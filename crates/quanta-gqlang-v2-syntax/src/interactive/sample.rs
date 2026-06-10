const INTERACTIVE_CLAUSE_DELIMITER_V1: &str = ";;";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SampleRootAstV1 {
    pub query_text: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompareRootAstV1 {
    pub baseline_ref: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportRootAstV1 {
    pub source_ref: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InteractiveRefAstV1 {
    pub kind: String,
    pub locator: String,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SampleRetrievalModeV1 {
    Lexical,
    Semantic,
    Hybrid,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SampleRetrievalIntentAstV1 {
    pub query: String,
    pub scope: Option<InteractiveRefAstV1>,
    pub mode: SampleRetrievalModeV1,
    pub limit: u32,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CompareMetricV1 {
    Exact,
    Structural,
    Semantic,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompareIntentAstV1 {
    pub baseline: InteractiveRefAstV1,
    pub candidate: InteractiveRefAstV1,
    pub metric: CompareMetricV1,
}

impl CompareIntentAstV1 {
    /// E03B `compare_target`: the explicit candidate a `COMPARE` is measured
    /// against. `COMPARE` requires this target by construction — the decoder
    /// rejects a payload without an explicit `candidate` ref — so there is no
    /// implicit/self compare.
    #[must_use]
    pub const fn compare_target(&self) -> &InteractiveRefAstV1 {
        &self.candidate
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ExportKindV1 {
    Json,
    Csv,
    Sarif,
    Markdown,
    GraphMl,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportIntentAstV1 {
    pub source: InteractiveRefAstV1,
    pub kind: ExportKindV1,
    pub profile: Option<String>,
}

/// E03B `bridge_receipt`: the typed, read-only descriptor an `EXPORT` hands to
/// the command bridge. `EXPORT` is bridge-only — it never mutates and never
/// promotes a gate verdict — so the receipt carries only what is exported
/// (`export_target`), the serialization `kind`, and an optional `profile`. There
/// is deliberately no approval-token or apply field: a mutating export is
/// unrepresentable here by construction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportBridgeReceiptV1 {
    pub export_target: InteractiveRefAstV1,
    pub kind: ExportKindV1,
    pub profile: Option<String>,
}

impl ExportIntentAstV1 {
    /// E03B `export_target`: the source reference whose materialized result is
    /// handed to the bridge for serialization.
    #[must_use]
    pub const fn export_target(&self) -> &InteractiveRefAstV1 {
        &self.source
    }

    /// The typed [`ExportBridgeReceiptV1`] this export hands to the command
    /// bridge. Bridge-only by construction: no mutation/approval field exists.
    #[must_use]
    pub fn bridge_receipt(&self) -> ExportBridgeReceiptV1 {
        ExportBridgeReceiptV1 {
            export_target: self.source.clone(),
            kind: self.kind,
            profile: self.profile.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SampleCompareExportAstErrorV1 {
    pub message: String,
}

impl SampleRootAstV1 {
    pub fn from_intent(intent: SampleRetrievalIntentAstV1) -> Self {
        Self {
            query_text: encode_sample_intent_v1(&intent),
        }
    }

    pub fn intent(&self) -> Result<SampleRetrievalIntentAstV1, SampleCompareExportAstErrorV1> {
        decode_sample_intent_v1(&self.query_text)
    }
}

impl CompareRootAstV1 {
    pub fn from_intent(intent: CompareIntentAstV1) -> Self {
        Self {
            baseline_ref: encode_compare_intent_v1(&intent),
        }
    }

    pub fn intent(&self) -> Result<CompareIntentAstV1, SampleCompareExportAstErrorV1> {
        decode_compare_intent_v1(&self.baseline_ref)
    }
}

impl ExportRootAstV1 {
    pub fn from_intent(intent: ExportIntentAstV1) -> Self {
        Self {
            source_ref: encode_export_intent_v1(&intent),
        }
    }

    pub fn intent(&self) -> Result<ExportIntentAstV1, SampleCompareExportAstErrorV1> {
        decode_export_intent_v1(&self.source_ref)
    }
}

pub fn decode_sample_intent_v1(
    raw: &str,
) -> Result<SampleRetrievalIntentAstV1, SampleCompareExportAstErrorV1> {
    let clauses = parse_key_value_clauses_v1(raw, "SAMPLE")?;
    reject_unknown_keys_v1("SAMPLE", &clauses, &["query", "scope", "mode", "limit"])?;

    Ok(SampleRetrievalIntentAstV1 {
        query: take_required_clause_v1("SAMPLE", &clauses, "query")?,
        scope: take_optional_clause_v1("SAMPLE", &clauses, "scope")?
            .map(|value| parse_interactive_ref_v1("SAMPLE", "scope", &value))
            .transpose()?,
        mode: parse_sample_mode_v1(&take_required_clause_v1("SAMPLE", &clauses, "mode")?)?,
        limit: parse_positive_u32_v1(
            "SAMPLE",
            "limit",
            &take_required_clause_v1("SAMPLE", &clauses, "limit")?,
        )?,
    })
}

pub fn decode_compare_intent_v1(
    raw: &str,
) -> Result<CompareIntentAstV1, SampleCompareExportAstErrorV1> {
    let clauses = parse_key_value_clauses_v1(raw, "COMPARE")?;
    reject_unknown_keys_v1("COMPARE", &clauses, &["baseline", "candidate", "metric"])?;

    Ok(CompareIntentAstV1 {
        baseline: parse_interactive_ref_v1(
            "COMPARE",
            "baseline",
            &take_required_clause_v1("COMPARE", &clauses, "baseline")?,
        )?,
        candidate: parse_interactive_ref_v1(
            "COMPARE",
            "candidate",
            &take_required_clause_v1("COMPARE", &clauses, "candidate")?,
        )?,
        metric: parse_compare_metric_v1(&take_required_clause_v1("COMPARE", &clauses, "metric")?)?,
    })
}

pub fn decode_export_intent_v1(
    raw: &str,
) -> Result<ExportIntentAstV1, SampleCompareExportAstErrorV1> {
    let clauses = parse_key_value_clauses_v1(raw, "EXPORT")?;
    reject_unknown_keys_v1("EXPORT", &clauses, &["source", "kind", "profile"])?;

    Ok(ExportIntentAstV1 {
        source: parse_interactive_ref_v1(
            "EXPORT",
            "source",
            &take_required_clause_v1("EXPORT", &clauses, "source")?,
        )?,
        kind: parse_export_kind_v1(&take_required_clause_v1("EXPORT", &clauses, "kind")?)?,
        profile: take_optional_clause_v1("EXPORT", &clauses, "profile")?,
    })
}

fn encode_sample_intent_v1(intent: &SampleRetrievalIntentAstV1) -> String {
    let mut clauses = vec![
        format!("query={}", intent.query),
        format!("mode={}", intent.mode.as_str_v1()),
        format!("limit={}", intent.limit),
    ];
    if let Some(scope) = &intent.scope {
        clauses.push(format!("scope={}", encode_interactive_ref_v1(scope)));
    }
    clauses.join(" ;; ")
}

fn encode_compare_intent_v1(intent: &CompareIntentAstV1) -> String {
    [
        format!("baseline={}", encode_interactive_ref_v1(&intent.baseline)),
        format!("candidate={}", encode_interactive_ref_v1(&intent.candidate)),
        format!("metric={}", intent.metric.as_str_v1()),
    ]
    .join(" ;; ")
}

fn encode_export_intent_v1(intent: &ExportIntentAstV1) -> String {
    let mut clauses = vec![
        format!("source={}", encode_interactive_ref_v1(&intent.source)),
        format!("kind={}", intent.kind.as_str_v1()),
    ];
    if let Some(profile) = &intent.profile {
        clauses.push(format!("profile={profile}"));
    }
    clauses.join(" ;; ")
}

impl SampleRetrievalModeV1 {
    fn as_str_v1(self) -> &'static str {
        match self {
            Self::Lexical => "lexical",
            Self::Semantic => "semantic",
            Self::Hybrid => "hybrid",
        }
    }
}

impl CompareMetricV1 {
    fn as_str_v1(self) -> &'static str {
        match self {
            Self::Exact => "exact",
            Self::Structural => "structural",
            Self::Semantic => "semantic",
        }
    }
}

impl ExportKindV1 {
    fn as_str_v1(self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Csv => "csv",
            Self::Sarif => "sarif",
            Self::Markdown => "markdown",
            Self::GraphMl => "graph-ml",
        }
    }
}

fn parse_sample_mode_v1(raw: &str) -> Result<SampleRetrievalModeV1, SampleCompareExportAstErrorV1> {
    match normalize_symbol_v1(raw).as_str() {
        "lexical" => Ok(SampleRetrievalModeV1::Lexical),
        "semantic" => Ok(SampleRetrievalModeV1::Semantic),
        "hybrid" => Ok(SampleRetrievalModeV1::Hybrid),
        normalized => Err(SampleCompareExportAstErrorV1 {
            message: format!("SAMPLE mode `{normalized}` is unsupported"),
        }),
    }
}

fn parse_compare_metric_v1(raw: &str) -> Result<CompareMetricV1, SampleCompareExportAstErrorV1> {
    match normalize_symbol_v1(raw).as_str() {
        "exact" => Ok(CompareMetricV1::Exact),
        "structural" => Ok(CompareMetricV1::Structural),
        "semantic" => Ok(CompareMetricV1::Semantic),
        normalized => Err(SampleCompareExportAstErrorV1 {
            message: format!("COMPARE metric `{normalized}` is unsupported"),
        }),
    }
}

fn parse_export_kind_v1(raw: &str) -> Result<ExportKindV1, SampleCompareExportAstErrorV1> {
    match normalize_symbol_v1(raw).as_str() {
        "json" => Ok(ExportKindV1::Json),
        "csv" => Ok(ExportKindV1::Csv),
        "sarif" => Ok(ExportKindV1::Sarif),
        "markdown" => Ok(ExportKindV1::Markdown),
        "graphml" | "graph-ml" => Ok(ExportKindV1::GraphMl),
        normalized => Err(SampleCompareExportAstErrorV1 {
            message: format!("EXPORT kind `{normalized}` is unsupported"),
        }),
    }
}

fn parse_interactive_ref_v1(
    root_name: &str,
    field_name: &str,
    raw: &str,
) -> Result<InteractiveRefAstV1, SampleCompareExportAstErrorV1> {
    let (raw_kind, raw_locator) =
        raw.split_once("::")
            .ok_or_else(|| SampleCompareExportAstErrorV1 {
                message: format!("{root_name} field `{field_name}` must use `<kind>::<locator>`"),
            })?;

    let kind = normalize_ref_kind_v1(root_name, field_name, raw_kind)?;
    let locator = raw_locator.trim();
    if locator.is_empty() {
        return Err(SampleCompareExportAstErrorV1 {
            message: format!("{root_name} field `{field_name}` locator is empty"),
        });
    }

    Ok(InteractiveRefAstV1 {
        kind,
        locator: locator.to_string(),
    })
}

fn encode_interactive_ref_v1(reference: &InteractiveRefAstV1) -> String {
    format!("{}::{}", reference.kind, reference.locator)
}

fn parse_key_value_clauses_v1(
    raw: &str,
    root_name: &str,
) -> Result<Vec<(String, String)>, SampleCompareExportAstErrorV1> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(SampleCompareExportAstErrorV1 {
            message: format!("{root_name} payload is empty"),
        });
    }

    let mut clauses = Vec::new();
    for (index, segment) in trimmed
        .split(INTERACTIVE_CLAUSE_DELIMITER_V1)
        .map(str::trim)
        .enumerate()
    {
        let clause_index = index + 1;
        if segment.is_empty() {
            return Err(SampleCompareExportAstErrorV1 {
                message: format!("{root_name} clause {clause_index} is empty"),
            });
        }

        let Some((raw_key, raw_value)) = segment.split_once('=') else {
            return Err(SampleCompareExportAstErrorV1 {
                message: format!("{root_name} clause {clause_index} must be key=value"),
            });
        };

        let key = normalize_symbol_v1(raw_key);
        if key.is_empty() {
            return Err(SampleCompareExportAstErrorV1 {
                message: format!("{root_name} clause {clause_index} has an empty key"),
            });
        }

        let value = raw_value.trim();
        if value.is_empty() {
            return Err(SampleCompareExportAstErrorV1 {
                message: format!("{root_name} clause `{key}` has an empty value"),
            });
        }

        if clauses.iter().any(|(existing_key, _)| existing_key == &key) {
            return Err(SampleCompareExportAstErrorV1 {
                message: format!("{root_name} clause `{key}` is duplicated"),
            });
        }

        clauses.push((key, value.to_string()));
    }

    Ok(clauses)
}

fn reject_unknown_keys_v1(
    root_name: &str,
    clauses: &[(String, String)],
    allowed: &[&str],
) -> Result<(), SampleCompareExportAstErrorV1> {
    for (key, _) in clauses {
        if !allowed.iter().any(|candidate| candidate == &key.as_str()) {
            return Err(SampleCompareExportAstErrorV1 {
                message: format!("{root_name} clause `{key}` is unsupported"),
            });
        }
    }
    Ok(())
}

fn take_required_clause_v1(
    root_name: &str,
    clauses: &[(String, String)],
    key: &str,
) -> Result<String, SampleCompareExportAstErrorV1> {
    take_optional_clause_v1(root_name, clauses, key)?.ok_or_else(|| SampleCompareExportAstErrorV1 {
        message: format!("{root_name} payload missing `{key}`"),
    })
}

fn take_optional_clause_v1(
    root_name: &str,
    clauses: &[(String, String)],
    key: &str,
) -> Result<Option<String>, SampleCompareExportAstErrorV1> {
    match clauses.iter().find(|(existing_key, _)| existing_key == key) {
        Some((_, value)) if value.trim().is_empty() => Err(SampleCompareExportAstErrorV1 {
            message: format!("{root_name} clause `{key}` has an empty value"),
        }),
        Some((_, value)) => Ok(Some(value.clone())),
        None => Ok(None),
    }
}

fn parse_positive_u32_v1(
    root_name: &str,
    field_name: &str,
    raw: &str,
) -> Result<u32, SampleCompareExportAstErrorV1> {
    let value = raw
        .parse::<u32>()
        .map_err(|err| SampleCompareExportAstErrorV1 {
            message: format!("{root_name} field `{field_name}` must be a positive u32: {err}"),
        })?;
    if value == 0 {
        return Err(SampleCompareExportAstErrorV1 {
            message: format!("{root_name} field `{field_name}` must be greater than zero"),
        });
    }
    Ok(value)
}

fn normalize_ref_kind_v1(
    root_name: &str,
    field_name: &str,
    raw: &str,
) -> Result<String, SampleCompareExportAstErrorV1> {
    let normalized = normalize_symbol_v1(raw);
    if normalized.is_empty() {
        return Err(SampleCompareExportAstErrorV1 {
            message: format!("{root_name} field `{field_name}` kind is empty"),
        });
    }
    if !normalized
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
    {
        return Err(SampleCompareExportAstErrorV1 {
            message: format!("{root_name} field `{field_name}` kind `{normalized}` is invalid"),
        });
    }
    Ok(normalized)
}

fn normalize_symbol_v1(raw: &str) -> String {
    raw.trim().to_ascii_lowercase().replace(['_', ' '], "-")
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::expect_used,
        reason = "interactive syntax tests assert validated intent decoding success"
    )]

    use super::{
        CompareIntentAstV1, CompareMetricV1, CompareRootAstV1, ExportIntentAstV1, ExportKindV1,
        ExportRootAstV1, InteractiveRefAstV1, SampleRetrievalIntentAstV1, SampleRetrievalModeV1,
        SampleRootAstV1,
    };

    #[test]
    fn sample_root_decodes_structured_retrieval_intent() {
        let root = SampleRootAstV1 {
            query_text: "query=error propagation in payments ;; scope=repo::frontend ;; mode=hybrid ;; limit=7".to_string(),
        };

        assert_eq!(
            root.intent(),
            Ok(SampleRetrievalIntentAstV1 {
                query: "error propagation in payments".to_string(),
                scope: Some(InteractiveRefAstV1 {
                    kind: "repo".to_string(),
                    locator: "frontend".to_string(),
                }),
                mode: SampleRetrievalModeV1::Hybrid,
                limit: 7,
            })
        );
    }

    #[test]
    fn compare_root_rejects_malformed_baseline_reference() {
        let root = CompareRootAstV1 {
            baseline_ref: "baseline=run-42 ;; candidate=run::43 ;; metric=semantic".to_string(),
        };

        assert_eq!(
            root.intent()
                .expect_err("baseline must be structured")
                .message,
            "COMPARE field `baseline` must use `<kind>::<locator>`"
        );
    }

    #[test]
    fn export_root_round_trips_structured_kind() {
        let root = ExportRootAstV1::from_intent(ExportIntentAstV1 {
            source: InteractiveRefAstV1 {
                kind: "result".to_string(),
                locator: "hotspots/latest".to_string(),
            },
            kind: ExportKindV1::Sarif,
            profile: Some("security-audit".to_string()),
        });

        assert_eq!(
            root.intent(),
            Ok(ExportIntentAstV1 {
                source: InteractiveRefAstV1 {
                    kind: "result".to_string(),
                    locator: "hotspots/latest".to_string(),
                },
                kind: ExportKindV1::Sarif,
                profile: Some("security-audit".to_string()),
            })
        );
    }

    #[test]
    fn compare_root_round_trips_structured_baseline_shape() {
        let root = CompareRootAstV1::from_intent(CompareIntentAstV1 {
            baseline: InteractiveRefAstV1 {
                kind: "baseline".to_string(),
                locator: "nightly-2026-06-07".to_string(),
            },
            candidate: InteractiveRefAstV1 {
                kind: "result".to_string(),
                locator: "run-123".to_string(),
            },
            metric: CompareMetricV1::Structural,
        });

        assert_eq!(
            root.intent(),
            Ok(CompareIntentAstV1 {
                baseline: InteractiveRefAstV1 {
                    kind: "baseline".to_string(),
                    locator: "nightly-2026-06-07".to_string(),
                },
                candidate: InteractiveRefAstV1 {
                    kind: "result".to_string(),
                    locator: "run-123".to_string(),
                },
                metric: CompareMetricV1::Structural,
            })
        );
    }
}
