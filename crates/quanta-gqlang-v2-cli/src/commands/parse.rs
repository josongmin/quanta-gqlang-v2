use super::query;

use quanta_gqlang_v2_ir::frontend::DocumentAstV1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseReportV1 {
    pub root_count: usize,
    pub root_kinds: Vec<&'static str>,
    pub root_summaries: Vec<String>,
}

impl ParseReportV1 {
    fn from_document_v1(doc: &DocumentAstV1) -> Self {
        Self {
            root_count: doc.roots.len(),
            root_kinds: doc.roots.iter().map(query::syntax_root_kind_v1).collect(),
            root_summaries: doc
                .roots
                .iter()
                .map(query::syntax_root_summary_v1)
                .collect(),
        }
    }

    pub fn render_v1(&self) -> String {
        let mut lines = vec![format!("parse.root_count={}", self.root_count)];
        for (index, (kind, summary)) in self
            .root_kinds
            .iter()
            .zip(self.root_summaries.iter())
            .enumerate()
        {
            lines.push(format!("parse.root[{index}].kind={kind}"));
            lines.push(format!("parse.root[{index}].summary={summary}"));
        }
        lines.join("\n")
    }
}

pub fn run(input: &str) -> Result<ParseReportV1, String> {
    let doc = query::parse_document_v1(input).map_err(query::render_syntax_page_v1)?;
    Ok(ParseReportV1::from_document_v1(&doc))
}
