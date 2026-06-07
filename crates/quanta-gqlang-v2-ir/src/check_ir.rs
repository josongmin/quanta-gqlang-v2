use quanta_gqlang_v2_sema::roots::check::TypedCheckRootV1;
pub use quanta_gqlang_v2_sema::roots::check::{
    CheckConfirmIntentV1, CheckExecutionModeV1, CheckPayloadErrorV1, CheckRootIntentV1,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckIrV1 {
    pub rule_id: String,
    pub scope: String,
}

impl CheckIrV1 {
    pub fn from_typed(spec: &TypedCheckRootV1) -> Self {
        Self {
            rule_id: spec.rule_id.clone(),
            scope: spec.scope.clone(),
        }
    }

    pub fn from_intent(intent: CheckRootIntentV1) -> Self {
        Self::from_typed(&TypedCheckRootV1::from_intent(intent))
    }

    pub fn intent(&self) -> Result<CheckRootIntentV1, CheckPayloadErrorV1> {
        CheckRootIntentV1::decode(&self.rule_id, &self.scope)
    }

    pub fn execution_mode(&self) -> Result<CheckExecutionModeV1, CheckPayloadErrorV1> {
        Ok(self.intent()?.execution_mode())
    }
}

#[cfg(test)]
mod tests {
    use super::{CheckConfirmIntentV1, CheckExecutionModeV1, CheckRootIntentV1};
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
        )
        .expect("intent");
        let ast = CheckRootAstV1::from_intent(intent.clone());
        let typed = analyze_root_v1(&AstRootV1::Check(ast)).expect("sema");

        let ir = lower_root_to_ir_v1(&typed).expect("ir");
        let PublicIrRootV1::Check(spec) = ir else {
            panic!("expected CHECK IR root");
        };

        assert_eq!(spec.intent().expect("decode"), intent);
        assert_eq!(
            spec.execution_mode().expect("mode"),
            CheckExecutionModeV1::Hybrid,
        );
    }
}
