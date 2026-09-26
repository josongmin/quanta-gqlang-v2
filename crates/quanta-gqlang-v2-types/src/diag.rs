use crate::ty::ValueTypeV1;
use quanta_gqlang_v2_hir::{HirNodeIdV1, LocatedSpanV2};
use std::fmt;

const MAX_DIAGNOSTIC_EXCERPT_BYTES: usize = 128;

fn diagnostic_excerpt(value: &str) -> &str {
    if value.len() <= MAX_DIAGNOSTIC_EXCERPT_BYTES {
        return value;
    }
    let mut end = MAX_DIAGNOSTIC_EXCERPT_BYTES;
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    &value[..end]
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeDiagnosticV2 {
    InvalidTypeName {
        location: LocatedSpanV2,
    },
    DuplicateRecordField {
        name: String,
        primary: LocatedSpanV2,
        secondary: LocatedSpanV2,
    },
    DuplicateUnionTag {
        tag: String,
        primary: LocatedSpanV2,
        secondary: LocatedSpanV2,
    },
    IntegerOutOfRange {
        node_id: HirNodeIdV1,
        location: LocatedSpanV2,
        text: String,
    },
    FloatNonFinite {
        location: LocatedSpanV2,
    },
    FloatUnderflow {
        node_id: HirNodeIdV1,
        location: LocatedSpanV2,
        text: String,
    },
    MalformedLiteral {
        node_id: HirNodeIdV1,
        location: LocatedSpanV2,
    },
    MissingNodeFact {
        node_id: HirNodeIdV1,
        location: LocatedSpanV2,
    },
    DuplicateNodeFact {
        node_id: HirNodeIdV1,
        location: LocatedSpanV2,
    },
    ForeignDocumentFact {
        node_id: HirNodeIdV1,
        location: LocatedSpanV2,
    },
    ExtraNodeFact {
        node_id: HirNodeIdV1,
    },
    UnusedContextualInput {
        node_id: HirNodeIdV1,
        location: LocatedSpanV2,
    },
    WrongNodeFactKind {
        node_id: HirNodeIdV1,
        location: LocatedSpanV2,
    },
    InvalidBindingDependency {
        node_id: HirNodeIdV1,
        target_node_id: HirNodeIdV1,
        location: LocatedSpanV2,
    },
    UnavailableBindingDependency {
        node_id: HirNodeIdV1,
        target_node_id: HirNodeIdV1,
        location: LocatedSpanV2,
    },
    InvalidDeferredRelationRow {
        root: HirNodeIdV1,
        location: LocatedSpanV2,
    },
    InvalidRelationPreEvaluationPlan {
        root: HirNodeIdV1,
        location: LocatedSpanV2,
    },
    UnavailableRelationRowDependency {
        root: HirNodeIdV1,
        target: HirNodeIdV1,
        location: LocatedSpanV2,
    },
    InvalidRelationPredicateInput {
        node_id: HirNodeIdV1,
        location: LocatedSpanV2,
    },
    CyclicDeclarationDependency {
        node_id: HirNodeIdV1,
        location: LocatedSpanV2,
    },
    InvalidRuleTypeBudget,
    InvalidRuleInput {
        location: LocatedSpanV2,
    },
    InvalidRuleHead {
        node_id: HirNodeIdV1,
        location: LocatedSpanV2,
    },
    UnknownRuleCallable {
        location: LocatedSpanV2,
    },
    RuleTypeResourceLimit {
        dimension: &'static str,
        location: LocatedSpanV2,
    },
    ContradictoryRuleType {
        location: LocatedSpanV2,
    },
    AmbiguousRuleType {
        location: LocatedSpanV2,
        candidates: usize,
    },
    UnanchoredRecursiveRule {
        location: LocatedSpanV2,
    },
    RulePredicateArity {
        location: LocatedSpanV2,
    },
    RelationExistsInRule {
        location: LocatedSpanV2,
    },
    SourceRegistryMismatch {
        location: LocatedSpanV2,
    },
    InvalidResolvedIdentity,
    HostPredicateIdentityLimit,
    UnknownOperator {
        node_id: HirNodeIdV1,
        symbol: String,
        location: LocatedSpanV2,
    },
    UnknownBuiltin {
        node_id: HirNodeIdV1,
        name: String,
        location: LocatedSpanV2,
    },
    BadOverload {
        node_id: HirNodeIdV1,
        name: String,
        operands: Vec<ValueTypeV1>,
        location: LocatedSpanV2,
    },
    BadProjection {
        node_id: HirNodeIdV1,
        base: ValueTypeV1,
        field: String,
        location: LocatedSpanV2,
    },
    HeterogeneousCollection {
        node_id: HirNodeIdV1,
        expected: ValueTypeV1,
        actual: ValueTypeV1,
        location: LocatedSpanV2,
    },
    InvalidStarArgument {
        node_id: HirNodeIdV1,
        location: LocatedSpanV2,
    },
    RegistryDuplicateId {
        registry: &'static str,
        id: u32,
    },
    RelationUsedAsValue,
    TypeShapeLimit,
    RedundantOptional,
}

impl TypeDiagnosticV2 {
    #[must_use]
    pub const fn primary(&self) -> Option<LocatedSpanV2> {
        match self {
            Self::DuplicateRecordField { primary, .. }
            | Self::DuplicateUnionTag { primary, .. } => Some(*primary),
            Self::InvalidTypeName { location }
            | Self::IntegerOutOfRange { location, .. }
            | Self::FloatNonFinite { location }
            | Self::FloatUnderflow { location, .. }
            | Self::MalformedLiteral { location, .. }
            | Self::MissingNodeFact { location, .. }
            | Self::DuplicateNodeFact { location, .. }
            | Self::ForeignDocumentFact { location, .. }
            | Self::UnusedContextualInput { location, .. }
            | Self::WrongNodeFactKind { location, .. }
            | Self::InvalidBindingDependency { location, .. }
            | Self::UnavailableBindingDependency { location, .. }
            | Self::InvalidDeferredRelationRow { location, .. }
            | Self::InvalidRelationPreEvaluationPlan { location, .. }
            | Self::UnavailableRelationRowDependency { location, .. }
            | Self::InvalidRelationPredicateInput { location, .. }
            | Self::CyclicDeclarationDependency { location, .. }
            | Self::InvalidRuleInput { location }
            | Self::InvalidRuleHead { location, .. }
            | Self::UnknownRuleCallable { location }
            | Self::RuleTypeResourceLimit { location, .. }
            | Self::ContradictoryRuleType { location }
            | Self::AmbiguousRuleType { location, .. }
            | Self::UnanchoredRecursiveRule { location }
            | Self::RulePredicateArity { location }
            | Self::RelationExistsInRule { location }
            | Self::SourceRegistryMismatch { location }
            | Self::UnknownOperator { location, .. }
            | Self::UnknownBuiltin { location, .. }
            | Self::BadOverload { location, .. }
            | Self::BadProjection { location, .. }
            | Self::HeterogeneousCollection { location, .. }
            | Self::InvalidStarArgument { location, .. } => Some(*location),
            Self::ExtraNodeFact { .. }
            | Self::InvalidRuleTypeBudget
            | Self::InvalidResolvedIdentity
            | Self::HostPredicateIdentityLimit
            | Self::RegistryDuplicateId { .. }
            | Self::RelationUsedAsValue
            | Self::TypeShapeLimit
            | Self::RedundantOptional => None,
        }
    }

    #[must_use]
    pub const fn secondary(&self) -> Option<LocatedSpanV2> {
        if let Self::DuplicateRecordField { secondary, .. }
        | Self::DuplicateUnionTag { secondary, .. } = self
        {
            Some(*secondary)
        } else {
            None
        }
    }

    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::InvalidTypeName { .. } => "types.invalid_type_name",
            Self::DuplicateRecordField { .. } => "types.duplicate_record_field",
            Self::DuplicateUnionTag { .. } => "types.duplicate_union_tag",
            Self::IntegerOutOfRange { .. } => "types.integer_out_of_range",
            Self::FloatNonFinite { .. } => "types.float_non_finite",
            Self::FloatUnderflow { .. } => "types.float_underflow",
            Self::MalformedLiteral { .. } => "types.malformed_literal",
            Self::MissingNodeFact { .. } => "types.missing_resolved_input",
            Self::DuplicateNodeFact { .. } => "types.duplicate_resolved_input",
            Self::ForeignDocumentFact { .. } => "types.foreign_document_input",
            Self::ExtraNodeFact { .. } => "types.extra_resolved_input",
            Self::UnusedContextualInput { .. } => "types.unused_contextual_input",
            Self::WrongNodeFactKind { .. } => "types.wrong_resolved_input_kind",
            Self::InvalidBindingDependency { .. } => "types.invalid_binding_dependency",
            Self::UnavailableBindingDependency { .. } => "types.unavailable_binding_dependency",
            Self::InvalidDeferredRelationRow { .. } => "types.invalid_deferred_relation_row",
            Self::InvalidRelationPreEvaluationPlan { .. } => {
                "types.invalid_relation_pre_evaluation_plan"
            }
            Self::UnavailableRelationRowDependency { .. } => {
                "types.unavailable_relation_row_dependency"
            }
            Self::InvalidRelationPredicateInput { .. } => "types.invalid_relation_predicate_input",
            Self::CyclicDeclarationDependency { .. } => "types.cyclic_declaration_dependency",
            Self::InvalidRuleTypeBudget => "types.invalid_rule_type_budget",
            Self::InvalidRuleInput { .. } => "types.invalid_rule_input",
            Self::InvalidRuleHead { .. } => "types.invalid_rule_head",
            Self::UnknownRuleCallable { .. } => "types.unknown_rule_callable",
            Self::RuleTypeResourceLimit { .. } => "types.rule_type_resource_limit",
            Self::ContradictoryRuleType { .. } => "types.contradictory_rule_type",
            Self::AmbiguousRuleType { .. } => "types.ambiguous_rule_type",
            Self::UnanchoredRecursiveRule { .. } => "types.unanchored_recursive_rule",
            Self::RulePredicateArity { .. } => "types.rule_predicate_arity",
            Self::RelationExistsInRule { .. } => "types.relation_exists_in_rule",
            Self::SourceRegistryMismatch { .. } => "types.source_registry_mismatch",
            Self::InvalidResolvedIdentity => "types.invalid_resolved_identity",
            Self::HostPredicateIdentityLimit => "types.host_predicate_identity_limit",
            Self::UnknownOperator { .. } => "types.unknown_operator",
            Self::UnknownBuiltin { .. } => "types.unknown_builtin",
            Self::BadOverload { .. } => "types.bad_overload",
            Self::BadProjection { .. } => "types.bad_projection",
            Self::HeterogeneousCollection { .. } => "types.heterogeneous_collection",
            Self::InvalidStarArgument { .. } => "types.invalid_star_argument",
            Self::RegistryDuplicateId { .. } => "types.registry_duplicate_id",
            Self::RelationUsedAsValue => "types.relation_used_as_value",
            Self::TypeShapeLimit => "types.type_shape_limit",
            Self::RedundantOptional => "types.redundant_optional",
        }
    }
}

impl fmt::Display for TypeDiagnosticV2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateRecordField { name, .. } => {
                write!(
                    f,
                    "{}: duplicate record field `{}`",
                    self.code(),
                    diagnostic_excerpt(name)
                )
            }
            Self::DuplicateUnionTag { tag, .. } => {
                write!(
                    f,
                    "{}: duplicate union tag `{}`",
                    self.code(),
                    diagnostic_excerpt(tag)
                )
            }
            Self::IntegerOutOfRange { .. } => {
                write!(f, "{}: integer literal is outside i128", self.code())
            }
            Self::FloatNonFinite { .. } => {
                write!(f, "{}: float literal is non-finite", self.code())
            }
            Self::FloatUnderflow { .. } => write!(
                f,
                "{}: nonzero float literal underflowed to zero",
                self.code()
            ),
            Self::MalformedLiteral { .. } => {
                write!(f, "{}: malformed numeric literal", self.code())
            }
            Self::BadOverload { name, .. } => {
                write!(
                    f,
                    "{}: no overload for `{}`",
                    self.code(),
                    diagnostic_excerpt(name)
                )
            }
            Self::BadProjection { field, .. } => {
                write!(
                    f,
                    "{}: cannot project field `{}`",
                    self.code(),
                    diagnostic_excerpt(field)
                )
            }
            Self::InvalidTypeName { .. }
            | Self::MissingNodeFact { .. }
            | Self::DuplicateNodeFact { .. }
            | Self::ForeignDocumentFact { .. }
            | Self::ExtraNodeFact { .. }
            | Self::UnusedContextualInput { .. }
            | Self::WrongNodeFactKind { .. }
            | Self::InvalidBindingDependency { .. }
            | Self::UnavailableBindingDependency { .. }
            | Self::InvalidDeferredRelationRow { .. }
            | Self::InvalidRelationPreEvaluationPlan { .. }
            | Self::UnavailableRelationRowDependency { .. }
            | Self::InvalidRelationPredicateInput { .. }
            | Self::CyclicDeclarationDependency { .. }
            | Self::InvalidRuleTypeBudget
            | Self::InvalidRuleInput { .. }
            | Self::InvalidRuleHead { .. }
            | Self::UnknownRuleCallable { .. }
            | Self::RuleTypeResourceLimit { .. }
            | Self::ContradictoryRuleType { .. }
            | Self::AmbiguousRuleType { .. }
            | Self::UnanchoredRecursiveRule { .. }
            | Self::RulePredicateArity { .. }
            | Self::RelationExistsInRule { .. }
            | Self::SourceRegistryMismatch { .. }
            | Self::InvalidResolvedIdentity
            | Self::HostPredicateIdentityLimit
            | Self::UnknownOperator { .. }
            | Self::UnknownBuiltin { .. }
            | Self::HeterogeneousCollection { .. }
            | Self::InvalidStarArgument { .. }
            | Self::RegistryDuplicateId { .. } => f.write_str(self.code()),
            Self::RelationUsedAsValue | Self::TypeShapeLimit | Self::RedundantOptional => {
                f.write_str(self.code())
            }
        }
    }
}
impl std::error::Error for TypeDiagnosticV2 {}

#[cfg(test)]
mod tests {
    use super::TypeDiagnosticV2;
    use crate::ValueTypeV1;
    use quanta_gqlang_v2_hir::{HirNodeIdV1, LocatedSpanV2, SourceIdV2, SpanV1};

    #[test]
    fn source_locations_are_projected_by_the_diagnostic_owner() {
        let primary = LocatedSpanV2 {
            source_id: SourceIdV2::from_raw(7),
            span: SpanV1::new(3, 5),
        };
        let secondary = LocatedSpanV2 {
            source_id: SourceIdV2::from_raw(7),
            span: SpanV1::new(12, 14),
        };
        let duplicate = TypeDiagnosticV2::DuplicateRecordField {
            name: "x".into(),
            primary,
            secondary,
        };
        assert_eq!(duplicate.primary(), Some(primary));
        assert_eq!(duplicate.secondary(), Some(secondary));
        let located = TypeDiagnosticV2::RelationExistsInRule { location: primary };
        assert_eq!(located.primary(), Some(primary));
        assert_eq!(located.secondary(), None);
        let locationless = TypeDiagnosticV2::InvalidRuleTypeBudget;
        assert_eq!(locationless.primary(), None);
        assert_eq!(locationless.secondary(), None);
    }

    #[test]
    fn every_type_diagnostic_pins_code_locations_and_display() {
        let primary = LocatedSpanV2 {
            source_id: SourceIdV2::from_raw(7),
            span: SpanV1::new(3, 5),
        };
        let secondary = LocatedSpanV2 {
            source_id: SourceIdV2::from_raw(7),
            span: SpanV1::new(12, 14),
        };
        let node_id = HirNodeIdV1::from_raw(11);
        let target_node_id = HirNodeIdV1::from_raw(12);

        let located = Some(primary);
        let cases = vec![
            (
                TypeDiagnosticV2::InvalidTypeName { location: primary },
                "types.invalid_type_name",
                located,
                None,
                "types.invalid_type_name",
            ),
            (
                TypeDiagnosticV2::DuplicateRecordField {
                    name: "field".into(),
                    primary,
                    secondary,
                },
                "types.duplicate_record_field",
                located,
                Some(secondary),
                "types.duplicate_record_field: duplicate record field `field`",
            ),
            (
                TypeDiagnosticV2::DuplicateUnionTag {
                    tag: "variant".into(),
                    primary,
                    secondary,
                },
                "types.duplicate_union_tag",
                located,
                Some(secondary),
                "types.duplicate_union_tag: duplicate union tag `variant`",
            ),
            (
                TypeDiagnosticV2::IntegerOutOfRange {
                    node_id,
                    location: primary,
                    text: "999".into(),
                },
                "types.integer_out_of_range",
                located,
                None,
                "types.integer_out_of_range: integer literal is outside i128",
            ),
            (
                TypeDiagnosticV2::FloatNonFinite { location: primary },
                "types.float_non_finite",
                located,
                None,
                "types.float_non_finite: float literal is non-finite",
            ),
            (
                TypeDiagnosticV2::FloatUnderflow {
                    node_id,
                    location: primary,
                    text: "1e-999".into(),
                },
                "types.float_underflow",
                located,
                None,
                "types.float_underflow: nonzero float literal underflowed to zero",
            ),
            (
                TypeDiagnosticV2::MalformedLiteral {
                    node_id,
                    location: primary,
                },
                "types.malformed_literal",
                located,
                None,
                "types.malformed_literal: malformed numeric literal",
            ),
            (
                TypeDiagnosticV2::MissingNodeFact {
                    node_id,
                    location: primary,
                },
                "types.missing_resolved_input",
                located,
                None,
                "types.missing_resolved_input",
            ),
            (
                TypeDiagnosticV2::DuplicateNodeFact {
                    node_id,
                    location: primary,
                },
                "types.duplicate_resolved_input",
                located,
                None,
                "types.duplicate_resolved_input",
            ),
            (
                TypeDiagnosticV2::ForeignDocumentFact {
                    node_id,
                    location: primary,
                },
                "types.foreign_document_input",
                located,
                None,
                "types.foreign_document_input",
            ),
            (
                TypeDiagnosticV2::ExtraNodeFact { node_id },
                "types.extra_resolved_input",
                None,
                None,
                "types.extra_resolved_input",
            ),
            (
                TypeDiagnosticV2::UnusedContextualInput {
                    node_id,
                    location: primary,
                },
                "types.unused_contextual_input",
                located,
                None,
                "types.unused_contextual_input",
            ),
            (
                TypeDiagnosticV2::WrongNodeFactKind {
                    node_id,
                    location: primary,
                },
                "types.wrong_resolved_input_kind",
                located,
                None,
                "types.wrong_resolved_input_kind",
            ),
            (
                TypeDiagnosticV2::InvalidBindingDependency {
                    node_id,
                    target_node_id,
                    location: primary,
                },
                "types.invalid_binding_dependency",
                located,
                None,
                "types.invalid_binding_dependency",
            ),
            (
                TypeDiagnosticV2::UnavailableBindingDependency {
                    node_id,
                    target_node_id,
                    location: primary,
                },
                "types.unavailable_binding_dependency",
                located,
                None,
                "types.unavailable_binding_dependency",
            ),
            (
                TypeDiagnosticV2::InvalidDeferredRelationRow {
                    root: node_id,
                    location: primary,
                },
                "types.invalid_deferred_relation_row",
                located,
                None,
                "types.invalid_deferred_relation_row",
            ),
            (
                TypeDiagnosticV2::InvalidRelationPreEvaluationPlan {
                    root: node_id,
                    location: primary,
                },
                "types.invalid_relation_pre_evaluation_plan",
                located,
                None,
                "types.invalid_relation_pre_evaluation_plan",
            ),
            (
                TypeDiagnosticV2::UnavailableRelationRowDependency {
                    root: node_id,
                    target: target_node_id,
                    location: primary,
                },
                "types.unavailable_relation_row_dependency",
                located,
                None,
                "types.unavailable_relation_row_dependency",
            ),
            (
                TypeDiagnosticV2::InvalidRelationPredicateInput {
                    node_id,
                    location: primary,
                },
                "types.invalid_relation_predicate_input",
                located,
                None,
                "types.invalid_relation_predicate_input",
            ),
            (
                TypeDiagnosticV2::CyclicDeclarationDependency {
                    node_id,
                    location: primary,
                },
                "types.cyclic_declaration_dependency",
                located,
                None,
                "types.cyclic_declaration_dependency",
            ),
            (
                TypeDiagnosticV2::InvalidRuleTypeBudget,
                "types.invalid_rule_type_budget",
                None,
                None,
                "types.invalid_rule_type_budget",
            ),
            (
                TypeDiagnosticV2::InvalidRuleInput { location: primary },
                "types.invalid_rule_input",
                located,
                None,
                "types.invalid_rule_input",
            ),
            (
                TypeDiagnosticV2::InvalidRuleHead {
                    node_id,
                    location: primary,
                },
                "types.invalid_rule_head",
                located,
                None,
                "types.invalid_rule_head",
            ),
            (
                TypeDiagnosticV2::UnknownRuleCallable { location: primary },
                "types.unknown_rule_callable",
                located,
                None,
                "types.unknown_rule_callable",
            ),
            (
                TypeDiagnosticV2::RuleTypeResourceLimit {
                    dimension: "steps",
                    location: primary,
                },
                "types.rule_type_resource_limit",
                located,
                None,
                "types.rule_type_resource_limit",
            ),
            (
                TypeDiagnosticV2::ContradictoryRuleType { location: primary },
                "types.contradictory_rule_type",
                located,
                None,
                "types.contradictory_rule_type",
            ),
            (
                TypeDiagnosticV2::AmbiguousRuleType {
                    location: primary,
                    candidates: 2,
                },
                "types.ambiguous_rule_type",
                located,
                None,
                "types.ambiguous_rule_type",
            ),
            (
                TypeDiagnosticV2::UnanchoredRecursiveRule { location: primary },
                "types.unanchored_recursive_rule",
                located,
                None,
                "types.unanchored_recursive_rule",
            ),
            (
                TypeDiagnosticV2::RulePredicateArity { location: primary },
                "types.rule_predicate_arity",
                located,
                None,
                "types.rule_predicate_arity",
            ),
            (
                TypeDiagnosticV2::RelationExistsInRule { location: primary },
                "types.relation_exists_in_rule",
                located,
                None,
                "types.relation_exists_in_rule",
            ),
            (
                TypeDiagnosticV2::SourceRegistryMismatch { location: primary },
                "types.source_registry_mismatch",
                located,
                None,
                "types.source_registry_mismatch",
            ),
            (
                TypeDiagnosticV2::InvalidResolvedIdentity,
                "types.invalid_resolved_identity",
                None,
                None,
                "types.invalid_resolved_identity",
            ),
            (
                TypeDiagnosticV2::HostPredicateIdentityLimit,
                "types.host_predicate_identity_limit",
                None,
                None,
                "types.host_predicate_identity_limit",
            ),
            (
                TypeDiagnosticV2::UnknownOperator {
                    node_id,
                    symbol: "??".into(),
                    location: primary,
                },
                "types.unknown_operator",
                located,
                None,
                "types.unknown_operator",
            ),
            (
                TypeDiagnosticV2::UnknownBuiltin {
                    node_id,
                    name: "missing".into(),
                    location: primary,
                },
                "types.unknown_builtin",
                located,
                None,
                "types.unknown_builtin",
            ),
            (
                TypeDiagnosticV2::BadOverload {
                    node_id,
                    name: "call".into(),
                    operands: vec![ValueTypeV1::int()],
                    location: primary,
                },
                "types.bad_overload",
                located,
                None,
                "types.bad_overload: no overload for `call`",
            ),
            (
                TypeDiagnosticV2::BadProjection {
                    node_id,
                    base: ValueTypeV1::int(),
                    field: "field".into(),
                    location: primary,
                },
                "types.bad_projection",
                located,
                None,
                "types.bad_projection: cannot project field `field`",
            ),
            (
                TypeDiagnosticV2::HeterogeneousCollection {
                    node_id,
                    expected: ValueTypeV1::int(),
                    actual: ValueTypeV1::string(),
                    location: primary,
                },
                "types.heterogeneous_collection",
                located,
                None,
                "types.heterogeneous_collection",
            ),
            (
                TypeDiagnosticV2::InvalidStarArgument {
                    node_id,
                    location: primary,
                },
                "types.invalid_star_argument",
                located,
                None,
                "types.invalid_star_argument",
            ),
            (
                TypeDiagnosticV2::RegistryDuplicateId {
                    registry: "operator",
                    id: 4,
                },
                "types.registry_duplicate_id",
                None,
                None,
                "types.registry_duplicate_id",
            ),
            (
                TypeDiagnosticV2::RelationUsedAsValue,
                "types.relation_used_as_value",
                None,
                None,
                "types.relation_used_as_value",
            ),
            (
                TypeDiagnosticV2::TypeShapeLimit,
                "types.type_shape_limit",
                None,
                None,
                "types.type_shape_limit",
            ),
            (
                TypeDiagnosticV2::RedundantOptional,
                "types.redundant_optional",
                None,
                None,
                "types.redundant_optional",
            ),
        ];

        assert_eq!(
            cases.len(),
            43,
            "every TypeDiagnosticV2 variant must be pinned exactly once"
        );
        assert_eq!(
            cases
                .iter()
                .map(|(diagnostic, ..)| diagnostic.code())
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            43,
            "each TypeDiagnosticV2 variant code must appear exactly once"
        );
        for (diagnostic, code, expected_primary, expected_secondary, display) in cases {
            assert_eq!(diagnostic.code(), code, "wrong code for {diagnostic:?}");
            assert_eq!(
                diagnostic.primary(),
                expected_primary,
                "wrong primary location for {diagnostic:?}"
            );
            assert_eq!(
                diagnostic.secondary(),
                expected_secondary,
                "wrong secondary location for {diagnostic:?}"
            );
            assert_eq!(
                diagnostic.to_string(),
                display,
                "wrong display for {diagnostic:?}"
            );
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeAuthorityDiagnosticV2 {
    InvalidAuthorityBudget,
    ResourceExhausted {
        dimension: &'static str,
        limit: usize,
        observed: usize,
    },
    DuplicateRelationSource {
        name: String,
        primary: LocatedSpanV2,
        secondary: LocatedSpanV2,
    },
    DuplicateRelationSourceId {
        id: String,
        primary: LocatedSpanV2,
        secondary: LocatedSpanV2,
    },
    InvalidSourceLocation {
        location: LocatedSpanV2,
    },
    SourceRegistryMismatch {
        location: LocatedSpanV2,
    },
    InvalidCanonicalName {
        name: String,
        location: LocatedSpanV2,
    },
    InvalidRelationAuthority {
        authority: &'static str,
        location: LocatedSpanV2,
    },
    InvalidRelationType(TypeDiagnosticV2),
}

impl TypeAuthorityDiagnosticV2 {
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::InvalidAuthorityBudget => "types.invalid_authority_budget",
            Self::ResourceExhausted { .. } => "types.resource_exhausted",
            Self::DuplicateRelationSource { .. } => "types.duplicate_relation_source",
            Self::DuplicateRelationSourceId { .. } => "types.duplicate_relation_source_id",
            Self::InvalidSourceLocation { .. } => "types.invalid_source_location",
            Self::SourceRegistryMismatch { .. } => "types.source_registry_mismatch",
            Self::InvalidCanonicalName { .. } => "types.invalid_canonical_name",
            Self::InvalidRelationAuthority { .. } => "types.invalid_relation_authority",
            Self::InvalidRelationType(error) => error.code(),
        }
    }
}

impl fmt::Display for TypeAuthorityDiagnosticV2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidAuthorityBudget => f.write_str(self.code()),
            Self::ResourceExhausted {
                dimension,
                limit,
                observed,
            } => write!(
                f,
                "{}: {dimension} limit {limit}, observed {observed}",
                self.code()
            ),
            Self::DuplicateRelationSource { name, .. } => write!(
                f,
                "{}: duplicate relation lookup name `{}`",
                self.code(),
                diagnostic_excerpt(name)
            ),
            Self::DuplicateRelationSourceId { id, .. } => {
                write!(
                    f,
                    "{}: duplicate relation source id `{}`",
                    self.code(),
                    diagnostic_excerpt(id)
                )
            }
            Self::InvalidCanonicalName { name, .. } => {
                write!(
                    f,
                    "{}: invalid canonical name `{}`",
                    self.code(),
                    diagnostic_excerpt(name)
                )
            }
            Self::InvalidRelationAuthority { authority, .. } => write!(
                f,
                "{}: invalid `{authority}` identity or shape",
                self.code()
            ),
            Self::InvalidSourceLocation { .. } | Self::SourceRegistryMismatch { .. } => {
                f.write_str(self.code())
            }
            Self::InvalidRelationType(error) => error.fmt(f),
        }
    }
}
impl std::error::Error for TypeAuthorityDiagnosticV2 {}
