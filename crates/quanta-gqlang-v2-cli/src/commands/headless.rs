use super::query;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HeadlessReadReportV1 {
    pub mode: &'static str,
    pub command_plane: &'static str,
    pub root_count: usize,
    pub root_kinds: Vec<&'static str>,
    pub no_result_reason: &'static str,
    pub degraded_reasons: Vec<String>,
    pub query_summary: query::QuerySummaryV1,
}

impl HeadlessReadReportV1 {
    pub fn render_v1(&self) -> String {
        let mut lines = vec![
            format!("headless.mode={}", self.mode),
            format!("headless.command_plane={}", self.command_plane),
            format!("headless.root_count={}", self.root_count),
            format!("headless.no_result_reason={}", self.no_result_reason),
        ];
        for (index, kind) in self.root_kinds.iter().enumerate() {
            lines.push(format!("headless.root[{index}].kind={kind}"));
        }
        for (index, reason) in self.degraded_reasons.iter().enumerate() {
            lines.push(format!("headless.degraded[{index}]={reason}"));
        }
        lines.push(self.query_summary.render_v1());
        lines.join("\n")
    }
}

pub fn run(input: &str) -> Result<HeadlessReadReportV1, String> {
    let summary = query::run(input)?;
    Ok(HeadlessReadReportV1 {
        mode: "read-only",
        command_plane: "forbidden",
        root_count: summary.root_count,
        root_kinds: summary.roots.iter().map(|root| root.syntax_kind).collect(),
        no_result_reason: "read-only-shell-has-no-execution-backend",
        degraded_reasons: summary
            .roots
            .iter()
            .filter_map(|root| {
                root.degraded_reason
                    .map(|reason| reason.code_v1().to_string())
            })
            .collect(),
        query_summary: summary,
    })
}
