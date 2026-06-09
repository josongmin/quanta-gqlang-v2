//! Read-only frontend surface for the `ir`-only consumers (`cli`, `lsp`).
//!
//! Per `tools/architecture/check_architecture.py`, `cli` and `lsp` may depend on
//! `ir` alone — never on `sema` or `syntax` directly. This module is the single,
//! intentionally narrow hub through which the syntax- and sema-owned frontend
//! types reach those crates. It re-exports ONLY the symbols the read-only
//! consumers actually use: the SELECT root/projection/field-ref types that the
//! LSP read-only summary and CLI query renderer need ARE surfaced, but the
//! internal SELECT filter/literal/shape vocabulary (`SelectFilterAstV1`,
//! `SelectLiteralAstV1`, `SelectQueryShapeV1`, `SelectTableRefAstV1`,
//! `SelectRootErrorV1`, `parse_select_shape_v1`) and the meta shape-error type
//! (`MetaCommandShapeErrorV1`) are deliberately withheld. The exports are grouped
//! by their owning layer so crate provenance stays visible at the boundary.

// Owned by `quanta-gqlang-v2-syntax` (re-exported here through `sema`): the parser
// entry points, the document/diagnostic carriers, and the untyped AST roots.
pub use quanta_gqlang_v2_sema::{
    document_statements_v1, format_document_v1, parse_document_v1, AstRootV1,
    CertifiedAdmissionPolicyAstV1, CertifiedTruthRequirementAstV1, CheckRootAstV1,
    CheckRootIntentV1, CompareRootAstV1, DescribeRootAstV1, DocumentAstV1, ExplainRootAstV1,
    ExplainShapeAstV1, ExportRootAstV1, FindRootAstV1, GateDeclAstV1, MetaCommandAstV1,
    MetaCommandShapeV1, ModelDeclAstV1, PackActivationModeAstV1, PackDeclAstV1, ProfileDeclAstV1,
    RuleDeclAstV1, SampleRootAstV1, SelectFieldRefAstV1, SelectProjectionAstV1, SelectRootAstV1,
    ShowRootAstV1, SyntaxDiagnosticPageV1, ViewDeclAstV1,
};
// Owned by `quanta-gqlang-v2-sema`: the semantic analyzer entry point, its
// diagnostic page, and the typed roots produced by analysis.
pub use quanta_gqlang_v2_sema::{
    analyze_document_v1, SemaDiagnosticPageV1, TypedCertifiedAdmissionPolicyV1,
    TypedCertifiedTruthRequirementV1, TypedDocumentV1, TypedMetaCommandV1,
    TypedPackActivationModeV1, TypedRootV1,
};
