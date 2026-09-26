use crate::{
    AggregationClassV1, CostClassV1, DeterminismV1, ExpressionEffectsV2, MonotonicityV1, PurityV1,
    TypeDiagnosticV2, ValueTypeV1,
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OperatorIdV2(u32);
impl OperatorIdV2 {
    #[must_use]
    pub const fn raw(self) -> u32 {
        self.0
    }

    /// Stable semantic spelling of every ID admitted by the closed registry.
    #[must_use]
    pub const fn symbol(self) -> Option<&'static str> {
        match self.0 {
            1..=3 | 102 | 107 | 111 => Some("-"),
            4..=6 | 101 | 106 | 110 => Some("+"),
            7 => Some("not"),
            103 | 108 | 112 | 113 => Some("*"),
            104 | 109 | 114 => Some("/"),
            105 => Some("%"),
            115 => Some("and"),
            116 => Some("or"),
            117 => Some("xor"),
            118 => Some("++"),
            200..=203 => Some("<"),
            210..=213 => Some("<="),
            220..=223 => Some(">"),
            230..=233 => Some(">="),
            300..=312 => Some("="),
            320..=332 => Some("!="),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct UnaryOverloadV1 {
    pub id: OperatorIdV2,
    pub operand: ValueTypeV1,
    pub result: ValueTypeV1,
    pub effects: ExpressionEffectsV2,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BinaryOverloadV1 {
    pub id: OperatorIdV2,
    pub lhs: ValueTypeV1,
    pub rhs: ValueTypeV1,
    pub result: ValueTypeV1,
    pub effects: ExpressionEffectsV2,
}

#[derive(Clone, Debug)]
pub struct OperatorRegistryV2 {
    unary: BTreeMap<String, Vec<UnaryOverloadV1>>,
    binary: BTreeMap<String, Vec<BinaryOverloadV1>>,
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::items_after_test_module,
    reason = "owner assertions colocated with private registry constructor"
)]
mod tests {
    use super::*;
    #[test]
    fn closed_registry_preserves_complete_operator_surface_and_explicit_ids() {
        let registry = OperatorRegistryV2::try_closed().expect("valid static registry");
        for symbol in [
            "+", "-", "*", "/", "%", "<", "<=", ">", ">=", "=", "!=", "and", "or", "xor", "++",
        ] {
            assert!(
                !registry.binary_overloads(symbol).is_empty(),
                "missing {symbol}"
            );
        }
        assert_eq!(
            registry.binary_overloads("<>"),
            registry.binary_overloads("!="),
            "surface not-equal aliases share the same stable overload identities"
        );
        for symbol in ["+", "-", "not"] {
            assert!(
                !registry.unary_overloads(symbol).is_empty(),
                "missing {symbol}"
            );
        }
        let ids = registry
            .binary
            .values()
            .flatten()
            .map(|entry| entry.id)
            .chain(registry.unary.values().flatten().map(|entry| entry.id))
            .collect::<BTreeSet<_>>();
        let count = registry.binary.values().map(Vec::len).sum::<usize>()
            + registry.unary.values().map(Vec::len).sum::<usize>();
        assert_eq!(count, 67, "the closed operator surface must be pinned");
        assert_eq!(ids.len(), count);
        for (symbol, overloads) in &registry.binary {
            assert!(overloads
                .iter()
                .all(|entry| entry.id.symbol() == Some(symbol.as_str())));
        }
        for (symbol, overloads) in &registry.unary {
            assert!(overloads
                .iter()
                .all(|entry| entry.id.symbol() == Some(symbol.as_str())));
        }
    }
    #[test]
    fn duplicate_id_and_semantic_domain_reject() {
        let one = u(1, "-", ValueTypeV1::int(), ValueTypeV1::int());
        let duplicate_id = u(1, "+", ValueTypeV1::int(), ValueTypeV1::int());
        assert_eq!(
            OperatorRegistryV2::try_from_entries(vec![one.clone(), duplicate_id], vec![])
                .unwrap_err(),
            TypeDiagnosticV2::RegistryDuplicateId {
                registry: "operator",
                id: 1
            }
        );
        let duplicate_domain = u(2, "-", ValueTypeV1::int(), ValueTypeV1::float());
        assert_eq!(
            OperatorRegistryV2::try_from_entries(vec![one, duplicate_domain], vec![]).unwrap_err(),
            TypeDiagnosticV2::RegistryDuplicateId {
                registry: "operator_signature",
                id: 2
            }
        );
    }

    #[test]
    fn every_operator_id_has_the_pinned_complete_effect_tuple() {
        let registry = OperatorRegistryV2::try_closed().expect("valid static registry");
        assert!(
            !registry.unary.is_empty(),
            "the unary effect matrix must not be vacuous"
        );
        assert!(
            !registry.binary.is_empty(),
            "the binary effect matrix must not be vacuous"
        );
        assert_eq!(
            registry.all_unary_overloads().count() + registry.all_binary_overloads().count(),
            67,
            "every admitted operator overload must be enumerated"
        );
        let scalar = |monotonicity| ExpressionEffectsV2 {
            purity: PurityV1::Pure,
            determinism: DeterminismV1::Deterministic,
            monotonicity,
            aggregation: AggregationClassV1::Scalar,
            cost: CostClassV1::Constant,
        };
        for (symbol, overloads) in &registry.unary {
            let expected = if symbol == "+" {
                scalar(MonotonicityV1::Monotone)
            } else {
                scalar(MonotonicityV1::Antitone)
            };
            assert!(
                overloads
                    .iter()
                    .all(|overload| overload.effects == expected),
                "bad unary effects for {symbol}"
            );
        }
        for (symbol, overloads) in &registry.binary {
            let expected = if matches!(symbol.as_str(), "+" | "and" | "or") {
                scalar(MonotonicityV1::Monotone)
            } else {
                scalar(MonotonicityV1::NonMonotone)
            };
            assert!(
                overloads
                    .iter()
                    .all(|overload| overload.effects == expected),
                "bad binary effects for {symbol}"
            );
        }
    }

    #[test]
    fn closed_registry_pins_every_operator_operand_and_result_semantics() {
        use crate::ScalarTyV1;
        let registry = OperatorRegistryV2::try_closed().expect("valid static registry");
        let duration = || ValueTypeV1::scalar(ScalarTyV1::Duration);
        let mut unary = registry
            .unary
            .iter()
            .flat_map(|(symbol, overloads)| {
                overloads.iter().map(move |overload| {
                    (
                        overload.id.raw(),
                        symbol.as_str(),
                        overload.operand.clone(),
                        overload.result.clone(),
                    )
                })
            })
            .collect::<Vec<_>>();
        unary.sort_by_key(|entry| entry.0);
        assert_eq!(
            unary,
            vec![
                (1, "-", ValueTypeV1::int(), ValueTypeV1::int()),
                (2, "-", ValueTypeV1::float(), ValueTypeV1::float()),
                (3, "-", duration(), duration()),
                (4, "+", ValueTypeV1::int(), ValueTypeV1::int()),
                (5, "+", ValueTypeV1::float(), ValueTypeV1::float()),
                (6, "+", duration(), duration()),
                (7, "not", ValueTypeV1::bool(), ValueTypeV1::bool()),
            ],
            "every unary operand and result must be pinned by exact id"
        );
        let mut binary = registry
            .binary
            .iter()
            .flat_map(|(symbol, overloads)| {
                overloads.iter().map(move |overload| {
                    (
                        overload.id.raw(),
                        symbol.as_str(),
                        overload.lhs.clone(),
                        overload.rhs.clone(),
                        overload.result.clone(),
                    )
                })
            })
            .collect::<Vec<_>>();
        binary.sort_by_key(|entry| entry.0);
        assert_eq!(
            binary,
            vec![
                (
                    101,
                    "+",
                    ValueTypeV1::int(),
                    ValueTypeV1::int(),
                    ValueTypeV1::int()
                ),
                (
                    102,
                    "-",
                    ValueTypeV1::int(),
                    ValueTypeV1::int(),
                    ValueTypeV1::int()
                ),
                (
                    103,
                    "*",
                    ValueTypeV1::int(),
                    ValueTypeV1::int(),
                    ValueTypeV1::int()
                ),
                (
                    104,
                    "/",
                    ValueTypeV1::int(),
                    ValueTypeV1::int(),
                    ValueTypeV1::int()
                ),
                (
                    105,
                    "%",
                    ValueTypeV1::int(),
                    ValueTypeV1::int(),
                    ValueTypeV1::int()
                ),
                (
                    106,
                    "+",
                    ValueTypeV1::float(),
                    ValueTypeV1::float(),
                    ValueTypeV1::float()
                ),
                (
                    107,
                    "-",
                    ValueTypeV1::float(),
                    ValueTypeV1::float(),
                    ValueTypeV1::float()
                ),
                (
                    108,
                    "*",
                    ValueTypeV1::float(),
                    ValueTypeV1::float(),
                    ValueTypeV1::float()
                ),
                (
                    109,
                    "/",
                    ValueTypeV1::float(),
                    ValueTypeV1::float(),
                    ValueTypeV1::float()
                ),
                (110, "+", duration(), duration(), duration()),
                (111, "-", duration(), duration(), duration()),
                (112, "*", duration(), duration(), duration()),
                (113, "*", duration(), ValueTypeV1::int(), duration()),
                (114, "/", duration(), ValueTypeV1::int(), duration()),
                (
                    115,
                    "and",
                    ValueTypeV1::bool(),
                    ValueTypeV1::bool(),
                    ValueTypeV1::bool()
                ),
                (
                    116,
                    "or",
                    ValueTypeV1::bool(),
                    ValueTypeV1::bool(),
                    ValueTypeV1::bool()
                ),
                (
                    117,
                    "xor",
                    ValueTypeV1::bool(),
                    ValueTypeV1::bool(),
                    ValueTypeV1::bool()
                ),
                (
                    118,
                    "++",
                    ValueTypeV1::string(),
                    ValueTypeV1::string(),
                    ValueTypeV1::string()
                ),
                (
                    200,
                    "<",
                    ValueTypeV1::int(),
                    ValueTypeV1::int(),
                    ValueTypeV1::bool()
                ),
                (
                    201,
                    "<",
                    ValueTypeV1::float(),
                    ValueTypeV1::float(),
                    ValueTypeV1::bool()
                ),
                (
                    202,
                    "<",
                    ValueTypeV1::string(),
                    ValueTypeV1::string(),
                    ValueTypeV1::bool()
                ),
                (203, "<", duration(), duration(), ValueTypeV1::bool()),
                (
                    210,
                    "<=",
                    ValueTypeV1::int(),
                    ValueTypeV1::int(),
                    ValueTypeV1::bool()
                ),
                (
                    211,
                    "<=",
                    ValueTypeV1::float(),
                    ValueTypeV1::float(),
                    ValueTypeV1::bool()
                ),
                (
                    212,
                    "<=",
                    ValueTypeV1::string(),
                    ValueTypeV1::string(),
                    ValueTypeV1::bool()
                ),
                (213, "<=", duration(), duration(), ValueTypeV1::bool()),
                (
                    220,
                    ">",
                    ValueTypeV1::int(),
                    ValueTypeV1::int(),
                    ValueTypeV1::bool()
                ),
                (
                    221,
                    ">",
                    ValueTypeV1::float(),
                    ValueTypeV1::float(),
                    ValueTypeV1::bool()
                ),
                (
                    222,
                    ">",
                    ValueTypeV1::string(),
                    ValueTypeV1::string(),
                    ValueTypeV1::bool()
                ),
                (223, ">", duration(), duration(), ValueTypeV1::bool()),
                (
                    230,
                    ">=",
                    ValueTypeV1::int(),
                    ValueTypeV1::int(),
                    ValueTypeV1::bool()
                ),
                (
                    231,
                    ">=",
                    ValueTypeV1::float(),
                    ValueTypeV1::float(),
                    ValueTypeV1::bool()
                ),
                (
                    232,
                    ">=",
                    ValueTypeV1::string(),
                    ValueTypeV1::string(),
                    ValueTypeV1::bool()
                ),
                (233, ">=", duration(), duration(), ValueTypeV1::bool()),
                (
                    300,
                    "=",
                    ValueTypeV1::bool(),
                    ValueTypeV1::bool(),
                    ValueTypeV1::bool()
                ),
                (
                    301,
                    "=",
                    ValueTypeV1::int(),
                    ValueTypeV1::int(),
                    ValueTypeV1::bool()
                ),
                (
                    302,
                    "=",
                    ValueTypeV1::float(),
                    ValueTypeV1::float(),
                    ValueTypeV1::bool()
                ),
                (
                    303,
                    "=",
                    ValueTypeV1::string(),
                    ValueTypeV1::string(),
                    ValueTypeV1::bool()
                ),
                (304, "=", duration(), duration(), ValueTypeV1::bool()),
                (
                    305,
                    "=",
                    ValueTypeV1::scalar(ScalarTyV1::Location),
                    ValueTypeV1::scalar(ScalarTyV1::Location),
                    ValueTypeV1::bool()
                ),
                (
                    306,
                    "=",
                    ValueTypeV1::scalar(ScalarTyV1::Path),
                    ValueTypeV1::scalar(ScalarTyV1::Path),
                    ValueTypeV1::bool()
                ),
                (
                    307,
                    "=",
                    ValueTypeV1::scalar(ScalarTyV1::RepoPath),
                    ValueTypeV1::scalar(ScalarTyV1::RepoPath),
                    ValueTypeV1::bool()
                ),
                (
                    308,
                    "=",
                    ValueTypeV1::scalar(ScalarTyV1::EntityRef),
                    ValueTypeV1::scalar(ScalarTyV1::EntityRef),
                    ValueTypeV1::bool()
                ),
                (
                    309,
                    "=",
                    ValueTypeV1::scalar(ScalarTyV1::SymbolRef),
                    ValueTypeV1::scalar(ScalarTyV1::SymbolRef),
                    ValueTypeV1::bool()
                ),
                (
                    310,
                    "=",
                    ValueTypeV1::scalar(ScalarTyV1::RuleRef),
                    ValueTypeV1::scalar(ScalarTyV1::RuleRef),
                    ValueTypeV1::bool()
                ),
                (
                    311,
                    "=",
                    ValueTypeV1::scalar(ScalarTyV1::SourceScope),
                    ValueTypeV1::scalar(ScalarTyV1::SourceScope),
                    ValueTypeV1::bool()
                ),
                (
                    312,
                    "=",
                    ValueTypeV1::scalar(ScalarTyV1::ProofTree),
                    ValueTypeV1::scalar(ScalarTyV1::ProofTree),
                    ValueTypeV1::bool()
                ),
                (
                    320,
                    "!=",
                    ValueTypeV1::bool(),
                    ValueTypeV1::bool(),
                    ValueTypeV1::bool()
                ),
                (
                    321,
                    "!=",
                    ValueTypeV1::int(),
                    ValueTypeV1::int(),
                    ValueTypeV1::bool()
                ),
                (
                    322,
                    "!=",
                    ValueTypeV1::float(),
                    ValueTypeV1::float(),
                    ValueTypeV1::bool()
                ),
                (
                    323,
                    "!=",
                    ValueTypeV1::string(),
                    ValueTypeV1::string(),
                    ValueTypeV1::bool()
                ),
                (324, "!=", duration(), duration(), ValueTypeV1::bool()),
                (
                    325,
                    "!=",
                    ValueTypeV1::scalar(ScalarTyV1::Location),
                    ValueTypeV1::scalar(ScalarTyV1::Location),
                    ValueTypeV1::bool()
                ),
                (
                    326,
                    "!=",
                    ValueTypeV1::scalar(ScalarTyV1::Path),
                    ValueTypeV1::scalar(ScalarTyV1::Path),
                    ValueTypeV1::bool()
                ),
                (
                    327,
                    "!=",
                    ValueTypeV1::scalar(ScalarTyV1::RepoPath),
                    ValueTypeV1::scalar(ScalarTyV1::RepoPath),
                    ValueTypeV1::bool()
                ),
                (
                    328,
                    "!=",
                    ValueTypeV1::scalar(ScalarTyV1::EntityRef),
                    ValueTypeV1::scalar(ScalarTyV1::EntityRef),
                    ValueTypeV1::bool()
                ),
                (
                    329,
                    "!=",
                    ValueTypeV1::scalar(ScalarTyV1::SymbolRef),
                    ValueTypeV1::scalar(ScalarTyV1::SymbolRef),
                    ValueTypeV1::bool()
                ),
                (
                    330,
                    "!=",
                    ValueTypeV1::scalar(ScalarTyV1::RuleRef),
                    ValueTypeV1::scalar(ScalarTyV1::RuleRef),
                    ValueTypeV1::bool()
                ),
                (
                    331,
                    "!=",
                    ValueTypeV1::scalar(ScalarTyV1::SourceScope),
                    ValueTypeV1::scalar(ScalarTyV1::SourceScope),
                    ValueTypeV1::bool()
                ),
                (
                    332,
                    "!=",
                    ValueTypeV1::scalar(ScalarTyV1::ProofTree),
                    ValueTypeV1::scalar(ScalarTyV1::ProofTree),
                    ValueTypeV1::bool()
                ),
            ],
            "every binary lhs, rhs, and result must be pinned by exact id"
        );
    }
}

impl OperatorRegistryV2 {
    pub(crate) fn all_unary_overloads(&self) -> impl Iterator<Item = &UnaryOverloadV1> {
        self.unary.values().flatten()
    }

    pub(crate) fn all_binary_overloads(&self) -> impl Iterator<Item = &BinaryOverloadV1> {
        self.binary.values().flatten()
    }
    fn try_from_entries(
        unary: Vec<(String, UnaryOverloadV1)>,
        binary: Vec<(String, BinaryOverloadV1)>,
    ) -> Result<Self, TypeDiagnosticV2> {
        let mut ids = BTreeSet::new();
        let mut unary_semantics: Vec<(String, ValueTypeV1)> = Vec::new();
        let mut binary_semantics: Vec<(String, ValueTypeV1, ValueTypeV1)> = Vec::new();
        let mut unary_map: BTreeMap<String, Vec<UnaryOverloadV1>> = BTreeMap::new();
        let mut binary_map: BTreeMap<String, Vec<BinaryOverloadV1>> = BTreeMap::new();
        for (symbol, overload) in unary {
            if !ids.insert(overload.id) {
                return Err(TypeDiagnosticV2::RegistryDuplicateId {
                    registry: "operator",
                    id: overload.id.raw(),
                });
            }
            let semantic = (symbol.clone(), overload.operand.clone());
            if unary_semantics.contains(&semantic) {
                return Err(TypeDiagnosticV2::RegistryDuplicateId {
                    registry: "operator_signature",
                    id: overload.id.raw(),
                });
            }
            unary_semantics.push(semantic);
            unary_map.entry(symbol).or_default().push(overload);
        }
        for (symbol, overload) in binary {
            if !ids.insert(overload.id) {
                return Err(TypeDiagnosticV2::RegistryDuplicateId {
                    registry: "operator",
                    id: overload.id.raw(),
                });
            }
            let semantic = (symbol.clone(), overload.lhs.clone(), overload.rhs.clone());
            if binary_semantics.contains(&semantic) {
                return Err(TypeDiagnosticV2::RegistryDuplicateId {
                    registry: "operator_signature",
                    id: overload.id.raw(),
                });
            }
            binary_semantics.push(semantic);
            binary_map.entry(symbol).or_default().push(overload);
        }
        Ok(Self {
            unary: unary_map,
            binary: binary_map,
        })
    }

    pub fn try_closed() -> Result<Self, TypeDiagnosticV2> {
        use crate::ScalarTyV1;
        let duration = || ValueTypeV1::scalar(ScalarTyV1::Duration);
        let unary = vec![
            u(1, "-", ValueTypeV1::int(), ValueTypeV1::int()),
            u(2, "-", ValueTypeV1::float(), ValueTypeV1::float()),
            u(3, "-", duration(), duration()),
            u(4, "+", ValueTypeV1::int(), ValueTypeV1::int()),
            u(5, "+", ValueTypeV1::float(), ValueTypeV1::float()),
            u(6, "+", duration(), duration()),
            u(7, "not", ValueTypeV1::bool(), ValueTypeV1::bool()),
        ];
        let mut binary = vec![
            b(
                101,
                "+",
                ValueTypeV1::int(),
                ValueTypeV1::int(),
                ValueTypeV1::int(),
            ),
            b(
                102,
                "-",
                ValueTypeV1::int(),
                ValueTypeV1::int(),
                ValueTypeV1::int(),
            ),
            b(
                103,
                "*",
                ValueTypeV1::int(),
                ValueTypeV1::int(),
                ValueTypeV1::int(),
            ),
            b(
                104,
                "/",
                ValueTypeV1::int(),
                ValueTypeV1::int(),
                ValueTypeV1::int(),
            ),
            b(
                105,
                "%",
                ValueTypeV1::int(),
                ValueTypeV1::int(),
                ValueTypeV1::int(),
            ),
            b(
                106,
                "+",
                ValueTypeV1::float(),
                ValueTypeV1::float(),
                ValueTypeV1::float(),
            ),
            b(
                107,
                "-",
                ValueTypeV1::float(),
                ValueTypeV1::float(),
                ValueTypeV1::float(),
            ),
            b(
                108,
                "*",
                ValueTypeV1::float(),
                ValueTypeV1::float(),
                ValueTypeV1::float(),
            ),
            b(
                109,
                "/",
                ValueTypeV1::float(),
                ValueTypeV1::float(),
                ValueTypeV1::float(),
            ),
            b(110, "+", duration(), duration(), duration()),
            b(111, "-", duration(), duration(), duration()),
            b(112, "*", duration(), duration(), duration()),
            b(113, "*", duration(), ValueTypeV1::int(), duration()),
            b(114, "/", duration(), ValueTypeV1::int(), duration()),
            b(
                115,
                "and",
                ValueTypeV1::bool(),
                ValueTypeV1::bool(),
                ValueTypeV1::bool(),
            ),
            b(
                116,
                "or",
                ValueTypeV1::bool(),
                ValueTypeV1::bool(),
                ValueTypeV1::bool(),
            ),
            b(
                117,
                "xor",
                ValueTypeV1::bool(),
                ValueTypeV1::bool(),
                ValueTypeV1::bool(),
            ),
            b(
                118,
                "++",
                ValueTypeV1::string(),
                ValueTypeV1::string(),
                ValueTypeV1::string(),
            ),
        ];
        for (base, symbol) in [(200, "<"), (210, "<="), (220, ">"), (230, ">=")] {
            binary.push(b(
                base,
                symbol,
                ValueTypeV1::int(),
                ValueTypeV1::int(),
                ValueTypeV1::bool(),
            ));
            binary.push(b(
                base + 1,
                symbol,
                ValueTypeV1::float(),
                ValueTypeV1::float(),
                ValueTypeV1::bool(),
            ));
            binary.push(b(
                base + 2,
                symbol,
                ValueTypeV1::string(),
                ValueTypeV1::string(),
                ValueTypeV1::bool(),
            ));
            binary.push(b(
                base + 3,
                symbol,
                duration(),
                duration(),
                ValueTypeV1::bool(),
            ));
        }
        let scalars = [
            ScalarTyV1::Bool,
            ScalarTyV1::Int,
            ScalarTyV1::Float,
            ScalarTyV1::String,
            ScalarTyV1::Duration,
            ScalarTyV1::Location,
            ScalarTyV1::Path,
            ScalarTyV1::RepoPath,
            ScalarTyV1::EntityRef,
            ScalarTyV1::SymbolRef,
            ScalarTyV1::RuleRef,
            ScalarTyV1::SourceScope,
            ScalarTyV1::ProofTree,
        ];
        let eq_ids = [
            300, 301, 302, 303, 304, 305, 306, 307, 308, 309, 310, 311, 312,
        ];
        let ne_ids = [
            320, 321, 322, 323, 324, 325, 326, 327, 328, 329, 330, 331, 332,
        ];
        for ((scalar, eq_id), ne_id) in scalars.into_iter().zip(eq_ids).zip(ne_ids) {
            binary.push(b(
                eq_id,
                "=",
                ValueTypeV1::scalar(scalar.clone()),
                ValueTypeV1::scalar(scalar.clone()),
                ValueTypeV1::bool(),
            ));
            binary.push(b(
                ne_id,
                "!=",
                ValueTypeV1::scalar(scalar.clone()),
                ValueTypeV1::scalar(scalar),
                ValueTypeV1::bool(),
            ));
        }
        Self::try_from_entries(unary, binary)
    }
    #[must_use]
    pub(crate) fn unary_overloads(&self, symbol: &str) -> &[UnaryOverloadV1] {
        self.unary.get(symbol).map_or(&[], Vec::as_slice)
    }
    #[must_use]
    pub(crate) fn binary_overloads(&self, symbol: &str) -> &[BinaryOverloadV1] {
        // The surface grammar admits SQL's `<>` alongside `!=`. Both spellings
        // denote the same closed operator IDs; aliasing at lookup avoids a
        // duplicate overload table or a second semantic identity.
        let canonical = if symbol == "<>" { "!=" } else { symbol };
        self.binary.get(canonical).map_or(&[], Vec::as_slice)
    }
}

fn u(
    id: u32,
    symbol: &str,
    operand: ValueTypeV1,
    result: ValueTypeV1,
) -> (String, UnaryOverloadV1) {
    (
        symbol.to_owned(),
        UnaryOverloadV1 {
            id: OperatorIdV2(id),
            operand,
            result,
            effects: operator_effects(symbol, true),
        },
    )
}
fn b(
    id: u32,
    symbol: &str,
    lhs: ValueTypeV1,
    rhs: ValueTypeV1,
    result: ValueTypeV1,
) -> (String, BinaryOverloadV1) {
    (
        symbol.to_owned(),
        BinaryOverloadV1 {
            id: OperatorIdV2(id),
            lhs,
            rhs,
            result,
            effects: operator_effects(symbol, false),
        },
    )
}

fn operator_effects(symbol: &str, unary: bool) -> ExpressionEffectsV2 {
    let monotonicity = if unary && matches!(symbol, "-" | "not") {
        MonotonicityV1::Antitone
    } else if (unary && symbol == "+") || (!unary && matches!(symbol, "+" | "and" | "or")) {
        MonotonicityV1::Monotone
    } else {
        MonotonicityV1::NonMonotone
    };
    ExpressionEffectsV2 {
        purity: PurityV1::Pure,
        determinism: DeterminismV1::Deterministic,
        monotonicity,
        aggregation: AggregationClassV1::Scalar,
        cost: CostClassV1::Constant,
    }
}
