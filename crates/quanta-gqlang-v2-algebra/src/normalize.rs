//! Provenance-preserving normalization from shared HIR relation/query shapes
//! into the explicit algebra (ticket 04, decisions `D3`/`D5`/`D23`/`D26`).
//!
//! The normalizer consumes the SAME shared HIR every other v3 surface consumes
//! ([`RelationExprHirV1`] / [`QueryHirV1`]) and lowers each construct into an
//! [`AlgebraNodeV1`] with explicit column provenance and cardinality. It runs
//! AFTER binding (`D3`), so callers may hand it binder context; correlation in
//! [`AlgebraOpV1::Apply`] is derived from VISIBLE outer columns, never a
//! root-local side channel.
//!
//! ## Composition symbol convention
//!
//! HIR records composition as an open-tagged [`CompositionV1`] (a symbol over
//! relation operands). This module is the closed authority that maps those
//! symbols to algebra operators. Unknown symbols and wrong arities reject with a
//! typed [`AlgebraDiagnosticV1`] — never a silent downgrade (`D10`).
//!
//! | symbol                       | operator   | operands |
//! |------------------------------|------------|----------|
//! | `join`                       | Join       | 2        |
//! | `cross`                      | Join(none) | 2        |
//! | `apply` / `lateral`          | Apply      | 2        |
//! | `union` / `union_all`        | Union      | 2        |
//! | `semijoin`                   | SemiJoin   | 2        |
//! | `antijoin`                   | AntiJoin   | 2        |
//! | `group_by`                   | Group      | 2        |
//! | `order_by`                   | Sort       | 2        |
//! | `sort`                       | Sort       | 1        |
//! | `limit`                      | Limit      | 1        |
//!
//! Sort/limit are modelled as unary compositions because the shared HIR has no
//! dedicated order/limit clause; their keys/counts are sidecar inputs supplied
//! through [`NormalizeOptionsV1`]. This keeps sort/limit semantics in the
//! algebra and independent of rendering (`D24`).

use crate::diagnostic::{AlgebraDiagnosticV1, IllFormedAggregateReasonV1};
use crate::node::{
    AggregateExprV1, AggregateV1, AlgebraNodeV1, AlgebraOpV1, AntiJoinV1, ApplyV1,
    ColumnProvenanceV1, ColumnV1, CorrelationColumnV1, FilterV1, GroupV1, JoinV1, LimitV1,
    ProjectV1, RelationSourceAuthorityV1, RelationSourceIdentityV1, ScanV1, SemiJoinV1, SortKeyV1,
    SortV1, UnionV1,
};
use quanta_gqlang_v2_hir::{
    CompositionV1, ProjectionItemV1, ProjectionV1, QueryHirV1, RelationExprHirV1,
    RelationExprKindV1, RelationFilterV1, RelationSourceV1, ValueExprHirV1, ValueExprKindV1,
};
use quanta_gqlang_v2_types::CardinalityV1;
use std::collections::{BTreeMap, BTreeSet};

/// Caller-supplied normalization context.
///
/// Two pieces of information cannot be recovered from the structure-only HIR and
/// must be supplied by the caller (which has the binder/type context the
/// normalizer deliberately does not re-derive):
///
/// - which callee names are AGGREGATES (so a projection over them lowers to
///   Group+Aggregate). Defaulting this would re-implement the closed builtin
///   registry from ticket 03, which is out of lane.
/// - the sort keys / limit counts for the unary `sort`/`limit` compositions.
#[derive(Clone, Debug, Default)]
pub struct NormalizeOptionsV1 {
    /// Callee names treated as aggregate folds.
    pub aggregates: BTreeSet<String>,
    /// Callee names whose `distinct` argument form eliminates duplicates before
    /// folding. Modelled as a marker callee `distinct` wrapping the real arg.
    pub distinct_marker: Option<String>,
    /// The `(count, offset)` window for the unary `limit` composition. The
    /// structure-only HIR has no limit clause, so the caller (which parsed the
    /// surface count/offset) supplies it here. `None` means "keep everything"
    /// (`count = u64::MAX`, `offset = 0`).
    pub limit_window: Option<(u64, u64)>,
    /// Exact relation identities and the missing-name contract issued by the
    /// compile context.
    pub relation_source_authority_v1: RelationSourceAuthorityV1,
}

impl NormalizeOptionsV1 {
    /// A context whose aggregate set is the given names, with `distinct` as the
    /// duplicate-elimination marker callee.
    #[must_use]
    pub fn with_aggregates<I, S>(aggregates: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            aggregates: aggregates.into_iter().map(Into::into).collect(),
            distinct_marker: Some("distinct".to_owned()),
            limit_window: None,
            relation_source_authority_v1: RelationSourceAuthorityV1::schema_open_v1(),
        }
    }

    #[must_use]
    pub fn with_relation_source_authority_v1(
        mut self,
        authority_v1: RelationSourceAuthorityV1,
    ) -> Self {
        self.relation_source_authority_v1 = authority_v1;
        self
    }

    /// Set the `(count, offset)` window threaded into the unary `limit`
    /// composition. Consumes and returns `self` for builder-style chaining.
    #[must_use]
    pub fn with_limit_window(mut self, count: u64, offset: u64) -> Self {
        self.limit_window = Some((count, offset));
        self
    }

    fn is_aggregate(&self, name: &str) -> bool {
        self.aggregates.contains(name)
    }
}

/// The output of one normalization run: the algebra root or a typed diagnostic.
///
/// Fail-closed: any unsupported/ill-formed construct yields `Err` with a typed
/// [`AlgebraDiagnosticV1`]; there is no partial/degraded success.
pub type NormalizeResultV1 = Result<AlgebraNodeV1, AlgebraDiagnosticV1>;

/// Normalize a whole query unit into algebra. The optional result shape is NOT
/// re-shaped here (render semantics are ticket 07); only the relation body is
/// lowered, preserving column provenance.
///
/// # Errors
///
/// Returns a typed [`AlgebraDiagnosticV1`] for any unsupported or ill-formed
/// construct in `query.body`.
pub fn normalize_query(query: &QueryHirV1, options: &NormalizeOptionsV1) -> NormalizeResultV1 {
    normalize_relation(&query.body, options)
}

/// Normalize a single relation expression into algebra.
///
/// # Errors
///
/// Returns a typed [`AlgebraDiagnosticV1`] for any unsupported or ill-formed
/// construct.
pub fn normalize_relation(
    relation: &RelationExprHirV1,
    options: &NormalizeOptionsV1,
) -> NormalizeResultV1 {
    // Collect alias -> relation bindings once over the whole tree so projection
    // provenance is ALIAS-AWARE: `u.id` over `users u` knows its source is
    // `users`, not the alias `u`. Aliases are unique per the binder (D3).
    let mut aliases: BTreeMap<String, String> = BTreeMap::new();
    collect_aliases(relation, &mut aliases);
    let ctx = Ctx {
        options,
        aliases: &aliases,
    };
    normalize_node(relation, &ctx)
}

/// Internal normalization context: the public options plus the alias->relation
/// map derived from relation sources in scope.
struct Ctx<'a> {
    options: &'a NormalizeOptionsV1,
    aliases: &'a BTreeMap<String, String>,
}

impl Ctx<'_> {
    /// Resolve an alias (or relation name) to its underlying relation name.
    fn relation_of(&self, name: &str) -> String {
        self.aliases
            .get(name)
            .cloned()
            .unwrap_or_else(|| name.to_owned())
    }

    fn source_identity_v1(
        &self,
        relation_v1: &str,
        span_v1: quanta_gqlang_v2_hir::SpanV1,
    ) -> Result<RelationSourceIdentityV1, AlgebraDiagnosticV1> {
        if let Some(identity_v1) = self
            .options
            .relation_source_authority_v1
            .exact_identity_v1(relation_v1)
        {
            return Ok(identity_v1.clone());
        }
        if self.options.relation_source_authority_v1.is_closed_v1() {
            return Err(AlgebraDiagnosticV1::UnknownRelationSource {
                relation: relation_v1.to_string(),
                span: span_v1,
            });
        }
        Ok(RelationSourceIdentityV1::SchemaOpen)
    }
}

fn collect_aliases(relation: &RelationExprHirV1, acc: &mut BTreeMap<String, String>) {
    match &relation.kind {
        RelationExprKindV1::Source(source) => {
            if let Some(alias) = &source.alias {
                let _ = acc.insert(alias.name.clone(), source.name.clone());
            }
        }
        RelationExprKindV1::Project(project) => collect_aliases(&project.input, acc),
        RelationExprKindV1::Filter(filter) => collect_aliases(&filter.input, acc),
        RelationExprKindV1::Compose(compose) => {
            for operand in &compose.operands {
                collect_aliases(operand, acc);
            }
        }
    }
}

fn normalize_node(relation: &RelationExprHirV1, ctx: &Ctx<'_>) -> NormalizeResultV1 {
    match &relation.kind {
        RelationExprKindV1::Source(source) => normalize_source(relation, source, ctx),
        RelationExprKindV1::Filter(filter) => normalize_filter(relation, filter, ctx),
        RelationExprKindV1::Project(project) => normalize_project(relation, project, ctx),
        RelationExprKindV1::Compose(compose) => normalize_compose(relation, compose, ctx),
    }
}

fn normalize_source(
    node: &RelationExprHirV1,
    source: &RelationSourceV1,
    ctx: &Ctx<'_>,
) -> NormalizeResultV1 {
    let alias = source.alias.as_ref().map(|a| a.name.clone());
    // A scan binds the whole row under the alias; the single output column is
    // the row binding with Source provenance and no specific field.
    let bind_name = alias.clone().unwrap_or_else(|| source.name.clone());
    let column = ColumnV1::new(
        bind_name,
        ColumnProvenanceV1::Source {
            relation: source.name.clone(),
            alias: alias.clone(),
            field: None,
        },
    );
    // A raw source is a BAG: duplicate rows are not silently eliminated (D26).
    Ok(AlgebraNodeV1::new(
        node.id,
        node.span,
        vec![column],
        CardinalityV1::Bag,
        AlgebraOpV1::Scan(ScanV1 {
            relation: source.name.clone(),
            source_identity_v1: ctx.source_identity_v1(&source.name, node.span)?,
            alias,
        }),
    ))
}

fn normalize_filter(
    node: &RelationExprHirV1,
    filter: &RelationFilterV1,
    ctx: &Ctx<'_>,
) -> NormalizeResultV1 {
    let input = normalize_node(&filter.input, ctx)?;
    // Filter preserves the input schema AND cardinality exactly.
    let columns = input.columns.clone();
    let cardinality = input.cardinality;
    Ok(AlgebraNodeV1::new(
        node.id,
        node.span,
        columns,
        cardinality,
        AlgebraOpV1::Filter(FilterV1 {
            input: Box::new(input),
            predicate: Box::new((*filter.predicate).clone()),
        }),
    ))
}

fn normalize_project(
    node: &RelationExprHirV1,
    project: &ProjectionV1,
    ctx: &Ctx<'_>,
) -> NormalizeResultV1 {
    let input = normalize_node(&project.input, ctx)?;

    // Split items into grouping keys (non-aggregate) and aggregate folds.
    let mut keys: Vec<ProjItem<'_>> = Vec::new();
    let mut aggregates: Vec<ProjItem<'_>> = Vec::new();
    for item in &project.items {
        match classify_item(item, ctx.options)? {
            ItemClass::Key => keys.push(ProjItem(item)),
            ItemClass::Aggregate => aggregates.push(ProjItem(item)),
        }
    }

    if aggregates.is_empty() {
        return Ok(plain_project(node, input, project, ctx));
    }

    // Grouped aggregation: produce an explicit Group over the keys feeding an
    // Aggregate over the folds. Keys and folds are kept STRICTLY separate.
    if project.items.is_empty() {
        return Err(AlgebraDiagnosticV1::IllFormedAggregate {
            reason: IllFormedAggregateReasonV1::EmptyAggregateProjection,
            span: node.span,
        });
    }

    let group_columns: Vec<ColumnV1> = keys.iter().map(|item| key_column(item.0, ctx)).collect();
    // A Group's output is the DISTINCT key tuples: a Set over its keys.
    let group = AlgebraNodeV1::new(
        node.id,
        node.span,
        group_columns,
        CardinalityV1::Set,
        AlgebraOpV1::Group(GroupV1 {
            input: Box::new(input),
            keys: keys.iter().map(|item| item.0.value.clone()).collect(),
        }),
    );

    let mut out_columns: Vec<ColumnV1> = group.columns.clone();
    let mut folds: Vec<AggregateExprV1> = Vec::new();
    for item in &aggregates {
        let fold = aggregate_expr(item.0, ctx.options)?;
        out_columns.push(ColumnV1::new(
            alias_or_default(item.0, &fold.function),
            ColumnProvenanceV1::Derived {
                origin: fold.origin,
            },
        ));
        folds.push(fold);
    }

    // Aggregate output is a Set keyed by the grouping keys (one row per group).
    Ok(AlgebraNodeV1::new(
        node.id,
        node.span,
        out_columns,
        CardinalityV1::Set,
        AlgebraOpV1::Aggregate(AggregateV1 {
            input: Box::new(group),
            aggregates: folds,
        }),
    ))
}

fn plain_project(
    node: &RelationExprHirV1,
    input: AlgebraNodeV1,
    project: &ProjectionV1,
    ctx: &Ctx<'_>,
) -> AlgebraNodeV1 {
    let columns: Vec<ColumnV1> = project
        .items
        .iter()
        .map(|item| project_column(item, ctx))
        .collect();
    // Projection NEVER changes duplicate semantics: project over a bag is a bag.
    let cardinality = input.cardinality;
    AlgebraNodeV1::new(
        node.id,
        node.span,
        columns,
        cardinality,
        AlgebraOpV1::Project(ProjectV1 {
            input: Box::new(input),
            items: project.items.iter().map(|i| i.value.clone()).collect(),
        }),
    )
}

fn normalize_compose(
    node: &RelationExprHirV1,
    compose: &CompositionV1,
    ctx: &Ctx<'_>,
) -> NormalizeResultV1 {
    let symbol = compose.op.symbol.as_str();
    match symbol {
        "join" | "cross" => binary_join(node, compose, ctx, symbol == "cross"),
        "apply" | "lateral" => binary_apply(node, compose, ctx),
        "union" | "union_all" => binary_union(node, compose, ctx, symbol == "union"),
        "semijoin" => binary_semi(node, compose, ctx),
        "antijoin" => binary_anti(node, compose, ctx),
        "group_by" => binary_group(node, compose, ctx),
        "order_by" => binary_order(node, compose, ctx),
        "sort" => unary_sort(node, compose, ctx),
        "limit" => unary_limit(node, compose, ctx),
        other => Err(AlgebraDiagnosticV1::UnsupportedComposition {
            symbol: other.to_owned(),
            span: node.span,
        }),
    }
}

fn two_operands<'c>(
    node: &RelationExprHirV1,
    compose: &'c CompositionV1,
) -> Result<(&'c RelationExprHirV1, &'c RelationExprHirV1), AlgebraDiagnosticV1> {
    match compose.operands.as_slice() {
        [left, right] => Ok((left, right)),
        other => Err(AlgebraDiagnosticV1::ArityMismatch {
            symbol: compose.op.symbol.clone(),
            expected: 2,
            found: other.len(),
            span: node.span,
        }),
    }
}

fn one_operand<'c>(
    node: &RelationExprHirV1,
    compose: &'c CompositionV1,
) -> Result<&'c RelationExprHirV1, AlgebraDiagnosticV1> {
    match compose.operands.as_slice() {
        [only] => Ok(only),
        other => Err(AlgebraDiagnosticV1::ArityMismatch {
            symbol: compose.op.symbol.clone(),
            expected: 1,
            found: other.len(),
            span: node.span,
        }),
    }
}

fn binary_join(
    node: &RelationExprHirV1,
    compose: &CompositionV1,
    ctx: &Ctx<'_>,
    cross: bool,
) -> NormalizeResultV1 {
    let (left_hir, right_hir) = two_operands(node, compose)?;
    let left = normalize_node(left_hir, ctx)?;
    let right = normalize_node(right_hir, ctx)?;
    // A join's output columns are the concatenation of both inputs; output is a
    // BAG (the cross multiplies multiplicities, no dedup).
    let mut columns = left.columns.clone();
    columns.extend(right.columns.clone());
    // `predicate = None` is RESERVED for the explicit `cross` operator (an opt-in
    // cartesian product). A plain `join` whose right operand carries no filter
    // predicate has a MISSING join condition; we must not fabricate a cross join
    // from it (D10, fail-closed). Validate post-extraction.
    let predicate = if cross {
        None
    } else {
        require_join_predicate(right_hir, &compose.op.symbol, node.span)?
    };
    Ok(AlgebraNodeV1::new(
        node.id,
        node.span,
        columns,
        CardinalityV1::Bag,
        AlgebraOpV1::Join(JoinV1 {
            left: Box::new(left),
            right: Box::new(right),
            predicate,
        }),
    ))
}

/// Pull a top-level filter predicate off the right operand of a join, if the
/// author wrote the join condition as a filter wrapper. Structure-only, so this
/// is the only place a join condition can come from in HIR.
fn join_predicate(right: &RelationExprHirV1) -> Option<Box<quanta_gqlang_v2_hir::BoolExprHirV1>> {
    if let RelationExprKindV1::Filter(filter) = &right.kind {
        Some(Box::new((*filter.predicate).clone()))
    } else {
        None
    }
}

/// Like [`join_predicate`] but fail-closed: a predicate is MANDATORY. Used for
/// join-family operators that have no cartesian form (`join`, `semijoin`,
/// `antijoin`). A missing predicate is never an implicit cross product — only
/// the explicit `cross` operator carries `predicate = None` (`D10`).
fn require_join_predicate(
    right: &RelationExprHirV1,
    symbol: &str,
    span: quanta_gqlang_v2_hir::SpanV1,
) -> Result<Option<Box<quanta_gqlang_v2_hir::BoolExprHirV1>>, AlgebraDiagnosticV1> {
    match join_predicate(right) {
        Some(p) => Ok(Some(p)),
        None => Err(AlgebraDiagnosticV1::EmptyJoinPredicate {
            symbol: symbol.to_owned(),
            span,
        }),
    }
}

fn binary_apply(
    node: &RelationExprHirV1,
    compose: &CompositionV1,
    ctx: &Ctx<'_>,
) -> NormalizeResultV1 {
    let (left_hir, right_hir) = two_operands(node, compose)?;
    let left = normalize_node(left_hir, ctx)?;
    let right = normalize_node(right_hir, ctx)?;

    // Derive VISIBLE correlation columns: which of the outer (left) columns the
    // inner (right) subtree references by name. This is the explicit carrier for
    // a correlated subquery (D23) — no side channel.
    //
    // A name that the inner subtree binds LOCALLY (its own source alias) shadows
    // the outer column of the same name: a reference to it resolves to the inner
    // binding, not the outer. Such names are therefore NOT outer references and
    // must not fabricate a correlation (which would also mislabel the inner
    // output column as Correlated when it is really local).
    let outer_names: Vec<String> = left.columns.iter().map(|c| c.name.clone()).collect();
    let mut local_bindings: BTreeMap<String, String> = BTreeMap::new();
    collect_aliases(right_hir, &mut local_bindings);
    let referenced: BTreeSet<String> = referenced_symbols(right_hir)
        .into_iter()
        .filter(|name| !local_bindings.contains_key(name))
        .collect();
    let mut correlation: Vec<CorrelationColumnV1> = Vec::new();
    for (idx, name) in outer_names.iter().enumerate() {
        if referenced.contains(name) {
            correlation.push(CorrelationColumnV1 {
                outer_column: idx,
                outer_name: name.clone(),
            });
        }
    }

    if correlation.is_empty() {
        // A degenerate apply with no correlation is a hidden side channel or a
        // mislabelled join: reject rather than silently downgrade.
        return Err(AlgebraDiagnosticV1::MissingCorrelation { span: node.span });
    }

    // Apply output cardinality (D26). Lateral expansion is order-preserving on the
    // LEFT dimension (for each outer row, in order, the correlated inner is
    // expanded), but a correlated inner may produce zero, one, or MANY rows per
    // outer row. So:
    //   - Sequence is PRESERVED: order does not change multiplicity.
    //   - Bag is PRESERVED: already duplicate-permitting.
    //   - Set DOWNGRADES to Bag: expanding each distinct outer row by a variable
    //     number of inner rows lets the same outer key recur across output rows,
    //     breaking the Set's distinctness guarantee. Preserving Set here would
    //     wrongly claim duplicate-elimination the lateral expansion does not provide.
    let cardinality = match left.cardinality {
        CardinalityV1::Set | CardinalityV1::Bag => CardinalityV1::Bag,
        CardinalityV1::Sequence => CardinalityV1::Sequence,
    };
    //
    // Correlated provenance is keyed by the outer symbol each right output column
    // DERIVES FROM, not by the column's output name. An inner projection may alias
    // an outer reference to a different name (`project { x: u }`): the output column
    // is named `x`, yet its value references outer `u`, so it must still carry
    // Correlated provenance pointing at `u`. Matching on the output name alone
    // would wrongly leave such a renamed column Derived (D23).
    let right_outer_refs = output_column_outer_refs(right_hir, &referenced);
    let mut columns = left.columns.clone();
    for (idx, col) in right.columns.iter().enumerate() {
        let outer_ref = right_outer_refs.get(idx).and_then(Option::as_ref);
        let provenance = if let Some(outer_name) = outer_ref {
            let outer = correlation
                .iter()
                .find(|c| &c.outer_name == outer_name)
                .map_or(0, |c| c.outer_column);
            ColumnProvenanceV1::Correlated {
                outer_column: outer,
                outer_name: outer_name.clone(),
            }
        } else {
            col.provenance.clone()
        };
        columns.push(ColumnV1::new(col.name.clone(), provenance));
    }

    Ok(AlgebraNodeV1::new(
        node.id,
        node.span,
        columns,
        cardinality,
        AlgebraOpV1::Apply(ApplyV1 {
            left: Box::new(left),
            right: Box::new(right),
            correlation,
        }),
    ))
}

fn binary_union(
    node: &RelationExprHirV1,
    compose: &CompositionV1,
    ctx: &Ctx<'_>,
    distinct: bool,
) -> NormalizeResultV1 {
    let (left_hir, right_hir) = two_operands(node, compose)?;
    let left = normalize_node(left_hir, ctx)?;
    let right = normalize_node(right_hir, ctx)?;
    if left.columns.len() != right.columns.len() {
        return Err(AlgebraDiagnosticV1::UnionArity {
            left: left.columns.len(),
            right: right.columns.len(),
            span: node.span,
        });
    }
    // Same count is necessary but not sufficient: the arms must also AGREE on each
    // positional output name. Otherwise the merged schema would silently adopt the
    // left arm's name and hide a genuine schema disagreement. Reject the first
    // mismatch rather than letting one arm's name win unannounced (D26).
    for (position, (l, r)) in left.columns.iter().zip(right.columns.iter()).enumerate() {
        if l.name != r.name {
            return Err(AlgebraDiagnosticV1::UnionColumnMismatch {
                position,
                left_name: l.name.clone(),
                right_name: r.name.clone(),
                span: node.span,
            });
        }
    }
    // Output columns merge positionally; provenance records both arms.
    let columns: Vec<ColumnV1> = left
        .columns
        .iter()
        .zip(right.columns.iter())
        .map(|(l, r)| {
            ColumnV1::new(
                l.name.clone(),
                ColumnProvenanceV1::Unioned {
                    left: Box::new(l.provenance.clone()),
                    right: Box::new(r.provenance.clone()),
                },
            )
        })
        .collect();
    // distinct => set semantics (dedup), else bag (multiplicity preserved). The
    // choice is explicit; there is no implicit default (D26).
    let cardinality = if distinct {
        CardinalityV1::Set
    } else {
        CardinalityV1::Bag
    };
    Ok(AlgebraNodeV1::new(
        node.id,
        node.span,
        columns,
        cardinality,
        AlgebraOpV1::Union(UnionV1 {
            left: Box::new(left),
            right: Box::new(right),
            distinct,
        }),
    ))
}

fn binary_semi(
    node: &RelationExprHirV1,
    compose: &CompositionV1,
    ctx: &Ctx<'_>,
) -> NormalizeResultV1 {
    let (left_hir, right_hir) = two_operands(node, compose)?;
    let left = normalize_node(left_hir, ctx)?;
    let right = normalize_node(right_hir, ctx)?;
    // Semi-join output is EXACTLY the left schema and cardinality.
    let columns = left.columns.clone();
    let cardinality = left.cardinality;
    // A semi-join has no cartesian form: a missing predicate is always a defect,
    // never an opt-in cross. Reject rather than fabricate `predicate = None`.
    let predicate = require_join_predicate(right_hir, &compose.op.symbol, node.span)?;
    Ok(AlgebraNodeV1::new(
        node.id,
        node.span,
        columns,
        cardinality,
        AlgebraOpV1::SemiJoin(SemiJoinV1 {
            left: Box::new(left),
            right: Box::new(right),
            predicate,
        }),
    ))
}

fn binary_anti(
    node: &RelationExprHirV1,
    compose: &CompositionV1,
    ctx: &Ctx<'_>,
) -> NormalizeResultV1 {
    let (left_hir, right_hir) = two_operands(node, compose)?;
    let left = normalize_node(left_hir, ctx)?;
    let right = normalize_node(right_hir, ctx)?;
    let columns = left.columns.clone();
    let cardinality = left.cardinality;
    // An anti-join, like a semi-join, has no cartesian form: a missing predicate
    // is always a defect. Reject rather than fabricate `predicate = None`.
    let predicate = require_join_predicate(right_hir, &compose.op.symbol, node.span)?;
    Ok(AlgebraNodeV1::new(
        node.id,
        node.span,
        columns,
        cardinality,
        AlgebraOpV1::AntiJoin(AntiJoinV1 {
            left: Box::new(left),
            right: Box::new(right),
            predicate,
        }),
    ))
}

/// Lower a `group_by` composition into an explicit [`AlgebraOpV1::Group`].
///
/// The surface emits `Compose("group_by", [input, keys])` where `keys` is a
/// `Project` whose items carry the group-key value expressions over a synthetic
/// `_keys` placeholder source (the surface cannot bind the real producer). The
/// algebra is the closed authority that turns that structure-only carrier into
/// a Group: the placeholder is discarded and the keys are partitioned over the
/// REAL `input`. Output columns are exactly the keys (alias-aware provenance),
/// and a Group's output is the DISTINCT key tuples — a Set (`D26`).
fn binary_group(
    node: &RelationExprHirV1,
    compose: &CompositionV1,
    ctx: &Ctx<'_>,
) -> NormalizeResultV1 {
    let (input_hir, keys_hir) = two_operands(node, compose)?;
    let input = normalize_node(input_hir, ctx)?;

    // The keys operand is the synthetic Project carrying the group-key items.
    let RelationExprKindV1::Project(keys_project) = &keys_hir.kind else {
        return Err(AlgebraDiagnosticV1::IllFormedAggregate {
            reason: IllFormedAggregateReasonV1::EmptyAggregateProjection,
            span: node.span,
        });
    };
    // A `group by` with no keys produces no partition tuple at all.
    if keys_project.items.is_empty() {
        return Err(AlgebraDiagnosticV1::IllFormedAggregate {
            reason: IllFormedAggregateReasonV1::EmptyAggregateProjection,
            span: node.span,
        });
    }

    // Grouping keys must be plain key expressions — an aggregate fold cannot be
    // a partition key (its value is per-partition, not per-row).
    for item in &keys_project.items {
        if let ItemClass::Aggregate = classify_item(item, ctx.options)? {
            return Err(AlgebraDiagnosticV1::IllFormedAggregate {
                reason: IllFormedAggregateReasonV1::NestedAggregate,
                span: item.value.span,
            });
        }
    }

    // Output columns are exactly the keys, with alias-aware Source provenance.
    let columns: Vec<ColumnV1> = keys_project
        .items
        .iter()
        .map(|item| key_column(item, ctx))
        .collect();
    Ok(AlgebraNodeV1::new(
        node.id,
        node.span,
        columns,
        CardinalityV1::Set,
        AlgebraOpV1::Group(GroupV1 {
            input: Box::new(input),
            keys: keys_project
                .items
                .iter()
                .map(|item| item.value.clone())
                .collect(),
        }),
    ))
}

/// Lower an `order_by` composition into an explicit [`AlgebraOpV1::Sort`].
///
/// The surface emits `Compose("order_by", [input, keys])` where `keys` is a
/// `Project` whose items carry the sort-key value expressions over a synthetic
/// `_keys` placeholder source (the surface cannot bind the real producer), and
/// each item's ALIAS folds the sort direction as the open tag `asc`/`desc`.
/// The algebra is the closed authority that decodes that structure-only carrier
/// into a Sort: the placeholder is discarded, the keys order over the REAL
/// `input`, and the direction tag is decoded (defaulting to ascending). Sort
/// makes order semantic, so the output schema is the input's and the
/// cardinality becomes a Sequence (`D24`). Mirrors [`binary_group`], but lowers
/// to Sort rather than Group.
fn binary_order(
    node: &RelationExprHirV1,
    compose: &CompositionV1,
    ctx: &Ctx<'_>,
) -> NormalizeResultV1 {
    let (input_hir, keys_hir) = two_operands(node, compose)?;
    let input = normalize_node(input_hir, ctx)?;

    // The keys operand is the synthetic Project carrying the sort-key items;
    // anything else is a malformed carrier with no algebra lowering.
    let RelationExprKindV1::Project(keys_project) = &keys_hir.kind else {
        return Err(AlgebraDiagnosticV1::UnsupportedComposition {
            symbol: compose.op.symbol.clone(),
            span: node.span,
        });
    };
    // An `order by` with no keys imposes no order at all.
    if keys_project.items.is_empty() {
        return Err(AlgebraDiagnosticV1::UnsupportedComposition {
            symbol: compose.op.symbol.clone(),
            span: node.span,
        });
    }

    // Decode each key: the value expression is carried verbatim and the
    // direction is read off the item alias (`asc`/`desc`), ascending by default.
    let keys: Vec<SortKeyV1> = keys_project
        .items
        .iter()
        .map(|item| SortKeyV1 {
            expr: item.value.clone(),
            direction: sort_direction_of(item),
        })
        .collect();

    // Sort preserves the input schema exactly and makes order semantic.
    let columns = input.columns.clone();
    Ok(AlgebraNodeV1::new(
        node.id,
        node.span,
        columns,
        CardinalityV1::Sequence,
        AlgebraOpV1::Sort(SortV1 {
            input: Box::new(input),
            keys,
        }),
    ))
}

/// Decode the sort direction folded into a key item's alias by the surface.
/// The alias is an open tag: `desc` means descending, everything else (incl.
/// the absent alias and `asc`) is ascending.
fn sort_direction_of(item: &ProjectionItemV1) -> crate::node::SortDirectionV1 {
    match item.alias.as_ref().map(|a| a.name.as_str()) {
        Some("desc") => crate::node::SortDirectionV1::Descending,
        _ => crate::node::SortDirectionV1::Ascending,
    }
}

fn unary_sort(
    node: &RelationExprHirV1,
    compose: &CompositionV1,
    ctx: &Ctx<'_>,
) -> NormalizeResultV1 {
    let only = one_operand(node, compose)?;
    let input = normalize_node(only, ctx)?;
    // Sort makes order semantic: output becomes a Sequence, multiplicity kept.
    let columns = input.columns.clone();
    // Sort keys: project items of an inner Project are reused as keys, else the
    // input's columns are the keys. Direction defaults to ascending; descending
    // is signalled by a unary `-` on the key (structure-only convention).
    let keys: Vec<SortKeyV1> = sort_keys_from(&input);
    Ok(AlgebraNodeV1::new(
        node.id,
        node.span,
        columns,
        CardinalityV1::Sequence,
        AlgebraOpV1::Sort(SortV1 {
            input: Box::new(input),
            keys,
        }),
    ))
}

fn unary_limit(
    node: &RelationExprHirV1,
    compose: &CompositionV1,
    ctx: &Ctx<'_>,
) -> NormalizeResultV1 {
    let only = one_operand(node, compose)?;
    let input = normalize_node(only, ctx)?;
    // Limit requires an ordered input for a deterministic kept-set; reject a
    // bag/set limit rather than silently producing non-deterministic rows.
    if input.cardinality != CardinalityV1::Sequence {
        return Err(AlgebraDiagnosticV1::UnorderedLimit { span: node.span });
    }
    let columns = input.columns.clone();
    // The window is a caller sidecar (`D24`): the structure-only HIR carries no
    // count/offset. Absent a supplied window, keep everything from offset zero.
    let (count, offset) = ctx.options.limit_window.unwrap_or((u64::MAX, 0));
    Ok(AlgebraNodeV1::new(
        node.id,
        node.span,
        columns,
        CardinalityV1::Sequence,
        AlgebraOpV1::Limit(LimitV1 {
            input: Box::new(input),
            count,
            offset,
        }),
    ))
}

// ---- helpers --------------------------------------------------------------

/// A borrowed projection item, kept distinct so key/aggregate vectors do not
/// own clones until the column schema is built.
struct ProjItem<'a>(&'a ProjectionItemV1);

enum ItemClass {
    Key,
    Aggregate,
}

fn classify_item(
    item: &ProjectionItemV1,
    options: &NormalizeOptionsV1,
) -> Result<ItemClass, AlgebraDiagnosticV1> {
    if is_aggregate_call(&item.value, options) {
        Ok(ItemClass::Aggregate)
    } else if contains_aggregate(&item.value, options) {
        // An aggregate nested inside a larger expression cannot be split into a
        // clean Group/Aggregate boundary.
        Err(AlgebraDiagnosticV1::IllFormedAggregate {
            reason: IllFormedAggregateReasonV1::NestedAggregate,
            span: item.value.span,
        })
    } else {
        Ok(ItemClass::Key)
    }
}

fn is_aggregate_call(value: &ValueExprHirV1, options: &NormalizeOptionsV1) -> bool {
    matches!(&value.kind, ValueExprKindV1::Call(call) if options.is_aggregate(&call.callee.name))
}

fn contains_aggregate(value: &ValueExprHirV1, options: &NormalizeOptionsV1) -> bool {
    match &value.kind {
        ValueExprKindV1::Literal(_) | ValueExprKindV1::Symbol(_) => false,
        ValueExprKindV1::Path(path) => contains_aggregate(&path.base, options),
        ValueExprKindV1::Unary(u) => contains_aggregate(&u.operand, options),
        ValueExprKindV1::Binary(b) => {
            contains_aggregate(&b.lhs, options) || contains_aggregate(&b.rhs, options)
        }
        ValueExprKindV1::Call(call) => {
            options.is_aggregate(&call.callee.name)
                || call.args.iter().any(|a| contains_aggregate(a, options))
        }
        ValueExprKindV1::Paren(inner) => contains_aggregate(inner, options),
        ValueExprKindV1::Record(record) => record
            .fields
            .iter()
            .any(|f| contains_aggregate(&f.value, options)),
        ValueExprKindV1::List(list) => list.elements.iter().any(|e| contains_aggregate(e, options)),
    }
}

fn aggregate_expr(
    item: &ProjectionItemV1,
    options: &NormalizeOptionsV1,
) -> Result<AggregateExprV1, AlgebraDiagnosticV1> {
    let ValueExprKindV1::Call(call) = &item.value.kind else {
        return Err(AlgebraDiagnosticV1::IllFormedAggregate {
            reason: IllFormedAggregateReasonV1::NestedAggregate,
            span: item.value.span,
        });
    };
    // Detect a `distinct(x)` marker arg to make duplicate-elimination explicit.
    let mut distinct = false;
    let mut args: Vec<ValueExprHirV1> = Vec::new();
    for arg in &call.args {
        if let (Some(marker), ValueExprKindV1::Call(inner)) =
            (options.distinct_marker.as_deref(), &arg.kind)
        {
            if inner.callee.name == marker {
                distinct = true;
                args.extend(inner.args.iter().cloned());
                continue;
            }
        }
        args.push(arg.clone());
    }
    Ok(AggregateExprV1 {
        function: call.callee.name.clone(),
        origin: item.value.id,
        args,
        distinct,
    })
}

fn key_column(item: &ProjectionItemV1, ctx: &Ctx<'_>) -> ColumnV1 {
    let name = alias_or_value_name(item);
    let provenance = source_provenance(&item.value, ctx).unwrap_or(ColumnProvenanceV1::Derived {
        origin: item.value.id,
    });
    ColumnV1::new(name, provenance)
}

fn project_column(item: &ProjectionItemV1, ctx: &Ctx<'_>) -> ColumnV1 {
    // A projection item whose value is a path off an alias carries Source
    // provenance so alias-aware field resolution is preserved; everything else
    // is Derived.
    let name = alias_or_value_name(item);
    let provenance = source_provenance(&item.value, ctx).unwrap_or(ColumnProvenanceV1::Derived {
        origin: item.value.id,
    });
    ColumnV1::new(name, provenance)
}

/// If the value is `alias.field` (a path rooted at a bare symbol), produce
/// Source provenance preserving the alias and resolving the alias back to its
/// underlying RELATION through the alias scope (D3 alias-aware resolution), so
/// the column knows which source it came from rather than just its alias.
fn source_provenance(value: &ValueExprHirV1, ctx: &Ctx<'_>) -> Option<ColumnProvenanceV1> {
    if let ValueExprKindV1::Path(path) = &value.kind {
        if let ValueExprKindV1::Symbol(symbol) = &path.base.kind {
            let field = path.segments.last().map(|s| s.name.clone());
            let alias = ctx
                .aliases
                .contains_key(&symbol.name)
                .then(|| symbol.name.clone());
            return Some(ColumnProvenanceV1::Source {
                relation: ctx.relation_of(&symbol.name),
                alias,
                field,
            });
        }
    }
    None
}

fn alias_or_value_name(item: &ProjectionItemV1) -> String {
    if let Some(alias) = &item.alias {
        return alias.name.clone();
    }
    value_default_name(&item.value)
}

fn alias_or_default(item: &ProjectionItemV1, default: &str) -> String {
    item.alias
        .as_ref()
        .map_or_else(|| default.to_owned(), |a| a.name.clone())
}

fn value_default_name(value: &ValueExprHirV1) -> String {
    match &value.kind {
        ValueExprKindV1::Symbol(symbol) => symbol.name.clone(),
        ValueExprKindV1::Path(path) => path
            .segments
            .last()
            .map_or_else(|| "col".to_owned(), |s| s.name.clone()),
        ValueExprKindV1::Literal(_)
        | ValueExprKindV1::Unary(_)
        | ValueExprKindV1::Binary(_)
        | ValueExprKindV1::Call(_)
        | ValueExprKindV1::Paren(_)
        | ValueExprKindV1::Record(_)
        | ValueExprKindV1::List(_) => "col".to_owned(),
    }
}

/// For each OUTPUT column of `relation` (in output order), the outer symbol its
/// defining value references, if any. The result aligns positionally with the
/// normalized node's `columns`, so a renamed inner projection (`project { x: u }`)
/// is recognised as deriving from outer `u` even though its output name is `x`.
///
/// `referenced` is the set of outer names that are genuine outer references
/// (local source bindings already filtered out by the caller), so a value's
/// reference only counts when it is in that set.
fn output_column_outer_refs(
    relation: &RelationExprHirV1,
    referenced: &BTreeSet<String>,
) -> Vec<Option<String>> {
    match &relation.kind {
        // A scan binds its own row locally; its single output column never
        // references the outer side.
        RelationExprKindV1::Source(_) => vec![None],
        // Filter preserves the input schema and ordering exactly.
        RelationExprKindV1::Filter(filter) => output_column_outer_refs(&filter.input, referenced),
        // Each projection item is one output column, in order. The column derives
        // from an outer symbol when its value references one.
        RelationExprKindV1::Project(project) => project
            .items
            .iter()
            .map(|item| value_outer_ref(&item.value, referenced))
            .collect(),
        // A composition's output schema is derived structurally elsewhere; without
        // re-deriving it here, mark every output column as not-outer-referencing so
        // we never fabricate a Correlated label. Direct outer references inside a
        // nested compose still surface through the visible correlation columns.
        RelationExprKindV1::Compose(_) => Vec::new(),
    }
}

/// The outer symbol a value expression references, if exactly one outer-referenced
/// symbol appears in it. Returns the lexicographically-first match for stability.
fn value_outer_ref(value: &ValueExprHirV1, referenced: &BTreeSet<String>) -> Option<String> {
    let mut symbols = BTreeSet::new();
    collect_value_symbols(value, &mut symbols);
    symbols.into_iter().find(|name| referenced.contains(name))
}

/// Collect every bare symbol name referenced anywhere in a relation subtree.
/// Used to derive visible correlation columns for an Apply.
fn referenced_symbols(relation: &RelationExprHirV1) -> BTreeSet<String> {
    let mut acc = BTreeSet::new();
    collect_relation_symbols(relation, &mut acc);
    acc
}

fn collect_relation_symbols(relation: &RelationExprHirV1, acc: &mut BTreeSet<String>) {
    match &relation.kind {
        RelationExprKindV1::Source(_) => {}
        RelationExprKindV1::Project(project) => {
            collect_relation_symbols(&project.input, acc);
            for item in &project.items {
                collect_value_symbols(&item.value, acc);
            }
        }
        RelationExprKindV1::Filter(filter) => {
            collect_relation_symbols(&filter.input, acc);
            collect_bool_symbols(&filter.predicate, acc);
        }
        RelationExprKindV1::Compose(compose) => {
            for operand in &compose.operands {
                collect_relation_symbols(operand, acc);
            }
        }
    }
}

fn collect_bool_symbols(node: &quanta_gqlang_v2_hir::BoolExprHirV1, acc: &mut BTreeSet<String>) {
    use quanta_gqlang_v2_hir::BoolExprKindV1;
    match &node.kind {
        BoolExprKindV1::Literal(_) => {}
        BoolExprKindV1::Value(value) => collect_value_symbols(value, acc),
        BoolExprKindV1::Compare(compare) => {
            collect_value_symbols(&compare.lhs, acc);
            collect_value_symbols(&compare.rhs, acc);
        }
        BoolExprKindV1::And(bin) | BoolExprKindV1::Or(bin) => {
            collect_bool_symbols(&bin.lhs, acc);
            collect_bool_symbols(&bin.rhs, acc);
        }
        BoolExprKindV1::Not(inner) | BoolExprKindV1::Paren(inner) => {
            collect_bool_symbols(inner, acc);
        }
        BoolExprKindV1::Quantifier(quant) => {
            // The quantifier binds its own variable names; references to them
            // inside the body resolve to those LOCAL binders, not to any outer
            // column of the same name. Collect the body's symbols into a scratch
            // set, drop the binder names, and only then merge the genuinely free
            // symbols into `acc`. Folding the binders in directly would let a
            // coincidental name collision fabricate a spurious Apply correlation.
            let mut body_symbols = BTreeSet::new();
            collect_bool_symbols(&quant.body, &mut body_symbols);
            for binder in &quant.binders {
                let _ = body_symbols.remove(binder);
            }
            acc.extend(body_symbols);
        }
    }
}

fn collect_value_symbols(value: &ValueExprHirV1, acc: &mut BTreeSet<String>) {
    match &value.kind {
        ValueExprKindV1::Literal(_) => {}
        ValueExprKindV1::Symbol(symbol) => {
            let _ = acc.insert(symbol.name.clone());
        }
        ValueExprKindV1::Path(path) => collect_value_symbols(&path.base, acc),
        ValueExprKindV1::Unary(u) => collect_value_symbols(&u.operand, acc),
        ValueExprKindV1::Binary(b) => {
            collect_value_symbols(&b.lhs, acc);
            collect_value_symbols(&b.rhs, acc);
        }
        ValueExprKindV1::Call(call) => {
            for arg in &call.args {
                collect_value_symbols(arg, acc);
            }
        }
        ValueExprKindV1::Paren(inner) => collect_value_symbols(inner, acc),
        ValueExprKindV1::Record(record) => {
            for field in &record.fields {
                collect_value_symbols(&field.value, acc);
            }
        }
        ValueExprKindV1::List(list) => {
            for element in &list.elements {
                collect_value_symbols(element, acc);
            }
        }
    }
}

/// Build sort keys from an already-normalized input. Each output column becomes
/// an ascending key; the algebra carries explicit keys rather than relying on a
/// rendering layer to re-derive order (D24).
fn sort_keys_from(input: &AlgebraNodeV1) -> Vec<SortKeyV1> {
    input
        .columns
        .iter()
        .map(|col| SortKeyV1 {
            expr: ValueExprHirV1::new(
                input.origin,
                input.span,
                ValueExprKindV1::Symbol(quanta_gqlang_v2_hir::SymbolRefV1 {
                    name: col.name.clone(),
                }),
            ),
            direction: crate::node::SortDirectionV1::Ascending,
        })
        .collect()
}
