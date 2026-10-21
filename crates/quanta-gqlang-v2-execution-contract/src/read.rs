//! Allocation-free observations of a retained relation. No view exposes its
//! cloneable DTO backing; copying a view keeps the original borrow lifetime.

use crate::{
    AlgebraFamilyV2, ExecutionBindingV3, ExecutionCallTargetV3, ExecutionCardinalityV3,
    ExecutionCorrelationV3, ExecutionExistentialTermV3, ExecutionExistsPolarityV3,
    ExecutionExpressionShapeV3, ExecutionExpressionV3, ExecutionFieldV3, ExecutionFiniteFloatV3,
    ExecutionJoinModeV3, ExecutionLiteralV3, ExecutionLogicalQuantifierV3, ExecutionNullOrderV3,
    ExecutionOperatorV3, ExecutionOrderKeyV3, ExecutionRecordFieldV3, ExecutionRelationAuthorityV3,
    ExecutionRelationV3, ExecutionRelationalOperatorV3, ExecutionScalarTypeV3, ExecutionSchemaV3,
    ExecutionSortDirectionV3, ExecutionSourceKindV3, ExecutionSourceV3, ExecutionUnionVariantV3,
    ExecutionValueTypeV3, SemanticItemIdV2, SourceIdV2, SourceQualifiedNodeIdV2,
};

macro_rules! read_view {
    ($view:ident, $dto:ident) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub struct $view<'a>(&'a $dto);
        impl $dto {
            #[must_use]
            pub fn read_v3(&self) -> $view<'_> {
                $view(self)
            }
        }
    };
}

macro_rules! read_slice {
    ($slice:ident, $dto:ident, $view:ident) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub struct $slice<'a>(&'a [$dto]);
        impl<'a> $slice<'a> {
            pub const fn len(self) -> usize {
                self.0.len()
            }
            pub const fn is_empty(self) -> bool {
                self.0.is_empty()
            }
            pub fn get(self, index: usize) -> Option<$view<'a>> {
                self.0.get(index).map($view)
            }
            pub fn first(self) -> Option<$view<'a>> {
                self.get(0)
            }
            pub fn iter(
                self,
            ) -> impl ExactSizeIterator<Item = $view<'a>> + DoubleEndedIterator + Clone {
                self.0.iter().map($view)
            }
        }
        impl<'a> IntoIterator for $slice<'a> {
            type Item = $view<'a>;
            type IntoIter = std::iter::Map<std::slice::Iter<'a, $dto>, fn(&'a $dto) -> $view<'a>>;
            fn into_iter(self) -> Self::IntoIter {
                self.0.iter().map($view)
            }
        }
        impl<'a> IntoIterator for &$slice<'a> {
            type Item = $view<'a>;
            type IntoIter = <$slice<'a> as IntoIterator>::IntoIter;
            fn into_iter(self) -> Self::IntoIter {
                (*self).into_iter()
            }
        }
    };
}

read_view!(ExecutionRelationReadV3, ExecutionRelationV3);
read_view!(
    ExecutionRelationalOperatorReadV3,
    ExecutionRelationalOperatorV3
);
read_view!(ExecutionExpressionReadV3, ExecutionExpressionV3);
read_view!(ExecutionSchemaReadV3, ExecutionSchemaV3);
read_view!(ExecutionFieldReadV3, ExecutionFieldV3);
read_view!(ExecutionOrderKeyReadV3, ExecutionOrderKeyV3);
read_view!(ExecutionBindingReadV3, ExecutionBindingV3);
read_view!(ExecutionSourceReadV3, ExecutionSourceV3);
read_view!(ExecutionCorrelationReadV3, ExecutionCorrelationV3);
read_view!(ExecutionExistentialTermReadV3, ExecutionExistentialTermV3);
read_view!(ExecutionOperatorReadV3, ExecutionOperatorV3);
read_view!(ExecutionValueTypeReadV3, ExecutionValueTypeV3);
read_view!(ExecutionRecordFieldReadV3, ExecutionRecordFieldV3);
read_view!(ExecutionUnionVariantReadV3, ExecutionUnionVariantV3);

read_slice!(
    ExecutionExpressionSliceReadV3,
    ExecutionExpressionV3,
    ExecutionExpressionReadV3
);
read_slice!(
    ExecutionRelationalOperatorSliceReadV3,
    ExecutionRelationalOperatorV3,
    ExecutionRelationalOperatorReadV3
);
read_slice!(
    ExecutionFieldSliceReadV3,
    ExecutionFieldV3,
    ExecutionFieldReadV3
);
read_slice!(
    ExecutionOrderKeySliceReadV3,
    ExecutionOrderKeyV3,
    ExecutionOrderKeyReadV3
);
read_slice!(
    ExecutionBindingSliceReadV3,
    ExecutionBindingV3,
    ExecutionBindingReadV3
);
read_slice!(
    ExecutionCorrelationSliceReadV3,
    ExecutionCorrelationV3,
    ExecutionCorrelationReadV3
);
read_slice!(
    ExecutionExistentialTermSliceReadV3,
    ExecutionExistentialTermV3,
    ExecutionExistentialTermReadV3
);
read_slice!(
    ExecutionValueTypeSliceReadV3,
    ExecutionValueTypeV3,
    ExecutionValueTypeReadV3
);
read_slice!(
    ExecutionRecordFieldSliceReadV3,
    ExecutionRecordFieldV3,
    ExecutionRecordFieldReadV3
);
read_slice!(
    ExecutionUnionVariantSliceReadV3,
    ExecutionUnionVariantV3,
    ExecutionUnionVariantReadV3
);

impl<'a> ExecutionRelationReadV3<'a> {
    pub fn item_id(self) -> SemanticItemIdV2 {
        self.0.item_id()
    }
    pub fn root(self) -> ExecutionRelationalOperatorReadV3<'a> {
        ExecutionRelationalOperatorReadV3(self.0.root())
    }
    pub fn expressions(self) -> ExecutionExpressionSliceReadV3<'a> {
        ExecutionExpressionSliceReadV3(self.0.expressions())
    }
}

impl<'a> ExecutionRelationalOperatorReadV3<'a> {
    pub fn family(self) -> AlgebraFamilyV2 {
        self.0.family()
    }
    pub fn join_mode(self) -> Option<ExecutionJoinModeV3> {
        self.0.join_mode()
    }
    pub fn limit_count(self) -> Option<u64> {
        self.0.limit_count()
    }
    pub fn stage(self) -> SourceQualifiedNodeIdV2 {
        self.0.stage()
    }
    pub fn source(self) -> Option<ExecutionSourceReadV3<'a>> {
        self.0.source().map(ExecutionSourceReadV3)
    }
    pub fn schema(self) -> ExecutionSchemaReadV3<'a> {
        ExecutionSchemaReadV3(self.0.schema())
    }
    pub fn predicates(self) -> &'a [SourceQualifiedNodeIdV2] {
        self.0.predicates()
    }
    pub fn expressions(self) -> &'a [SourceQualifiedNodeIdV2] {
        self.0.expressions()
    }
    pub fn correlations(self) -> ExecutionCorrelationSliceReadV3<'a> {
        ExecutionCorrelationSliceReadV3(self.0.correlations())
    }
    pub fn existential_terms(self) -> ExecutionExistentialTermSliceReadV3<'a> {
        ExecutionExistentialTermSliceReadV3(self.0.existential_terms())
    }
    pub fn partition_input(self) -> ExecutionBindingSliceReadV3<'a> {
        ExecutionBindingSliceReadV3(self.0.partition_input())
    }
    pub fn children(self) -> ExecutionRelationalOperatorSliceReadV3<'a> {
        ExecutionRelationalOperatorSliceReadV3(self.0.children())
    }
}

impl<'a> ExecutionSchemaReadV3<'a> {
    pub fn fields(self) -> ExecutionFieldSliceReadV3<'a> {
        ExecutionFieldSliceReadV3(self.0.fields())
    }
    pub fn cardinality(self) -> ExecutionCardinalityV3 {
        self.0.cardinality()
    }
    pub fn partition_keys(self) -> &'a [SourceQualifiedNodeIdV2] {
        self.0.partition_keys()
    }
    pub fn order(self) -> ExecutionOrderKeySliceReadV3<'a> {
        ExecutionOrderKeySliceReadV3(self.0.order())
    }
}

impl<'a> ExecutionFieldReadV3<'a> {
    pub fn name(self) -> &'a str {
        self.0.name()
    }
    pub fn value_type(self) -> ExecutionValueTypeReadV3<'a> {
        ExecutionValueTypeReadV3(self.0.value_type())
    }
}

impl<'a> ExecutionOrderKeyReadV3<'a> {
    pub fn expression(self) -> SourceQualifiedNodeIdV2 {
        self.0.expression()
    }
    pub fn direction(self) -> ExecutionSortDirectionV3 {
        self.0.direction()
    }
    pub fn null_order(self) -> ExecutionNullOrderV3 {
        self.0.null_order()
    }
    pub fn value_type(self) -> ExecutionValueTypeReadV3<'a> {
        ExecutionValueTypeReadV3(self.0.value_type())
    }
}

impl<'a> ExecutionExpressionReadV3<'a> {
    pub fn id(self) -> SourceQualifiedNodeIdV2 {
        self.0.id()
    }
    pub fn value_type(self) -> ExecutionValueTypeReadV3<'a> {
        ExecutionValueTypeReadV3(self.0.value_type())
    }
    pub fn shape(self) -> ExecutionExpressionShapeReadV3<'a> {
        use ExecutionExpressionShapeReadV3 as R;
        use ExecutionExpressionShapeV3 as D;
        match self.0.shape() {
            D::Literal(value) => R::Literal(literal_read(value)),
            D::Binding(value) => R::Binding(ExecutionBindingReadV3(value)),
            D::Path { base, segments } => R::Path {
                base,
                segments: ExecutionStringSliceReadV3(segments),
            },
            D::Unary { operator, operand } => R::Unary {
                operator: ExecutionOperatorReadV3(operator),
                operand,
            },
            D::Binary { operator, lhs, rhs } => R::Binary {
                operator: ExecutionOperatorReadV3(operator),
                lhs,
                rhs,
            },
            D::Call { target, arguments } => R::Call {
                target: call_target_read(target),
                arguments,
            },
            D::Paren(value) => R::Paren(value),
            D::Record(fields) => R::Record(ExecutionRecordExpressionSliceReadV3(fields)),
            D::List(items) => R::List(items),
            D::BoolValue(value) => R::BoolValue(value),
            D::Compare { operator, lhs, rhs } => R::Compare {
                operator: ExecutionOperatorReadV3(operator),
                lhs,
                rhs,
            },
            D::And { operator, lhs, rhs } => R::And {
                operator: ExecutionOperatorReadV3(operator),
                lhs,
                rhs,
            },
            D::Or { operator, lhs, rhs } => R::Or {
                operator: ExecutionOperatorReadV3(operator),
                lhs,
                rhs,
            },
            D::Not { operator, operand } => R::Not {
                operator: ExecutionOperatorReadV3(operator),
                operand,
            },
            D::LogicalQuantifier {
                kind,
                binders,
                body,
            } => R::LogicalQuantifier {
                kind: *kind,
                binders: ExecutionBindingSliceReadV3(binders),
                body,
            },
            D::RelationExists {
                polarity,
                relation_root,
                correlations,
            } => R::RelationExists {
                polarity: *polarity,
                relation_root,
                correlations: ExecutionCorrelationSliceReadV3(correlations),
            },
        }
    }
}

impl<'a> ExecutionBindingReadV3<'a> {
    pub fn source_id(self) -> SourceIdV2 {
        self.0.source_id()
    }
    pub fn owner_node(self) -> u32 {
        self.0.owner_node()
    }
    pub fn ordinal(self) -> usize {
        self.0.ordinal()
    }
    pub fn name(self) -> &'a str {
        self.0.name()
    }
}

impl<'a> ExecutionSourceReadV3<'a> {
    pub fn authority(self) -> ExecutionRelationAuthorityV3 {
        self.0.authority()
    }
    pub fn kind(self) -> ExecutionSourceKindV3 {
        self.0.kind()
    }
    pub fn namespace(self) -> Option<&'a str> {
        self.0.namespace()
    }
    pub fn name(self) -> &'a str {
        self.0.name()
    }
    pub fn host_binding_identity(self) -> Option<[u8; 32]> {
        self.0.host_binding_identity()
    }
    pub fn composition_binding(self) -> Option<ExecutionBindingReadV3<'a>> {
        self.0.composition_binding().map(ExecutionBindingReadV3)
    }
}

impl<'a> ExecutionCorrelationReadV3<'a> {
    pub fn binding(self) -> ExecutionBindingReadV3<'a> {
        ExecutionBindingReadV3(self.0.binding())
    }
    pub fn reference(self) -> SourceQualifiedNodeIdV2 {
        self.0.reference()
    }
}

impl<'a> ExecutionExistentialTermReadV3<'a> {
    pub fn expression(self) -> SourceQualifiedNodeIdV2 {
        self.0.expression()
    }
    pub fn polarity(self) -> ExecutionExistsPolarityV3 {
        self.0.polarity()
    }
    pub fn correlations(self) -> ExecutionCorrelationSliceReadV3<'a> {
        ExecutionCorrelationSliceReadV3(self.0.correlations())
    }
}

impl<'a> ExecutionOperatorReadV3<'a> {
    pub fn symbol(self) -> &'a str {
        self.0.symbol()
    }
    pub fn overload_id(self) -> u32 {
        self.0.overload_id()
    }
}

impl<'a> ExecutionRecordFieldReadV3<'a> {
    pub fn name(self) -> &'a str {
        self.0.name()
    }
    pub fn value_type(self) -> ExecutionValueTypeReadV3<'a> {
        ExecutionValueTypeReadV3(self.0.value_type())
    }
}

impl<'a> ExecutionUnionVariantReadV3<'a> {
    pub fn tag(self) -> &'a str {
        self.0.tag()
    }
    pub fn value_type(self) -> ExecutionValueTypeReadV3<'a> {
        ExecutionValueTypeReadV3(self.0.value_type())
    }
}

impl<'a> ExecutionValueTypeReadV3<'a> {
    pub fn shape(self) -> ExecutionValueTypeShapeReadV3<'a> {
        use ExecutionValueTypeShapeReadV3 as R;
        use ExecutionValueTypeV3 as D;
        match self.0 {
            D::Scalar(value) => R::Scalar(*value),
            D::Null => R::Null,
            D::Tuple(items) => R::Tuple(ExecutionValueTypeSliceReadV3(items)),
            D::Record(fields) => R::Record(ExecutionRecordFieldSliceReadV3(fields)),
            D::Optional(value) => R::Optional(Self(value)),
            D::Collection {
                element,
                cardinality,
            } => R::Collection {
                element: Self(element),
                cardinality: *cardinality,
            },
            D::Union(variants) => R::Union(ExecutionUnionVariantSliceReadV3(variants)),
            D::Unknown => R::Unknown,
        }
    }
    pub fn kind(self) -> &'static str {
        match self.0 {
            ExecutionValueTypeV3::Scalar(_) => "scalar",
            ExecutionValueTypeV3::Null => "null",
            ExecutionValueTypeV3::Tuple(_) => "tuple",
            ExecutionValueTypeV3::Record(_) => "record",
            ExecutionValueTypeV3::Optional(_) => "optional",
            ExecutionValueTypeV3::Collection { .. } => "collection",
            ExecutionValueTypeV3::Union(_) => "union",
            ExecutionValueTypeV3::Unknown => "unknown",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecutionValueTypeShapeReadV3<'a> {
    Scalar(ExecutionScalarTypeV3),
    Null,
    Tuple(ExecutionValueTypeSliceReadV3<'a>),
    Record(ExecutionRecordFieldSliceReadV3<'a>),
    Optional(ExecutionValueTypeReadV3<'a>),
    Collection {
        element: ExecutionValueTypeReadV3<'a>,
        cardinality: ExecutionCardinalityV3,
    },
    Union(ExecutionUnionVariantSliceReadV3<'a>),
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecutionLiteralReadV3<'a> {
    Int(&'a i128),
    Float(ExecutionFiniteFloatV3),
    Bool(&'a bool),
    String(&'a str),
    Null,
}

fn literal_read(value: &ExecutionLiteralV3) -> ExecutionLiteralReadV3<'_> {
    match value {
        ExecutionLiteralV3::Int(value) => ExecutionLiteralReadV3::Int(value),
        ExecutionLiteralV3::Float(value) => ExecutionLiteralReadV3::Float(*value),
        ExecutionLiteralV3::Bool(value) => ExecutionLiteralReadV3::Bool(value),
        ExecutionLiteralV3::String(value) => ExecutionLiteralReadV3::String(value),
        ExecutionLiteralV3::Null => ExecutionLiteralReadV3::Null,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecutionCallTargetReadV3<'a> {
    Builtin { name: &'a str },
    Predicate { name: &'a str },
    HostPredicate { fingerprint: &'a str },
}

fn call_target_read(value: &ExecutionCallTargetV3) -> ExecutionCallTargetReadV3<'_> {
    match value {
        ExecutionCallTargetV3::Builtin { name } => ExecutionCallTargetReadV3::Builtin { name },
        ExecutionCallTargetV3::Predicate { name } => ExecutionCallTargetReadV3::Predicate { name },
        ExecutionCallTargetV3::HostPredicate { fingerprint } => {
            ExecutionCallTargetReadV3::HostPredicate { fingerprint }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExecutionStringSliceReadV3<'a>(&'a [String]);
impl<'a> ExecutionStringSliceReadV3<'a> {
    pub fn len(self) -> usize {
        self.0.len()
    }
    pub fn is_empty(self) -> bool {
        self.0.is_empty()
    }
    pub fn get(self, index: usize) -> Option<&'a str> {
        self.0.get(index).map(String::as_str)
    }
    pub fn first(self) -> Option<&'a str> {
        self.get(0)
    }
    pub fn iter(self) -> impl ExactSizeIterator<Item = &'a str> + DoubleEndedIterator + Clone {
        self.0.iter().map(String::as_str)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExecutionRecordExpressionSliceReadV3<'a>(&'a [(String, SourceQualifiedNodeIdV2)]);
impl<'a> ExecutionRecordExpressionSliceReadV3<'a> {
    pub fn len(self) -> usize {
        self.0.len()
    }
    pub fn is_empty(self) -> bool {
        self.0.is_empty()
    }
    pub fn get(self, index: usize) -> Option<(&'a str, SourceQualifiedNodeIdV2)> {
        self.0.get(index).map(|(name, id)| (name.as_str(), *id))
    }
    pub fn iter(
        self,
    ) -> impl ExactSizeIterator<Item = (&'a str, SourceQualifiedNodeIdV2)> + DoubleEndedIterator + Clone
    {
        self.0.iter().map(|(name, id)| (name.as_str(), *id))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecutionExpressionShapeReadV3<'a> {
    Literal(ExecutionLiteralReadV3<'a>),
    Binding(ExecutionBindingReadV3<'a>),
    Path {
        base: &'a SourceQualifiedNodeIdV2,
        segments: ExecutionStringSliceReadV3<'a>,
    },
    Unary {
        operator: ExecutionOperatorReadV3<'a>,
        operand: &'a SourceQualifiedNodeIdV2,
    },
    Binary {
        operator: ExecutionOperatorReadV3<'a>,
        lhs: &'a SourceQualifiedNodeIdV2,
        rhs: &'a SourceQualifiedNodeIdV2,
    },
    Call {
        target: ExecutionCallTargetReadV3<'a>,
        arguments: &'a [Option<SourceQualifiedNodeIdV2>],
    },
    Paren(&'a SourceQualifiedNodeIdV2),
    Record(ExecutionRecordExpressionSliceReadV3<'a>),
    List(&'a [SourceQualifiedNodeIdV2]),
    BoolValue(&'a SourceQualifiedNodeIdV2),
    Compare {
        operator: ExecutionOperatorReadV3<'a>,
        lhs: &'a SourceQualifiedNodeIdV2,
        rhs: &'a SourceQualifiedNodeIdV2,
    },
    And {
        operator: ExecutionOperatorReadV3<'a>,
        lhs: &'a SourceQualifiedNodeIdV2,
        rhs: &'a SourceQualifiedNodeIdV2,
    },
    Or {
        operator: ExecutionOperatorReadV3<'a>,
        lhs: &'a SourceQualifiedNodeIdV2,
        rhs: &'a SourceQualifiedNodeIdV2,
    },
    Not {
        operator: ExecutionOperatorReadV3<'a>,
        operand: &'a SourceQualifiedNodeIdV2,
    },
    LogicalQuantifier {
        kind: ExecutionLogicalQuantifierV3,
        binders: ExecutionBindingSliceReadV3<'a>,
        body: &'a SourceQualifiedNodeIdV2,
    },
    RelationExists {
        polarity: ExecutionExistsPolarityV3,
        relation_root: &'a SourceQualifiedNodeIdV2,
        correlations: ExecutionCorrelationSliceReadV3<'a>,
    },
}
