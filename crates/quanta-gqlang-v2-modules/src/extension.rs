//! Declarative extension / model packs and their validation + conflict rules
//! (deliverables 6, 7; `D28`).
//!
//! An extension pack adds host-agnostic, declarative SEMANTIC FACTS about
//! modeled targets (e.g. "this framework symbol is a taint source"). It is:
//!
//! - frontend-owned and host-agnostic: facts are opaque `(target, attribute,
//!   value)` rows, never runtime substrate DTOs or backend wiring (ticket
//!   pinned semantics 3, `D7`/`D14`),
//! - declarative and additive: a pack adds rows; it cannot override the closed
//!   builtin/operator/type core (ticket pinned semantics 8 — there is no API
//!   here to mutate core operators),
//! - versioned + testable: rows carry a `model_version` and validate against a
//!   fixed schema.
//!
//! Conflict rule (`D28`): when two packs assert the SAME `(target, attribute)`
//! with DIFFERENT values, composition fails closed with
//! [`ModuleDiagnosticV1::ModelConflict`] — there is no last-wins. Identical rows
//! from two packs compose silently (idempotent agreement).

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use crate::diagnostic::ModuleDiagnosticV1;
use crate::version::VersionV1;

/// One declarative model fact: an attribute/value asserted about a target.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ModelFactV1 {
    /// The modeled target key (e.g. fully-qualified framework symbol).
    pub target: String,
    /// The attribute being asserted (e.g. `taint_kind`).
    pub attribute: String,
    /// The asserted value.
    pub value: String,
}

impl ModelFactV1 {
    /// Construct a model fact.
    #[must_use]
    pub fn new(
        target: impl Into<String>,
        attribute: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        Self {
            target: target.into(),
            attribute: attribute.into(),
            value: value.into(),
        }
    }
}

/// A declarative extension/model pack: a versioned, named set of model facts
/// plus the closed schema (attribute -> allowed values) it must conform to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelPackV1 {
    /// The model pack name (used in conflict diagnostics).
    pub name: String,
    /// The model-data schema version (additive/versioned per deliverable 6).
    pub model_version: VersionV1,
    /// The declared facts, canonically ordered/de-duplicated.
    pub facts: BTreeSet<ModelFactV1>,
}

impl ModelPackV1 {
    /// Construct an empty model pack.
    #[must_use]
    pub fn new(name: impl Into<String>, model_version: VersionV1) -> Self {
        Self {
            name: name.into(),
            model_version,
            facts: BTreeSet::new(),
        }
    }

    /// Add a model fact (idempotent).
    #[must_use]
    pub fn with_fact(mut self, fact: ModelFactV1) -> Self {
        let _ = self.facts.insert(fact);
        self
    }

    /// Validate every fact against `schema`: each fact's attribute must be a
    /// declared schema key, and its value must be in that attribute's allowed
    /// set. Fail-closed with [`ModuleDiagnosticV1::InvalidModelData`] on the
    /// first sorted violation.
    pub fn validate(&self, schema: &ModelSchemaV1) -> Result<(), ModuleDiagnosticV1> {
        for fact in &self.facts {
            match schema.allowed.get(&fact.attribute) {
                Some(allowed) if allowed.contains(&fact.value) => {}
                Some(_) => {
                    return Err(ModuleDiagnosticV1::InvalidModelData {
                        pack: self.name.clone(),
                        detail: format!(
                            "attribute `{}` value `{}` is not in the schema's allowed set",
                            fact.attribute, fact.value
                        ),
                    });
                }
                None => {
                    return Err(ModuleDiagnosticV1::InvalidModelData {
                        pack: self.name.clone(),
                        detail: format!(
                            "attribute `{}` is not declared in the schema",
                            fact.attribute
                        ),
                    });
                }
            }
        }
        Ok(())
    }
}

/// The closed schema model packs validate against: each attribute maps to its
/// allowed value set. An attribute absent from the schema is rejected; this is
/// what keeps extension data additive-but-closed rather than free-form.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ModelSchemaV1 {
    allowed: BTreeMap<String, BTreeSet<String>>,
}

impl ModelSchemaV1 {
    /// An empty schema (admits nothing).
    #[must_use]
    pub fn new() -> Self {
        Self {
            allowed: BTreeMap::new(),
        }
    }

    /// Declare that `attribute` admits `value` (builder style; cumulative).
    #[must_use]
    pub fn allow(mut self, attribute: impl Into<String>, value: impl Into<String>) -> Self {
        let _ = self
            .allowed
            .entry(attribute.into())
            .or_default()
            .insert(value.into());
        self
    }
}

/// Compose multiple validated model packs into a single fact base, failing
/// closed on incompatible overlap (`D28`).
///
/// Each pack is first validated against `schema`. Then facts are merged: two
/// packs asserting the same `(target, attribute)` with the SAME value agree
/// (idempotent); with DIFFERENT values they conflict and reject with
/// [`ModuleDiagnosticV1::ModelConflict`]. Packs are processed in sorted name
/// order so the diagnostic is deterministic regardless of input order.
pub fn compose_models(
    packs: &[ModelPackV1],
    schema: &ModelSchemaV1,
) -> Result<ComposedModelV1, ModuleDiagnosticV1> {
    // Deterministic processing order, independent of caller-supplied order.
    let mut ordered: Vec<&ModelPackV1> = packs.iter().collect();
    ordered.sort_by(|a, b| a.name.cmp(&b.name));

    for pack in &ordered {
        pack.validate(schema)?;
    }

    // (target, attribute) -> (value, asserting pack).
    let mut merged: BTreeMap<(String, String), (String, String)> = BTreeMap::new();
    for pack in &ordered {
        for fact in &pack.facts {
            let key = (fact.target.clone(), fact.attribute.clone());
            match merged.get(&key) {
                Some((existing_value, existing_pack)) if *existing_value == fact.value => {
                    let _ = existing_pack;
                }
                Some((existing_value, existing_pack)) => {
                    // Conflicting values for the same target attribute.
                    let mut packs_pair = vec![existing_pack.clone(), pack.name.clone()];
                    let mut values_pair = vec![existing_value.clone(), fact.value.clone()];
                    // Sort the value/pack pair jointly for a stable rendering.
                    if packs_pair.first() > packs_pair.get(1) {
                        packs_pair.reverse();
                        values_pair.reverse();
                    }
                    return Err(ModuleDiagnosticV1::ModelConflict {
                        target: fact.target.clone(),
                        attribute: fact.attribute.clone(),
                        values: values_pair,
                        packs: packs_pair,
                    });
                }
                None => {
                    let _ = merged.insert(key, (fact.value.clone(), pack.name.clone()));
                }
            }
        }
    }

    let facts = merged
        .into_iter()
        .map(|((target, attribute), (value, _))| ModelFactV1 {
            target,
            attribute,
            value,
        })
        .collect();

    Ok(ComposedModelV1 { facts })
}

/// The composed fact base of several agreeing model packs.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ComposedModelV1 {
    facts: BTreeSet<ModelFactV1>,
}

impl ComposedModelV1 {
    /// All composed facts, canonically ordered.
    pub fn facts(&self) -> impl Iterator<Item = &ModelFactV1> {
        self.facts.iter()
    }

    /// Number of composed facts.
    #[must_use]
    pub fn len(&self) -> usize {
        self.facts.len()
    }

    /// Whether the composed model is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.facts.is_empty()
    }
}
