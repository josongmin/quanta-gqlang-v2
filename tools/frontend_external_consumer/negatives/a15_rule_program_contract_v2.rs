//! Negative case A15_rule_program_contract_v2 (class A: owner symbol through the facade root).
//!
//! `RuleProgramContractV2` is a logic owner contract type. It must stay unreachable from
//! `quanta_gqlang_v2_frontend_contract`: the facade is a curated allowlist, not
//! a re-export of an owner root.
//!
//! Expected: E0432, "no `RuleProgramContractV2` in the root".
use quanta_gqlang_v2_frontend_contract::RuleProgramContractV2;

fn main() {}
