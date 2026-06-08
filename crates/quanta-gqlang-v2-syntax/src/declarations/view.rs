use std::collections::BTreeMap;

use crate::roots::select::{parse_select_root_v1, SelectRootAstV1};

const DECLARATION_CLAUSE_DELIMITER_V1: &str = ";;";

pub const VIEW_DECL_ACCEPTED_FORMS_V1: &[&str] = &[
    "VIEW <name> ;; query-ref=<query_ref>",
    "VIEW <name> ;; body=SELECT <projection> FROM <table> ...",
];
pub const RULE_DECL_ACCEPTED_FORM_V1: &str = "RULE <name> ;; check-ref=<rule_or_check_ref>";
pub const MODEL_DECL_ACCEPTED_FORM_V1: &str =
    "MODEL <name> ;; provider=<provider_ref> ;; source=<source_ref>";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ViewDeclAstV1 {
    // Shared parser ownership still passes the full declaration tail through this carrier.
    pub name: String,
}

impl ViewDeclAstV1 {
    pub fn shape_v1(&self) -> Result<ViewDeclShapeAstV1, DeclarationShapeErrorV1> {
        decode_view_decl_shape_v1(&self.name)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuleDeclAstV1 {
    // Shared parser ownership still passes the full declaration tail through this carrier.
    pub name: String,
}

impl RuleDeclAstV1 {
    pub fn shape_v1(&self) -> Result<RuleDeclShapeAstV1, DeclarationShapeErrorV1> {
        decode_rule_decl_shape_v1(&self.name)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelDeclAstV1 {
    // Shared parser ownership still passes the full declaration tail through this carrier.
    pub name: String,
}

impl ModelDeclAstV1 {
    pub fn shape_v1(&self) -> Result<ModelDeclShapeAstV1, DeclarationShapeErrorV1> {
        decode_model_decl_shape_v1(&self.name)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ViewDeclShapeAstV1 {
    pub name: String,
    pub query: ViewQueryShapeAstV1,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ViewQueryShapeAstV1 {
    QueryRef(String),
    InlineSelect(SelectRootAstV1),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuleDeclShapeAstV1 {
    pub name: String,
    pub linkage: RuleCheckLinkageAstV1,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuleCheckLinkageAstV1 {
    pub check_ref: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelDeclShapeAstV1 {
    pub name: String,
    pub binding: ModelSourceBindingAstV1,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelSourceBindingAstV1 {
    pub provider: String,
    pub source: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeclarationShapeErrorV1 {
    pub message: String,
}

pub fn decode_view_decl_shape_v1(
    raw_payload: &str,
) -> Result<ViewDeclShapeAstV1, DeclarationShapeErrorV1> {
    let (name, clauses) = split_decl_payload_v1(raw_payload, "VIEW")?;
    reject_unknown_clauses_v1("VIEW", &clauses, &["query-ref", "body"])?;

    let query_ref = clauses.get("query-ref").cloned();
    let inline_body = clauses.get("body").cloned();
    let query = match (query_ref, inline_body) {
        (Some(_), Some(_)) => {
            return Err(DeclarationShapeErrorV1 {
                message: "VIEW declaration must choose either `query-ref` or `body`, not both"
                    .to_string(),
            });
        }
        (Some(query_ref), None) => ViewQueryShapeAstV1::QueryRef(query_ref),
        (None, Some(inline_body)) => ViewQueryShapeAstV1::InlineSelect(
            parse_select_root_v1(&inline_body).map_err(|err| DeclarationShapeErrorV1 {
                message: format!("VIEW inline body is invalid: {}", err.message),
            })?,
        ),
        (None, None) => {
            return Err(DeclarationShapeErrorV1 {
                message: format!(
                    "VIEW declaration requires `query-ref` or `body`; accepted forms: {}",
                    VIEW_DECL_ACCEPTED_FORMS_V1.join(" | ")
                ),
            });
        }
    };

    Ok(ViewDeclShapeAstV1 { name, query })
}

pub fn decode_rule_decl_shape_v1(
    raw_payload: &str,
) -> Result<RuleDeclShapeAstV1, DeclarationShapeErrorV1> {
    let (name, clauses) = split_decl_payload_v1(raw_payload, "RULE")?;
    reject_unknown_clauses_v1("RULE", &clauses, &["check-ref"])?;

    let Some(check_ref) = clauses.get("check-ref").cloned() else {
        return Err(DeclarationShapeErrorV1 {
            message: format!(
                "RULE declaration requires `check-ref`; accepted form: {RULE_DECL_ACCEPTED_FORM_V1}"
            ),
        });
    };

    Ok(RuleDeclShapeAstV1 {
        name,
        linkage: RuleCheckLinkageAstV1 { check_ref },
    })
}

pub fn decode_model_decl_shape_v1(
    raw_payload: &str,
) -> Result<ModelDeclShapeAstV1, DeclarationShapeErrorV1> {
    let (name, clauses) = split_decl_payload_v1(raw_payload, "MODEL")?;
    reject_unknown_clauses_v1("MODEL", &clauses, &["provider", "source"])?;

    let Some(provider) = clauses.get("provider").cloned() else {
        return Err(DeclarationShapeErrorV1 {
            message: format!(
                "MODEL declaration requires `provider`; accepted form: {MODEL_DECL_ACCEPTED_FORM_V1}"
            ),
        });
    };
    let Some(source) = clauses.get("source").cloned() else {
        return Err(DeclarationShapeErrorV1 {
            message: format!(
                "MODEL declaration requires `source`; accepted form: {MODEL_DECL_ACCEPTED_FORM_V1}"
            ),
        });
    };

    Ok(ModelDeclShapeAstV1 {
        name,
        binding: ModelSourceBindingAstV1 { provider, source },
    })
}

fn split_decl_payload_v1(
    raw_payload: &str,
    decl_kind: &str,
) -> Result<(String, BTreeMap<String, String>), DeclarationShapeErrorV1> {
    let trimmed = raw_payload.trim();
    if trimmed.is_empty() {
        return Err(DeclarationShapeErrorV1 {
            message: format!("{decl_kind} declaration is empty"),
        });
    }

    let mut segments = trimmed
        .split(DECLARATION_CLAUSE_DELIMITER_V1)
        .map(str::trim)
        .enumerate();
    let (_, raw_name) = segments.next().ok_or_else(|| DeclarationShapeErrorV1 {
        message: format!("{decl_kind} declaration is missing a name"),
    })?;
    let name = parse_decl_name_v1(raw_name, decl_kind)?;

    let mut clauses = BTreeMap::new();
    for (index, segment) in segments {
        let clause_number = index + 1;
        if segment.is_empty() {
            return Err(DeclarationShapeErrorV1 {
                message: format!("{decl_kind} clause {clause_number} is empty"),
            });
        }
        let Some((raw_key, raw_value)) = segment.split_once('=') else {
            return Err(DeclarationShapeErrorV1 {
                message: format!("{decl_kind} clause {clause_number} must use key=value form"),
            });
        };

        let key = normalize_clause_symbol_v1(raw_key);
        let value = raw_value.trim();
        if key.is_empty() || value.is_empty() {
            return Err(DeclarationShapeErrorV1 {
                message: format!(
                    "{decl_kind} clause {clause_number} must have non-empty key and value"
                ),
            });
        }
        if clauses.insert(key.clone(), value.to_string()).is_some() {
            return Err(DeclarationShapeErrorV1 {
                message: format!("{decl_kind} clause `{key}` is duplicated"),
            });
        }
    }

    Ok((name, clauses))
}

fn parse_decl_name_v1(raw_name: &str, decl_kind: &str) -> Result<String, DeclarationShapeErrorV1> {
    let name = raw_name.trim();
    if name.is_empty() {
        return Err(DeclarationShapeErrorV1 {
            message: format!("{decl_kind} declaration name is missing"),
        });
    }
    if name.contains('=') {
        return Err(DeclarationShapeErrorV1 {
            message: format!(
                "{decl_kind} declaration name must be the first clause before any key=value pairs"
            ),
        });
    }
    if name.split_whitespace().count() != 1 {
        return Err(DeclarationShapeErrorV1 {
            message: format!("{decl_kind} declaration name must be a single identifier token"),
        });
    }
    Ok(name.to_string())
}

fn reject_unknown_clauses_v1(
    decl_kind: &str,
    clauses: &BTreeMap<String, String>,
    allowed: &[&str],
) -> Result<(), DeclarationShapeErrorV1> {
    for key in clauses.keys() {
        if !allowed.iter().any(|candidate| key == candidate) {
            return Err(DeclarationShapeErrorV1 {
                message: format!("{decl_kind} clause `{key}` is unsupported"),
            });
        }
    }
    Ok(())
}

fn normalize_clause_symbol_v1(raw_value: &str) -> String {
    raw_value
        .trim()
        .to_ascii_lowercase()
        .replace(['_', ' '], "-")
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::expect_used,
        reason = "declaration tests assert shape decoding success for validated fixtures"
    )]
    #![expect(
        clippy::panic,
        reason = "unexpected declaration variants must fail closed in unit tests"
    )]

    use super::{
        DeclarationShapeErrorV1, ModelDeclAstV1, ModelDeclShapeAstV1, ModelSourceBindingAstV1,
        RuleCheckLinkageAstV1, RuleDeclAstV1, RuleDeclShapeAstV1, ViewDeclAstV1,
        ViewDeclShapeAstV1, ViewQueryShapeAstV1,
    };

    #[test]
    fn view_shape_accepts_query_reference_payload() {
        let decl = ViewDeclAstV1 {
            name: "catalog.users_by_repo ;; query-ref=queries/catalog/users_by_repo".to_string(),
        };

        assert_eq!(
            decl.shape_v1(),
            Ok(ViewDeclShapeAstV1 {
                name: "catalog.users_by_repo".to_string(),
                query: ViewQueryShapeAstV1::QueryRef("queries/catalog/users_by_repo".to_string()),
            })
        );
    }

    #[test]
    fn view_shape_accepts_inline_select_body() {
        let decl = ViewDeclAstV1 {
            name: "catalog.users ;; body=SELECT id, profile.name FROM catalog.users LIMIT 25"
                .to_string(),
        };
        let shape = decl.shape_v1().expect("VIEW shape should parse");

        match shape.query {
            ViewQueryShapeAstV1::InlineSelect(select) => {
                assert_eq!(
                    select.raw_query_v1(),
                    "SELECT id, profile.name FROM catalog.users LIMIT 25"
                );
            }
            ViewQueryShapeAstV1::QueryRef(_) => panic!("expected inline SELECT body"),
        }
        assert_eq!(shape.name, "catalog.users");
    }

    #[test]
    fn view_shape_rejects_missing_body_or_reference() {
        let decl = ViewDeclAstV1 {
            name: "catalog.users".to_string(),
        };

        assert_eq!(
            decl.shape_v1(),
            Err(DeclarationShapeErrorV1 {
                message: "VIEW declaration requires `query-ref` or `body`; accepted forms: VIEW <name> ;; query-ref=<query_ref> | VIEW <name> ;; body=SELECT <projection> FROM <table> ..."
                    .to_string(),
            })
        );
    }

    #[test]
    fn rule_shape_requires_check_reference() {
        let decl = RuleDeclAstV1 {
            name: "owasp.sql-injection ;; check-ref=checks/owasp/sql-injection".to_string(),
        };

        assert_eq!(
            decl.shape_v1(),
            Ok(RuleDeclShapeAstV1 {
                name: "owasp.sql-injection".to_string(),
                linkage: RuleCheckLinkageAstV1 {
                    check_ref: "checks/owasp/sql-injection".to_string(),
                },
            })
        );
    }

    #[test]
    fn model_shape_requires_provider_and_source_bindings() {
        let decl = ModelDeclAstV1 {
            name: "github.repo ;; provider=github ;; source=repositories/default".to_string(),
        };

        assert_eq!(
            decl.shape_v1(),
            Ok(ModelDeclShapeAstV1 {
                name: "github.repo".to_string(),
                binding: ModelSourceBindingAstV1 {
                    provider: "github".to_string(),
                    source: "repositories/default".to_string(),
                },
            })
        );
    }

    #[test]
    fn model_shape_rejects_incomplete_binding() {
        let decl = ModelDeclAstV1 {
            name: "github.repo ;; provider=github".to_string(),
        };

        assert_eq!(
            decl.shape_v1(),
            Err(DeclarationShapeErrorV1 {
                message: "MODEL declaration requires `source`; accepted form: MODEL <name> ;; provider=<provider_ref> ;; source=<source_ref>"
                    .to_string(),
            })
        );
    }
}
