use quanta_gqlang_v2_sema::roots::explain::{
    decode_typed_explain_shape_v1, TypedExplainEvidenceModeV1 as SemaExplainEvidenceModeV1,
    TypedExplainShapeErrorV1 as SemaExplainShapeErrorV1, TypedExplainShapeV1 as SemaExplainShapeV1,
    TypedExplainTruthExpectationV1 as SemaExplainTruthExpectationV1,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExplainIrV1 {
    // The explain shape is decoded + validated ONCE, at lowering, and stored
    // structured here. `shape()` returns it without re-decoding — the IR no
    // longer re-parses the target string on every access.
    shape: ExplainShapeIrV1,
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
    /// Decode + validate the explain shape ONCE (called from lowering). The
    /// structured shape is stored, so reads never re-decode the target string.
    pub fn from_typed_target_v1(target: Option<&str>) -> Result<Self, ExplainShapeIrErrorV1> {
        let shape = decode_typed_explain_shape_v1(target)
            .map(ExplainShapeIrV1::from)
            .map_err(ExplainShapeIrErrorV1::from)?;
        Ok(Self { shape })
    }

    /// The validated explain shape. Infallible — it was validated at lowering.
    #[must_use]
    pub fn shape(&self) -> &ExplainShapeIrV1 {
        &self.shape
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
            // The assert above pins Ok; in-src tests stay panic-free, so the
            // impossible reject arm early-returns instead of unwrapping.
            // nosemgrep: qglang-v2-no-silent-err-discard
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
            &ExplainShapeIrV1 {
                target: "view::user_authz".to_string(),
                evidence: ExplainEvidenceModeIrV1::Counterexample,
                truth_expectation: ExplainTruthExpectationIrV1::Refute,
            }
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
                code: "invalid_explain_target",
                message: "EXPLAIN truth expectation clause is duplicated".to_string(),
            }),
        );
    }
}
