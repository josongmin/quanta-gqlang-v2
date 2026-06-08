const EXPLAIN_CLAUSE_DELIMITER_V1: &str = ";;";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExplainRootAstV1 {
    pub target: Option<String>,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum ExplainEvidenceModeV1 {
    #[default]
    Auto,
    Trace,
    Citation,
    Counterexample,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum ExplainTruthExpectationV1 {
    #[default]
    Inspect,
    Confirm,
    Refute,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExplainShapeAstV1 {
    pub target: String,
    pub evidence: ExplainEvidenceModeV1,
    pub truth_expectation: ExplainTruthExpectationV1,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExplainShapeErrorV1 {
    pub message: String,
}

impl ExplainRootAstV1 {
    pub fn shape(&self) -> Result<ExplainShapeAstV1, ExplainShapeErrorV1> {
        decode_explain_shape_v1(self.target.as_deref())
    }
}

pub fn decode_explain_shape_v1(
    raw_target: Option<&str>,
) -> Result<ExplainShapeAstV1, ExplainShapeErrorV1> {
    let raw_target = raw_target.ok_or_else(|| ExplainShapeErrorV1 {
        message: "EXPLAIN target is missing".to_string(),
    })?;
    let trimmed = raw_target.trim();
    if trimmed.is_empty() {
        return Err(ExplainShapeErrorV1 {
            message: "EXPLAIN target is empty".to_string(),
        });
    }

    let mut target: Option<String> = None;
    let mut evidence = ExplainEvidenceModeV1::default();
    let mut truth_expectation = ExplainTruthExpectationV1::default();
    let mut evidence_seen = false;
    let mut truth_seen = false;

    for (index, segment) in trimmed
        .split(EXPLAIN_CLAUSE_DELIMITER_V1)
        .map(str::trim)
        .enumerate()
    {
        let segment_index = index + 1;
        if segment.is_empty() {
            return Err(ExplainShapeErrorV1 {
                message: format!("EXPLAIN clause {segment_index} is empty"),
            });
        }

        // The first segment is the positional target unless it is actually a
        // known `key=value` clause. Checking the clause-key whitelist (rather than
        // just "contains '='") lets parameterized targets such as
        // `graph::reachability(seed=auth)` carry an `=` without being mistaken for
        // a clause.
        if index == 0 && !is_known_clause_segment_v1(segment) {
            target = Some(segment.to_string());
            continue;
        }

        let Some((raw_key, raw_value)) = segment.split_once('=') else {
            return Err(ExplainShapeErrorV1 {
                message: format!("EXPLAIN clause {segment_index} must be key=value"),
            });
        };

        let key = normalize_clause_symbol_v1(raw_key);
        let value = raw_value.trim();
        if value.is_empty() {
            return Err(ExplainShapeErrorV1 {
                message: format!("EXPLAIN clause '{key}' has an empty value"),
            });
        }

        match key.as_str() {
            "target" => {
                if target.replace(value.to_string()).is_some() {
                    return Err(ExplainShapeErrorV1 {
                        message: "EXPLAIN target clause is duplicated".to_string(),
                    });
                }
            }
            "evidence" => {
                if evidence_seen {
                    return Err(ExplainShapeErrorV1 {
                        message: "EXPLAIN evidence clause is duplicated".to_string(),
                    });
                }
                evidence = parse_evidence_mode_v1(value)?;
                evidence_seen = true;
            }
            "truth" | "truth-expectation" => {
                if truth_seen {
                    return Err(ExplainShapeErrorV1 {
                        message: "EXPLAIN truth expectation clause is duplicated".to_string(),
                    });
                }
                truth_expectation = parse_truth_expectation_v1(value)?;
                truth_seen = true;
            }
            _ => {
                return Err(ExplainShapeErrorV1 {
                    message: format!("EXPLAIN clause '{key}' is unsupported"),
                });
            }
        }
    }

    let target = target.ok_or_else(|| ExplainShapeErrorV1 {
        message: "EXPLAIN target clause is missing".to_string(),
    })?;

    Ok(ExplainShapeAstV1 {
        target,
        evidence,
        truth_expectation,
    })
}

fn parse_evidence_mode_v1(raw_value: &str) -> Result<ExplainEvidenceModeV1, ExplainShapeErrorV1> {
    match normalize_clause_symbol_v1(raw_value).as_str() {
        "auto" => Ok(ExplainEvidenceModeV1::Auto),
        "trace" => Ok(ExplainEvidenceModeV1::Trace),
        "citation" => Ok(ExplainEvidenceModeV1::Citation),
        "counterexample" => Ok(ExplainEvidenceModeV1::Counterexample),
        normalized => Err(ExplainShapeErrorV1 {
            message: format!("EXPLAIN evidence mode '{normalized}' is unsupported"),
        }),
    }
}

fn parse_truth_expectation_v1(
    raw_value: &str,
) -> Result<ExplainTruthExpectationV1, ExplainShapeErrorV1> {
    match normalize_clause_symbol_v1(raw_value).as_str() {
        "inspect" => Ok(ExplainTruthExpectationV1::Inspect),
        "confirm" => Ok(ExplainTruthExpectationV1::Confirm),
        "refute" => Ok(ExplainTruthExpectationV1::Refute),
        normalized => Err(ExplainShapeErrorV1 {
            message: format!("EXPLAIN truth expectation '{normalized}' is unsupported"),
        }),
    }
}

fn is_known_clause_segment_v1(segment: &str) -> bool {
    match segment.split_once('=') {
        Some((raw_key, _)) => matches!(
            normalize_clause_symbol_v1(raw_key).as_str(),
            "target" | "evidence" | "truth" | "truth-expectation"
        ),
        None => false,
    }
}

fn normalize_clause_symbol_v1(raw_value: &str) -> String {
    raw_value
        .trim()
        .to_ascii_lowercase()
        .replace(['_', ' '], "-")
}

#[cfg(test)]
mod tests {
    use super::{
        ExplainEvidenceModeV1, ExplainRootAstV1, ExplainShapeAstV1, ExplainShapeErrorV1,
        ExplainTruthExpectationV1,
    };

    #[test]
    fn explain_shape_defaults_to_auto_inspect() {
        let root = ExplainRootAstV1 {
            target: Some("graph::reachability(seed)".to_string()),
        };

        assert_eq!(
            root.shape(),
            Ok(ExplainShapeAstV1 {
                target: "graph::reachability(seed)".to_string(),
                evidence: ExplainEvidenceModeV1::Auto,
                truth_expectation: ExplainTruthExpectationV1::Inspect,
            })
        );
    }

    #[test]
    fn explain_shape_decodes_structured_suffixes() {
        let root = ExplainRootAstV1 {
            target: Some(
                "rule::owasp.sql-injection ;; evidence=trace ;; truth_expectation=confirm"
                    .to_string(),
            ),
        };

        assert_eq!(
            root.shape(),
            Ok(ExplainShapeAstV1 {
                target: "rule::owasp.sql-injection".to_string(),
                evidence: ExplainEvidenceModeV1::Trace,
                truth_expectation: ExplainTruthExpectationV1::Confirm,
            })
        );
    }

    #[test]
    fn explain_shape_accepts_parameterized_target_containing_equals() {
        // Regression: a positional target that itself contains `=` (a parameterized
        // analysis target) used to be sliced at the `=` and rejected as an
        // unsupported clause.
        let root = ExplainRootAstV1 {
            target: Some("graph::reachability(seed=auth) ;; evidence=trace".to_string()),
        };

        assert_eq!(
            root.shape(),
            Ok(ExplainShapeAstV1 {
                target: "graph::reachability(seed=auth)".to_string(),
                evidence: ExplainEvidenceModeV1::Trace,
                truth_expectation: ExplainTruthExpectationV1::Inspect,
            })
        );
    }

    #[test]
    fn explain_shape_rejects_duplicate_truth_expectation() {
        let root = ExplainRootAstV1 {
            target: Some("node::sink ;; truth=inspect ;; truth=refute".to_string()),
        };

        assert_eq!(
            root.shape(),
            Err(ExplainShapeErrorV1 {
                message: "EXPLAIN truth expectation clause is duplicated".to_string(),
            })
        );
    }
}
