use crate::{TypeDiagnosticV2, ValueTypeV1};
use quanta_gqlang_v2_hir::{
    FrontendRequestMaterializationRootV3, FrontendRetainedStoragePermitV3,
    FrontendStorageRefusalV3, SourceIdV2,
};

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

    /// Source predicate invocation is not an aggregate call. All other
    /// effects remain conservative until a body-effect proof exists.
    #[must_use]
    pub(crate) const fn conservative_source_predicate() -> Self {
        Self {
            purity: PurityV1::Impure,
            determinism: DeterminismV1::NonDeterministic,
            monotonicity: MonotonicityV1::NonMonotone,
            aggregation: AggregationClassV1::Scalar,
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

    /// Stable semantic name of every overload admitted by the closed registry.
    #[must_use]
    pub const fn name(self) -> Option<&'static str> {
        match self.0 {
            1 | 2 => Some("count"),
            3 | 4 => Some("sum"),
            5 | 6 => Some("min"),
            7 | 8 => Some("max"),
            9 | 10 => Some("avg"),
            11 => Some("starts_with"),
            12 => Some("ends_with"),
            13 => Some("contains"),
            14 => Some("length"),
            15 => Some("is_null"),
            16 => Some("size"),
            17 => Some("path_length"),
            18 => Some("path_locations"),
            19 => Some("explain"),
            20 => Some("proof_size"),
            21 => Some("now"),
            _ => None,
        }
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

/// Closed builtin inventory retained by the original source request. The
/// cloneable V2 registry stays private; readers expose copied scalar facts.
pub struct AdmittedBuiltinRegistryV3 {
    inner: std::sync::Arc<AdmittedBuiltinRegistryInnerV3>,
}

pub(crate) struct AdmittedBuiltinRegistryInnerV3 {
    pub(crate) registry: BuiltinRegistryV2,
    _storage: FrontendRetainedStoragePermitV3,
}

impl std::fmt::Debug for AdmittedBuiltinRegistryInnerV3 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("AdmittedBuiltinRegistryInnerV3")
            .field("registry", &self.registry)
            .finish_non_exhaustive()
    }
}

/// Scalar observations carry no source authority on their own.
pub trait BuiltinEvidenceReaderV3 {
    fn family_id(&self, name: &str) -> Option<BuiltinFamilyIdV2>;
    fn effects_of_resolved(&self, id: BuiltinIdV2) -> Option<ExpressionEffectsV2>;
}

impl BuiltinEvidenceReaderV3 for BuiltinRegistryV2 {
    fn family_id(&self, name: &str) -> Option<BuiltinFamilyIdV2> {
        BuiltinRegistryV2::family_id(self, name)
    }

    fn effects_of_resolved(&self, id: BuiltinIdV2) -> Option<ExpressionEffectsV2> {
        BuiltinRegistryV2::effects_of_resolved(self, id)
    }
}

impl BuiltinEvidenceReaderV3 for AdmittedBuiltinRegistryV3 {
    fn family_id(&self, name: &str) -> Option<BuiltinFamilyIdV2> {
        self.inner.registry.family_id(name)
    }

    fn effects_of_resolved(&self, id: BuiltinIdV2) -> Option<ExpressionEffectsV2> {
        self.inner.registry.effects_of_resolved(id)
    }
}

impl AdmittedBuiltinRegistryV3 {
    /// Recheck the original builtin debit against the exact issued request.
    pub fn validate_owner_v3(
        &self,
        root: &FrontendRequestMaterializationRootV3<'_>,
        source_id: SourceIdV2,
        source: &str,
    ) -> Result<(), FrontendStorageRefusalV3> {
        self.inner
            ._storage
            .validate_owner_v3(root, source_id, source)
    }

    pub(crate) fn shared_inner(&self) -> std::sync::Arc<AdmittedBuiltinRegistryInnerV3> {
        std::sync::Arc::clone(&self.inner)
    }
    #[must_use]
    pub fn family_id(&self, name: &str) -> Option<BuiltinFamilyIdV2> {
        self.inner.registry.family_id(name)
    }

    #[must_use]
    pub fn effects_of_resolved(&self, id: BuiltinIdV2) -> Option<ExpressionEffectsV2> {
        self.inner.registry.effects_of_resolved(id)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BuiltinRegistryRequestErrorV3 {
    Storage(FrontendStorageRefusalV3),
    SourceMismatch,
    DemandOverflow,
    Registry(TypeDiagnosticV2),
}

impl BuiltinRegistryV2 {
    pub(crate) fn all_entries(&self) -> impl Iterator<Item = &BuiltinEntryV2> {
        self.entries.iter()
    }
    #[expect(
        clippy::indexing_slicing,
        reason = "enumerate index is within the same entries slice"
    )]
    fn try_from_entries(entries: Vec<BuiltinEntryV2>) -> Result<Self, TypeDiagnosticV2> {
        for (index, entry) in entries.iter().enumerate() {
            let earlier = &entries[..index];
            if earlier.iter().any(|prior| prior.id == entry.id) {
                return Err(TypeDiagnosticV2::RegistryDuplicateId {
                    registry: "builtin",
                    id: entry.id.raw(),
                });
            }
            if earlier.iter().any(|prior| {
                (prior.name == entry.name && prior.family_id != entry.family_id)
                    || (prior.family_id == entry.family_id && prior.name != entry.name)
            }) {
                return Err(TypeDiagnosticV2::RegistryDuplicateId {
                    registry: "builtin_family",
                    id: entry.id.raw(),
                });
            }
            if earlier.iter().any(|prior| {
                prior.name == entry.name
                    && prior.family_id == entry.family_id
                    && domains_overlap(&prior.signature, &entry.signature)
            }) {
                return Err(TypeDiagnosticV2::RegistryDuplicateId {
                    registry: "builtin_signature",
                    id: entry.id.raw(),
                });
            }
        }
        Ok(Self { entries })
    }

    /// Reserve the pinned 21-entry builtin inventory before any String, Vec,
    /// or nested type Box is born. The inventory is defined by `try_closed`;
    /// adding an entry requires updating this source-count census together.
    pub fn try_closed_with_request_owner_v3(
        root: &FrontendRequestMaterializationRootV3<'_>,
        source_id: SourceIdV2,
        source: &str,
        parent: &FrontendRetainedStoragePermitV3,
    ) -> Result<AdmittedBuiltinRegistryV3, BuiltinRegistryRequestErrorV3> {
        parent
            .validate_owner_v3(root, source_id, source)
            .map_err(BuiltinRegistryRequestErrorV3::Storage)?;
        let name_bytes = (1..=21_u32).try_fold(0_usize, |total, id| {
            total.checked_add(BuiltinIdV2(id).name()?.len())
        });
        let retained_bytes = 21_usize
            .checked_mul(std::mem::size_of::<BuiltinEntryV2>())
            .and_then(|bytes| bytes.checked_add(name_bytes?))
            .and_then(|bytes| {
                20_usize
                    .checked_mul(std::mem::size_of::<ValueTypeV1>())
                    .and_then(|params| bytes.checked_add(params))
            })
            .and_then(|bytes| {
                2_usize
                    .checked_mul(std::mem::size_of::<ValueTypeV1>())
                    .and_then(|boxes| bytes.checked_add(boxes))
            })
            .and_then(|bytes| {
                // The Arc payload includes the permit itself, so cloned Type
                // kernels keep its debit until their final shared owner drops.
                std::mem::size_of::<AdmittedBuiltinRegistryInnerV3>()
                    .checked_add(2 * std::mem::size_of::<usize>())
                    .and_then(|arc| {
                        arc.checked_add(std::mem::align_of::<AdmittedBuiltinRegistryInnerV3>() - 1)
                    })
                    .and_then(|arc| bytes.checked_add(arc))
            })
            .ok_or(BuiltinRegistryRequestErrorV3::DemandOverflow)?;
        let storage = root
            .reserve_v3(source_id, source, retained_bytes)
            .map_err(BuiltinRegistryRequestErrorV3::Storage)?;
        let registry = Self::try_closed().map_err(BuiltinRegistryRequestErrorV3::Registry)?;
        Ok(AdmittedBuiltinRegistryV3 {
            inner: std::sync::Arc::new(AdmittedBuiltinRegistryInnerV3 {
                registry,
                _storage: storage,
            }),
        })
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

    /// Effects of the selected builtin overload itself, before composing
    /// effects from its arguments. The resolved ID, not the family name, is
    /// the authority because overloads may have distinct effects.
    #[must_use]
    pub fn effects_of_resolved(&self, id: BuiltinIdV2) -> Option<ExpressionEffectsV2> {
        self.entries
            .iter()
            .find(|entry| entry.id == id)
            .map(|entry| entry.effects)
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
    use std::collections::BTreeSet;
    #[test]
    fn closed_registry_preserves_builtin_families_metadata_and_ids() {
        let registry = BuiltinRegistryV2::try_closed().expect("valid static registry");
        for (name, family) in [
            ("count", 1u32),
            ("sum", 2),
            ("min", 3),
            ("max", 4),
            ("avg", 5),
            ("starts_with", 6),
            ("ends_with", 7),
            ("contains", 8),
            ("length", 9),
            ("is_null", 10),
            ("size", 11),
            ("path_length", 12),
            ("path_locations", 13),
            ("explain", 14),
            ("proof_size", 15),
            ("now", 16),
        ] {
            assert_eq!(
                registry.family_id(name),
                Some(BuiltinFamilyIdV2(family)),
                "wrong family id for {name}"
            );
        }
        assert_eq!(registry.entries.len(), 21);
        assert!(registry
            .entries
            .iter()
            .all(|entry| entry.id.name() == Some(entry.name.as_str())));
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
        assert_eq!(
            now.effects,
            ExpressionEffectsV2 {
                purity: PurityV1::Impure,
                determinism: DeterminismV1::NonDeterministic,
                monotonicity: MonotonicityV1::NonMonotone,
                aggregation: AggregationClassV1::Scalar,
                cost: CostClassV1::Constant,
            }
        );
        let count = registry
            .lookup_family(registry.family_id("count").expect("count"))
            .collect::<Vec<_>>();
        assert_eq!(
            count.len(),
            2,
            "count admits exactly the any and star overloads"
        );
        assert!(count.iter().any(|entry| matches!(
            &entry.signature,
            CallableSignatureV2::AnySingle { result } if *result == ValueTypeV1::int()
        )));
        assert!(count.iter().any(|entry| matches!(
            &entry.signature,
            CallableSignatureV2::AggregateStar { result } if *result == ValueTypeV1::int()
        )));
    }

    #[test]
    fn closed_registry_storage_census_matches_pinned_producer() {
        let registry = BuiltinRegistryV2::try_closed().expect("valid static registry");
        assert_eq!(registry.entries.len(), 21);
        assert_eq!(registry.entries.capacity(), 21);
        let mut parameter_slots = 0;
        let mut boxed_collections = 0;
        for entry in &registry.entries {
            assert_eq!(entry.name.capacity(), entry.name.len());
            match &entry.signature {
                CallableSignatureV2::Fixed { params, result } => {
                    parameter_slots += params.len();
                    assert_eq!(params.capacity(), params.len());
                    boxed_collections += params
                        .iter()
                        .filter(|parameter| parameter.as_collection().is_some())
                        .count();
                    boxed_collections += usize::from(result.as_collection().is_some());
                }
                CallableSignatureV2::AnySingle { result }
                | CallableSignatureV2::AggregateStar { result } => {
                    boxed_collections += usize::from(result.as_collection().is_some());
                }
            }
        }
        assert_eq!(parameter_slots, 20);
        assert_eq!(boxed_collections, 2);
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
            BuiltinRegistryV2::try_from_entries(vec![first.clone(), duplicate_id]).unwrap_err(),
            TypeDiagnosticV2::RegistryDuplicateId {
                registry: "builtin",
                id: 1
            }
        );
        let duplicate_signature = e(
            2,
            1,
            "f",
            fixed(vec![ValueTypeV1::int()], ValueTypeV1::float()),
            effects,
        );
        assert_eq!(
            BuiltinRegistryV2::try_from_entries(vec![first, duplicate_signature]).unwrap_err(),
            TypeDiagnosticV2::RegistryDuplicateId {
                registry: "builtin_signature",
                id: 2
            }
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
            BuiltinRegistryV2::try_from_entries(vec![first.clone(), overlap]).unwrap_err(),
            TypeDiagnosticV2::RegistryDuplicateId {
                registry: "builtin_signature",
                id: 2
            }
        );
        let aliased_name = e(3, 2, "f", fixed(vec![], ValueTypeV1::bool()), effects);
        assert_eq!(
            BuiltinRegistryV2::try_from_entries(vec![first.clone(), aliased_name]).unwrap_err(),
            TypeDiagnosticV2::RegistryDuplicateId {
                registry: "builtin_family",
                id: 3
            }
        );
        let aliased_family = e(4, 1, "g", fixed(vec![], ValueTypeV1::bool()), effects);
        assert_eq!(
            BuiltinRegistryV2::try_from_entries(vec![first, aliased_family]).unwrap_err(),
            TypeDiagnosticV2::RegistryDuplicateId {
                registry: "builtin_family",
                id: 4
            }
        );
    }

    #[test]
    fn resolved_intrinsic_effects_do_not_inherit_nested_aggregation() {
        let scalar = ExpressionEffectsV2::pure_scalar();
        let aggregate = ExpressionEffectsV2 {
            aggregation: AggregationClassV1::Aggregate,
            cost: CostClassV1::Linear,
            ..scalar
        };
        let registry = BuiltinRegistryV2::try_from_entries(vec![
            e(
                31,
                31,
                "abs",
                fixed(vec![ValueTypeV1::int()], ValueTypeV1::int()),
                scalar,
            ),
            e(
                32,
                32,
                "sum",
                fixed(vec![ValueTypeV1::int()], ValueTypeV1::int()),
                aggregate,
            ),
        ])
        .expect("nonoverlapping fixture");
        let abs = registry
            .effects_of_resolved(BuiltinIdV2(31))
            .expect("resolved abs");
        let sum = registry
            .effects_of_resolved(BuiltinIdV2(32))
            .expect("resolved sum");
        assert_eq!(abs.aggregation, AggregationClassV1::Scalar);
        assert_eq!(sum.aggregation, AggregationClassV1::Aggregate);
        assert_eq!(abs.join(sum).aggregation, AggregationClassV1::Aggregate);
        assert_eq!(registry.effects_of_resolved(BuiltinIdV2(33)), None);
    }

    #[test]
    fn resolved_effects_distinguish_overloads_within_one_family() {
        let scalar = ExpressionEffectsV2::pure_scalar();
        let nonmonotone = ExpressionEffectsV2 {
            monotonicity: MonotonicityV1::NonMonotone,
            ..scalar
        };
        let registry = BuiltinRegistryV2::try_from_entries(vec![
            e(
                41,
                41,
                "f",
                fixed(vec![ValueTypeV1::int()], ValueTypeV1::int()),
                scalar,
            ),
            e(
                42,
                41,
                "f",
                fixed(vec![ValueTypeV1::float()], ValueTypeV1::float()),
                nonmonotone,
            ),
        ])
        .expect("disjoint overload domains");
        assert_eq!(registry.family_id("f"), Some(BuiltinFamilyIdV2(41)));
        assert_eq!(registry.effects_of_resolved(BuiltinIdV2(41)), Some(scalar));
        assert_eq!(
            registry.effects_of_resolved(BuiltinIdV2(42)),
            Some(nonmonotone)
        );
    }
}
