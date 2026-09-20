//! The explicit relational algebra node family (ticket 04, decision `D5`).
//!
//! These nodes replace root-local query lowering with a first-class algebra that
//! can represent join / subquery / grouping shapes. Every node carries:
//!
//! - the originating HIR node id ([`HirNodeIdV1`]) and span, so diagnostics and
//!   explain trees map back to source (`D20`);
//! - an explicit OUTPUT SCHEMA of columns with PROVENANCE ([`ColumnV1`]), so
//!   alias-aware field resolution and downstream planning never re-derive where a
//!   column came from;
//! - an explicit OUTPUT CARDINALITY ([`CardinalityV1`] from ticket 03), so
//!   set/bag/sequence multiplicity and duplicate-elimination behaviour is
//!   testable per operator family (`D26`).
//!
//! Grouping keys and aggregate expressions are kept STRICTLY SEPARATE: a
//! [`GroupV1`] carries only key columns, an [`AggregateV1`] carries only the
//! folds. Sort and limit semantics ([`SortV1`]/[`LimitV1`]) are independent of
//! rendering — they live in the algebra, not in a presentation layer (`D24`).
//!
//! Correlated / lateral subqueries are represented EXPLICITLY by [`ApplyV1`]
//! ("apply"/lateral join): the right input is evaluated once per left row and may
//! reference the left row's columns through visible CORRELATION columns. There is
//! no root-local side channel and no planner-only heuristic (`D23`).

use std::collections::BTreeMap;

use quanta_gqlang_v2_hir::{BoolExprHirV1, HirNodeIdV1, SpanV1, ValueExprHirV1};
use quanta_gqlang_v2_types::CardinalityV1;

/// Closed language-owned relation identities. Authored names are resolved to
/// these identities once during normalization; downstream consumers never use
/// a string spelling as builtin authority.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum BuiltinRelationV1 {
    WorkspaceFiles,
}

impl BuiltinRelationV1 {
    #[must_use]
    pub const fn authored_name_v1(self) -> &'static str {
        match self {
            Self::WorkspaceFiles => "files",
        }
    }
}

/// Resolved identity of an algebra relation source.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RelationSourceIdentityV1 {
    Builtin(BuiltinRelationV1),
    CompositionBinding(String),
    Named(String),
    SchemaOpen,
}

/// Exact relation identities plus the authority's missing-name contract.
///
/// Structural compilation remains explicitly schema-open while still being
/// able to install exact composition-local bindings. Semantic compilation is
/// closed: an authored name absent from its exact map is a typed error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RelationSourceAuthorityV1 {
    SchemaOpen(BTreeMap<String, RelationSourceIdentityV1>),
    Closed(BTreeMap<String, RelationSourceIdentityV1>),
}

impl Default for RelationSourceAuthorityV1 {
    fn default() -> Self {
        Self::schema_open_v1()
    }
}

impl RelationSourceAuthorityV1 {
    #[must_use]
    pub const fn schema_open_v1() -> Self {
        Self::SchemaOpen(BTreeMap::new())
    }

    #[must_use]
    pub const fn closed_v1(identities_v1: BTreeMap<String, RelationSourceIdentityV1>) -> Self {
        Self::Closed(identities_v1)
    }

    pub fn insert_exact_v1(
        &mut self,
        authored_name_v1: impl Into<String>,
        identity_v1: RelationSourceIdentityV1,
    ) {
        match self {
            Self::SchemaOpen(identities_v1) | Self::Closed(identities_v1) => {
                identities_v1.insert(authored_name_v1.into(), identity_v1);
            }
        }
    }

    pub(crate) fn exact_identity_v1(
        &self,
        authored_name_v1: &str,
    ) -> Option<&RelationSourceIdentityV1> {
        match self {
            Self::SchemaOpen(identities_v1) | Self::Closed(identities_v1) => {
                identities_v1.get(authored_name_v1)
            }
        }
    }

    pub(crate) const fn is_closed_v1(&self) -> bool {
        matches!(self, Self::Closed(_))
    }
}

/// Where a single output column originated.
///
/// Provenance is explicit so alias-aware resolution and downstream planning do
/// not re-walk the HIR. A column is either pulled straight from a base relation
/// source (`Source`), produced by a projection/aggregate expression (`Derived`),
/// captured from an outer query for a correlated [`ApplyV1`] (`Correlated`), or
/// merged from union arms that agree on a name (`Unioned`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ColumnProvenanceV1 {
    /// Column read directly from a named relation source, optionally via an
    /// alias. `field` is the source-side field name (the segment after the
    /// alias, e.g. the `id` in `u.id`); `None` means the whole row binding.
    Source {
        /// The relation source name as written (e.g. `users`).
        relation: String,
        /// The binding alias in scope (e.g. `u`), if the source was aliased.
        alias: Option<String>,
        /// The source field selected, if a specific field was projected.
        field: Option<String>,
    },
    /// Column produced by a derived expression (projection item or aggregate
    /// fold) over the operator's input. Points back at the HIR node that
    /// produced the value so explain can show the expression.
    Derived {
        /// HIR id of the producing value/aggregate expression.
        origin: HirNodeIdV1,
    },
    /// Column captured from the OUTER (left) input of a correlated
    /// [`ApplyV1`]. Records the outer column it correlates to so the
    /// correlation is visible rather than implicit.
    Correlated {
        /// Index of the outer column this correlates to.
        outer_column: usize,
        /// Name of the outer column, for stable explain output.
        outer_name: String,
    },
    /// Column merged from positionally-aligned [`UnionV1`] arms.
    Unioned {
        /// Provenance of the same column position in the left arm.
        left: Box<ColumnProvenanceV1>,
        /// Provenance of the same column position in the right arm.
        right: Box<ColumnProvenanceV1>,
    },
}

/// One output column of an algebra node: its visible name plus its provenance.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ColumnV1 {
    /// The column's visible output name (the alias the rest of the query sees).
    pub name: String,
    /// Where the column came from.
    pub provenance: ColumnProvenanceV1,
}

impl ColumnV1 {
    /// Construct an output column.
    #[must_use]
    pub fn new(name: impl Into<String>, provenance: ColumnProvenanceV1) -> Self {
        Self {
            name: name.into(),
            provenance,
        }
    }
}

/// An algebra node: an operator payload plus its identity, output schema and
/// output cardinality.
#[derive(Clone, Debug, PartialEq)]
pub struct AlgebraNodeV1 {
    /// HIR id of the construct this node was normalized from.
    pub origin: HirNodeIdV1,
    /// Source byte range of the originating construct.
    pub span: SpanV1,
    /// Ordered output columns with provenance.
    pub columns: Vec<ColumnV1>,
    /// Output multiplicity / duplicate-elimination / order semantics.
    pub cardinality: CardinalityV1,
    /// The operator payload.
    pub op: AlgebraOpV1,
}

impl AlgebraNodeV1 {
    /// Assemble a node from its parts.
    ///
    /// Enforces the structural invariant that an [`AlgebraOpV1::Aggregate`]'s
    /// direct input is always an [`AlgebraOpV1::Group`] node (see
    /// [`AggregateV1`]). The check is a `debug_assert`: a malformed
    /// `Aggregate(input=non-Group)` fails fast in debug/test builds rather than
    /// being silently misclassified downstream, while release builds pay no
    /// runtime cost.
    #[must_use]
    pub fn new(
        origin: HirNodeIdV1,
        span: SpanV1,
        columns: Vec<ColumnV1>,
        cardinality: CardinalityV1,
        op: AlgebraOpV1,
    ) -> Self {
        if let AlgebraOpV1::Aggregate(aggregate) = &op {
            debug_assert!(
                matches!(aggregate.input.op, AlgebraOpV1::Group(_)),
                "Aggregate input must be a Group node (scalar aggregates use \
                 Group(keys=[])), but found {:?}",
                aggregate.input.family()
            );
        }
        Self {
            origin,
            span,
            columns,
            cardinality,
            op,
        }
    }

    /// The operator family tag of this node, for diagnostics and explain.
    #[must_use]
    pub fn family(&self) -> AlgebraFamilyV1 {
        self.op.family()
    }
}

/// The closed set of algebra operator payloads.
#[derive(Clone, Debug, PartialEq)]
pub enum AlgebraOpV1 {
    /// Leaf: scan a named relation source. Output cardinality is the source's
    /// declared cardinality (a source defaults to a `Bag` until typing narrows
    /// it — duplicates are NOT silently eliminated, see `D26`).
    Scan(ScanV1),
    /// Project: re-shape columns of one input. Pure column rewrite; does NOT
    /// change duplicate semantics (a `project` over a bag stays a bag), so
    /// duplicate elimination is never implicit in a projection.
    Project(ProjectV1),
    /// Filter: retain rows of one input that satisfy a predicate. Preserves the
    /// input cardinality exactly (filtering a bag yields a bag).
    Filter(FilterV1),
    /// Join: pair rows of two independent inputs on a predicate. Output is a
    /// bag: the cross of two bags multiplies multiplicities; no dedup.
    Join(JoinV1),
    /// Apply (lateral / correlated join): evaluate `right` once per `left` row,
    /// with `left`'s columns visible as correlation columns. This is the
    /// explicit carrier for correlated subqueries — no side channel.
    Apply(ApplyV1),
    /// Group: partition one input by key columns. Output rows are the DISTINCT
    /// key tuples, so a group is always a `Set` over its keys. Aggregates fold
    /// per partition in a following [`AggregateV1`]; keys and folds never mix.
    Group(GroupV1),
    /// Aggregate: fold each partition produced by a [`GroupV1`] input into
    /// per-group values. Carries ONLY aggregate expressions; grouping keys live
    /// in the input `Group` node. Duplicate sensitivity of each fold is explicit
    /// per [`AggregateExprV1::distinct`].
    Aggregate(AggregateV1),
    /// Sort: order one input by sort keys. Output cardinality becomes
    /// `Sequence` (order is now semantic); multiplicity of rows is preserved.
    Sort(SortV1),
    /// Limit: keep at most `count` rows (after optional `offset`). Requires an
    /// ordered (`Sequence`) input for a deterministic result; preserves
    /// cardinality kind.
    Limit(LimitV1),
    /// Union: combine two union-compatible inputs. `distinct` selects set
    /// (duplicate-eliminating) vs bag (multiplicity-preserving) semantics
    /// explicitly — there is no implicit default (`D26`).
    Union(UnionV1),
    /// SemiJoin: keep rows of `left` that have at least one match in `right`.
    /// Output columns and cardinality are exactly `left`'s; `right` contributes
    /// no columns and no multiplicity.
    SemiJoin(SemiJoinV1),
    /// AntiJoin: keep rows of `left` that have NO match in `right`. Output
    /// columns and cardinality are exactly `left`'s.
    AntiJoin(AntiJoinV1),
}

impl AlgebraOpV1 {
    /// The family tag of this payload.
    #[must_use]
    pub fn family(&self) -> AlgebraFamilyV1 {
        match self {
            Self::Scan(_) => AlgebraFamilyV1::Scan,
            Self::Project(_) => AlgebraFamilyV1::Project,
            Self::Filter(_) => AlgebraFamilyV1::Filter,
            Self::Join(_) => AlgebraFamilyV1::Join,
            Self::Apply(_) => AlgebraFamilyV1::Apply,
            Self::Group(_) => AlgebraFamilyV1::Group,
            Self::Aggregate(_) => AlgebraFamilyV1::Aggregate,
            Self::Sort(_) => AlgebraFamilyV1::Sort,
            Self::Limit(_) => AlgebraFamilyV1::Limit,
            Self::Union(_) => AlgebraFamilyV1::Union,
            Self::SemiJoin(_) => AlgebraFamilyV1::SemiJoin,
            Self::AntiJoin(_) => AlgebraFamilyV1::AntiJoin,
        }
    }
}

/// The operator family of a node, used for stable diagnostic and explain text.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AlgebraFamilyV1 {
    /// [`AlgebraOpV1::Scan`].
    Scan,
    /// [`AlgebraOpV1::Project`].
    Project,
    /// [`AlgebraOpV1::Filter`].
    Filter,
    /// [`AlgebraOpV1::Join`].
    Join,
    /// [`AlgebraOpV1::Apply`].
    Apply,
    /// [`AlgebraOpV1::Group`].
    Group,
    /// [`AlgebraOpV1::Aggregate`].
    Aggregate,
    /// [`AlgebraOpV1::Sort`].
    Sort,
    /// [`AlgebraOpV1::Limit`].
    Limit,
    /// [`AlgebraOpV1::Union`].
    Union,
    /// [`AlgebraOpV1::SemiJoin`].
    SemiJoin,
    /// [`AlgebraOpV1::AntiJoin`].
    AntiJoin,
}

impl AlgebraFamilyV1 {
    /// Stable lowercase keyword for this family, shared by CLI/LSP/tests.
    #[must_use]
    pub const fn keyword(self) -> &'static str {
        match self {
            Self::Scan => "scan",
            Self::Project => "project",
            Self::Filter => "filter",
            Self::Join => "join",
            Self::Apply => "apply",
            Self::Group => "group",
            Self::Aggregate => "aggregate",
            Self::Sort => "sort",
            Self::Limit => "limit",
            Self::Union => "union",
            Self::SemiJoin => "semijoin",
            Self::AntiJoin => "antijoin",
        }
    }
}

/// Leaf scan of a named relation source.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScanV1 {
    /// The relation source name as written.
    pub relation: String,
    /// Typed source identity resolved by the frontend context.
    pub source_identity_v1: RelationSourceIdentityV1,
    /// The binding alias in scope, if the source was aliased.
    pub alias: Option<String>,
}

/// Project: re-shape input columns through projection expressions.
#[derive(Clone, Debug, PartialEq)]
pub struct ProjectV1 {
    /// The single input node.
    pub input: Box<AlgebraNodeV1>,
    /// Ordered projection expressions producing the output columns. Each item's
    /// position aligns with the owning node's `columns` schema.
    pub items: Vec<ValueExprHirV1>,
}

/// Filter: retain input rows satisfying a predicate.
#[derive(Clone, Debug, PartialEq)]
pub struct FilterV1 {
    /// The single input node.
    pub input: Box<AlgebraNodeV1>,
    /// The retain predicate (shared boolean kernel).
    pub predicate: Box<BoolExprHirV1>,
}

/// Join: pair rows of two independent inputs on a predicate.
#[derive(Clone, Debug, PartialEq)]
pub struct JoinV1 {
    /// Left input.
    pub left: Box<AlgebraNodeV1>,
    /// Right input.
    pub right: Box<AlgebraNodeV1>,
    /// Join predicate, if any. `None` is an explicit cross/cartesian join.
    pub predicate: Option<Box<BoolExprHirV1>>,
}

/// Apply (lateral / correlated join).
#[derive(Clone, Debug, PartialEq)]
pub struct ApplyV1 {
    /// Outer (left) input; evaluated first, drives the correlation.
    pub left: Box<AlgebraNodeV1>,
    /// Inner (right) input; evaluated once per `left` row. Its correlated
    /// columns (those whose provenance is [`ColumnProvenanceV1::Correlated`])
    /// reference `left`'s columns.
    pub right: Box<AlgebraNodeV1>,
    /// The explicit correlation columns: which of `left`'s columns the right
    /// input depends on. Empty means a non-correlated (degenerate) apply, which
    /// normalization rejects in favour of a plain [`JoinV1`].
    pub correlation: Vec<CorrelationColumnV1>,
}

/// A single visible correlation column carried into the inner side of an
/// [`ApplyV1`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CorrelationColumnV1 {
    /// Index into the outer (left) node's `columns`.
    pub outer_column: usize,
    /// Name of the outer column, duplicated for stable explain output.
    pub outer_name: String,
}

/// Group: partition input by key columns. Keys ONLY; no aggregates here.
#[derive(Clone, Debug, PartialEq)]
pub struct GroupV1 {
    /// The single input node being partitioned.
    pub input: Box<AlgebraNodeV1>,
    /// Ordered grouping key expressions. Output columns are exactly these keys,
    /// distinct per partition.
    pub keys: Vec<ValueExprHirV1>,
}

/// Aggregate: fold each partition of a [`GroupV1`] input.
///
/// STRUCTURAL INVARIANT: the input to an Aggregate MUST be a [`GroupV1`] node
/// (`AlgebraOpV1::Group`). Grouping keys never live on the Aggregate — they are
/// the output columns of the input Group, and the Aggregate carries only folds.
/// Scalar aggregates (no grouping keys) are represented as
/// `Aggregate(input=Group(keys=[]))`, NOT as an Aggregate over a Scan/Filter.
/// This invariant is checked by a `debug_assert` in [`AlgebraNodeV1::new`], so a
/// malformed `Aggregate(input=non-Group)` fails fast in debug/test builds rather
/// than being silently misclassified downstream.
#[derive(Clone, Debug, PartialEq)]
pub struct AggregateV1 {
    /// The grouping input. MUST be a [`GroupV1`] node (`AlgebraOpV1::Group`); a
    /// scalar aggregate uses `Group(keys=[])`. See the struct-level invariant.
    pub input: Box<AlgebraNodeV1>,
    /// Ordered aggregate folds. Aggregates ONLY; grouping keys live in `input`.
    pub aggregates: Vec<AggregateExprV1>,
}

/// A single aggregate fold with explicit duplicate sensitivity.
#[derive(Clone, Debug, PartialEq)]
pub struct AggregateExprV1 {
    /// The aggregate callee name as written (e.g. `count`, `sum`).
    pub function: String,
    /// HIR id of the originating call expression, for provenance/explain.
    pub origin: HirNodeIdV1,
    /// The folded argument expressions (e.g. the `x` in `sum(x)`). `count(*)`
    /// has no argument and yields an empty list.
    pub args: Vec<ValueExprHirV1>,
    /// Whether this fold eliminates duplicates of its argument BEFORE folding
    /// (`count(distinct x)` vs `count(x)`). This makes bag-vs-set aggregate
    /// behaviour explicit and testable (`D26`), instead of leaking SQL bag
    /// defaults.
    pub distinct: bool,
}

/// Sort: order input by sort keys.
#[derive(Clone, Debug, PartialEq)]
pub struct SortV1 {
    /// The single input node.
    pub input: Box<AlgebraNodeV1>,
    /// Ordered sort keys. Earlier keys are more significant.
    pub keys: Vec<SortKeyV1>,
}

/// One sort key: an expression plus a direction.
#[derive(Clone, Debug, PartialEq)]
pub struct SortKeyV1 {
    /// The key expression.
    pub expr: ValueExprHirV1,
    /// Sort direction.
    pub direction: SortDirectionV1,
}

/// Sort direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SortDirectionV1 {
    /// Ascending order.
    Ascending,
    /// Descending order.
    Descending,
}

impl SortDirectionV1 {
    /// Stable keyword for explain/diagnostics.
    #[must_use]
    pub const fn keyword(self) -> &'static str {
        match self {
            Self::Ascending => "asc",
            Self::Descending => "desc",
        }
    }
}

/// Limit: keep at most `count` rows after `offset`.
#[derive(Clone, Debug, PartialEq)]
pub struct LimitV1 {
    /// The single input node.
    pub input: Box<AlgebraNodeV1>,
    /// Maximum number of rows to keep.
    pub count: u64,
    /// Rows to skip before keeping, defaulting to zero.
    pub offset: u64,
}

/// Union: combine two union-compatible inputs.
#[derive(Clone, Debug, PartialEq)]
pub struct UnionV1 {
    /// Left arm.
    pub left: Box<AlgebraNodeV1>,
    /// Right arm.
    pub right: Box<AlgebraNodeV1>,
    /// Whether duplicates across arms are eliminated (`set` union) or preserved
    /// (`bag` union). Explicit, never defaulted.
    pub distinct: bool,
}

/// SemiJoin: keep `left` rows with at least one `right` match.
#[derive(Clone, Debug, PartialEq)]
pub struct SemiJoinV1 {
    /// Left input; its columns and cardinality are the output.
    pub left: Box<AlgebraNodeV1>,
    /// Right input; tested for existence only, contributes no columns.
    pub right: Box<AlgebraNodeV1>,
    /// Match predicate, if any.
    pub predicate: Option<Box<BoolExprHirV1>>,
}

/// AntiJoin: keep `left` rows with NO `right` match.
#[derive(Clone, Debug, PartialEq)]
pub struct AntiJoinV1 {
    /// Left input; its columns and cardinality are the output.
    pub left: Box<AlgebraNodeV1>,
    /// Right input; tested for non-existence only, contributes no columns.
    pub right: Box<AlgebraNodeV1>,
    /// Match predicate, if any.
    pub predicate: Option<Box<BoolExprHirV1>>,
}
