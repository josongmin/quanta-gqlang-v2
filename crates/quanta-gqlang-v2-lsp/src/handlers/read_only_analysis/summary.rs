#[path = "common.rs"]
mod common;
#[path = "composition.rs"]
mod composition;
#[path = "declarations.rs"]
mod declarations;
#[path = "meta.rs"]
mod meta;
#[path = "select.rs"]
mod select;

use self::common::root_summary_v1;
use self::composition::composition_summary_v1;
use self::declarations::declaration_summary_from_carrier_v1;
use self::meta::meta_summary_v1;
use self::select::select_summary_v1;
use super::ReadOnlyRootSummaryV1;
use quanta_gqlang_v2_ir::frontend::{
    AstRootV1, CompareRootAstV1, DescribeRootAstV1, ExportRootAstV1, FindRootAstV1, GateDeclAstV1,
    ModelDeclAstV1, PackDeclAstV1, ProfileDeclAstV1, RuleDeclAstV1, SampleRootAstV1, ShowRootAstV1,
    ViewDeclAstV1,
};

/// Build the read-only presentation from a canonically-validated typed root. This
/// is pure formatting — every accept/reject decision was already made by the
/// parser — so it can never diverge from the grammar.
pub(super) fn summary_from_root_v1(root: &AstRootV1) -> ReadOnlyRootSummaryV1 {
    match root {
        AstRootV1::Select(spec) => select_summary_v1(spec),
        AstRootV1::Find(FindRootAstV1 { subkind, seed }) => root_summary_v1(
            "FIND",
            "query",
            format!("find::{}", subkind.as_str()),
            vec![format!("seed={}", seed.trim())],
        ),
        AstRootV1::Check(spec) => {
            let summary = match spec.intent() {
                Ok(intent) => vec![
                    format!("class={}", intent.class),
                    format!("scope={:?}", intent.scope),
                    format!("pattern={:?}", intent.pattern),
                    format!("confirm={:?}", intent.confirm),
                ],
                Err(_) => vec![format!("scope={}", spec.scope.trim())],
            };
            root_summary_v1("CHECK", "query", spec.rule_id.clone(), summary)
        }
        AstRootV1::Explain(spec) => {
            let (typed_target, summary) = match spec.shape() {
                Ok(shape) => (
                    shape.target.clone(),
                    vec![
                        format!("evidence={:?}", shape.evidence),
                        format!("truth={:?}", shape.truth_expectation),
                    ],
                ),
                Err(_) => (spec.target.clone().unwrap_or_default(), Vec::new()),
            };
            root_summary_v1("EXPLAIN", "query", typed_target, summary)
        }
        AstRootV1::Show(ShowRootAstV1 { category }) => root_summary_v1(
            "SHOW",
            "interactive",
            category.clone(),
            vec![format!("catalog_category={category}")],
        ),
        AstRootV1::Describe(DescribeRootAstV1 { target }) => root_summary_v1(
            "DESCRIBE",
            "interactive",
            target.clone(),
            vec![format!("target={target}")],
        ),
        AstRootV1::Sample(SampleRootAstV1 { query_text }) => root_summary_v1(
            "SAMPLE",
            "interactive",
            query_text.clone(),
            vec![format!("payload={query_text}")],
        ),
        AstRootV1::Compare(CompareRootAstV1 { baseline_ref }) => root_summary_v1(
            "COMPARE",
            "interactive",
            baseline_ref.clone(),
            vec![format!("payload={baseline_ref}")],
        ),
        AstRootV1::Export(ExportRootAstV1 { source_ref }) => root_summary_v1(
            "EXPORT",
            "interactive",
            source_ref.clone(),
            vec![format!("payload={source_ref}")],
        ),
        AstRootV1::View(ViewDeclAstV1 { name }) => {
            declaration_summary_from_carrier_v1("VIEW", name)
        }
        AstRootV1::Rule(RuleDeclAstV1 { name }) => {
            declaration_summary_from_carrier_v1("RULE", name)
        }
        AstRootV1::Model(ModelDeclAstV1 { name }) => {
            declaration_summary_from_carrier_v1("MODEL", name)
        }
        AstRootV1::Pack(PackDeclAstV1 { name }) => {
            declaration_summary_from_carrier_v1("PACK", name)
        }
        AstRootV1::Profile(ProfileDeclAstV1 { name }) => {
            declaration_summary_from_carrier_v1("PROFILE", name)
        }
        AstRootV1::Gate(GateDeclAstV1 { name }) => {
            declaration_summary_from_carrier_v1("GATE", name)
        }
        AstRootV1::Meta(command) => meta_summary_v1(command),
        AstRootV1::Composition(spec) => {
            // D2: the composition block IS the first statement. Summarize the
            // grouped block — the body's own root summary is reused for the body
            // symbol/label rather than re-derived (no shadow grammar).
            let lets: Vec<(String, &AstRootV1)> = spec
                .lets
                .iter()
                .map(|binding| (binding.name.clone(), binding.producer.as_ref()))
                .collect();
            composition_summary_v1(&lets, &spec.with_name, spec.body.as_ref())
        }
    }
}
