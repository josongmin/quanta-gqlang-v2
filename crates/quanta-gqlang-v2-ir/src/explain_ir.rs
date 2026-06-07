use quanta_gqlang_v2_sema::roots::explain::{
    decode_typed_explain_shape_v1, TypedExplainEvidenceModeV1 as SemaExplainEvidenceModeV1,
    TypedExplainShapeErrorV1 as SemaExplainShapeErrorV1, TypedExplainShapeV1 as SemaExplainShapeV1,
    TypedExplainTruthExpectationV1 as SemaExplainTruthExpectationV1,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExplainIrV1 {
    pub target: Option<String>,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum ExplainEvidenceModeIrV1 {
    #[default]
    Auto,
    Trace,
    Citation,
    Counterexample,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum ExplainTruthExpectationIrV1 {
    #[default]
    Inspect,
    Confirm,
    Refute,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExplainShapeIrV1 {
    pub target: String,
    pub evidence: ExplainEvidenceModeIrV1,
    pub truth_expectation: ExplainTruthExpectationIrV1,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExplainShapeIrErrorV1 {
    pub message: String,
}

impl ExplainIrV1 {
    pub fn shape(&self) -> Result<ExplainShapeIrV1, ExplainShapeIrErrorV1> {
        decode_typed_explain_shape_v1(self.target.as_deref())
            .map(ExplainShapeIrV1::from)
            .map_err(ExplainShapeIrErrorV1::from)
    }
}

impl From<SemaExplainShapeV1> for ExplainShapeIrV1 {
    fn from(shape: SemaExplainShapeV1) -> Self {
        Self {
            target: shape.target,
            evidence: shape.evidence.into(),
            truth_expectation: shape.truth_expectation.into(),
        }
    }
}

impl From<SemaExplainEvidenceModeV1> for ExplainEvidenceModeIrV1 {
    fn from(mode: SemaExplainEvidenceModeV1) -> Self {
        match mode {
            SemaExplainEvidenceModeV1::Auto => Self::Auto,
            SemaExplainEvidenceModeV1::Trace => Self::Trace,
            SemaExplainEvidenceModeV1::Citation => Self::Citation,
            SemaExplainEvidenceModeV1::Counterexample => Self::Counterexample,
        }
    }
}

impl From<SemaExplainTruthExpectationV1> for ExplainTruthExpectationIrV1 {
    fn from(expectation: SemaExplainTruthExpectationV1) -> Self {
        match expectation {
            SemaExplainTruthExpectationV1::Inspect => Self::Inspect,
            SemaExplainTruthExpectationV1::Confirm => Self::Confirm,
            SemaExplainTruthExpectationV1::Refute => Self::Refute,
        }
    }
}

impl From<SemaExplainShapeErrorV1> for ExplainShapeIrErrorV1 {
    fn from(error: SemaExplainShapeErrorV1) -> Self {
        Self {
            message: error.message,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ExplainEvidenceModeIrV1, ExplainShapeIrV1, ExplainTruthExpectationIrV1};
    use crate::{lower_root_to_ir_v1, PublicIrRootV1};
    use quanta_gqlang_v2_sema::{analyze_root_v1, TypedRootV1};
    use quanta_gqlang_v2_syntax::ast::AstRootV1;
    use quanta_gqlang_v2_syntax::roots::explain::ExplainRootAstV1;

    #[test]
    fn explain_ir_preserves_structured_shape_from_sema() {
        let ast_root = AstRootV1::Explain(ExplainRootAstV1 {
            target: Some("view::user_authz ;; evidence=counterexample ;; truth=refute".to_string()),
        });

        let typed_root = analyze_root_v1(&ast_root);
        assert!(matches!(typed_root, Ok(TypedRootV1::Explain(_))));
        let typed_root = match typed_root {
            Ok(root) => root,
            Err(_) => return,
        };

        let ir_root = lower_root_to_ir_v1(&typed_root);
        assert!(matches!(ir_root, Ok(PublicIrRootV1::Explain(_))));
        let spec = match ir_root {
            Ok(PublicIrRootV1::Explain(spec)) => spec,
            _ => return,
        };

        assert_eq!(
            spec.shape(),
            Ok(ExplainShapeIrV1 {
                target: "view::user_authz".to_string(),
                evidence: ExplainEvidenceModeIrV1::Counterexample,
                truth_expectation: ExplainTruthExpectationIrV1::Refute,
            })
        );
    }

    #[test]
    fn explain_ir_surfaces_shape_errors() {
        let typed_root =
            TypedRootV1::Explain(quanta_gqlang_v2_sema::roots::explain::TypedExplainRootV1 {
                target: Some("view::user_authz ;; truth=inspect ;; truth=confirm".to_string()),
            });

        assert_eq!(
            lower_root_to_ir_v1(&typed_root),
            Err(crate::IrLoweringErrorV1 {
                message: "EXPLAIN truth expectation clause is duplicated".to_string(),
            }),
        );
    }
}
