use crate::commands::query;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DoctorStatusV1 {
    ParseError,
    SemaError,
    NoResult,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DoctorReportV1 {
    pub status: DoctorStatusV1,
    pub root_kinds: Vec<&'static str>,
    pub parse_diagnostics: Vec<String>,
    pub sema_diagnostics: Vec<String>,
    pub no_result_reason: Option<String>,
    pub degraded_reasons: Vec<String>,
    pub query_summary: Option<query::QuerySummaryV1>,
}

impl DoctorReportV1 {
    pub fn render_v1(&self) -> String {
        let mut lines = vec![format!("doctor.status={}", self.status.as_str_v1())];
        for (index, kind) in self.root_kinds.iter().enumerate() {
            lines.push(format!("doctor.root[{index}].kind={kind}"));
        }
        for (index, diagnostic) in self.parse_diagnostics.iter().enumerate() {
            lines.push(format!("doctor.parse_diagnostic[{index}]={diagnostic}"));
        }
        for (index, diagnostic) in self.sema_diagnostics.iter().enumerate() {
            lines.push(format!("doctor.sema_diagnostic[{index}]={diagnostic}"));
        }
        if let Some(reason) = &self.no_result_reason {
            lines.push(format!("doctor.no_result_reason={reason}"));
        }
        for (index, reason) in self.degraded_reasons.iter().enumerate() {
            lines.push(format!("doctor.degraded_reason[{index}]={reason}"));
        }
        if let Some(summary) = &self.query_summary {
            lines.push(summary.render_v1());
        }
        lines.join("\n")
    }
}

impl DoctorStatusV1 {
    pub const fn as_str_v1(&self) -> &'static str {
        match self {
            Self::ParseError => "parse-error",
            Self::SemaError => "sema-error",
            Self::NoResult => "no-result",
        }
    }
}

pub fn run_v1(input: &str) -> Result<DoctorReportV1, String> {
    let doc = match query::parse_document_v1(input) {
        Ok(doc) => doc,
        Err(page) => {
            return Ok(DoctorReportV1 {
                status: DoctorStatusV1::ParseError,
                root_kinds: Vec::new(),
                parse_diagnostics: page
                    .diagnostics
                    .iter()
                    .map(|diag| format!("{}: {}", diag.code, diag.message))
                    .collect(),
                sema_diagnostics: Vec::new(),
                no_result_reason: None,
                degraded_reasons: Vec::new(),
                query_summary: None,
            });
        }
    };

    let typed = match query::analyze_document_v1(&doc) {
        Ok(typed) => typed,
        Err(page) => {
            return Ok(DoctorReportV1 {
                status: DoctorStatusV1::SemaError,
                root_kinds: doc.roots.iter().map(query::syntax_root_kind_v1).collect(),
                parse_diagnostics: Vec::new(),
                sema_diagnostics: page.diagnostics,
                no_result_reason: None,
                degraded_reasons: Vec::new(),
                query_summary: None,
            });
        }
    };

    let ir_roots = query::lower_document_to_ir_v1(&typed).map_err(query::render_ir_error_v1)?;
    let summary = query::build_query_summary_v1(&doc, &typed, &ir_roots);
    let degraded_reasons = summary
        .roots
        .iter()
        .filter_map(|root| match root.ir_kind {
            "find" => Some("runtime-host-routing-unavailable".to_string()),
            "check" => Some("semantic-confirm-lane-unavailable".to_string()),
            "explain" => Some("truth-evidence-materialization-unavailable".to_string()),
            "retrieval" => Some("retrieval-result-materialization-unavailable".to_string()),
            "interactive" => Some("catalog-and-interactive-host-truth-unavailable".to_string()),
            _ => None,
        })
        .collect();

    Ok(DoctorReportV1 {
        status: DoctorStatusV1::NoResult,
        root_kinds: summary.roots.iter().map(|root| root.syntax_kind).collect(),
        parse_diagnostics: Vec::new(),
        sema_diagnostics: Vec::new(),
        no_result_reason: Some("read-only-shell-has-no-execution-backend".to_string()),
        degraded_reasons,
        query_summary: Some(summary),
    })
}
