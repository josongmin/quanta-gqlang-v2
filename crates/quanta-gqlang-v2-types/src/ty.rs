//! The closed `v3` type universe (ticket 03, decisions `D4`/`D20`/`D29`).
//!
//! This module owns the single, closed, backend-independent set of types the
//! frontend assigns to value expressions. Per `D4` type checking uses a closed
//! registry of types and operators; per `D20` result kinds, source locations,
//! witness/path payloads, and explainability are *semantic* types here, not
//! CLI-only presentation. No backend/provider state participates in any lookup.
//!
//! The universe has three layers:
//!
//! 1. [`ScalarTyV1`] — the closed scalar leaf types (incl. semantic
//!    reference/domain types and the path/witness/proof value family).
//! 2. [`TypeV1`] — the recursive structural layer: tuples, records, optionals,
//!    collections, unions, and the relation/set row family with cardinality.
//! 3. nullability/optionality rules carried by [`TypeV1::Optional`] plus the
//!    [`TypeV1::Null`] bottom-of-absence type.
//!
//! Types are value objects: `Clone + PartialEq + Eq`, structurally compared.

use std::fmt;

/// A closed scalar leaf type.
///
/// "Closed" means this enum is exhaustive: there is no open string tag and no
/// backend hook. Every scalar a `v3` expression can hold appears here.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ScalarTyV1 {
    /// Boolean truth value.
    Bool,
    /// Signed integer.
    Int,
    /// Floating / decimal number.
    Float,
    /// Text string.
    String,
    /// A time span / duration quantity.
    Duration,
    /// A source location (file + range) value.
    Location,
    /// A path value (an ordered route, e.g. a file path or a graph path head).
    Path,
    /// A canonical repository-relative source path.
    RepoPath,
    /// A reference to a domain entity.
    EntityRef,
    /// A reference to a bound symbol/name in the program.
    SymbolRef,
    /// A reference to a declared rule.
    RuleRef,
    /// A source-scope handle (a bounded collection of sources/inputs).
    SourceScope,
    /// A proof / explanation tree value.
    ProofTree,
}

impl ScalarTyV1 {
    /// Stable lowercase keyword for this scalar, used in diagnostics and the
    /// public surface so CLI/LSP/tests share one spelling.
    #[must_use]
    pub const fn keyword(&self) -> &'static str {
        match self {
            Self::Bool => "bool",
            Self::Int => "int",
            Self::Float => "float",
            Self::String => "string",
            Self::Duration => "duration",
            Self::Location => "location",
            Self::Path => "path",
            Self::RepoPath => "repo_path",
            Self::EntityRef => "entity_ref",
            Self::SymbolRef => "symbol_ref",
            Self::RuleRef => "rule_ref",
            Self::SourceScope => "source_scope",
            Self::ProofTree => "proof_tree",
        }
    }

    /// Whether this scalar is one of the numeric tower (`int`, `float`).
    #[must_use]
    pub const fn is_numeric(&self) -> bool {
        matches!(self, Self::Int | Self::Float)
    }

    /// Whether this scalar belongs to the path/witness/proof value family.
    #[must_use]
    pub const fn is_proof_family(&self) -> bool {
        matches!(self, Self::Path | Self::Location | Self::ProofTree)
    }
}

impl fmt::Display for ScalarTyV1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.keyword())
    }
}

/// Multiplicity/cardinality of a relation or set type.
///
/// Per `D26` multiplicity and duplicate-elimination must be explicit and
/// testable; this enum is the frontend-owned carrier for that distinction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CardinalityV1 {
    /// A set: duplicate rows are eliminated; order is not semantic.
    Set,
    /// A bag/multiset: duplicates are preserved; order is not semantic.
    Bag,
    /// An ordered sequence: duplicates preserved and order is semantic.
    Sequence,
}

impl CardinalityV1 {
    /// Stable keyword naming the cardinality itself.
    #[must_use]
    pub const fn keyword(&self) -> &'static str {
        match self {
            Self::Set => "set",
            Self::Bag => "bag",
            Self::Sequence => "sequence",
        }
    }

    /// Surface collection keyword: a `Sequence` collection renders as `list`,
    /// a `Set` as `set`, a `Bag` as `bag`. This is the author-facing spelling.
    #[must_use]
    pub const fn collection_keyword(&self) -> &'static str {
        match self {
            Self::Set => "set",
            Self::Bag => "bag",
            Self::Sequence => "list",
        }
    }
}

/// A named field within a record type. Field order is preserved and significant
/// for display, but record *equality* is order-sensitive here so that two
/// records compare equal only when they list the same fields in the same order
/// (matching the order-preserving HIR record/shape nodes from ticket 01).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RecordFieldTyV1 {
    /// Field name.
    pub name: String,
    /// Field type.
    pub ty: TypeV1,
}

impl RecordFieldTyV1 {
    /// Construct a record field type.
    #[must_use]
    pub fn new(name: impl Into<String>, ty: TypeV1) -> Self {
        Self {
            name: name.into(),
            ty,
        }
    }
}

/// A named variant within an algebraic-datatype/union type.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct UnionVariantTyV1 {
    /// Variant tag.
    pub tag: String,
    /// Payload type carried by this variant.
    pub payload: TypeV1,
}

impl UnionVariantTyV1 {
    /// Construct a union variant.
    #[must_use]
    pub fn new(tag: impl Into<String>, payload: TypeV1) -> Self {
        Self {
            tag: tag.into(),
            payload,
        }
    }
}

/// A `v3` type.
///
/// The closed structural layer over [`ScalarTyV1`]. Recursive variants own their
/// children so a `TypeV1` is a self-contained value tree.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TypeV1 {
    /// A scalar leaf type.
    Scalar(ScalarTyV1),
    /// The type of the `null`/absent literal. Assignable into any
    /// [`TypeV1::Optional`]; distinct from a present value of any scalar type.
    Null,
    /// A positional tuple (fixed arity, heterogeneous element types).
    Tuple(Vec<TypeV1>),
    /// A named record (a tagged tuple). Field order is preserved.
    Record(Vec<RecordFieldTyV1>),
    /// An optional/nullable type: a present `T` or absent (`null`).
    Optional(Box<TypeV1>),
    /// A homogeneous collection with explicit cardinality semantics.
    Collection {
        /// Element type.
        element: Box<TypeV1>,
        /// Multiplicity / duplicate / order semantics.
        cardinality: CardinalityV1,
    },
    /// A tagged union / algebraic datatype. Variant order is preserved but two
    /// unions are equal only when they carry the same variants in the same
    /// order.
    Union(Vec<UnionVariantTyV1>),
    /// A relation: rows of a record shape with explicit cardinality. The row
    /// type must be a [`TypeV1::Record`]; constructing helpers enforce that.
    Relation {
        /// Per-row record type.
        row: Box<TypeV1>,
        /// Multiplicity / duplicate / order semantics of the row collection.
        cardinality: CardinalityV1,
    },
}

impl TypeV1 {
    /// Convenience constructor for a scalar type.
    #[must_use]
    pub const fn scalar(scalar: ScalarTyV1) -> Self {
        Self::Scalar(scalar)
    }

    /// `bool` scalar.
    #[must_use]
    pub const fn bool() -> Self {
        Self::Scalar(ScalarTyV1::Bool)
    }

    /// `int` scalar.
    #[must_use]
    pub const fn int() -> Self {
        Self::Scalar(ScalarTyV1::Int)
    }

    /// `float` scalar.
    #[must_use]
    pub const fn float() -> Self {
        Self::Scalar(ScalarTyV1::Float)
    }

    /// `string` scalar.
    #[must_use]
    pub const fn string() -> Self {
        Self::Scalar(ScalarTyV1::String)
    }

    /// Wrap a type as optional. Wrapping an already-optional type is idempotent
    /// so `optional(optional(T)) == optional(T)`.
    #[must_use]
    pub fn optional(inner: TypeV1) -> Self {
        if inner.is_optional() {
            inner
        } else {
            Self::Optional(Box::new(inner))
        }
    }

    /// A homogeneous collection of `element` with the given cardinality.
    #[must_use]
    pub fn collection(element: TypeV1, cardinality: CardinalityV1) -> Self {
        Self::Collection {
            element: Box::new(element),
            cardinality,
        }
    }

    /// A `list<T>` — a sequence collection (ordered, duplicates preserved).
    #[must_use]
    pub fn list(element: TypeV1) -> Self {
        Self::collection(element, CardinalityV1::Sequence)
    }

    /// A `set<T>` — a set collection (unordered, duplicate-eliminated).
    #[must_use]
    pub fn set(element: TypeV1) -> Self {
        Self::collection(element, CardinalityV1::Set)
    }

    /// A record from named fields.
    #[must_use]
    pub fn record(fields: Vec<RecordFieldTyV1>) -> Self {
        Self::Record(fields)
    }

    /// Whether this type is the absent/null type.
    #[must_use]
    pub const fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    /// Whether this type is optional/nullable.
    #[must_use]
    pub const fn is_optional(&self) -> bool {
        matches!(self, Self::Optional(_))
    }

    /// The underlying type with one optional layer removed, or the type itself
    /// when not optional.
    #[must_use]
    pub fn unwrap_optional(&self) -> &TypeV1 {
        if let Self::Optional(inner) = self {
            inner
        } else {
            self
        }
    }

    /// Whether a value of type `self` may be supplied where `expected` is
    /// required, under the closed nullability/optionality rules:
    ///
    /// * identical types are assignable;
    /// * `null` is assignable into any optional type;
    /// * a present `T` is assignable into `optional<T>` (and `optional<U>` for
    ///   any `U` that `T` is assignable to);
    /// * a `collection<S>` is assignable into a same-cardinality
    ///   `collection<E>` when `S` is assignable into `E`. Because the empty
    ///   list/collection literal types to `collection<null>` (its element type
    ///   is unknown), `null` as the *element* of a collection is assignable into
    ///   any element type — this is what keeps an empty `[]` assignable into any
    ///   concrete `list<T>` "via later context" (see `type_list`). Note this
    ///   element rule is element-only: a bare `null` value is still only
    ///   assignable into an optional, not into a concrete scalar;
    /// * `int` is NOT implicitly assignable to `float` — numeric widening is an
    ///   explicit operator/overload concern, not a silent subtype rule, so the
    ///   universe stays predictable for downstream planning.
    #[must_use]
    pub fn is_assignable_to(&self, expected: &TypeV1) -> bool {
        if self == expected {
            return true;
        }
        if let Self::Optional(inner) = expected {
            return self.is_null() || self.is_assignable_to(inner);
        }
        if let (
            Self::Collection {
                element: src,
                cardinality: src_card,
            },
            Self::Collection {
                element: dst,
                cardinality: dst_card,
            },
        ) = (self, expected)
        {
            // Same cardinality is required: a `list` (sequence) is not a `set`.
            return src_card == dst_card && (src.is_null() || src.is_assignable_to(dst));
        }
        false
    }
}

impl fmt::Display for TypeV1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Scalar(scalar) => write!(f, "{scalar}"),
            Self::Null => f.write_str("null"),
            Self::Tuple(elements) => {
                f.write_str("tuple<")?;
                write_comma_list(f, elements)?;
                f.write_str(">")
            }
            Self::Record(fields) => {
                f.write_str("{ ")?;
                for (idx, field) in fields.iter().enumerate() {
                    if idx > 0 {
                        f.write_str(", ")?;
                    }
                    write!(f, "{}: {}", field.name, field.ty)?;
                }
                f.write_str(" }")
            }
            Self::Optional(inner) => write!(f, "optional<{inner}>"),
            Self::Collection {
                element,
                cardinality,
            } => write!(f, "{}<{element}>", cardinality.collection_keyword()),
            Self::Union(variants) => {
                f.write_str("union<")?;
                for (idx, variant) in variants.iter().enumerate() {
                    if idx > 0 {
                        f.write_str(" | ")?;
                    }
                    write!(f, "{}: {}", variant.tag, variant.payload)?;
                }
                f.write_str(">")
            }
            Self::Relation { row, cardinality } => {
                write!(f, "relation<{}, {row}>", cardinality.keyword())
            }
        }
    }
}

fn write_comma_list(f: &mut fmt::Formatter<'_>, items: &[TypeV1]) -> fmt::Result {
    for (idx, item) in items.iter().enumerate() {
        if idx > 0 {
            f.write_str(", ")?;
        }
        write!(f, "{item}")?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every scalar must map to a stable, unique lowercase keyword and round-trip
    /// through `Display`. This locks the public spelling shared by CLI/LSP/tests.
    #[test]
    fn scalar_keywords_are_stable_unique_and_display_match() {
        let all = [
            (ScalarTyV1::Bool, "bool"),
            (ScalarTyV1::Int, "int"),
            (ScalarTyV1::Float, "float"),
            (ScalarTyV1::String, "string"),
            (ScalarTyV1::Duration, "duration"),
            (ScalarTyV1::Location, "location"),
            (ScalarTyV1::Path, "path"),
            (ScalarTyV1::RepoPath, "repo_path"),
            (ScalarTyV1::EntityRef, "entity_ref"),
            (ScalarTyV1::SymbolRef, "symbol_ref"),
            (ScalarTyV1::RuleRef, "rule_ref"),
            (ScalarTyV1::SourceScope, "source_scope"),
            (ScalarTyV1::ProofTree, "proof_tree"),
        ];
        let mut seen = std::collections::BTreeSet::new();
        for (scalar, keyword) in &all {
            assert_eq!(scalar.keyword(), *keyword);
            assert_eq!(format!("{scalar}"), *keyword);
            assert!(seen.insert(*keyword), "duplicate keyword {keyword}");
        }
    }

    /// Only `int` and `float` are numeric; references/proof-family scalars are not.
    #[test]
    fn is_numeric_only_for_int_and_float() {
        assert!(ScalarTyV1::Int.is_numeric());
        assert!(ScalarTyV1::Float.is_numeric());
        assert!(!ScalarTyV1::Bool.is_numeric());
        assert!(!ScalarTyV1::String.is_numeric());
        assert!(!ScalarTyV1::Duration.is_numeric());
        assert!(!ScalarTyV1::Path.is_numeric());
        assert!(!ScalarTyV1::RepoPath.is_numeric());
    }

    /// The proof/witness family is exactly `path`, `location`, `proof_tree`.
    #[test]
    fn is_proof_family_membership() {
        assert!(ScalarTyV1::Path.is_proof_family());
        assert!(!ScalarTyV1::RepoPath.is_proof_family());
        assert!(ScalarTyV1::Location.is_proof_family());
        assert!(ScalarTyV1::ProofTree.is_proof_family());
        assert!(!ScalarTyV1::Int.is_proof_family());
        assert!(!ScalarTyV1::EntityRef.is_proof_family());
        assert!(!ScalarTyV1::Duration.is_proof_family());
    }

    #[test]
    fn repo_path_is_distinct_from_broad_path_and_only_self_assignable_v1() {
        let broad_v1 = TypeV1::scalar(ScalarTyV1::Path);
        let repo_v1 = TypeV1::scalar(ScalarTyV1::RepoPath);
        assert_ne!(broad_v1, repo_v1);
        assert!(!broad_v1.is_assignable_to(&repo_v1));
        assert!(!repo_v1.is_assignable_to(&broad_v1));
        assert!(repo_v1.is_assignable_to(&repo_v1));
    }

    /// Cardinality has two distinct keyword spellings: the cardinality name and
    /// the author-facing collection keyword (`Sequence` renders as `list`).
    #[test]
    fn cardinality_keywords_distinguish_name_from_collection_spelling() {
        assert_eq!(CardinalityV1::Set.keyword(), "set");
        assert_eq!(CardinalityV1::Bag.keyword(), "bag");
        assert_eq!(CardinalityV1::Sequence.keyword(), "sequence");

        assert_eq!(CardinalityV1::Set.collection_keyword(), "set");
        assert_eq!(CardinalityV1::Bag.collection_keyword(), "bag");
        assert_eq!(CardinalityV1::Sequence.collection_keyword(), "list");
    }

    /// Constructors build the documented values.
    #[test]
    fn union_variant_and_record_field_constructors() {
        let variant = UnionVariantTyV1::new("ok", TypeV1::int());
        assert_eq!(variant.tag, "ok");
        assert_eq!(variant.payload, TypeV1::int());

        let field = RecordFieldTyV1::new("id", TypeV1::string());
        assert_eq!(field.name, "id");
        assert_eq!(field.ty, TypeV1::string());
    }

    /// Wrapping an already-optional type is idempotent (documented invariant):
    /// `optional(optional(T)) == optional(T)`.
    #[test]
    fn optional_wrapping_is_idempotent() {
        let once = TypeV1::optional(TypeV1::int());
        let twice = TypeV1::optional(once.clone());
        assert_eq!(once, twice);
        assert_eq!(twice, TypeV1::Optional(Box::new(TypeV1::int())));
        assert!(once.is_optional());
    }

    /// `unwrap_optional` strips exactly one optional layer and is a no-op on
    /// non-optional types.
    #[test]
    fn unwrap_optional_strips_one_layer() {
        let opt = TypeV1::optional(TypeV1::string());
        assert_eq!(opt.unwrap_optional(), &TypeV1::string());
        // Non-optional types are returned as-is.
        let plain = TypeV1::int();
        assert_eq!(plain.unwrap_optional(), &TypeV1::int());
        assert_eq!(TypeV1::Null.unwrap_optional(), &TypeV1::Null);
    }

    /// `is_null` is true only for the `Null` type.
    #[test]
    fn is_null_only_for_null() {
        assert!(TypeV1::Null.is_null());
        assert!(!TypeV1::int().is_null());
        assert!(!TypeV1::optional(TypeV1::int()).is_null());
    }

    /// The set helper builds a set-cardinality collection; list builds a sequence.
    #[test]
    fn set_and_list_helpers_carry_cardinality() {
        let s = TypeV1::set(TypeV1::int());
        assert_eq!(
            s,
            TypeV1::Collection {
                element: Box::new(TypeV1::int()),
                cardinality: CardinalityV1::Set,
            }
        );
        let l = TypeV1::list(TypeV1::int());
        assert_eq!(
            l,
            TypeV1::Collection {
                element: Box::new(TypeV1::int()),
                cardinality: CardinalityV1::Sequence,
            }
        );
        // A set is never assignable into a list (cardinality differs).
        assert!(!s.is_assignable_to(&l));
        assert!(!l.is_assignable_to(&s));
    }

    /// `Display` for every recursive structural variant must match the documented
    /// rendering grammar exactly.
    #[test]
    fn display_renders_every_structural_variant() {
        // Tuple.
        let tuple = TypeV1::Tuple(vec![TypeV1::int(), TypeV1::string(), TypeV1::Null]);
        assert_eq!(format!("{tuple}"), "tuple<int, string, null>");

        // Record (order preserved).
        let record = TypeV1::record(vec![
            RecordFieldTyV1::new("id", TypeV1::int()),
            RecordFieldTyV1::new("name", TypeV1::string()),
        ]);
        assert_eq!(format!("{record}"), "{ id: int, name: string }");

        // Optional.
        assert_eq!(
            format!("{}", TypeV1::optional(TypeV1::int())),
            "optional<int>"
        );

        // Collections render by their author-facing keyword.
        assert_eq!(format!("{}", TypeV1::list(TypeV1::int())), "list<int>");
        assert_eq!(format!("{}", TypeV1::set(TypeV1::string())), "set<string>");
        assert_eq!(
            format!("{}", TypeV1::collection(TypeV1::int(), CardinalityV1::Bag)),
            "bag<int>"
        );

        // Union (variant order preserved, ` | ` separated).
        let union = TypeV1::Union(vec![
            UnionVariantTyV1::new("ok", TypeV1::int()),
            UnionVariantTyV1::new("err", TypeV1::string()),
        ]);
        assert_eq!(format!("{union}"), "union<ok: int | err: string>");

        // Relation renders cardinality name then row.
        let relation = TypeV1::Relation {
            row: Box::new(TypeV1::record(vec![RecordFieldTyV1::new(
                "x",
                TypeV1::int(),
            )])),
            cardinality: CardinalityV1::Set,
        };
        assert_eq!(format!("{relation}"), "relation<set, { x: int }>");

        // Empty tuple and empty record degenerate cases.
        assert_eq!(format!("{}", TypeV1::Tuple(vec![])), "tuple<>");
        assert_eq!(format!("{}", TypeV1::record(vec![])), "{  }");
    }

    /// Record equality is order-sensitive (documented): the same fields in a
    /// different order are NOT equal.
    #[test]
    fn record_equality_is_order_sensitive() {
        let a = TypeV1::record(vec![
            RecordFieldTyV1::new("a", TypeV1::int()),
            RecordFieldTyV1::new("b", TypeV1::string()),
        ]);
        let b = TypeV1::record(vec![
            RecordFieldTyV1::new("b", TypeV1::string()),
            RecordFieldTyV1::new("a", TypeV1::int()),
        ]);
        assert_ne!(a, b);
    }

    /// A present `T` is assignable into `optional<U>` whenever `T` is assignable
    /// into `U`, transitively through a collection element.
    #[test]
    fn assignability_through_optional_and_collection_elements() {
        // present int into optional<int>
        assert!(TypeV1::int().is_assignable_to(&TypeV1::optional(TypeV1::int())));
        // list<int> into list<optional<int>> via element rule
        assert!(TypeV1::list(TypeV1::int())
            .is_assignable_to(&TypeV1::list(TypeV1::optional(TypeV1::int()))));
        // bare null NOT assignable into a concrete scalar
        assert!(!TypeV1::Null.is_assignable_to(&TypeV1::int()));
        // int NOT silently widened to float
        assert!(!TypeV1::int().is_assignable_to(&TypeV1::float()));
        // unrelated structural types are not assignable
        assert!(!TypeV1::int().is_assignable_to(&TypeV1::string()));
        assert!(!TypeV1::Tuple(vec![TypeV1::int()]).is_assignable_to(&TypeV1::int()));
    }
}
