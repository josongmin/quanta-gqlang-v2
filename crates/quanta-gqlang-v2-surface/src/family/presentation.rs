//! SHOW, DESCRIBE, SAMPLE, COMPARE, and EXPORT family grammars.

use super::{
    at_clause, boxed_program_item_v3, clause_key, expect_string, note_clause, parse_ref, parse_u32,
    reject, CursorV1, ParseDiagnosticV1, PresentationKindV1, ProgramItemHirV1, SpanV1, TokenKindV1,
};
use quanta_gqlang_v2_hir::{CompareMetricV1, ExportFormatV1, SampleModeV1, ShowCategoryV1};

// ---- SHOW ------------------------------------------------------------------

pub(super) fn parse_show(
    cur: &mut CursorV1<'_>,
    source: &str,
) -> Result<ProgramItemHirV1, ParseDiagnosticV1> {
    let kw = cur.expect_keyword("show", "'show' to start a catalog query")?;
    let (cat, cat_span) = cur.expect_ident("a SHOW category")?;
    let category = match cat.as_str() {
        "capabilities" => ShowCategoryV1::Capabilities,
        "models" => ShowCategoryV1::Models,
        "views" => ShowCategoryV1::Views,
        "rules" => ShowCategoryV1::Rules,
        "packs" => ShowCategoryV1::Packs,
        "profiles" => ShowCategoryV1::Profiles,
        "gates" => ShowCategoryV1::Gates,
        other => {
            return Err(reject(
                cur,
                cat_span,
                format!("unsupported SHOW category '{other}'"),
            ))
        }
    };
    let span = kw.span.merge(&cat_span);
    let kind = PresentationKindV1::Show { category };
    cur.reserve_show_box_v1(source, span)?;
    Ok(ProgramItemHirV1::from(cur.builder.presentation(kind, span)))
}

// ---- DESCRIBE --------------------------------------------------------------

pub(super) fn parse_describe(
    cur: &mut CursorV1<'_>,
    source: &str,
) -> Result<ProgramItemHirV1, ParseDiagnosticV1> {
    let kw = cur.expect_keyword("describe", "'describe' to start an entity lookup")?;
    let (entity_kind, _kind_span) = cur.expect_ident("an entity kind")?;
    // `kind::path`
    cur.expect(&TokenKindV1::Colon, "':' in a 'kind::path' target")?;
    cur.expect(&TokenKindV1::Colon, "'::' in a 'kind::path' target")?;
    let path = parse_ref(cur, source, "an entity path")?;
    let span = kw.span.merge(&path.span);
    let kind = PresentationKindV1::Describe {
        kind: entity_kind,
        path,
    };
    let presentation = cur.builder.presentation(kind, span);
    boxed_program_item_v3(cur, presentation, span)
}

// ---- SAMPLE ----------------------------------------------------------------

pub(super) fn parse_sample(
    cur: &mut CursorV1<'_>,
    source: &str,
) -> Result<ProgramItemHirV1, ParseDiagnosticV1> {
    let kw = cur.expect_keyword("sample", "'sample' to start a retrieval")?;
    // `query="text"`
    let (q_key, _q_span) = cur.expect_ident("the 'query' key")?;
    if q_key != "query" {
        return Err(reject(
            cur,
            _q_span,
            "SAMPLE must start with a 'query' clause",
        ));
    }
    cur.expect_op("=", "'=' after 'query'")?;
    let (query_text, q_text_span) = expect_string(cur, "a quoted query text")?;
    if query_text.is_empty() {
        return Err(reject(
            cur,
            q_text_span,
            "SAMPLE query_text must not be empty",
        ));
    }
    let mut end = q_text_span.end;

    let mut mode = SampleModeV1::Lexical;
    let mut limit: Option<(u32, SpanV1)> = None;
    let mut scope = None;

    let mut seen = 0u64;
    while at_clause(cur) {
        let (key, key_span) = clause_key(cur)?;
        note_clause(cur, &mut seen, "SAMPLE", &key, key_span)?;
        match key.as_str() {
            "mode" => {
                let (v, sp) = cur.expect_ident("a sample mode")?;
                end = sp.end;
                mode = match v.as_str() {
                    "lexical" => SampleModeV1::Lexical,
                    "semantic" => SampleModeV1::Semantic,
                    "hybrid" => SampleModeV1::Hybrid,
                    other => {
                        return Err(reject(
                            cur,
                            sp,
                            format!("unsupported sample mode '{other}'"),
                        ))
                    }
                };
            }
            "limit" => {
                let (n, sp) = parse_u32(cur, "a positive limit")?;
                end = sp.end;
                limit = Some((n, sp));
            }
            "scope" => {
                let r = parse_ref(cur, source, "a scope reference")?;
                end = r.span.end;
                scope = Some(r);
            }
            other => {
                return Err(reject(
                    cur,
                    key_span,
                    format!("unsupported SAMPLE clause '{other}'"),
                ))
            }
        }
    }

    let limit = match limit {
        Some((n, _sp)) if n > 0 => n,
        Some((_, sp)) => return Err(reject(cur, sp, "SAMPLE limit must be > 0")),
        None => return Err(reject(cur, kw.span, "SAMPLE requires a 'limit' clause")),
    };

    let span = kw.span.merge(&SpanV1::point(end));
    let kind = PresentationKindV1::Sample {
        query_text,
        mode,
        limit,
        scope,
    };
    let presentation = cur.builder.presentation(kind, span);
    boxed_program_item_v3(cur, presentation, span)
}

// ---- COMPARE ---------------------------------------------------------------

pub(super) fn parse_compare(
    cur: &mut CursorV1<'_>,
    source: &str,
) -> Result<ProgramItemHirV1, ParseDiagnosticV1> {
    let kw = cur.expect_keyword("compare", "'compare' to start a delta")?;
    // `baseline=<ref>`
    let (b_key, b_span) = cur.expect_ident("the 'baseline' key")?;
    if b_key != "baseline" {
        return Err(reject(
            cur,
            b_span,
            "COMPARE must start with a 'baseline' clause",
        ));
    }
    cur.expect_op("=", "'=' after 'baseline'")?;
    let baseline_ref = parse_ref(cur, source, "a baseline reference")?;
    let mut end = baseline_ref.span.end;

    let mut candidate_ref = None;
    let mut metric = CompareMetricV1::Exact;

    let mut seen = 0u64;
    while at_clause(cur) {
        let (key, key_span) = clause_key(cur)?;
        note_clause(cur, &mut seen, "COMPARE", &key, key_span)?;
        match key.as_str() {
            "candidate" => {
                let r = parse_ref(cur, source, "a candidate reference")?;
                end = r.span.end;
                candidate_ref = Some(r);
            }
            "metric" => {
                let (v, sp) = cur.expect_ident("a comparison metric")?;
                end = sp.end;
                metric = match v.as_str() {
                    "exact" => CompareMetricV1::Exact,
                    "structural" => CompareMetricV1::Structural,
                    "semantic" => CompareMetricV1::Semantic,
                    other => return Err(reject(cur, sp, format!("unsupported metric '{other}'"))),
                };
            }
            other => {
                return Err(reject(
                    cur,
                    key_span,
                    format!("unsupported COMPARE clause '{other}'"),
                ))
            }
        }
    }

    let candidate_ref = candidate_ref.ok_or_else(|| {
        reject(
            cur,
            kw.span,
            "COMPARE requires an explicit 'candidate' clause",
        )
    })?;

    let span = kw.span.merge(&SpanV1::point(end));
    let kind = PresentationKindV1::Compare {
        baseline_ref,
        candidate_ref,
        metric,
    };
    let presentation = cur.builder.presentation(kind, span);
    boxed_program_item_v3(cur, presentation, span)
}

// ---- EXPORT ----------------------------------------------------------------

pub(super) fn parse_export(
    cur: &mut CursorV1<'_>,
    source: &str,
) -> Result<ProgramItemHirV1, ParseDiagnosticV1> {
    let kw = cur.expect_keyword("export", "'export' to start a bridge receipt")?;
    let (s_key, s_span) = cur.expect_ident("the 'source' key")?;
    if s_key != "source" {
        return Err(reject(
            cur,
            s_span,
            "EXPORT must start with a 'source' clause",
        ));
    }
    cur.expect_op("=", "'=' after 'source'")?;
    let source_ref = parse_ref(cur, source, "a source reference")?;
    let mut end = source_ref.span.end;

    let mut format = None;
    let mut profile = None;

    let mut seen = 0u64;
    while at_clause(cur) {
        let (key, key_span) = clause_key(cur)?;
        note_clause(cur, &mut seen, "EXPORT", &key, key_span)?;
        match key.as_str() {
            "kind" => {
                let r = parse_ref(cur, source, "an export kind")?;
                end = r.span.end;
                format = Some(match r.name.as_str() {
                    "json" => ExportFormatV1::Json,
                    "csv" => ExportFormatV1::Csv,
                    "sarif" => ExportFormatV1::Sarif,
                    "markdown" => ExportFormatV1::Markdown,
                    "graph-ml" => ExportFormatV1::GraphMl,
                    other => {
                        return Err(reject(
                            cur,
                            r.span,
                            format!("unsupported export kind '{other}'"),
                        ))
                    }
                });
            }
            "profile" => {
                let r = parse_ref(cur, source, "a profile reference")?;
                end = r.span.end;
                profile = Some(r);
            }
            other => {
                return Err(reject(
                    cur,
                    key_span,
                    format!("unsupported EXPORT clause '{other}'"),
                ))
            }
        }
    }

    let format = format.ok_or_else(|| reject(cur, kw.span, "EXPORT requires a 'kind' clause"))?;

    let span = kw.span.merge(&SpanV1::point(end));
    let kind = PresentationKindV1::Export {
        source_ref,
        format,
        profile,
    };
    let presentation = cur.builder.presentation(kind, span);
    boxed_program_item_v3(cur, presentation, span)
}
