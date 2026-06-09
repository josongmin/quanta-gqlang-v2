use super::{DiagnosticClassV1, ReadOnlyLspErrorV1};
use quanta_gqlang_v2_ir::diagnostic_class::{
    front_door_diagnostic_class_v1, FrontDoorDiagnosticClassV1,
};
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
    // The syntax-code -> display-class mapping is owned by the ir front-door
    // authority so the LSP surface and the conformance rails cannot drift.
    match front_door_diagnostic_class_v1(code) {
        FrontDoorDiagnosticClassV1::Sema => DiagnosticClassV1::Sema,
        FrontDoorDiagnosticClassV1::Syntax => DiagnosticClassV1::Syntax,
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
