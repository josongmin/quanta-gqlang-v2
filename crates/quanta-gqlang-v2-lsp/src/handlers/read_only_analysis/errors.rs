use super::{DiagnosticClassV1, ReadOnlyLspErrorV1};
use quanta_gqlang_v2_ir::frontend::SyntaxDiagnosticPageV1;

pub(super) fn syntax_page_to_lsp_error_v1(page: SyntaxDiagnosticPageV1) -> ReadOnlyLspErrorV1 {
    let Some(diagnostic) = page.diagnostics.into_iter().next() else {
        return syntax_error_v1(
            "unknown_syntax_error",
            "syntax rejected input without diagnostics",
        );
    };
    ReadOnlyLspErrorV1 {
        class: diagnostic_class_for_syntax_code_v1(diagnostic.code),
        code: diagnostic.code,
        message: diagnostic.message,
    }
}

pub(super) fn diagnostic_class_for_syntax_code_v1(code: &'static str) -> DiagnosticClassV1 {
    // The canonical parser is the single source of diagnostics. A handful of its
    // codes are semantic-value rejections (an unsupported enum value / activation
    // for an otherwise well-formed statement) which the read-only surface presents
    // as the `sema` class; every other shape/keyword error stays `syntax`.
    match code {
        "unsupported_show_category"
        | "unsupported_find_subkind"
        | "unsupported_describe_kind"
        | "use_pack_invalid_activation" => DiagnosticClassV1::Sema,
        _ => DiagnosticClassV1::Syntax,
    }
}

pub(super) fn syntax_error_v1(
    code: &'static str,
    message: impl Into<String>,
) -> ReadOnlyLspErrorV1 {
    ReadOnlyLspErrorV1 {
        class: DiagnosticClassV1::Syntax,
        code,
        message: message.into(),
    }
}
