use crate::handlers::read_only_analysis::analyze_query_v1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HoverCardV1 {
    pub root_keyword: String,
    pub category: String,
    pub typed_target: String,
    pub summary: Vec<String>,
}

/// Thin adapter over the shared read-only analysis owner: present the first
/// statement's validated summary as a hover card.
pub fn handle_hover(query: &str) -> Result<HoverCardV1, String> {
    analyze_query_v1(query)
        .map(|summary| HoverCardV1 {
            root_keyword: summary.root_keyword.to_string(),
            category: summary.category.to_string(),
            typed_target: summary.typed_target,
            summary: summary.summary,
        })
        .map_err(|err| err.render_v1())
}
