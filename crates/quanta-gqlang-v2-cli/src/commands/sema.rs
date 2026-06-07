use super::query;

use quanta_gqlang_v2_ir::frontend::TypedDocumentV1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SemaReportV1 {
    pub root_count: usize,
    pub root_kinds: Vec<&'static str>,
    pub root_summaries: Vec<String>,
}

impl SemaReportV1 {
    fn from_typed_document_v1(doc: &TypedDocumentV1) -> Self {
        Self {
            root_count: doc.roots.len(),
            root_kinds: doc.roots.iter().map(query::typed_root_kind_v1).collect(),
            root_summaries: doc.roots.iter().map(query::typed_root_summary_v1).collect(),
        }
    }

    pub fn render_v1(&self) -> String {
        let mut lines = vec![format!("sema.root_count={}", self.root_count)];
        for (index, (kind, summary)) in self
            .root_kinds
            .iter()
            .zip(self.root_summaries.iter())
            .enumerate()
        {
            lines.push(format!("sema.root[{index}].kind={kind}"));
            lines.push(format!("sema.root[{index}].summary={summary}"));
        }
        lines.join("\n")
    }
}

pub fn run(input: &str) -> Result<SemaReportV1, String> {
    let doc = query::parse_document_v1(input).map_err(query::render_syntax_page_v1)?;
    let typed = query::analyze_document_v1(&doc).map_err(query::render_sema_page_v1)?;
    Ok(SemaReportV1::from_typed_document_v1(&typed))
}
