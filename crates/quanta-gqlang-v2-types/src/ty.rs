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
