//! Checked, closed frontend type values.

use crate::diag::TypeDiagnosticV2;
use quanta_gqlang_v2_hir::{CheckedLocatedSpanV2, LocatedSpanV2};
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

pub const MAX_TYPE_DEPTH: usize = 64;
pub const MAX_TYPE_NODES: usize = 4096;
pub const MAX_TYPE_NAME_BYTES: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct TypeShapeFactsV1 {
    depth: usize,
    nodes: usize,
    contains_relation: bool,
}

impl TypeShapeFactsV1 {
    const LEAF: Self = Self {
        depth: 1,
        nodes: 1,
        contains_relation: false,
    };

    fn try_parent<'a>(
        children: impl IntoIterator<Item = &'a TypeShapeFactsV1>,
        contains_relation: bool,
    ) -> Result<Self, TypeDiagnosticV2> {
        let mut shape = Self {
            contains_relation,
            ..Self::LEAF
        };
        for child in children {
            shape.try_push_child(child)?;
        }
        Ok(shape)
    }

    fn try_push_child(&mut self, child: &Self) -> Result<(), TypeDiagnosticV2> {
        let depth = self.depth.max(
            child
                .depth
                .checked_add(1)
                .ok_or(TypeDiagnosticV2::TypeShapeLimit { location: None })?,
        );
        let nodes = self
            .nodes
            .checked_add(child.nodes)
            .ok_or(TypeDiagnosticV2::TypeShapeLimit { location: None })?;
        if depth > MAX_TYPE_DEPTH || nodes > MAX_TYPE_NODES {
            return Err(TypeDiagnosticV2::TypeShapeLimit { location: None });
        }
        self.depth = depth;
        self.nodes = nodes;
        self.contains_relation |= child.contains_relation;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ScalarTyV1 {
    Bool,
    Int,
    Float,
    String,
    Duration,
    Location,
    Path,
    RepoPath,
    EntityRef,
    SymbolRef,
    RuleRef,
    SourceScope,
    ProofTree,
}

impl ScalarTyV1 {
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
}

impl fmt::Display for ScalarTyV1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.keyword())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CardinalityV1 {
    Set,
    Bag,
    Sequence,
}

#[derive(Clone, Debug)]
pub struct RecordFieldTypeV1 {
    name: String,
    value_type: ValueTypeV1,
    location: CheckedLocatedSpanV2,
}

impl RecordFieldTypeV1 {
    pub fn try_new(
        name: &str,
        value_type: ValueTypeV1,
        location: CheckedLocatedSpanV2,
    ) -> Result<Self, TypeDiagnosticV2> {
        if !canonical_type_name(name) {
            return Err(TypeDiagnosticV2::InvalidTypeName {
                location: location.location(),
            });
        }
        Ok(Self {
            name: name.to_owned(),
            value_type,
            location,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RecordFieldV1 {
    name: String,
    value_type: ValueTypeV1,
}
impl RecordFieldV1 {
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
    #[must_use]
    pub const fn value_type(&self) -> &ValueTypeV1 {
        &self.value_type
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RecordTypeV1 {
    fields: Arc<[RecordFieldV1]>,
    shape: TypeShapeFactsV1,
}

impl RecordTypeV1 {
    pub fn try_new(fields: Vec<RecordFieldTypeV1>) -> Result<Self, TypeDiagnosticV2> {
        let shape = TypeShapeFactsV1::try_parent(
            fields.iter().map(|field| &field.value_type.0.shape),
            false,
        )?;
        let mut seen = std::collections::BTreeMap::new();
        for field in &fields {
            if let Some(original) = seen.insert(field.name.clone(), field.location.location()) {
                return Err(TypeDiagnosticV2::DuplicateRecordField {
                    name: field.name.clone(),
                    primary: field.location.location(),
                    secondary: original,
                });
            }
        }
        Ok(Self {
            fields: fields
                .into_iter()
                .map(|field| RecordFieldV1 {
                    name: field.name,
                    value_type: field.value_type,
                })
                .collect::<Vec<_>>()
                .into(),
            shape,
        })
    }
    #[must_use]
    pub fn fields(&self) -> &[RecordFieldV1] {
        &self.fields
    }
    #[must_use]
    pub fn field(&self, name: &str) -> Option<&RecordFieldV1> {
        self.fields.iter().find(|field| field.name == name)
    }
}

/// Builds an internal record from borrowed child types. Shape and name checks
/// happen before an admitted child is cloned into an owned field.
pub(crate) struct RecordTypeBuilderV1 {
    fields: Vec<RecordFieldTypeV1>,
    shape: TypeShapeFactsV1,
    seen: BTreeMap<String, LocatedSpanV2>,
    duplicate: Option<TypeDiagnosticV2>,
    shape_limit: Option<TypeDiagnosticV2>,
}

impl RecordTypeBuilderV1 {
    pub(crate) fn new() -> Self {
        Self {
            fields: Vec::new(),
            shape: TypeShapeFactsV1::LEAF,
            seen: BTreeMap::new(),
            duplicate: None,
            shape_limit: None,
        }
    }

    pub(crate) fn try_push(
        &mut self,
        name: &str,
        value_type: &ValueTypeV1,
        location: CheckedLocatedSpanV2,
    ) -> Result<(), TypeDiagnosticV2> {
        if !canonical_type_name(name) {
            return Err(TypeDiagnosticV2::InvalidTypeName {
                location: location.location(),
            });
        }
        if self.shape_limit.is_some() {
            return Ok(());
        }
        if let Err(error) = self.shape.try_push_child(&value_type.0.shape) {
            self.shape_limit = Some(error.at_location(location.location()));
            return Ok(());
        }
        if let Some(original) = self.seen.get(name) {
            if self.duplicate.is_none() {
                self.duplicate = Some(TypeDiagnosticV2::DuplicateRecordField {
                    name: name.to_owned(),
                    primary: location.location(),
                    secondary: *original,
                });
            }
        } else if self.duplicate.is_none() {
            self.seen.insert(name.to_owned(), location.location());
        }
        if self.duplicate.is_none() {
            self.fields.push(RecordFieldTypeV1::try_new(
                name,
                value_type.clone(),
                location,
            )?);
        }
        Ok(())
    }

    pub(crate) fn shape_limit(&self) -> Option<&TypeDiagnosticV2> {
        self.shape_limit.as_ref()
    }

    pub(crate) fn valid_name(name: &str) -> bool {
        canonical_type_name(name)
    }

    pub(crate) fn finish(self) -> Result<RecordTypeV1, TypeDiagnosticV2> {
        if let Some(shape_limit) = self.shape_limit {
            return Err(shape_limit);
        }
        if let Some(duplicate) = self.duplicate {
            return Err(duplicate);
        }
        Ok(RecordTypeV1 {
            fields: self
                .fields
                .into_iter()
                .map(|field| RecordFieldV1 {
                    name: field.name,
                    value_type: field.value_type,
                })
                .collect::<Vec<_>>()
                .into(),
            shape: self.shape,
        })
    }
}

#[derive(Clone, Debug)]
pub struct UnionVariantTypeV1 {
    tag: String,
    value_type: ValueTypeV1,
    location: CheckedLocatedSpanV2,
}

impl UnionVariantTypeV1 {
    pub fn try_new(
        tag: &str,
        value_type: ValueTypeV1,
        location: CheckedLocatedSpanV2,
    ) -> Result<Self, TypeDiagnosticV2> {
        if !canonical_type_name(tag) {
            return Err(TypeDiagnosticV2::InvalidTypeName {
                location: location.location(),
            });
        }
        Ok(Self {
            tag: tag.to_owned(),
            value_type,
            location,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct UnionVariantV1 {
    tag: String,
    value_type: ValueTypeV1,
}
impl UnionVariantV1 {
    #[must_use]
    pub fn tag(&self) -> &str {
        &self.tag
    }
    #[must_use]
    pub const fn value_type(&self) -> &ValueTypeV1 {
        &self.value_type
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct UnionTypeV1 {
    variants: Arc<[UnionVariantV1]>,
    shape: TypeShapeFactsV1,
}

impl UnionTypeV1 {
    pub fn try_new(variants: Vec<UnionVariantTypeV1>) -> Result<Self, TypeDiagnosticV2> {
        let shape = TypeShapeFactsV1::try_parent(
            variants.iter().map(|variant| &variant.value_type.0.shape),
            false,
        )?;
        let mut seen = std::collections::BTreeMap::new();
        for variant in &variants {
            if let Some(original) = seen.insert(variant.tag.clone(), variant.location.location()) {
                return Err(TypeDiagnosticV2::DuplicateUnionTag {
                    tag: variant.tag.clone(),
                    primary: variant.location.location(),
                    secondary: original,
                });
            }
        }
        Ok(Self {
            variants: variants
                .into_iter()
                .map(|variant| UnionVariantV1 {
                    tag: variant.tag,
                    value_type: variant.value_type,
                })
                .collect::<Vec<_>>()
                .into(),
            shape,
        })
    }
    #[must_use]
    pub fn variants(&self) -> &[UnionVariantV1] {
        &self.variants
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TupleTypeV1 {
    items: Arc<[ValueTypeV1]>,
    shape: TypeShapeFactsV1,
}
impl TupleTypeV1 {
    pub fn try_new(items: Vec<ValueTypeV1>) -> Result<Self, TypeDiagnosticV2> {
        let shape = TypeShapeFactsV1::try_parent(items.iter().map(|item| &item.0.shape), false)?;
        Ok(Self {
            items: items.into(),
            shape,
        })
    }
    #[must_use]
    pub fn items(&self) -> &[ValueTypeV1] {
        &self.items
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct OptionalTypeV1 {
    inner: Box<ValueTypeV1>,
    shape: TypeShapeFactsV1,
}
impl OptionalTypeV1 {
    pub fn try_new(inner: ValueTypeV1) -> Result<Self, TypeDiagnosticV2> {
        if inner.0.as_optional().is_some() {
            return Err(TypeDiagnosticV2::RedundantOptional);
        }
        let shape = TypeShapeFactsV1::try_parent([&inner.0.shape], false)?;
        Ok(Self {
            inner: Box::new(inner),
            shape,
        })
    }
    #[must_use]
    pub const fn inner(&self) -> &ValueTypeV1 {
        &self.inner
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CollectionTypeV1 {
    element: Box<ValueTypeV1>,
    cardinality: CardinalityV1,
    shape: TypeShapeFactsV1,
}
impl CollectionTypeV1 {
    pub fn try_new(
        element: ValueTypeV1,
        cardinality: CardinalityV1,
    ) -> Result<Self, TypeDiagnosticV2> {
        let shape = TypeShapeFactsV1::try_parent([&element.0.shape], false)?;
        Ok(Self {
            element: Box::new(element),
            cardinality,
            shape,
        })
    }
    #[must_use]
    pub const fn element(&self) -> &ValueTypeV1 {
        &self.element
    }
    #[must_use]
    pub const fn cardinality(&self) -> CardinalityV1 {
        self.cardinality
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RelationTypeV1 {
    row: RecordTypeV1,
    cardinality: CardinalityV1,
    shape: TypeShapeFactsV1,
}

impl RelationTypeV1 {
    pub fn try_new(
        row: RecordTypeV1,
        cardinality: CardinalityV1,
    ) -> Result<Self, TypeDiagnosticV2> {
        let shape = TypeShapeFactsV1::try_parent([&row.shape], true)?;
        Ok(Self {
            row,
            cardinality,
            shape,
        })
    }
    #[must_use]
    pub const fn row(&self) -> &RecordTypeV1 {
        &self.row
    }
    #[must_use]
    pub const fn cardinality(&self) -> CardinalityV1 {
        self.cardinality
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum TypeKindV1 {
    Scalar(ScalarTyV1),
    Null,
    Tuple(TupleTypeV1),
    Record(RecordTypeV1),
    Optional(OptionalTypeV1),
    Collection(CollectionTypeV1),
    Union(UnionTypeV1),
    Relation(RelationTypeV1),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TypeV1 {
    kind: TypeKindV1,
    shape: TypeShapeFactsV1,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ValueTypeV1(TypeV1);

impl ValueTypeV1 {
    pub fn try_from_type(ty: TypeV1) -> Result<Self, TypeDiagnosticV2> {
        if ty.shape.contains_relation {
            return Err(TypeDiagnosticV2::RelationUsedAsValue);
        }
        Ok(Self(ty))
    }
    #[must_use]
    pub const fn as_type(&self) -> &TypeV1 {
        &self.0
    }
    #[must_use]
    pub fn into_type(self) -> TypeV1 {
        self.0
    }
    #[must_use]
    pub const fn scalar(scalar: ScalarTyV1) -> Self {
        Self(TypeV1::scalar(scalar))
    }
    #[must_use]
    pub const fn bool() -> Self {
        Self(TypeV1::bool())
    }
    #[must_use]
    pub const fn int() -> Self {
        Self(TypeV1::int())
    }
    #[must_use]
    pub const fn float() -> Self {
        Self(TypeV1::float())
    }
    #[must_use]
    pub const fn string() -> Self {
        Self(TypeV1::string())
    }
    #[must_use]
    pub const fn null() -> Self {
        Self(TypeV1::null())
    }
    pub fn try_tuple(items: Vec<Self>) -> Result<Self, TypeDiagnosticV2> {
        Self::try_from_type(TypeV1::try_tuple(items)?)
    }
    pub fn try_optional(inner: Self) -> Result<Self, TypeDiagnosticV2> {
        Self::try_from_type(TypeV1::try_optional(inner)?)
    }
    pub fn try_collection(
        element: Self,
        cardinality: CardinalityV1,
    ) -> Result<Self, TypeDiagnosticV2> {
        Self::try_from_type(TypeV1::try_collection(element, cardinality)?)
    }
    pub fn try_list(element: Self) -> Result<Self, TypeDiagnosticV2> {
        Self::try_collection(element, CardinalityV1::Sequence)
    }
    pub fn try_record(record: RecordTypeV1) -> Result<Self, TypeDiagnosticV2> {
        Self::try_from_type(TypeV1::record(record))
    }
    #[must_use]
    pub fn is_assignable_to(&self, expected: &Self) -> bool {
        self.0.is_assignable_to(&expected.0)
    }
    #[must_use]
    pub const fn as_tuple(&self) -> Option<&TupleTypeV1> {
        self.0.as_tuple()
    }
    #[must_use]
    pub const fn as_record(&self) -> Option<&RecordTypeV1> {
        self.0.as_record()
    }
    #[must_use]
    pub const fn as_union(&self) -> Option<&UnionTypeV1> {
        self.0.as_union()
    }
    #[must_use]
    pub const fn as_optional(&self) -> Option<&OptionalTypeV1> {
        self.0.as_optional()
    }
    #[must_use]
    pub const fn as_collection(&self) -> Option<&CollectionTypeV1> {
        self.0.as_collection()
    }
    #[must_use]
    pub const fn is_null(&self) -> bool {
        self.0.is_null()
    }
    #[must_use]
    pub const fn depth(&self) -> usize {
        self.0.depth()
    }
    #[must_use]
    pub const fn node_count(&self) -> usize {
        self.0.node_count()
    }
}

impl fmt::Display for ValueTypeV1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl TypeV1 {
    #[must_use]
    pub const fn scalar(scalar: ScalarTyV1) -> Self {
        Self {
            kind: TypeKindV1::Scalar(scalar),
            shape: TypeShapeFactsV1::LEAF,
        }
    }
    #[must_use]
    pub const fn bool() -> Self {
        Self::scalar(ScalarTyV1::Bool)
    }
    #[must_use]
    pub const fn int() -> Self {
        Self::scalar(ScalarTyV1::Int)
    }
    #[must_use]
    pub const fn float() -> Self {
        Self::scalar(ScalarTyV1::Float)
    }
    #[must_use]
    pub const fn string() -> Self {
        Self::scalar(ScalarTyV1::String)
    }
    #[must_use]
    pub const fn null() -> Self {
        Self {
            kind: TypeKindV1::Null,
            shape: TypeShapeFactsV1::LEAF,
        }
    }
    pub fn try_tuple(items: Vec<ValueTypeV1>) -> Result<Self, TypeDiagnosticV2> {
        let tuple = TupleTypeV1::try_new(items)?;
        Ok(Self {
            shape: tuple.shape,
            kind: TypeKindV1::Tuple(tuple),
        })
    }
    pub fn try_optional(inner: ValueTypeV1) -> Result<Self, TypeDiagnosticV2> {
        let optional = OptionalTypeV1::try_new(inner)?;
        Ok(Self {
            shape: optional.shape,
            kind: TypeKindV1::Optional(optional),
        })
    }
    pub fn try_collection(
        element: ValueTypeV1,
        cardinality: CardinalityV1,
    ) -> Result<Self, TypeDiagnosticV2> {
        let collection = CollectionTypeV1::try_new(element, cardinality)?;
        Ok(Self {
            shape: collection.shape,
            kind: TypeKindV1::Collection(collection),
        })
    }
    pub fn try_list(element: ValueTypeV1) -> Result<Self, TypeDiagnosticV2> {
        Self::try_collection(element, CardinalityV1::Sequence)
    }
    #[must_use]
    pub fn record(record: RecordTypeV1) -> Self {
        Self {
            shape: record.shape,
            kind: TypeKindV1::Record(record),
        }
    }
    #[must_use]
    pub fn union(union: UnionTypeV1) -> Self {
        Self {
            shape: union.shape,
            kind: TypeKindV1::Union(union),
        }
    }
    #[must_use]
    pub fn relation(relation: RelationTypeV1) -> Self {
        Self {
            shape: relation.shape,
            kind: TypeKindV1::Relation(relation),
        }
    }
    #[must_use]
    pub const fn as_scalar(&self) -> Option<&ScalarTyV1> {
        if let TypeKindV1::Scalar(value) = &self.kind {
            Some(value)
        } else {
            None
        }
    }
    #[must_use]
    pub const fn is_null(&self) -> bool {
        matches!(self.kind, TypeKindV1::Null)
    }
    #[must_use]
    pub const fn as_tuple(&self) -> Option<&TupleTypeV1> {
        if let TypeKindV1::Tuple(value) = &self.kind {
            Some(value)
        } else {
            None
        }
    }
    #[must_use]
    pub const fn as_record(&self) -> Option<&RecordTypeV1> {
        if let TypeKindV1::Record(value) = &self.kind {
            Some(value)
        } else {
            None
        }
    }
    #[must_use]
    pub const fn as_union(&self) -> Option<&UnionTypeV1> {
        if let TypeKindV1::Union(value) = &self.kind {
            Some(value)
        } else {
            None
        }
    }
    #[must_use]
    pub const fn as_optional(&self) -> Option<&OptionalTypeV1> {
        if let TypeKindV1::Optional(value) = &self.kind {
            Some(value)
        } else {
            None
        }
    }
    #[must_use]
    pub const fn as_collection(&self) -> Option<&CollectionTypeV1> {
        if let TypeKindV1::Collection(value) = &self.kind {
            Some(value)
        } else {
            None
        }
    }
    #[must_use]
    pub const fn as_relation(&self) -> Option<&RelationTypeV1> {
        if let TypeKindV1::Relation(value) = &self.kind {
            Some(value)
        } else {
            None
        }
    }
    #[must_use]
    pub const fn depth(&self) -> usize {
        self.shape.depth
    }
    #[must_use]
    pub const fn node_count(&self) -> usize {
        self.shape.nodes
    }
    #[must_use]
    pub const fn contains_relation(&self) -> bool {
        self.shape.contains_relation
    }

    fn is_assignable_to(&self, expected: &Self) -> bool {
        if self == expected {
            return true;
        }
        if let Some(optional) = expected.as_optional() {
            return self.is_null() || self.is_assignable_to(optional.inner().as_type());
        }
        match (self.as_collection(), expected.as_collection()) {
            (Some(source), Some(target)) => {
                source.cardinality == target.cardinality
                    && source.element.is_assignable_to(&target.element)
            }
            _ => false,
        }
    }
}

impl fmt::Display for TypeV1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            TypeKindV1::Scalar(scalar) => write!(f, "{scalar}"),
            TypeKindV1::Null => f.write_str("null"),
            TypeKindV1::Tuple(tuple) => {
                f.write_str("tuple<")?;
                for (index, item) in tuple.items.iter().enumerate() {
                    if index > 0 {
                        f.write_str(", ")?;
                    }
                    write!(f, "{item}")?;
                }
                f.write_str(">")
            }
            TypeKindV1::Record(record) => {
                f.write_str("{ ")?;
                for (index, field) in record.fields.iter().enumerate() {
                    if index > 0 {
                        f.write_str(", ")?;
                    }
                    write!(f, "{}: {}", field.name, field.value_type)?;
                }
                f.write_str(" }")
            }
            TypeKindV1::Optional(optional) => write!(f, "optional<{}>", optional.inner),
            TypeKindV1::Collection(collection) => {
                let name = match collection.cardinality {
                    CardinalityV1::Set => "set",
                    CardinalityV1::Bag => "bag",
                    CardinalityV1::Sequence => "list",
                };
                write!(f, "{name}<{}>", collection.element)
            }
            TypeKindV1::Union(union) => {
                f.write_str("union<")?;
                for (index, variant) in union.variants.iter().enumerate() {
                    if index > 0 {
                        f.write_str(", ")?;
                    }
                    write!(f, "{}: {}", variant.tag, variant.value_type)?;
                }
                f.write_str(">")
            }
            TypeKindV1::Relation(relation) => {
                write!(f, "relation<{}>", Self::record(relation.row.clone()))
            }
        }
    }
}

fn canonical_type_name(value: &str) -> bool {
    if value.len() > MAX_TYPE_NAME_BYTES {
        return false;
    }
    let mut bytes = value.bytes();
    let Some(first) = bytes.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == b'_')
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
}

/// Finite IEEE-754 value with both signed zeros canonicalized to positive zero.
/// Equality and hashing compare canonical bits; ordering uses `f64::total_cmp`.
/// Excluding NaNs and normalizing zero makes these contracts agree.
#[derive(Clone, Copy, Debug)]
pub struct FiniteF64V2(u64);
impl FiniteF64V2 {
    pub fn try_new(value: f64, location: LocatedSpanV2) -> Result<Self, TypeDiagnosticV2> {
        if !value.is_finite() {
            return Err(TypeDiagnosticV2::FloatNonFinite { location });
        }
        Ok(Self(if value == 0.0 {
            0.0f64.to_bits()
        } else {
            value.to_bits()
        }))
    }
    #[must_use]
    pub fn get(self) -> f64 {
        f64::from_bits(self.0)
    }
}
impl PartialEq for FiniteF64V2 {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl Eq for FiniteF64V2 {}
impl Hash for FiniteF64V2 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}
impl PartialOrd for FiniteF64V2 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for FiniteF64V2 {
    fn cmp(&self, other: &Self) -> Ordering {
        self.get().total_cmp(&other.get())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TypedLiteralValueV2 {
    Int(i128),
    Float(FiniteF64V2),
    Bool(bool),
    String(String),
    Null,
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "unit test asserts checked builder construction and rejection"
)]
mod tests {
    use super::{
        RecordFieldTypeV1, RecordTypeBuilderV1, RecordTypeV1, TypeV1, UnionTypeV1,
        UnionVariantTypeV1, ValueTypeV1, MAX_TYPE_NODES,
    };
    use quanta_gqlang_v2_hir::{
        LocatedSpanV2, SourceIdV2, SourceRegistrationV2, SourceRegistryViewV2, SpanV1,
    };

    #[test]
    fn oversized_builder_retains_only_admitted_owned_child() {
        let source_id = SourceIdV2::from_raw(7);
        let location = LocatedSpanV2 {
            source_id,
            span: SpanV1::new(0, 1),
        };
        let registry = SourceRegistryViewV2::try_new(vec![SourceRegistrationV2 {
            source_id,
            source_len: 1,
        }])
        .expect("source registry");
        let checked = registry
            .try_check_location(location)
            .expect("field location");
        let large = ValueTypeV1::try_tuple(vec![ValueTypeV1::int(); MAX_TYPE_NODES - 2])
            .expect("4095-node child");
        let mut builder = RecordTypeBuilderV1::new();
        for index in 0..MAX_TYPE_NODES {
            builder
                .try_push(&format!("g{index}"), &large, checked.clone())
                .expect("defer parent limit until field priority is known");
        }
        assert_eq!(builder.fields.len(), 1, "only the admitted child is owned");
        assert_eq!(
            builder.shape_limit().and_then(|error| error.primary()),
            Some(location)
        );
        assert_eq!(
            builder.finish().unwrap_err().code(),
            "types.type_shape_limit"
        );

        let mut duplicate = RecordTypeBuilderV1::new();
        duplicate
            .try_push("same", &large, checked.clone())
            .expect("first field");
        duplicate
            .try_push("same", &large, checked)
            .expect("shape error is deferred");
        assert_eq!(
            duplicate.finish().unwrap_err().code(),
            "types.type_shape_limit",
            "parent shape precedes duplicate detection at the public constructor boundary"
        );
    }

    #[test]
    fn wide_type_clones_share_children_without_changing_shape() {
        let source_id = SourceIdV2::from_raw(7);
        let registry = SourceRegistryViewV2::try_new(vec![SourceRegistrationV2 {
            source_id,
            source_len: 1,
        }])
        .expect("source registry");
        let checked = registry
            .try_check_location(LocatedSpanV2 {
                source_id,
                span: SpanV1::new(0, 1),
            })
            .expect("field location");

        let tuple = ValueTypeV1::try_tuple(vec![ValueTypeV1::int(); MAX_TYPE_NODES - 2])
            .expect("4095-node tuple");
        let tuple_copy = tuple.clone();
        assert_eq!(tuple, tuple_copy);
        assert_eq!(tuple.node_count(), MAX_TYPE_NODES - 1);
        assert!(std::ptr::eq(
            tuple.as_tuple().expect("tuple").items().as_ptr(),
            tuple_copy.as_tuple().expect("tuple copy").items().as_ptr(),
        ));

        let record = ValueTypeV1::try_record(
            RecordTypeV1::try_new(vec![RecordFieldTypeV1::try_new(
                "value",
                tuple,
                checked.clone(),
            )
            .expect("field")])
            .expect("4096-node record"),
        )
        .expect("value record");
        let record_copy = record.clone();
        assert_eq!(record, record_copy);
        assert_eq!(record.node_count(), MAX_TYPE_NODES);
        assert!(std::ptr::eq(
            record.as_record().expect("record").fields().as_ptr(),
            record_copy
                .as_record()
                .expect("record copy")
                .fields()
                .as_ptr(),
        ));

        let union = ValueTypeV1::try_from_type(TypeV1::union(
            UnionTypeV1::try_new(vec![
                UnionVariantTypeV1::try_new("first", ValueTypeV1::int(), checked.clone())
                    .expect("first variant"),
                UnionVariantTypeV1::try_new("second", ValueTypeV1::bool(), checked)
                    .expect("second variant"),
            ])
            .expect("union"),
        ))
        .expect("value union");
        let union_copy = union.clone();
        assert_eq!(union, union_copy);
        assert!(std::ptr::eq(
            union.as_union().expect("union").variants().as_ptr(),
            union_copy
                .as_union()
                .expect("union copy")
                .variants()
                .as_ptr(),
        ));
    }
}
