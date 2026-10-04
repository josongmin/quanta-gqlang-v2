//! Catalog declarations and pack activation directives.

use super::{
    at_clause, clause_key, note_clause, parse_keyword_value, parse_ref, parse_required_ref_list,
    reject,
};
use crate::cursor::CursorV1;
use crate::diag::ParseDiagnosticV1;
use crate::query::parse_query;
use quanta_gqlang_v2_hir::{
    ActivationV1, AdmissionV1, CatalogPayloadV1, GateBindingV1, MetaKindV1, ProgramItemHirV1,
    SpanV1, UsePackActivationV1, UsePackHirV1, ViewBodyV1,
};

// ---- VIEW ------------------------------------------------------------------

pub(super) fn parse_view(
    cur: &mut CursorV1<'_>,
    source: &str,
) -> Result<ProgramItemHirV1, ParseDiagnosticV1> {
    let kw = cur.expect_keyword("view", "'view' to declare a view")?;
    let name_ref = parse_ref(cur, source, "a view name")?;
    let (key, key_span) = clause_key(cur)?;
    let (body, end) = match key.as_str() {
        "query-ref" => {
            let r = parse_ref(cur, source, "a query reference")?;
            (ViewBodyV1::QueryRef(r.clone()), r.span.end)
        }
        "select" => {
            // inline SELECT: the v3 `from | ...` pipe is the shared query carrier.
            let query = parse_query(cur)?;
            let e = query.span.end;
            (ViewBodyV1::Inline(Box::new(query)), e)
        }
        other => {
            return Err(reject(
                cur,
                key_span,
                format!("VIEW body must be 'query-ref' or 'select', got '{other}'"),
            ))
        }
    };
    let span = kw.span.merge(&SpanV1::point(end));
    let decl = cur.builder.catalog_decl(
        name_ref.name,
        name_ref.span,
        CatalogPayloadV1::View(body),
        span,
    );
    Ok(ProgramItemHirV1::from(decl))
}

// ---- MODEL -----------------------------------------------------------------

pub(super) fn parse_model(
    cur: &mut CursorV1<'_>,
    source: &str,
) -> Result<ProgramItemHirV1, ParseDiagnosticV1> {
    let kw = cur.expect_keyword("model", "'model' to declare a model")?;
    let name_ref = parse_ref(cur, source, "a model name")?;
    let mut provider = None;
    let mut model_source = None;
    let mut end = name_ref.span.end;

    let mut seen: Vec<String> = Vec::new();
    while at_clause(cur) {
        let (key, key_span) = clause_key(cur)?;
        note_clause(cur, &mut seen, "MODEL", &key, key_span)?;
        match key.as_str() {
            "provider" => {
                let r = parse_ref(cur, source, "a provider tag")?;
                end = r.span.end;
                provider = Some(r.name);
            }
            "source" => {
                let r = parse_ref(cur, source, "a source reference")?;
                end = r.span.end;
                model_source = Some(r);
            }
            other => {
                return Err(reject(
                    cur,
                    key_span,
                    format!("unsupported MODEL clause '{other}'"),
                ))
            }
        }
    }

    let provider = provider.ok_or_else(|| reject(cur, kw.span, "MODEL requires a 'provider'"))?;
    let model_source =
        model_source.ok_or_else(|| reject(cur, kw.span, "MODEL requires a 'source'"))?;
    let span = kw.span.merge(&SpanV1::point(end));
    let decl = cur.builder.catalog_decl(
        name_ref.name,
        name_ref.span,
        CatalogPayloadV1::Model {
            provider,
            source: model_source,
        },
        span,
    );
    Ok(ProgramItemHirV1::from(decl))
}

// ---- PACK ------------------------------------------------------------------

pub(super) fn parse_pack(
    cur: &mut CursorV1<'_>,
    source: &str,
) -> Result<ProgramItemHirV1, ParseDiagnosticV1> {
    let kw = cur.expect_keyword("pack", "'pack' to declare a pack")?;
    let name_ref = parse_ref(cur, source, "a pack name")?;
    let mut pack_source = None;
    let mut activation = None;
    let mut end = name_ref.span.end;

    let mut seen: Vec<String> = Vec::new();
    while at_clause(cur) {
        let (key, key_span) = clause_key(cur)?;
        note_clause(cur, &mut seen, "PACK", &key, key_span)?;
        match key.as_str() {
            "source" => {
                let r = parse_ref(cur, source, "a source reference")?;
                end = r.span.end;
                pack_source = Some(r);
            }
            "activation" => {
                let (v, sp) = activation_value(cur)?;
                end = sp.end;
                activation = Some(v);
            }
            other => {
                return Err(reject(
                    cur,
                    key_span,
                    format!("unsupported PACK clause '{other}'"),
                ))
            }
        }
    }

    let pack_source =
        pack_source.ok_or_else(|| reject(cur, kw.span, "PACK requires a 'source'"))?;
    let activation =
        activation.ok_or_else(|| reject(cur, kw.span, "PACK requires an 'activation'"))?;
    let span = kw.span.merge(&SpanV1::point(end));
    let decl = cur.builder.catalog_decl(
        name_ref.name,
        name_ref.span,
        CatalogPayloadV1::Pack {
            source: pack_source,
            activation,
        },
        span,
    );
    Ok(ProgramItemHirV1::from(decl))
}

// ---- PROFILE ---------------------------------------------------------------

pub(super) fn parse_profile(
    cur: &mut CursorV1<'_>,
    source: &str,
) -> Result<ProgramItemHirV1, ParseDiagnosticV1> {
    let kw = cur.expect_keyword("profile", "'profile' to declare a profile")?;
    let name_ref = parse_ref(cur, source, "a profile name")?;
    let mut capabilities = Vec::new();
    let mut admission = None;
    let mut end = name_ref.span.end;

    let mut seen: Vec<String> = Vec::new();
    while at_clause(cur) {
        let (key, key_span) = clause_key(cur)?;
        note_clause(cur, &mut seen, "PROFILE", &key, key_span)?;
        match key.as_str() {
            "capabilities" => {
                let refs = parse_required_ref_list(
                    cur,
                    source,
                    "a capability reference",
                    key_span,
                    "PROFILE capabilities list must not be empty",
                )?;
                end = refs.last().map(|r| r.span.end).unwrap_or(end);
                capabilities = refs;
            }
            "admission" => {
                let (v, sp) = parse_keyword_value(cur, "an admission policy")?;
                end = sp.end;
                admission = Some(match v.as_str() {
                    "allowlist" => AdmissionV1::Allowlist,
                    "review-required" => AdmissionV1::ReviewRequired,
                    "certified-only" => AdmissionV1::CertifiedOnly,
                    other => {
                        return Err(reject(
                            cur,
                            sp,
                            format!("unsupported admission policy '{other}'"),
                        ))
                    }
                });
            }
            other => {
                return Err(reject(
                    cur,
                    key_span,
                    format!("unsupported PROFILE clause '{other}'"),
                ))
            }
        }
    }

    let admission =
        admission.ok_or_else(|| reject(cur, kw.span, "PROFILE requires an 'admission'"))?;
    if capabilities.is_empty() {
        return Err(reject(
            cur,
            kw.span,
            "PROFILE requires a 'capabilities' clause",
        ));
    }
    let span = kw.span.merge(&SpanV1::point(end));
    let decl = cur.builder.catalog_decl(
        name_ref.name,
        name_ref.span,
        CatalogPayloadV1::Profile {
            capabilities,
            admission,
        },
        span,
    );
    Ok(ProgramItemHirV1::from(decl))
}

// ---- GATE ------------------------------------------------------------------

pub(super) fn parse_gate(
    cur: &mut CursorV1<'_>,
    source: &str,
) -> Result<ProgramItemHirV1, ParseDiagnosticV1> {
    let kw = cur.expect_keyword("gate", "'gate' to declare a gate")?;
    let name_ref = parse_ref(cur, source, "a gate name")?;
    let mut checks = Vec::new();
    let mut binding = None;
    let mut end = name_ref.span.end;

    let mut seen: Vec<String> = Vec::new();
    while at_clause(cur) {
        let (key, key_span) = clause_key(cur)?;
        note_clause(cur, &mut seen, "GATE", &key, key_span)?;
        match key.as_str() {
            "checks" => {
                let refs = parse_required_ref_list(
                    cur,
                    source,
                    "a check reference",
                    key_span,
                    "GATE checks list must not be empty",
                )?;
                end = refs.last().map(|r| r.span.end).unwrap_or(end);
                checks = refs;
            }
            "binding" => {
                let (v, sp) = cur.expect_ident("a binding policy")?;
                end = sp.end;
                binding = Some(match v.as_str() {
                    "all" => GateBindingV1::All,
                    "any" => GateBindingV1::Any,
                    other => {
                        return Err(reject(
                            cur,
                            sp,
                            format!("unsupported binding policy '{other}'"),
                        ))
                    }
                });
            }
            other => {
                return Err(reject(
                    cur,
                    key_span,
                    format!("unsupported GATE clause '{other}'"),
                ))
            }
        }
    }

    let binding = binding.ok_or_else(|| reject(cur, kw.span, "GATE requires a 'binding'"))?;
    if checks.is_empty() {
        return Err(reject(cur, kw.span, "GATE requires a 'checks' clause"));
    }
    let span = kw.span.merge(&SpanV1::point(end));
    let decl = cur.builder.catalog_decl(
        name_ref.name,
        name_ref.span,
        CatalogPayloadV1::Gate { checks, binding },
        span,
    );
    Ok(ProgramItemHirV1::from(decl))
}

// ---- RULE binding (ruledef) ------------------------------------------------

pub(super) fn parse_ruledef(
    cur: &mut CursorV1<'_>,
    source: &str,
) -> Result<ProgramItemHirV1, ParseDiagnosticV1> {
    let kw = cur.expect_keyword("ruledef", "'ruledef' to bind a catalog rule")?;
    let name_ref = parse_ref(cur, source, "a rule name")?;
    let (key, key_span) = clause_key(cur)?;
    if key != "check-ref" {
        return Err(reject(
            cur,
            key_span,
            format!("RULE binding requires a 'check-ref' clause, got '{key}'"),
        ));
    }
    let check_ref = parse_ref(cur, source, "a check reference")?;
    let span = kw.span.merge(&check_ref.span);
    let decl = cur.builder.catalog_decl(
        name_ref.name,
        name_ref.span,
        CatalogPayloadV1::RuleBinding { check_ref },
        span,
    );
    Ok(ProgramItemHirV1::from(decl))
}

// ---- META: USE PACK --------------------------------------------------------

pub(super) fn parse_use_pack(
    cur: &mut CursorV1<'_>,
    source: &str,
) -> Result<ProgramItemHirV1, ParseDiagnosticV1> {
    let kw = cur.expect_keyword("use", "'use' to start a meta directive")?;
    let pack_word = cur.expect_keyword("pack", "'pack' after 'use'")?;
    let pack_ref = parse_ref(cur, source, "a pack id")?;
    let mut end = pack_ref.span.end;

    let mut activation = None;
    let mut lockfile = None;

    let mut seen: Vec<String> = Vec::new();
    while at_clause(cur) {
        let (key, key_span) = clause_key(cur)?;
        note_clause(cur, &mut seen, "USE PACK", &key, key_span)?;
        match key.as_str() {
            "activation" => {
                let (v, sp) = activation_value(cur)?;
                end = sp.end;
                // USE PACK only admits session/workspace (manual is reject in v2).
                if matches!(v, ActivationV1::Manual) {
                    return Err(reject(
                        cur,
                        sp,
                        "USE PACK activation must be 'session' or 'workspace'",
                    ));
                }
                activation = Some(v);
            }
            "lockfile" => {
                let r = parse_ref(cur, source, "a lockfile path")?;
                end = r.span.end;
                if !r.name.ends_with(".lock") {
                    return Err(reject(
                        cur,
                        r.span,
                        "USE PACK lockfile must end with '.lock'",
                    ));
                }
                lockfile = Some(r.name);
            }
            other => {
                return Err(reject(
                    cur,
                    key_span,
                    format!("unsupported USE PACK clause '{other}'"),
                ))
            }
        }
    }

    let activation = activation
        .ok_or_else(|| reject(cur, pack_word.span, "USE PACK requires an 'activation'"))?;
    let lockfile = lockfile
        .ok_or_else(|| reject(cur, pack_word.span, "USE PACK requires a '.lock' lockfile"))?;
    let span = kw.span.merge(&SpanV1::point(end));
    let meta = cur.builder.meta(
        MetaKindV1::UsePack(UsePackHirV1 {
            pack_id: pack_ref.name,
            activation: match activation {
                ActivationV1::Session => UsePackActivationV1::Session,
                ActivationV1::Workspace => UsePackActivationV1::Workspace,
                ActivationV1::Manual => {
                    return Err(reject(
                        cur,
                        pack_word.span,
                        "USE PACK activation must be 'session' or 'workspace'",
                    ));
                }
            },
            lockfile,
        }),
        span,
    );
    Ok(ProgramItemHirV1::from(meta))
}

// ---- META: REQUIRE CERTIFIED -----------------------------------------------

pub(super) fn parse_require_certified(
    cur: &mut CursorV1<'_>,
) -> Result<ProgramItemHirV1, ParseDiagnosticV1> {
    let kw = cur.expect_keyword("require", "'require' to start a meta directive")?;
    let cert = cur.expect_keyword("certified", "'certified' after 'require'")?;
    let span = kw.span.merge(&cert.span);
    let meta = cur.builder.meta(MetaKindV1::RequireCertified, span);
    Ok(ProgramItemHirV1::from(meta))
}

/// Parse an activation policy keyword (`manual`/`session`/`workspace`).
fn activation_value(cur: &mut CursorV1<'_>) -> Result<(ActivationV1, SpanV1), ParseDiagnosticV1> {
    let (v, sp) = cur.expect_ident("an activation policy")?;
    let act = match v.as_str() {
        "manual" => ActivationV1::Manual,
        "session" => ActivationV1::Session,
        "workspace" => ActivationV1::Workspace,
        other => return Err(cur.unsupported(sp, format!("unsupported activation '{other}'"))),
    };
    Ok((act, sp))
}
