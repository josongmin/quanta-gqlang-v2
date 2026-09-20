use crate::{TypeDiagnosticV2, TypeV1};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)] pub enum PurityV1 { Pure, Impure }
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)] pub enum DeterminismV1 { Deterministic, NonDeterministic }
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)] pub enum MonotonicityV1 { Monotone, Antitone, NonMonotone }
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)] pub enum AggregationClassV1 { Scalar, Aggregate }
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)] pub enum CostClassV1 { Constant, Linear, SuperLinear }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ExpressionEffectsV2 {
    pub purity: PurityV1,
    pub determinism: DeterminismV1,
    pub monotonicity: MonotonicityV1,
    pub aggregation: AggregationClassV1,
    pub cost: CostClassV1,
}
impl ExpressionEffectsV2 {
    #[must_use] pub const fn pure_scalar() -> Self { Self { purity: PurityV1::Pure, determinism: DeterminismV1::Deterministic, monotonicity: MonotonicityV1::Monotone, aggregation: AggregationClassV1::Scalar, cost: CostClassV1::Constant } }
}
pub type BuiltinMetaV1 = ExpressionEffectsV2;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BuiltinIdV2(u32);
impl BuiltinIdV2 { #[must_use] pub const fn from_raw(raw: u32) -> Self { Self(raw) } #[must_use] pub const fn raw(self) -> u32 { self.0 } }

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum CallableSignatureV2 {
    Fixed { params: Vec<TypeV1>, result: TypeV1 },
    AnySingle { result: TypeV1 },
    AggregateStar { result: TypeV1 },
}
impl CallableSignatureV2 {
    #[must_use] pub fn result(&self) -> &TypeV1 { match self { Self::Fixed { result, .. } | Self::AnySingle { result } | Self::AggregateStar { result } => result } }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BuiltinEntryV2 {
    pub id: BuiltinIdV2,
    pub name: String,
    pub signature: CallableSignatureV2,
    pub effects: ExpressionEffectsV2,
}

#[derive(Clone, Debug)]
pub struct BuiltinRegistryV2 { entries: Vec<BuiltinEntryV2> }

impl BuiltinRegistryV2 {
    pub fn try_closed(entries: Vec<BuiltinEntryV2>) -> Result<Self, TypeDiagnosticV2> {
        let mut ids = BTreeSet::new();
        let mut semantics = BTreeSet::new();
        for entry in &entries {
            if !ids.insert(entry.id) { return Err(TypeDiagnosticV2::RegistryDuplicateId { registry: "builtin", id: entry.id.raw() }); }
            if !semantics.insert((entry.name.clone(), entry.signature.clone())) {
                return Err(TypeDiagnosticV2::RegistryDuplicateId { registry: "builtin_signature", id: entry.id.raw() });
            }
        }
        Ok(Self { entries })
    }

    #[must_use]
    pub fn closed() -> Self {
        let aggregate = ExpressionEffectsV2 { purity: PurityV1::Pure, determinism: DeterminismV1::Deterministic, monotonicity: MonotonicityV1::Monotone, aggregation: AggregationClassV1::Aggregate, cost: CostClassV1::Linear };
        let scalar = ExpressionEffectsV2::pure_scalar();
        let entries = vec![
            BuiltinEntryV2 { id: BuiltinIdV2(1), name: "count".into(), signature: CallableSignatureV2::AnySingle { result: TypeV1::int() }, effects: aggregate },
            BuiltinEntryV2 { id: BuiltinIdV2(2), name: "count".into(), signature: CallableSignatureV2::AggregateStar { result: TypeV1::int() }, effects: aggregate },
            BuiltinEntryV2 { id: BuiltinIdV2(3), name: "sum".into(), signature: CallableSignatureV2::Fixed { params: vec![TypeV1::int()], result: TypeV1::int() }, effects: aggregate },
            BuiltinEntryV2 { id: BuiltinIdV2(4), name: "sum".into(), signature: CallableSignatureV2::Fixed { params: vec![TypeV1::float()], result: TypeV1::float() }, effects: aggregate },
            BuiltinEntryV2 { id: BuiltinIdV2(5), name: "starts_with".into(), signature: CallableSignatureV2::Fixed { params: vec![TypeV1::string(), TypeV1::string()], result: TypeV1::bool() }, effects: scalar },
            BuiltinEntryV2 { id: BuiltinIdV2(6), name: "is_null".into(), signature: CallableSignatureV2::AnySingle { result: TypeV1::bool() }, effects: scalar },
        ];
        Self::try_closed(entries).expect("closed builtin IDs and signatures are statically unique")
    }
    #[must_use] pub fn lookup_all(&self, name: &str) -> impl Iterator<Item = &BuiltinEntryV2> { self.entries.iter().filter(move |entry| entry.name == name) }
}
