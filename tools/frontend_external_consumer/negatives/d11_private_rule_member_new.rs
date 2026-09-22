//! Negative case D11_private_rule_member_new (class D: private constructor).
//!
//! `RuleProgramMemberV2::new` is private: a rule-program member is minted only
//! by the assembly stage that checked the rule program, never by a consumer.
//!
//! Expected: E0624, "associated function `new` is private".
use quanta_gqlang_v2_frontend_contract::RuleProgramMemberV2;

fn main() {
    let _ = RuleProgramMemberV2::new;
}
