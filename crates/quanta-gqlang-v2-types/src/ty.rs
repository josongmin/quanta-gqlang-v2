//! Checked, closed frontend type values.

use crate::diag::TypeDiagnosticV2;
use quanta_gqlang_v2_hir::LocatedSpanV2;
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ScalarTyV1 {
    Bool, Int, Float, String, Duration, Location, Path, RepoPath, EntityRef,
    SymbolRef, RuleRef, SourceScope, ProofTree,
}

impl ScalarTyV1 {
    #[must_use]
    pub const fn keyword(&self) -> &'static str {
        match self {
            Self::Bool => "bool", Self::Int => "int", Self::Float => "float",
            Self::String => "string", Self::Duration => "duration",
            Self::Location => "location", Self::Path => "path",
            Self::RepoPath => "repo_path", Self::EntityRef => "entity_ref",
            Self::SymbolRef => "symbol_ref", Self::RuleRef => "rule_ref",
            Self::SourceScope => "source_scope", Self::ProofTree => "proof_tree",
        }
    }
}

impl fmt::Display for ScalarTyV1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str(self.keyword()) }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CardinalityV1 { Set, Bag, Sequence }

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RecordFieldTypeV1 {
    name: String,
    ty: Box<TypeV1>,
    location: LocatedSpanV2,
}

impl RecordFieldTypeV1 {
    pub fn try_new(
        name: impl Into<String>, ty: TypeV1, location: LocatedSpanV2,
    ) -> Result<Self, TypeDiagnosticV2> {
        let name = name.into();
        if name.trim().is_empty() { return Err(TypeDiagnosticV2::InvalidTypeName { location }); }
        Ok(Self { name, ty: Box::new(ty), location })
    }
    #[must_use] pub fn name(&self) -> &str { &self.name }
    #[must_use] pub fn ty(&self) -> &TypeV1 { &self.ty }
    #[must_use] pub const fn location(&self) -> LocatedSpanV2 { self.location }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RecordTypeV1 { fields: Vec<RecordFieldTypeV1> }

impl RecordTypeV1 {
    pub fn try_new(fields: Vec<RecordFieldTypeV1>) -> Result<Self, TypeDiagnosticV2> {
        let mut seen = std::collections::BTreeMap::new();
        for field in &fields {
            if let Some(original) = seen.insert(field.name.clone(), field.location) {
                return Err(TypeDiagnosticV2::DuplicateRecordField {
                    name: field.name.clone(), primary: field.location, secondary: original,
                });
            }
        }
        Ok(Self { fields })
    }
    #[must_use] pub fn fields(&self) -> &[RecordFieldTypeV1] { &self.fields }
    #[must_use] pub fn field(&self, name: &str) -> Option<&RecordFieldTypeV1> {
        self.fields.iter().find(|field| field.name == name)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct UnionVariantTypeV1 {
    tag: String,
    ty: Box<TypeV1>,
    location: LocatedSpanV2,
}

impl UnionVariantTypeV1 {
    pub fn try_new(
        tag: impl Into<String>, ty: TypeV1, location: LocatedSpanV2,
    ) -> Result<Self, TypeDiagnosticV2> {
        let tag = tag.into();
        if tag.trim().is_empty() { return Err(TypeDiagnosticV2::InvalidTypeName { location }); }
        Ok(Self { tag, ty: Box::new(ty), location })
    }
    #[must_use] pub fn tag(&self) -> &str { &self.tag }
    #[must_use] pub fn ty(&self) -> &TypeV1 { &self.ty }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct UnionTypeV1 { variants: Vec<UnionVariantTypeV1> }

impl UnionTypeV1 {
    pub fn try_new(variants: Vec<UnionVariantTypeV1>) -> Result<Self, TypeDiagnosticV2> {
        let mut seen = std::collections::BTreeMap::new();
        for variant in &variants {
            if let Some(original) = seen.insert(variant.tag.clone(), variant.location) {
                return Err(TypeDiagnosticV2::DuplicateUnionTag {
                    tag: variant.tag.clone(), primary: variant.location, secondary: original,
                });
            }
        }
        Ok(Self { variants })
    }
    #[must_use] pub fn variants(&self) -> &[UnionVariantTypeV1] { &self.variants }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RelationTypeV1 { row: RecordTypeV1, cardinality: CardinalityV1 }

impl RelationTypeV1 {
    #[must_use] pub const fn new(row: RecordTypeV1, cardinality: CardinalityV1) -> Self {
        Self { row, cardinality }
    }
    #[must_use] pub const fn row(&self) -> &RecordTypeV1 { &self.row }
    #[must_use] pub const fn cardinality(&self) -> CardinalityV1 { self.cardinality }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TypeV1 {
    Scalar(ScalarTyV1), Null, Tuple(Vec<TypeV1>), Record(RecordTypeV1),
    Optional(Box<TypeV1>),
    Collection { element: Box<TypeV1>, cardinality: CardinalityV1 },
    Union(UnionTypeV1), Relation(RelationTypeV1),
}

impl TypeV1 {
    #[must_use] pub const fn scalar(scalar: ScalarTyV1) -> Self { Self::Scalar(scalar) }
    #[must_use] pub const fn bool() -> Self { Self::Scalar(ScalarTyV1::Bool) }
    #[must_use] pub const fn int() -> Self { Self::Scalar(ScalarTyV1::Int) }
    #[must_use] pub const fn float() -> Self { Self::Scalar(ScalarTyV1::Float) }
    #[must_use] pub const fn string() -> Self { Self::Scalar(ScalarTyV1::String) }
    #[must_use] pub fn optional(inner: Self) -> Self {
        if matches!(inner, Self::Optional(_)) { inner } else { Self::Optional(Box::new(inner)) }
    }
    #[must_use] pub fn collection(element: Self, cardinality: CardinalityV1) -> Self {
        Self::Collection { element: Box::new(element), cardinality }
    }
    #[must_use] pub fn list(element: Self) -> Self { Self::collection(element, CardinalityV1::Sequence) }
    #[must_use] pub fn set(element: Self) -> Self { Self::collection(element, CardinalityV1::Set) }
    #[must_use] pub const fn record(record: RecordTypeV1) -> Self { Self::Record(record) }
    #[must_use] pub const fn relation(relation: RelationTypeV1) -> Self { Self::Relation(relation) }
    #[must_use] pub fn is_assignable_to(&self, expected: &Self) -> bool {
        if self == expected { return true; }
        if let Self::Optional(inner) = expected {
            return matches!(self, Self::Null) || self.is_assignable_to(inner);
        }
        match (self, expected) {
            (Self::Collection { element: source, cardinality: source_cardinality },
             Self::Collection { element: target, cardinality: target_cardinality }) => {
                source_cardinality == target_cardinality
                    && (matches!(source.as_ref(), Self::Null) || source.is_assignable_to(target))
            }
            _ => false,
        }
    }
}

impl fmt::Display for TypeV1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Scalar(scalar) => write!(f, "{scalar}"), Self::Null => f.write_str("null"),
            Self::Tuple(items) => write!(f, "tuple<{items:?}>"),
            Self::Record(record) => {
                f.write_str("{ ")?;
                for (index, field) in record.fields.iter().enumerate() {
                    if index > 0 { f.write_str(", ")?; }
                    write!(f, "{}: {}", field.name, field.ty)?;
                }
                f.write_str(" }")
            }
            Self::Optional(inner) => write!(f, "optional<{inner}>"),
            Self::Collection { element, cardinality } => {
                let name = match cardinality { CardinalityV1::Set => "set", CardinalityV1::Bag => "bag", CardinalityV1::Sequence => "list" };
                write!(f, "{name}<{element}>")
            }
            Self::Union(union) => write!(f, "union<{:?}>", union.variants),
            Self::Relation(relation) => write!(f, "relation<{}>", Self::Record(relation.row.clone())),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct FiniteF64V2(u64);

impl FiniteF64V2 {
    pub fn try_new(value: f64, location: LocatedSpanV2) -> Result<Self, TypeDiagnosticV2> {
        if !value.is_finite() { return Err(TypeDiagnosticV2::FloatNonFinite { location }); }
        Ok(Self(if value == 0.0 { 0.0f64.to_bits() } else { value.to_bits() }))
    }
    #[must_use] pub fn get(self) -> f64 { f64::from_bits(self.0) }
}
impl PartialEq for FiniteF64V2 { fn eq(&self, other: &Self) -> bool { self.0 == other.0 } }
impl Eq for FiniteF64V2 {}
impl Hash for FiniteF64V2 { fn hash<H: Hasher>(&self, state: &mut H) { self.0.hash(state); } }
impl PartialOrd for FiniteF64V2 { fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) } }
impl Ord for FiniteF64V2 { fn cmp(&self, other: &Self) -> Ordering { self.get().total_cmp(&other.get()) } }

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TypedLiteralValueV2 {
    Int(i128), Float(FiniteF64V2), Bool(bool), String(String), Null,
}
