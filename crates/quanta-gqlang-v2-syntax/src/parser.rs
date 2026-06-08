use crate::ast::{AstRootV1, DocumentAstV1};
use crate::declarations::pack::{GateDeclAstV1, PackDeclAstV1, ProfileDeclAstV1};
use crate::declarations::view::{ModelDeclAstV1, RuleDeclAstV1, ViewDeclAstV1};
use crate::diagnostics::{SyntaxDiagnosticPageV1, SyntaxDiagnosticV1};
use crate::interactive::meta::MetaCommandAstV1;
use crate::interactive::sample::{CompareRootAstV1, ExportRootAstV1, SampleRootAstV1};
use crate::interactive::show::{parse_describe_root_v1, parse_show_root_v1};
use crate::roots::check::CheckRootAstV1;
use crate::roots::explain::ExplainRootAstV1;
use crate::roots::find::{FindRootAstV1, FindSubkindV1};

pub fn parse_document_v1(input: &str) -> Result<DocumentAstV1, SyntaxDiagnosticPageV1> {
    if input.trim().is_empty() {
        return Err(diagnostic_page_v1("empty_document", "document is empty"));
    }

    let mut roots = Vec::new();
    let mut diagnostics = Vec::new();
    for statement in document_statements_v1(input) {
        match parse_root_v1(&statement) {
            Ok(root) => roots.push(root),
            // Error recovery: keep parsing the remaining statements so a single
            // request surfaces every malformed statement at once instead of
            // stopping at the first failure.
            Err(page) => diagnostics.extend(page.diagnostics),
        }
    }

    if !diagnostics.is_empty() {
        return Err(SyntaxDiagnosticPageV1 { diagnostics });
    }

    Ok(DocumentAstV1 {
        roots,
        with_clause: None,
    })
}

/// Split a document into statements: exactly one per non-blank line. The canonical
/// form produced by `format_document_v1` is one root per line, so statement
/// boundaries are physical newlines. This is deliberately a *line*-oriented
/// contract with no implicit line continuation: a statement may not span lines and
/// a string literal may not contain a newline. The alternative — gluing lines that
/// do not begin with a root keyword onto the previous statement — is ambiguous on a
/// terminator-less grammar (it silently swallows a typo'd root keyword, wrongly
/// splits a wrapped line that begins with a keyword-like identifier, and rewrites
/// newlines
/// inside literals), so every line is parsed independently and surfaces its own
/// honest diagnostic. Blank lines are ignored.
#[must_use]
pub fn document_statements_v1(input: &str) -> Vec<String> {
    input
        .lines()
        .map(str::trim)
        .filter(|statement| !statement.is_empty())
        .map(str::to_string)
        .collect()
}

fn parse_root_v1(input: &str) -> Result<AstRootV1, SyntaxDiagnosticPageV1> {
    let (head, tail) = split_head_tail_v1(input);
    let head = head.to_ascii_uppercase();

    match head.as_str() {
        "SELECT" => parse_select_root_v1(tail),
        "FIND" => parse_find_root_v1(tail),
        "CHECK" => parse_check_root_v1(tail),
        "EXPLAIN" => parse_explain_root_v1(tail),
        "SHOW" => parse_show_interactive_root_v1(tail),
        "DESCRIBE" => parse_describe_interactive_root_v1(tail),
        "SAMPLE" => parse_sample_interactive_root_v1(tail),
        "COMPARE" => parse_compare_interactive_root_v1(tail),
        "EXPORT" => parse_export_interactive_root_v1(tail),
        "VIEW" => parse_view_declaration_root_v1(tail),
        "RULE" => parse_rule_declaration_root_v1(tail),
        "MODEL" => parse_model_declaration_root_v1(tail),
        "PACK" => parse_pack_declaration_root_v1(tail),
        "PROFILE" => parse_profile_declaration_root_v1(tail),
        "GATE" => parse_gate_declaration_root_v1(tail),
        "USE" => parse_use_meta_v1(tail),
        "REQUIRE" => parse_require_meta_v1(tail),
        _ => Err(diagnostic_page_v1(
            "unsupported_root",
            format!("unsupported root keyword `{}`", head),
        )),
    }
}

fn parse_select_root_v1(tail: &str) -> Result<AstRootV1, SyntaxDiagnosticPageV1> {
    let raw = if tail.trim().is_empty() {
        "SELECT".to_string()
    } else {
        format!("SELECT {tail}")
    };
    crate::roots::select::parse_select_root_v1(&raw)
        .map(AstRootV1::Select)
        .map_err(|err| diagnostic_page_v1(err.code, err.message))
}

fn parse_find_root_v1(tail: &str) -> Result<AstRootV1, SyntaxDiagnosticPageV1> {
    let (subkind, seed) = split_required_pair_v1(
        "find_missing_subkind_or_seed",
        "FIND requires `<subkind> <seed>`",
        tail,
    )?;

    let spec = FindRootAstV1 {
        subkind: parse_find_subkind_v1(&subkind)?,
        seed,
    };
    spec.parse_seed()
        .map_err(|err| diagnostic_page_v1("invalid_find_seed", err.message.clone()))?;
    Ok(AstRootV1::Find(spec))
}

fn parse_check_root_v1(tail: &str) -> Result<AstRootV1, SyntaxDiagnosticPageV1> {
    let (rule_id, scope) = split_required_pair_v1(
        "check_missing_rule_or_scope",
        "CHECK requires `<rule_id> <scope>`",
        tail,
    )?;

    let spec = CheckRootAstV1 { rule_id, scope };
    spec.intent()
        .map_err(|err| diagnostic_page_v1("invalid_check_payload", err.message))?;
    Ok(AstRootV1::Check(spec))
}

fn parse_explain_root_v1(tail: &str) -> Result<AstRootV1, SyntaxDiagnosticPageV1> {
    let spec = ExplainRootAstV1 {
        target: non_empty_option_v1(tail),
    };
    spec.shape()
        .map_err(|err| diagnostic_page_v1("invalid_explain_shape", err.message))?;
    Ok(AstRootV1::Explain(spec))
}

fn parse_show_interactive_root_v1(tail: &str) -> Result<AstRootV1, SyntaxDiagnosticPageV1> {
    let category = required_tail_v1("show_missing_category", "SHOW requires category", tail)?;
    parse_show_root_v1(&category)
        .map(AstRootV1::Show)
        .map_err(|err| diagnostic_page_v1(err.code, err.message))
}

fn parse_describe_interactive_root_v1(tail: &str) -> Result<AstRootV1, SyntaxDiagnosticPageV1> {
    let target = required_tail_v1("describe_missing_target", "DESCRIBE requires target", tail)?;
    parse_describe_root_v1(&target)
        .map(AstRootV1::Describe)
        .map_err(|err| diagnostic_page_v1(err.code, err.message))
}

fn parse_sample_interactive_root_v1(tail: &str) -> Result<AstRootV1, SyntaxDiagnosticPageV1> {
    let query_text = required_tail_v1(
        "sample_missing_query",
        "SAMPLE requires structured payload",
        tail,
    )?;
    let root = SampleRootAstV1 { query_text };
    let intent = root
        .intent()
        .map_err(|err| diagnostic_page_v1("invalid_sample_payload", err.message))?;
    Ok(AstRootV1::Sample(SampleRootAstV1::from_intent(intent)))
}

fn parse_compare_interactive_root_v1(tail: &str) -> Result<AstRootV1, SyntaxDiagnosticPageV1> {
    let baseline_ref = required_tail_v1(
        "compare_missing_baseline",
        "COMPARE requires structured payload",
        tail,
    )?;
    let root = CompareRootAstV1 { baseline_ref };
    let intent = root
        .intent()
        .map_err(|err| diagnostic_page_v1("invalid_compare_payload", err.message))?;
    Ok(AstRootV1::Compare(CompareRootAstV1::from_intent(intent)))
}

fn parse_export_interactive_root_v1(tail: &str) -> Result<AstRootV1, SyntaxDiagnosticPageV1> {
    let source_ref = required_tail_v1(
        "export_missing_source",
        "EXPORT requires structured payload",
        tail,
    )?;
    let root = ExportRootAstV1 { source_ref };
    let intent = root
        .intent()
        .map_err(|err| diagnostic_page_v1("invalid_export_payload", err.message))?;
    Ok(AstRootV1::Export(ExportRootAstV1::from_intent(intent)))
}

fn parse_view_declaration_root_v1(tail: &str) -> Result<AstRootV1, SyntaxDiagnosticPageV1> {
    let name = required_tail_v1("view_missing_name", "VIEW requires payload", tail)?;
    let spec = ViewDeclAstV1 { name };
    spec.shape_v1()
        .map_err(|err| diagnostic_page_v1("invalid_view_declaration", err.message))?;
    Ok(AstRootV1::View(spec))
}

fn parse_rule_declaration_root_v1(tail: &str) -> Result<AstRootV1, SyntaxDiagnosticPageV1> {
    let name = required_tail_v1("rule_missing_name", "RULE requires payload", tail)?;
    let spec = RuleDeclAstV1 { name };
    spec.shape_v1()
        .map_err(|err| diagnostic_page_v1("invalid_rule_declaration", err.message))?;
    Ok(AstRootV1::Rule(spec))
}

fn parse_model_declaration_root_v1(tail: &str) -> Result<AstRootV1, SyntaxDiagnosticPageV1> {
    let name = required_tail_v1("model_missing_name", "MODEL requires payload", tail)?;
    let spec = ModelDeclAstV1 { name };
    spec.shape_v1()
        .map_err(|err| diagnostic_page_v1("invalid_model_declaration", err.message))?;
    Ok(AstRootV1::Model(spec))
}

fn parse_pack_declaration_root_v1(tail: &str) -> Result<AstRootV1, SyntaxDiagnosticPageV1> {
    let name = required_tail_v1("pack_missing_name", "PACK requires payload", tail)?;
    let spec = PackDeclAstV1 { name };
    spec.shape()
        .map_err(|err| diagnostic_page_v1("invalid_pack_declaration", err.message))?;
    Ok(AstRootV1::Pack(spec))
}

fn parse_profile_declaration_root_v1(tail: &str) -> Result<AstRootV1, SyntaxDiagnosticPageV1> {
    let name = required_tail_v1("profile_missing_name", "PROFILE requires payload", tail)?;
    let spec = ProfileDeclAstV1 { name };
    spec.shape()
        .map_err(|err| diagnostic_page_v1("invalid_profile_declaration", err.message))?;
    Ok(AstRootV1::Profile(spec))
}

fn parse_gate_declaration_root_v1(tail: &str) -> Result<AstRootV1, SyntaxDiagnosticPageV1> {
    let name = required_tail_v1("gate_missing_name", "GATE requires payload", tail)?;
    let spec = GateDeclAstV1 { name };
    spec.shape()
        .map_err(|err| diagnostic_page_v1("invalid_gate_declaration", err.message))?;
    Ok(AstRootV1::Gate(spec))
}

fn parse_use_meta_v1(tail: &str) -> Result<AstRootV1, SyntaxDiagnosticPageV1> {
    let (keyword, pack_tail) =
        split_required_pair_v1("use_missing_pack", "USE PACK requires pack id", tail)?;
    if keyword.eq_ignore_ascii_case("PACK") {
        let (pack_id, fields) = parse_named_payload_v1(&pack_tail, "USE PACK")?;
        let mut activation_mode = None;
        let mut lockfile_path = None;

        for field in fields {
            match field.key.as_str() {
                "activation" => set_optional_meta_field_v1(
                    &mut activation_mode,
                    field.value,
                    "duplicate_use_pack_activation",
                    "USE PACK cannot repeat `activation` clause",
                )?,
                "lockfile" => set_optional_meta_field_v1(
                    &mut lockfile_path,
                    field.value,
                    "duplicate_use_pack_lockfile",
                    "USE PACK cannot repeat `lockfile` clause",
                )?,
                other => {
                    return Err(diagnostic_page_v1(
                        "unsupported_use_pack_clause",
                        format!("USE PACK does not support `{other}` clause"),
                    ));
                }
            }
        }

        let command = MetaCommandAstV1::UsePack {
            pack_id,
            activation_mode,
            lockfile_path,
        };
        command
            .shape()
            .map_err(|err| diagnostic_page_v1(err.code, err.message))?;
        Ok(AstRootV1::Meta(command))
    } else {
        Err(diagnostic_page_v1(
            "unsupported_use_meta",
            format!("unsupported USE target `{keyword}`"),
        ))
    }
}

fn parse_require_meta_v1(tail: &str) -> Result<AstRootV1, SyntaxDiagnosticPageV1> {
    if tail.eq_ignore_ascii_case("CERTIFIED") {
        let command = MetaCommandAstV1::RequireCertified;
        command
            .shape()
            .map_err(|err| diagnostic_page_v1(err.code, err.message))?;
        Ok(AstRootV1::Meta(command))
    } else {
        Err(diagnostic_page_v1(
            "unsupported_require_meta",
            format!("unsupported REQUIRE target `{tail}`"),
        ))
    }
}

fn parse_find_subkind_v1(input: &str) -> Result<FindSubkindV1, SyntaxDiagnosticPageV1> {
    match input.to_ascii_lowercase().as_str() {
        "reachability" => Ok(FindSubkindV1::Reachability),
        "flow" => Ok(FindSubkindV1::Flow),
        "slice" => Ok(FindSubkindV1::Slice),
        "taint" => Ok(FindSubkindV1::Taint),
        "path" => Ok(FindSubkindV1::Path),
        "points_to" => Ok(FindSubkindV1::PointsTo),
        "summary" => Ok(FindSubkindV1::Summary),
        _ => Err(diagnostic_page_v1(
            "unsupported_find_subkind",
            format!("unsupported FIND subkind `{input}`"),
        )),
    }
}

fn split_head_tail_v1(input: &str) -> (&str, &str) {
    let trimmed = input.trim();
    match trimmed.split_once(char::is_whitespace) {
        Some((head, tail)) => (head, tail.trim()),
        None => (trimmed, ""),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PayloadFieldV1 {
    key: String,
    value: String,
}

fn parse_named_payload_v1(
    input: &str,
    root_keyword: &'static str,
) -> Result<(String, Vec<PayloadFieldV1>), SyntaxDiagnosticPageV1> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(diagnostic_page_v1(
            "missing_named_target",
            format!("{root_keyword} requires a target name"),
        ));
    }

    let mut parts = trimmed.split(";;");
    let subject = parts.next().unwrap_or_default().trim();
    if subject.is_empty() {
        return Err(diagnostic_page_v1(
            "missing_named_target",
            format!("{root_keyword} requires a target name"),
        ));
    }

    let mut fields = Vec::new();
    for chunk in parts {
        fields.push(parse_payload_field_v1(chunk)?);
    }

    Ok((subject.to_string(), fields))
}

fn parse_payload_field_v1(input: &str) -> Result<PayloadFieldV1, SyntaxDiagnosticPageV1> {
    let chunk = input.trim();
    if chunk.is_empty() {
        return Err(diagnostic_page_v1(
            "empty_payload_clause",
            "payload clause cannot be empty",
        ));
    }

    let Some((key, value)) = chunk.split_once('=') else {
        return Err(diagnostic_page_v1(
            "invalid_payload_clause",
            format!("payload clause `{chunk}` must be `key=value`"),
        ));
    };

    let key = key.trim();
    let value = value.trim();
    if key.is_empty() || value.is_empty() {
        return Err(diagnostic_page_v1(
            "invalid_payload_clause",
            format!("payload clause `{chunk}` must include non-empty key and value"),
        ));
    }

    Ok(PayloadFieldV1 {
        key: key.to_ascii_lowercase(),
        value: value.to_string(),
    })
}

fn split_required_pair_v1(
    code: &'static str,
    message: &'static str,
    input: &str,
) -> Result<(String, String), SyntaxDiagnosticPageV1> {
    let trimmed = input.trim();
    match trimmed.split_once(char::is_whitespace) {
        Some((head, tail)) if !tail.trim().is_empty() => {
            Ok((head.trim().to_string(), tail.trim().to_string()))
        }
        _ => Err(diagnostic_page_v1(code, message)),
    }
}

fn required_tail_v1(
    code: &'static str,
    message: &'static str,
    input: &str,
) -> Result<String, SyntaxDiagnosticPageV1> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        Err(diagnostic_page_v1(code, message))
    } else {
        Ok(trimmed.to_string())
    }
}

fn set_optional_meta_field_v1(
    slot: &mut Option<String>,
    value: String,
    code: &'static str,
    message: &'static str,
) -> Result<(), SyntaxDiagnosticPageV1> {
    if slot.replace(value).is_some() {
        return Err(diagnostic_page_v1(code, message));
    }
    Ok(())
}

fn non_empty_option_v1(input: &str) -> Option<String> {
    let trimmed = input.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

fn diagnostic_page_v1(code: &'static str, message: impl Into<String>) -> SyntaxDiagnosticPageV1 {
    SyntaxDiagnosticPageV1 {
        diagnostics: vec![SyntaxDiagnosticV1 {
            code,
            message: message.into(),
            span: None,
        }],
    }
}
