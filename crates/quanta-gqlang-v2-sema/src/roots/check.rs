use quanta_gqlang_v2_syntax::roots::check::CheckRootAstV1;
pub use quanta_gqlang_v2_syntax::roots::check::{
    CheckConfirmIntentV1, CheckExecutionModeV1, CheckPayloadErrorV1, CheckRootIntentV1,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedCheckRootV1 {
    pub rule_id: String,
    pub scope: String,
}

impl TypedCheckRootV1 {
    pub fn from_ast(ast: &CheckRootAstV1) -> Self {
        Self {
            rule_id: ast.rule_id.clone(),
            scope: ast.scope.clone(),
        }
    }

    pub fn from_intent(intent: CheckRootIntentV1) -> Self {
        Self::from_ast(&CheckRootAstV1::from_intent(intent))
    }

    pub fn intent(&self) -> Result<CheckRootIntentV1, CheckPayloadErrorV1> {
        CheckRootIntentV1::decode(&self.rule_id, &self.scope)
    }

    pub fn execution_mode(&self) -> Result<CheckExecutionModeV1, CheckPayloadErrorV1> {
        Ok(self.intent()?.execution_mode())
    }

    pub fn to_ast(&self) -> Result<CheckRootAstV1, CheckPayloadErrorV1> {
        Ok(CheckRootAstV1::from_intent(self.intent()?))
    }
}

#[cfg(test)]
mod tests {
    use super::{CheckConfirmIntentV1, CheckExecutionModeV1, CheckRootIntentV1, TypedCheckRootV1};
    use crate::{analyze_root_v1, TypedRootV1};
    use quanta_gqlang_v2_syntax::ast::AstRootV1;
    use quanta_gqlang_v2_syntax::roots::check::CheckRootAstV1;

    #[test]
    fn sema_check_root_preserves_pattern_only_intent() {
        let intent = CheckRootIntentV1::new(
            "rule.pattern/path-filter",
            "pattern",
            None,
            Some("path ~= \"src/**/*.rs\"".to_string()),
            CheckConfirmIntentV1::Prompt,
        )
        .expect("intent");
        let root = CheckRootAstV1::from_intent(intent.clone());

        let typed = analyze_root_v1(&AstRootV1::Check(root.clone())).expect("sema");
        let TypedRootV1::Check(spec) = typed else {
            panic!("expected CHECK typed root");
        };

        assert_eq!(spec.intent().expect("decode"), intent);
        assert_eq!(
            spec.execution_mode().expect("mode"),
            CheckExecutionModeV1::PatternOnly,
        );
        assert_eq!(spec.to_ast().expect("rebind"), root);
    }

    #[test]
    fn typed_check_root_can_be_built_directly_from_intent() {
        let intent = CheckRootIntentV1::new(
            "rule.semantic/ownership",
            "policy",
            Some("repo://frontend".to_string()),
            None,
            CheckConfirmIntentV1::Skip,
        )
        .expect("intent");
        let typed = TypedCheckRootV1::from_intent(intent.clone());

        assert_eq!(typed.intent().expect("decode"), intent);
        assert_eq!(
            typed.execution_mode().expect("mode"),
            CheckExecutionModeV1::SemanticOnly,
        );
    }
}
