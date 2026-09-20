//! Checked typed callable signatures for external predicate and rule authority.

use crate::budget::ModuleBudgetSessionV2;
use crate::graph::is_segment;
use crate::ModuleDiagnosticV2;
use quanta_gqlang_v2_hir::{CheckedLocatedSpanV2, LocatedSpanV2, SourceRegistryViewV2};
use quanta_gqlang_v2_types::{
    ExpressionEffectsV2, ResolvedPredicateCallableV2, ResolvedPredicateIdV2, ValueTypeV1,
};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CallableKindV2 {
    Predicate,
    Rule,
}
impl CallableKindV2 {
    #[must_use]
    pub const fn tag(self) -> &'static str {
        match self {
            Self::Predicate => "predicate",
            Self::Rule => "rule",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocatedParameterInputV2 {
    pub name: String,
    pub value_type: ValueTypeV1,
    pub location: CheckedLocatedSpanV2,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignatureInputV2 {
    pub parameters: Vec<LocatedParameterInputV2>,
    pub result: ValueTypeV1,
    pub callable_kind: CallableKindV2,
    pub effects: ExpressionEffectsV2,
    pub location: CheckedLocatedSpanV2,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignatureParameterV2 {
    name: String,
    value_type: ValueTypeV1,
    location: LocatedSpanV2,
}
impl SignatureParameterV2 {
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
    #[must_use]
    pub const fn value_type(&self) -> &ValueTypeV1 {
        &self.value_type
    }
    #[must_use]
    pub const fn location(&self) -> LocatedSpanV2 {
        self.location
    }
}

/// Typed callable declaration. Source locations are excluded from semantic
/// equality so separately authored identical interfaces remain identical.
#[derive(Clone, Debug)]
pub struct SignatureV2 {
    parameters: Vec<SignatureParameterV2>,
    result: ValueTypeV1,
    callable_kind: CallableKindV2,
    effects: ExpressionEffectsV2,
}
impl SignatureV2 {
    pub(crate) fn try_new(
        input: SignatureInputV2,
        source_registry: &SourceRegistryViewV2,
        session: &mut ModuleBudgetSessionV2,
    ) -> Result<Self, ModuleDiagnosticV2> {
        session.charge_node()?;
        check_registry(&input.location, source_registry)?;
        for parameter in &input.parameters {
            session.charge_node()?;
            session.charge_string(&parameter.name)?;
            check_registry(&parameter.location, source_registry)?;
        }
        let mut seen: BTreeMap<String, CheckedLocatedSpanV2> = BTreeMap::new();
        let mut checked = Vec::with_capacity(input.parameters.len());
        for parameter in input.parameters {
            if !is_segment(&parameter.name) {
                return Err(ModuleDiagnosticV2::new(
                    "modules.invalid_signature_parameter",
                    "invalid signature parameter name",
                    &parameter.location,
                ));
            }
            if let Some(original) = seen.get(&parameter.name) {
                return Err(ModuleDiagnosticV2::conflict(
                    "modules.duplicate_signature_parameter",
                    "duplicate signature parameter",
                    &parameter.location,
                    original,
                ));
            }
            seen.insert(parameter.name.clone(), parameter.location.clone());
            checked.push(SignatureParameterV2 {
                name: parameter.name,
                value_type: parameter.value_type,
                location: parameter.location.location(),
            });
        }
        Ok(Self {
            parameters: checked,
            result: input.result,
            callable_kind: input.callable_kind,
            effects: input.effects,
        })
    }
    pub fn try_as_resolved_predicate(
        &self,
        id: ResolvedPredicateIdV2,
    ) -> Result<ResolvedPredicateCallableV2, ModuleDiagnosticV2> {
        ResolvedPredicateCallableV2::try_new(
            id,
            self.parameters
                .iter()
                .map(|parameter| parameter.value_type.clone())
                .collect(),
            self.result.clone(),
            self.effects,
        )
        .map_err(|error| {
            ModuleDiagnosticV2::without_location("modules.invalid_signature", error.to_string())
        })
    }
    #[must_use]
    pub fn parameters(&self) -> &[SignatureParameterV2] {
        &self.parameters
    }
    #[must_use]
    pub const fn result(&self) -> &ValueTypeV1 {
        &self.result
    }
    #[must_use]
    pub const fn callable_kind(&self) -> CallableKindV2 {
        self.callable_kind
    }
    #[must_use]
    pub const fn effects(&self) -> ExpressionEffectsV2 {
        self.effects
    }
}

fn check_registry(
    location: &CheckedLocatedSpanV2,
    source_registry: &SourceRegistryViewV2,
) -> Result<(), ModuleDiagnosticV2> {
    if location.registry_identity() != &source_registry.identity() {
        return Err(ModuleDiagnosticV2::new(
            "modules.source_registry_mismatch",
            "signature location belongs to another source registry",
            location,
        ));
    }
    Ok(())
}
impl PartialEq for SignatureV2 {
    fn eq(&self, other: &Self) -> bool {
        self.parameters
            .iter()
            .map(|p| (&p.name, &p.value_type))
            .eq(other.parameters.iter().map(|p| (&p.name, &p.value_type)))
            && self.result == other.result
            && self.callable_kind == other.callable_kind
            && self.effects == other.effects
    }
}
impl Eq for SignatureV2 {}
