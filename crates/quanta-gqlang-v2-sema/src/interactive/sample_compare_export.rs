use quanta_gqlang_v2_syntax::interactive::sample::{
    CompareIntentAstV1 as SyntaxCompareIntentAstV1, CompareMetricV1 as SyntaxCompareMetricV1,
    CompareRootAstV1, ExportIntentAstV1 as SyntaxExportIntentAstV1,
    ExportKindV1 as SyntaxExportKindV1, ExportRootAstV1,
    InteractiveRefAstV1 as SyntaxInteractiveRefAstV1,
    SampleCompareExportAstErrorV1 as SyntaxSampleCompareExportAstErrorV1,
    SampleRetrievalIntentAstV1 as SyntaxSampleRetrievalIntentAstV1,
    SampleRetrievalModeV1 as SyntaxSampleRetrievalModeV1, SampleRootAstV1,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypedSampleCompareExportRootV1 {
    Sample { query_text: String },
    Compare { baseline_ref: String },
    Export { source_ref: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypedSampleCompareExportShapeV1 {
    Sample(TypedSampleRetrievalIntentV1),
    Compare(TypedCompareIntentV1),
    Export(TypedExportIntentV1),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedInteractiveRefV1 {
    pub kind: String,
    pub locator: String,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TypedSampleRetrievalModeV1 {
    Lexical,
    Semantic,
    Hybrid,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedSampleRetrievalIntentV1 {
    pub query: String,
    pub scope: Option<TypedInteractiveRefV1>,
    pub mode: TypedSampleRetrievalModeV1,
    pub limit: u32,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TypedCompareMetricV1 {
    Exact,
    Structural,
    Semantic,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedCompareIntentV1 {
    pub baseline: TypedInteractiveRefV1,
    pub candidate: TypedInteractiveRefV1,
    pub metric: TypedCompareMetricV1,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TypedExportKindV1 {
    Json,
    Csv,
    Sarif,
    Markdown,
    GraphMl,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedExportIntentV1 {
    pub source: TypedInteractiveRefV1,
    pub kind: TypedExportKindV1,
    pub profile: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedSampleCompareExportErrorV1 {
    pub message: String,
}

impl TypedSampleCompareExportRootV1 {
    pub fn from_sample_ast(ast: &SampleRootAstV1) -> Result<Self, TypedSampleCompareExportErrorV1> {
        let intent = ast.intent()?;
        Ok(Self::Sample {
            query_text: SampleRootAstV1::from_intent(intent).query_text,
        })
    }

    pub fn from_compare_ast(
        ast: &CompareRootAstV1,
    ) -> Result<Self, TypedSampleCompareExportErrorV1> {
        let intent = ast.intent()?;
        Ok(Self::Compare {
            baseline_ref: CompareRootAstV1::from_intent(intent).baseline_ref,
        })
    }

    pub fn from_export_ast(ast: &ExportRootAstV1) -> Result<Self, TypedSampleCompareExportErrorV1> {
        let intent = ast.intent()?;
        Ok(Self::Export {
            source_ref: ExportRootAstV1::from_intent(intent).source_ref,
        })
    }

    pub fn shape(
        &self,
    ) -> Result<TypedSampleCompareExportShapeV1, TypedSampleCompareExportErrorV1> {
        match self {
            Self::Sample { query_text } => Ok(TypedSampleCompareExportShapeV1::Sample(
                SampleRootAstV1 {
                    query_text: query_text.clone(),
                }
                .intent()?
                .into(),
            )),
            Self::Compare { baseline_ref } => Ok(TypedSampleCompareExportShapeV1::Compare(
                CompareRootAstV1 {
                    baseline_ref: baseline_ref.clone(),
                }
                .intent()?
                .into(),
            )),
            Self::Export { source_ref } => Ok(TypedSampleCompareExportShapeV1::Export(
                ExportRootAstV1 {
                    source_ref: source_ref.clone(),
                }
                .intent()?
                .into(),
            )),
        }
    }
}

pub fn analyze_sample_root_v1(
    ast: &SampleRootAstV1,
) -> Result<TypedSampleCompareExportRootV1, TypedSampleCompareExportErrorV1> {
    TypedSampleCompareExportRootV1::from_sample_ast(ast)
}

pub fn analyze_compare_root_v1(
    ast: &CompareRootAstV1,
) -> Result<TypedSampleCompareExportRootV1, TypedSampleCompareExportErrorV1> {
    TypedSampleCompareExportRootV1::from_compare_ast(ast)
}

pub fn analyze_export_root_v1(
    ast: &ExportRootAstV1,
) -> Result<TypedSampleCompareExportRootV1, TypedSampleCompareExportErrorV1> {
    TypedSampleCompareExportRootV1::from_export_ast(ast)
}

impl From<SyntaxInteractiveRefAstV1> for TypedInteractiveRefV1 {
    fn from(value: SyntaxInteractiveRefAstV1) -> Self {
        Self {
            kind: value.kind,
            locator: value.locator,
        }
    }
}

impl From<SyntaxSampleRetrievalModeV1> for TypedSampleRetrievalModeV1 {
    fn from(value: SyntaxSampleRetrievalModeV1) -> Self {
        match value {
            SyntaxSampleRetrievalModeV1::Lexical => Self::Lexical,
            SyntaxSampleRetrievalModeV1::Semantic => Self::Semantic,
            SyntaxSampleRetrievalModeV1::Hybrid => Self::Hybrid,
        }
    }
}

impl From<SyntaxSampleRetrievalIntentAstV1> for TypedSampleRetrievalIntentV1 {
    fn from(value: SyntaxSampleRetrievalIntentAstV1) -> Self {
        Self {
            query: value.query,
            scope: value.scope.map(Into::into),
            mode: value.mode.into(),
            limit: value.limit,
        }
    }
}

impl From<SyntaxCompareMetricV1> for TypedCompareMetricV1 {
    fn from(value: SyntaxCompareMetricV1) -> Self {
        match value {
            SyntaxCompareMetricV1::Exact => Self::Exact,
            SyntaxCompareMetricV1::Structural => Self::Structural,
            SyntaxCompareMetricV1::Semantic => Self::Semantic,
        }
    }
}

impl From<SyntaxCompareIntentAstV1> for TypedCompareIntentV1 {
    fn from(value: SyntaxCompareIntentAstV1) -> Self {
        Self {
            baseline: value.baseline.into(),
            candidate: value.candidate.into(),
            metric: value.metric.into(),
        }
    }
}

impl From<SyntaxExportKindV1> for TypedExportKindV1 {
    fn from(value: SyntaxExportKindV1) -> Self {
        match value {
            SyntaxExportKindV1::Json => Self::Json,
            SyntaxExportKindV1::Csv => Self::Csv,
            SyntaxExportKindV1::Sarif => Self::Sarif,
            SyntaxExportKindV1::Markdown => Self::Markdown,
            SyntaxExportKindV1::GraphMl => Self::GraphMl,
        }
    }
}

impl From<SyntaxExportIntentAstV1> for TypedExportIntentV1 {
    fn from(value: SyntaxExportIntentAstV1) -> Self {
        Self {
            source: value.source.into(),
            kind: value.kind.into(),
            profile: value.profile,
        }
    }
}

impl From<SyntaxSampleCompareExportAstErrorV1> for TypedSampleCompareExportErrorV1 {
    fn from(value: SyntaxSampleCompareExportAstErrorV1) -> Self {
        Self {
            message: value.message,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        analyze_compare_root_v1, analyze_export_root_v1, analyze_sample_root_v1,
        TypedCompareIntentV1, TypedCompareMetricV1, TypedExportIntentV1, TypedExportKindV1,
        TypedInteractiveRefV1, TypedSampleCompareExportRootV1, TypedSampleCompareExportShapeV1,
        TypedSampleRetrievalIntentV1, TypedSampleRetrievalModeV1,
    };
    use quanta_gqlang_v2_syntax::interactive::sample::{
        CompareRootAstV1, ExportRootAstV1, SampleRootAstV1,
    };

    #[test]
    fn typed_sample_shape_is_preserved_from_ast() {
        let typed = analyze_sample_root_v1(&SampleRootAstV1 {
            query_text:
                "query=callgraph entrypoints ;; scope=repo::frontend ;; mode=semantic ;; limit=5"
                    .to_string(),
        })
        .expect("sample");

        assert_eq!(
            typed.shape(),
            Ok(TypedSampleCompareExportShapeV1::Sample(
                TypedSampleRetrievalIntentV1 {
                    query: "callgraph entrypoints".to_string(),
                    scope: Some(TypedInteractiveRefV1 {
                        kind: "repo".to_string(),
                        locator: "frontend".to_string(),
                    }),
                    mode: TypedSampleRetrievalModeV1::Semantic,
                    limit: 5,
                }
            ))
        );
    }

    #[test]
    fn typed_compare_shape_is_preserved_from_ast() {
        let typed = analyze_compare_root_v1(&CompareRootAstV1 {
            baseline_ref:
                "baseline=baseline::nightly-2026-06-07 ;; candidate=result::run-123 ;; metric=exact"
                    .to_string(),
        })
        .expect("compare");

        assert_eq!(
            typed.shape(),
            Ok(TypedSampleCompareExportShapeV1::Compare(
                TypedCompareIntentV1 {
                    baseline: TypedInteractiveRefV1 {
                        kind: "baseline".to_string(),
                        locator: "nightly-2026-06-07".to_string(),
                    },
                    candidate: TypedInteractiveRefV1 {
                        kind: "result".to_string(),
                        locator: "run-123".to_string(),
                    },
                    metric: TypedCompareMetricV1::Exact,
                }
            ))
        );
    }

    #[test]
    fn typed_export_shape_surfaces_decode_errors() {
        let error = analyze_export_root_v1(&ExportRootAstV1 {
            source_ref: "source=result::run-123 ;; kind=pdf".to_string(),
        })
        .expect_err("export kind must be validated");

        assert_eq!(error.message, "EXPORT kind `pdf` is unsupported");
    }

    #[test]
    fn raw_variant_shape_decode_stays_available_for_shared_dispatch() {
        let typed = TypedSampleCompareExportRootV1::Export {
            source_ref: "source=result::hotspots/latest ;; kind=sarif ;; profile=security"
                .to_string(),
        };

        assert_eq!(
            typed.shape(),
            Ok(TypedSampleCompareExportShapeV1::Export(
                TypedExportIntentV1 {
                    source: TypedInteractiveRefV1 {
                        kind: "result".to_string(),
                        locator: "hotspots/latest".to_string(),
                    },
                    kind: TypedExportKindV1::Sarif,
                    profile: Some("security".to_string()),
                }
            ))
        );
    }
}
