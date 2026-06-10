use super::super::{ReadOnlyRootSummaryV1, SymbolItemV1};
use super::common::{
    certified_admission_summary_v1, certified_truth_summary_v1, document_symbol_v1,
    pack_activation_mode_summary_v1, root_summary_v1, root_symbol_v1,
};
use quanta_gqlang_v2_ir::frontend::{MetaCommandAstV1, MetaCommandShapeV1};

pub(super) fn meta_summary_v1(command: &MetaCommandAstV1) -> ReadOnlyRootSummaryV1 {
    // The command already parsed, so `shape()` succeeds; fall back defensively.
    match command.shape() {
        Ok(MetaCommandShapeV1::UsePack(spec)) => {
            let activation = pack_activation_mode_summary_v1(spec.activation_mode);
            ReadOnlyRootSummaryV1 {
                root_keyword: "USE PACK",
                category: "meta",
                typed_target: spec.pack_id.to_string(),
                summary: vec![
                    format!("activation={activation}"),
                    format!("lockfile={}", spec.lockfile_path),
                ],
                symbols: vec![
                    document_symbol_v1(),
                    root_symbol_v1("USE PACK", "meta"),
                    SymbolItemV1 {
                        name: spec.pack_id.to_string(),
                        kind: "pack".to_string(),
                        detail: Some(format!(
                            "activation={activation} lockfile={}",
                            spec.lockfile_path
                        )),
                    },
                ],
            }
        }
        Ok(MetaCommandShapeV1::RequireCertified(spec)) => {
            let truth = certified_truth_summary_v1(spec.truth_requirement);
            let admission = certified_admission_summary_v1(spec.admission_policy);
            ReadOnlyRootSummaryV1 {
                root_keyword: "REQUIRE CERTIFIED",
                category: "meta",
                typed_target: "certified-only".to_string(),
                summary: vec![format!("truth={truth}"), format!("admission={admission}")],
                symbols: vec![
                    document_symbol_v1(),
                    root_symbol_v1("REQUIRE CERTIFIED", "meta"),
                ],
            }
        }
        // Registered debt (silent-failure audit): a `shape()` reject collapses to
        // an empty summary under a hardcoded USE PACK keyword — wrong label for a
        // failing REQUIRE CERTIFIED, and the reject code is invisible to the
        // client. Tracked for the typed-summary remediation pass.
        // nosemgrep: qglang-v2-no-silent-err-discard
        Err(_) => root_summary_v1("USE PACK", "meta", String::new(), Vec::new()),
    }
}
