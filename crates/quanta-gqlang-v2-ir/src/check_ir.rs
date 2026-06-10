use quanta_gqlang_v2_sema::roots::check::TypedCheckRootV1;
pub use quanta_gqlang_v2_sema::roots::check::{
    CheckConfirmIntentV1, CheckEmitIntentV1, CheckExecutionModeV1, CheckPayloadErrorV1,
    CheckRootIntentV1,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckIrV1 {
    // The CHECK intent is decoded + validated ONCE, at lowering, and stored
    // structured here. `intent()` returns it without re-decoding the
    // `check:v1|...` scope payload on every access (the audit's SMELL-1).
    intent: CheckRootIntentV1,
}

impl CheckIrV1 {
    /// Decode + validate the CHECK payload ONCE (called from lowering). Fails
    /// closed on a malformed payload; the decoded intent is then stored.
    pub fn from_typed(spec: &TypedCheckRootV1) -> Result<Self, CheckPayloadErrorV1> {
        Ok(Self {
            intent: CheckRootIntentV1::decode(&spec.rule_id, &spec.scope)?,
        })
    }

    pub fn from_intent(intent: CheckRootIntentV1) -> Self {
        Self { intent }
    }

    /// The validated CHECK intent. Infallible — it was decoded at lowering.
    #[must_use]
    pub fn intent(&self) -> &CheckRootIntentV1 {
        &self.intent
    }

    #[must_use]
    pub fn execution_mode(&self) -> CheckExecutionModeV1 {
        self.intent.execution_mode()
    }
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::expect_used,
        reason = "CHECK IR tests assert validated sema and lowering success"
    )]
    #![expect(
        clippy::panic,
        reason = "unexpected lowered variants must fail closed in unit tests"
    )]

    use super::{CheckConfirmIntentV1, CheckEmitIntentV1, CheckExecutionModeV1, CheckRootIntentV1};
    use crate::{lower_root_to_ir_v1, PublicIrRootV1};
    use quanta_gqlang_v2_sema::analyze_root_v1;
    use quanta_gqlang_v2_syntax::ast::AstRootV1;
    use quanta_gqlang_v2_syntax::roots::check::CheckRootAstV1;

    #[test]
    fn ir_lowering_preserves_hybrid_check_intent() {
        let intent = CheckRootIntentV1::new(
            "rule.hybrid/shell-audit",
            "security",
            Some("repo://runtime".to_string()),
            Some("call = \"std::process::Command\"".to_string()),
            CheckConfirmIntentV1::Require,
            CheckEmitIntentV1::Findings,
        )
        .expect("intent");
        let ast = CheckRootAstV1::from_intent(intent.clone());
        let typed = analyze_root_v1(&AstRootV1::Check(ast)).expect("sema");

        let ir = lower_root_to_ir_v1(&typed).expect("ir");
        let PublicIrRootV1::Check(spec) = ir else {
            panic!("expected CHECK IR root");
        };

        assert_eq!(spec.intent(), &intent);
        assert_eq!(
            spec.execution_mode(),
            CheckExecutionModeV1::Hybrid,
        );
    }
}
