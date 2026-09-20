use crate::{TypeDiagnosticV2, ValueTypeV1};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PurityV1 {
    Pure,
    Impure,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DeterminismV1 {
    Deterministic,
    NonDeterministic,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MonotonicityV1 {
    Monotone,
    Antitone,
    NonMonotone,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AggregationClassV1 {
    Scalar,
    Aggregate,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CostClassV1 {
    Constant,
    Linear,
    SuperLinear,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExpressionEffectsV2 {
    pub purity: PurityV1,
    pub determinism: DeterminismV1,
    pub monotonicity: MonotonicityV1,
    pub aggregation: AggregationClassV1,
    pub cost: CostClassV1,
}
impl ExpressionEffectsV2 {
    #[must_use]
    pub const fn pure_scalar() -> Self {
        Self {
            purity: PurityV1::Pure,
            determinism: DeterminismV1::Deterministic,
            monotonicity: MonotonicityV1::Monotone,
            aggregation: AggregationClassV1::Scalar,
            cost: CostClassV1::Constant,
        }
    }

    #[must_use]
    pub(crate) const fn conservative_unknown() -> Self {
        Self {
            purity: PurityV1::Impure,
            determinism: DeterminismV1::NonDeterministic,
            monotonicity: MonotonicityV1::NonMonotone,
            aggregation: AggregationClassV1::Aggregate,
            cost: CostClassV1::SuperLinear,
        }
    }

    #[must_use]
    pub fn join(self, other: Self) -> Self {
        let monotonicity = if self.monotonicity == MonotonicityV1::NonMonotone
            || other.monotonicity == MonotonicityV1::NonMonotone
            || self.monotonicity != other.monotonicity
        {
            MonotonicityV1::NonMonotone
        } else {
            self.monotonicity
        };
        Self {
            purity: self.purity.max(other.purity),
            determinism: self.determinism.max(other.determinism),
            monotonicity,
            aggregation: self.aggregation.max(other.aggregation),
            cost: self.cost.max(other.cost),
        }
    }

    #[must_use]
    pub fn compose(self, children: impl IntoIterator<Item = Self>) -> Self {
        children.into_iter().fold(self, Self::join)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BuiltinIdV2(u32);
impl BuiltinIdV2 {
    #[must_use]
    pub const fn raw(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BuiltinFamilyIdV2(u32);
impl BuiltinFamilyIdV2 {
    #[must_use]
    pub const fn raw(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum CallableSignatureV2 {
    Fixed {
        params: Vec<ValueTypeV1>,
        result: ValueTypeV1,
    },
    AnySingle {
        result: ValueTypeV1,
    },
    AggregateStar {
        result: ValueTypeV1,
    },
}
impl CallableSignatureV2 {
    #[must_use]
    pub(crate) fn result(&self) -> &ValueTypeV1 {
        match self {
            Self::Fixed { result, .. }
            | Self::AnySingle { result }
            | Self::AggregateStar { result } => result,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct BuiltinEntryV2 {
    pub id: BuiltinIdV2,
    pub family_id: BuiltinFamilyIdV2,
    pub name: String,
    pub signature: CallableSignatureV2,
    pub effects: ExpressionEffectsV2,
}

#[derive(Clone, Debug)]
pub struct BuiltinRegistryV2 {
    entries: Vec<BuiltinEntryV2>,
}

impl BuiltinRegistryV2 {
    fn try_from_entries(entries: Vec<BuiltinEntryV2>) -> Result<Self, TypeDiagnosticV2> {
        let mut ids = BTreeSet::new();
        let mut names = BTreeMap::new();
        let mut families = BTreeMap::new();
        let mut admitted_domains: Vec<(&str, BuiltinFamilyIdV2, &CallableSignatureV2)> = Vec::new();
        for entry in &entries {
            if !ids.insert(entry.id) {
                return Err(TypeDiagnosticV2::RegistryDuplicateId {
                    registry: "builtin",
                    id: entry.id.raw(),
                });
            }
            if names
                .insert(entry.name.as_str(), entry.family_id)
                .is_some_and(|family| family != entry.family_id)
                || families
                    .insert(entry.family_id, entry.name.as_str())
                    .is_some_and(|name| name != entry.name)
            {
                return Err(TypeDiagnosticV2::RegistryDuplicateId {
                    registry: "builtin_family",
                    id: entry.id.raw(),
                });
            }
            if admitted_domains.iter().any(|(name, family, signature)| {
                *name == entry.name
                    && *family == entry.family_id
                    && domains_overlap(signature, &entry.signature)
            }) {
                return Err(TypeDiagnosticV2::RegistryDuplicateId {
                    registry: "builtin_signature",
                    id: entry.id.raw(),
                });
            }
            admitted_domains.push((&entry.name, entry.family_id, &entry.signature));
        }
        Ok(Self { entries })
    }

    pub fn try_closed() -> Result<Self, TypeDiagnosticV2> {
        let aggregate = ExpressionEffectsV2 {
            purity: PurityV1::Pure,
            determinism: DeterminismV1::Deterministic,
            monotonicity: MonotonicityV1::Monotone,
            aggregation: AggregationClassV1::Aggregate,
            cost: CostClassV1::Linear,
        };
        let scalar = ExpressionEffectsV2::pure_scalar();
        let linear = ExpressionEffectsV2 {
            cost: CostClassV1::Linear,
            ..scalar
        };
        let explain = ExpressionEffectsV2 {
            monotonicity: MonotonicityV1::NonMonotone,
            cost: CostClassV1::SuperLinear,
            ..scalar
        };
        let now = ExpressionEffectsV2 {
            purity: PurityV1::Impure,
            determinism: DeterminismV1::NonDeterministic,
            monotonicity: MonotonicityV1::NonMonotone,
            ..scalar
        };
        let path = ValueTypeV1::scalar(crate::ScalarTyV1::Path);
        let location = ValueTypeV1::scalar(crate::ScalarTyV1::Location);
        let proof = ValueTypeV1::scalar(crate::ScalarTyV1::ProofTree);
        let entries = vec![
            e(
                1,
                1,
                "count",
                CallableSignatureV2::AnySingle {
                    result: ValueTypeV1::int(),
                },
                aggregate,
            ),
            e(
                2,
                1,
                "count",
                CallableSignatureV2::AggregateStar {
                    result: ValueTypeV1::int(),
                },
                aggregate,
            ),
            e(
                3,
                2,
                "sum",
                fixed(vec![ValueTypeV1::int()], ValueTypeV1::int()),
                aggregate,
            ),
            e(
                4,
                2,
                "sum",
                fixed(vec![ValueTypeV1::float()], ValueTypeV1::float()),
                aggregate,
            ),
            e(
                5,
                3,
                "min",
                fixed(vec![ValueTypeV1::int()], ValueTypeV1::int()),
                aggregate,
            ),
            e(
                6,
                3,
                "min",
                fixed(vec![ValueTypeV1::float()], ValueTypeV1::float()),
                aggregate,
            ),
            e(
                7,
                4,
                "max",
                fixed(vec![ValueTypeV1::int()], ValueTypeV1::int()),
                aggregate,
            ),
            e(
                8,
                4,
                "max",
                fixed(vec![ValueTypeV1::float()], ValueTypeV1::float()),
                aggregate,
            ),
            e(
                9,
                5,
                "avg",
                fixed(vec![ValueTypeV1::int()], ValueTypeV1::float()),
                aggregate,
            ),
            e(
                10,
                5,
                "avg",
                fixed(vec![ValueTypeV1::float()], ValueTypeV1::float()),
                aggregate,
            ),
            e(
                11,
                6,
                "starts_with",
                fixed(
                    vec![ValueTypeV1::string(), ValueTypeV1::string()],
                    ValueTypeV1::bool(),
                ),
                linear,
            ),
            e(
                12,
                7,
                "ends_with",
                fixed(
                    vec![ValueTypeV1::string(), ValueTypeV1::string()],
                    ValueTypeV1::bool(),
                ),
                linear,
            ),
            e(
                13,
                8,
                "contains",
                fixed(
                    vec![ValueTypeV1::string(), ValueTypeV1::string()],
                    ValueTypeV1::bool(),
                ),
                linear,
            ),
            e(
                14,
                9,
                "length",
                fixed(vec![ValueTypeV1::string()], ValueTypeV1::int()),
                linear,
            ),
            e(
                15,
                10,
                "is_null",
                CallableSignatureV2::AnySingle {
                    result: ValueTypeV1::bool(),
                },
                scalar,
            ),
            e(
                16,
                11,
                "size",
                fixed(
                    vec![ValueTypeV1::try_list(ValueTypeV1::int())?],
                    ValueTypeV1::int(),
                ),
                linear,
            ),
            e(
                17,
                12,
                "path_length",
                fixed(vec![path.clone()], ValueTypeV1::int()),
                linear,
            ),
            e(
                18,
                13,
                "path_locations",
                fixed(vec![path], ValueTypeV1::try_list(location)?),
                linear,
            ),
            e(
                19,
                14,
                "explain",
                fixed(vec![ValueTypeV1::bool()], proof.clone()),
                explain,
            ),
            e(
                20,
                15,
                "proof_size",
                fixed(vec![proof], ValueTypeV1::int()),
                linear,
            ),
            e(
                21,
                16,
                "now",
                fixed(vec![], ValueTypeV1::scalar(crate::ScalarTyV1::Duration)),
                now,
            ),
        ];
        Self::try_from_entries(entries)
    }
    pub(crate) fn lookup_family(
        &self,
        family: BuiltinFamilyIdV2,
    ) -> impl Iterator<Item = &BuiltinEntryV2> {
        self.entries
            .iter()
            .filter(move |entry| entry.family_id == family)
    }

    pub(crate) fn agreed_fixed_parameters(
        &self,
        family: BuiltinFamilyIdV2,
        arity: usize,
    ) -> Vec<Option<ValueTypeV1>> {
        let matching = self
            .lookup_family(family)
            .filter_map(|entry| match &entry.signature {
                CallableSignatureV2::Fixed { params, .. } if params.len() == arity => Some(params),
                CallableSignatureV2::Fixed { .. }
                | CallableSignatureV2::AnySingle { .. }
                | CallableSignatureV2::AggregateStar { .. } => None,
            })
            .collect::<Vec<_>>();
        if matching.is_empty() {
            return vec![None; arity];
        }
        (0..arity)
            .map(|index| {
                let first = matching.first().and_then(|params| params.get(index))?;
                matching
                    .iter()
                    .all(|params| params.get(index) == Some(first))
                    .then(|| first.clone())
            })
            .collect()
    }
    #[must_use]
    pub fn family_id(&self, name: &str) -> Option<BuiltinFamilyIdV2> {
        self.entries
            .iter()
            .find(|entry| entry.name == name)
            .map(|entry| entry.family_id)
    }
}

fn domains_overlap(left: &CallableSignatureV2, right: &CallableSignatureV2) -> bool {
    match (left, right) {
        (CallableSignatureV2::AggregateStar { .. }, CallableSignatureV2::AggregateStar { .. }) => {
            true
        }
        (CallableSignatureV2::AnySingle { .. }, CallableSignatureV2::AnySingle { .. }) => true,
        (CallableSignatureV2::AnySingle { .. }, CallableSignatureV2::Fixed { params, .. })
        | (CallableSignatureV2::Fixed { params, .. }, CallableSignatureV2::AnySingle { .. }) => {
            params.len() == 1
        }
        (
            CallableSignatureV2::Fixed { params: left, .. },
            CallableSignatureV2::Fixed { params: right, .. },
        ) => left == right,
        _ => false,
    }
}

fn fixed(params: Vec<ValueTypeV1>, result: ValueTypeV1) -> CallableSignatureV2 {
    CallableSignatureV2::Fixed { params, result }
}
fn e(
    id: u32,
    family: u32,
    name: &str,
    signature: CallableSignatureV2,
    effects: ExpressionEffectsV2,
) -> BuiltinEntryV2 {
    BuiltinEntryV2 {
        id: BuiltinIdV2(id),
        family_id: BuiltinFamilyIdV2(family),
        name: name.into(),
        signature,
        effects,
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, clippy::expect_used, reason = "owner assertions")]
mod tests {
    use super::*;
    #[test]
    fn closed_registry_preserves_builtin_families_metadata_and_ids() {
        let registry = BuiltinRegistryV2::try_closed().expect("valid static registry");
        for name in [
            "count",
            "sum",
            "min",
            "max",
            "avg",
            "starts_with",
            "ends_with",
            "contains",
            "length",
            "is_null",
            "size",
            "path_length",
            "path_locations",
            "explain",
            "proof_size",
            "now",
        ] {
            assert!(registry.family_id(name).is_some(), "missing {name}");
        }
        assert_eq!(registry.entries.len(), 21);
        assert_eq!(
            registry
                .entries
                .iter()
                .map(|entry| entry.id)
                .collect::<BTreeSet<_>>()
                .len(),
            21
        );
        let now = registry
            .entries
            .iter()
            .find(|entry| entry.name == "now")
            .expect("now");
        assert_eq!(now.effects.determinism, DeterminismV1::NonDeterministic);
        let count = registry
            .lookup_family(registry.family_id("count").expect("count"))
            .collect::<Vec<_>>();
        assert!(count
            .iter()
            .any(|entry| matches!(entry.signature, CallableSignatureV2::AggregateStar { .. })));
    }

    #[test]
    fn closed_registry_pins_every_id_name_signature_and_effect_tuple() {
        fn signature_key(signature: &CallableSignatureV2) -> String {
            match signature {
                CallableSignatureV2::Fixed { params, result } => format!(
                    "({})->{result}",
                    params
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(",")
                ),
                CallableSignatureV2::AnySingle { result } => format!("any->{result}"),
                CallableSignatureV2::AggregateStar { result } => format!("star->{result}"),
            }
        }
        let scalar = ExpressionEffectsV2::pure_scalar();
        let linear = ExpressionEffectsV2 {
            cost: CostClassV1::Linear,
            ..scalar
        };
        let aggregate = ExpressionEffectsV2 {
            aggregation: AggregationClassV1::Aggregate,
            cost: CostClassV1::Linear,
            ..scalar
        };
        let explain = ExpressionEffectsV2 {
            monotonicity: MonotonicityV1::NonMonotone,
            cost: CostClassV1::SuperLinear,
            ..scalar
        };
        let now = ExpressionEffectsV2 {
            purity: PurityV1::Impure,
            determinism: DeterminismV1::NonDeterministic,
            monotonicity: MonotonicityV1::NonMonotone,
            ..scalar
        };
        let expected = vec![
            (1, 1, "count", "any->int", aggregate),
            (2, 1, "count", "star->int", aggregate),
            (3, 2, "sum", "(int)->int", aggregate),
            (4, 2, "sum", "(float)->float", aggregate),
            (5, 3, "min", "(int)->int", aggregate),
            (6, 3, "min", "(float)->float", aggregate),
            (7, 4, "max", "(int)->int", aggregate),
            (8, 4, "max", "(float)->float", aggregate),
            (9, 5, "avg", "(int)->float", aggregate),
            (10, 5, "avg", "(float)->float", aggregate),
            (11, 6, "starts_with", "(string,string)->bool", linear),
            (12, 7, "ends_with", "(string,string)->bool", linear),
            (13, 8, "contains", "(string,string)->bool", linear),
            (14, 9, "length", "(string)->int", linear),
            (15, 10, "is_null", "any->bool", scalar),
            (16, 11, "size", "(list<int>)->int", linear),
            (17, 12, "path_length", "(path)->int", linear),
            (18, 13, "path_locations", "(path)->list<location>", linear),
            (19, 14, "explain", "(bool)->proof_tree", explain),
            (20, 15, "proof_size", "(proof_tree)->int", linear),
            (21, 16, "now", "()->duration", now),
        ];
        let registry = BuiltinRegistryV2::try_closed().expect("valid static registry");
        let actual = registry
            .entries
            .iter()
            .map(|entry| {
                (
                    entry.id.raw(),
                    entry.family_id.raw(),
                    entry.name.as_str(),
                    signature_key(&entry.signature),
                    entry.effects,
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            actual,
            expected
                .into_iter()
                .map(|(id, family, name, signature, effects)| {
                    (id, family, name, signature.to_owned(), effects)
                })
                .collect::<Vec<_>>()
        );
    }
    #[test]
    fn duplicate_builtin_id_and_signature_reject() {
        let effects = ExpressionEffectsV2::pure_scalar();
        let first = e(
            1,
            1,
            "f",
            fixed(vec![ValueTypeV1::int()], ValueTypeV1::int()),
            effects,
        );
        let duplicate_id = e(1, 2, "g", fixed(vec![], ValueTypeV1::bool()), effects);
        assert_eq!(
            BuiltinRegistryV2::try_from_entries(vec![first.clone(), duplicate_id])
                .unwrap_err()
                .code(),
            "types.registry_duplicate_id"
        );
        let duplicate_signature = e(
            2,
            1,
            "f",
            fixed(vec![ValueTypeV1::int()], ValueTypeV1::float()),
            effects,
        );
        assert_eq!(
            BuiltinRegistryV2::try_from_entries(vec![first, duplicate_signature])
                .unwrap_err()
                .code(),
            "types.registry_duplicate_id"
        );
    }
    #[test]
    fn registry_rejects_family_aliases_and_overlapping_any_single() {
        let effects = ExpressionEffectsV2::pure_scalar();
        let first = e(
            1,
            1,
            "f",
            CallableSignatureV2::AnySingle {
                result: ValueTypeV1::int(),
            },
            effects,
        );
        let overlap = e(
            2,
            1,
            "f",
            fixed(vec![ValueTypeV1::string()], ValueTypeV1::bool()),
            effects,
        );
        assert_eq!(
            BuiltinRegistryV2::try_from_entries(vec![first.clone(), overlap])
                .unwrap_err()
                .code(),
            "types.registry_duplicate_id"
        );
        let aliased_name = e(3, 2, "f", fixed(vec![], ValueTypeV1::bool()), effects);
        assert_eq!(
            BuiltinRegistryV2::try_from_entries(vec![first.clone(), aliased_name])
                .unwrap_err()
                .code(),
            "types.registry_duplicate_id"
        );
        let aliased_family = e(4, 1, "g", fixed(vec![], ValueTypeV1::bool()), effects);
        assert_eq!(
            BuiltinRegistryV2::try_from_entries(vec![first, aliased_family])
                .unwrap_err()
                .code(),
            "types.registry_duplicate_id"
        );
    }
}
