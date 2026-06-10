use quanta_gqlang_v2_syntax::roots::explain::{
    decode_explain_shape_v1, ExplainEvidenceModeV1 as SyntaxExplainEvidenceModeV1,
    ExplainShapeAstV1 as SyntaxExplainShapeAstV1, ExplainShapeErrorV1 as SyntaxExplainShapeErrorV1,
    ExplainTruthExpectationV1 as SyntaxExplainTruthExpectationV1,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedExplainRootV1 {
    pub target: Option<String>,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum TypedExplainEvidenceModeV1 {
    #[default]
    Auto,
    Trace,
    Citation,
    Counterexample,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum TypedExplainTruthExpectationV1 {
    #[default]
    Inspect,
    Confirm,
    Refute,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedExplainShapeV1 {
    pub target: String,
    pub evidence: TypedExplainEvidenceModeV1,
    pub truth_expectation: TypedExplainTruthExpectationV1,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedExplainShapeErrorV1 {
    pub message: String,
}

impl TypedExplainRootV1 {
    pub fn shape(&self) -> Result<TypedExplainShapeV1, TypedExplainShapeErrorV1> {
        decode_typed_explain_shape_v1(self.target.as_deref())
    }
}

pub fn decode_typed_explain_shape_v1(
    raw_target: Option<&str>,
) -> Result<TypedExplainShapeV1, TypedExplainShapeErrorV1> {
    decode_explain_shape_v1(raw_target)
        .map(TypedExplainShapeV1::from)
        .map_err(TypedExplainShapeErrorV1::from)
}

impl From<SyntaxExplainShapeAstV1> for TypedExplainShapeV1 {
    fn from(shape: SyntaxExplainShapeAstV1) -> Self {
        Self {
            target: shape.target,
            evidence: shape.evidence.into(),
            truth_expectation: shape.truth_expectation.into(),
        }
    }
}

impl From<SyntaxExplainEvidenceModeV1> for TypedExplainEvidenceModeV1 {
    fn from(mode: SyntaxExplainEvidenceModeV1) -> Self {
        match mode {
            SyntaxExplainEvidenceModeV1::Auto => Self::Auto,
            SyntaxExplainEvidenceModeV1::Trace => Self::Trace,
            SyntaxExplainEvidenceModeV1::Citation => Self::Citation,
            SyntaxExplainEvidenceModeV1::Counterexample => Self::Counterexample,
        }
    }
}

impl From<SyntaxExplainTruthExpectationV1> for TypedExplainTruthExpectationV1 {
    fn from(expectation: SyntaxExplainTruthExpectationV1) -> Self {
        match expectation {
            SyntaxExplainTruthExpectationV1::Inspect => Self::Inspect,
            SyntaxExplainTruthExpectationV1::Confirm => Self::Confirm,
            SyntaxExplainTruthExpectationV1::Refute => Self::Refute,
        }
    }
}

impl From<SyntaxExplainShapeErrorV1> for TypedExplainShapeErrorV1 {
    fn from(error: SyntaxExplainShapeErrorV1) -> Self {
        Self {
            message: error.message,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{TypedExplainEvidenceModeV1, TypedExplainShapeV1, TypedExplainTruthExpectationV1};
    use crate::{analyze_root_v1, TypedRootV1};
    use quanta_gqlang_v2_syntax::ast::AstRootV1;
    use quanta_gqlang_v2_syntax::roots::explain::ExplainRootAstV1;

    #[test]
    fn typed_explain_shape_is_preserved_from_ast() {
        let root = AstRootV1::Explain(ExplainRootAstV1 {
            target: Some(
                "model::payments.charge ;; evidence=citation ;; truth=inspect".to_string(),
            ),
        });

        let typed_root = analyze_root_v1(&root);
        assert!(matches!(typed_root, Ok(TypedRootV1::Explain(_))));

        let spec = match typed_root {
            Ok(TypedRootV1::Explain(spec)) => spec,
            _ => return,
        };

        assert_eq!(
            spec.shape(),
            Ok(TypedExplainShapeV1 {
                target: "model::payments.charge".to_string(),
                evidence: TypedExplainEvidenceModeV1::Citation,
                truth_expectation: TypedExplainTruthExpectationV1::Inspect,
            })
        );
    }

    #[test]
    fn typed_explain_shape_surfaces_decode_errors() {
        let root = AstRootV1::Explain(ExplainRootAstV1 {
            target: Some(
                "model::payments.charge ;; evidence=trace ;; evidence=citation".to_string(),
            ),
        });

        let typed_root = analyze_root_v1(&root);
        assert_eq!(
            typed_root,
            Err(crate::SemaDiagnosticPageV1 {
                diagnostics: vec![crate::SemaDiagnosticV1 {
                    code: "invalid_explain_target",
                    message: "EXPLAIN evidence clause is duplicated".to_string(),
                }],
            }),
        );
    }
}
