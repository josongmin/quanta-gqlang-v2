use crate::handlers::starter_scaffold;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ZeroQuerySuggestionV1 {
    pub label: &'static str,
    pub root_kind: &'static str,
    pub query_scaffold: &'static str,
    pub behavior: &'static str,
    pub reason_code: &'static str,
    pub reason_detail: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ZeroQueryGroupV1 {
    pub id: &'static str,
    pub title: &'static str,
    pub suggestions: Vec<ZeroQuerySuggestionV1>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ZeroQueryResponseV1 {
    pub mode: &'static str,
    pub provenance: &'static str,
    pub groups: Vec<ZeroQueryGroupV1>,
}

impl ZeroQueryResponseV1 {
    #[must_use]
    pub fn render_v1(&self) -> String {
        let mut lines = vec![
            format!("zero_query.mode={}", self.mode),
            format!("zero_query.provenance={}", self.provenance),
        ];

        for (group_index, group) in self.groups.iter().enumerate() {
            lines.push(format!("zero_query.group[{group_index}].id={}", group.id));
            lines.push(format!(
                "zero_query.group[{group_index}].title={}",
                group.title
            ));
            for (suggestion_index, suggestion) in group.suggestions.iter().enumerate() {
                lines.push(format!(
                    "zero_query.group[{group_index}].suggestion[{suggestion_index}].label={}",
                    suggestion.label
                ));
                lines.push(format!(
                    "zero_query.group[{group_index}].suggestion[{suggestion_index}].root_kind={}",
                    suggestion.root_kind
                ));
                lines.push(format!(
                    "zero_query.group[{group_index}].suggestion[{suggestion_index}].query={}",
                    suggestion.query_scaffold
                ));
                lines.push(format!(
                    "zero_query.group[{group_index}].suggestion[{suggestion_index}].behavior={}",
                    suggestion.behavior
                ));
                lines.push(format!(
                    "zero_query.group[{group_index}].suggestion[{suggestion_index}].reason.code={}",
                    suggestion.reason_code
                ));
                lines.push(format!(
                    "zero_query.group[{group_index}].suggestion[{suggestion_index}].reason.detail={}",
                    suggestion.reason_detail
                ));
            }
        }

        lines.join("\n")
    }
}

pub fn handle_zero_query() -> Result<ZeroQueryResponseV1, String> {
    starter_scaffold::build_starter_scaffold_v1().map(ZeroQueryResponseV1::from_scaffold_v1)
}

impl ZeroQueryResponseV1 {
    fn from_scaffold_v1(scaffold: starter_scaffold::StarterScaffoldReportV1) -> Self {
        Self {
            mode: scaffold.mode.label_v1(),
            provenance: scaffold.provenance.label_v1(),
            groups: scaffold
                .groups
                .into_iter()
                .map(|group| ZeroQueryGroupV1 {
                    id: group.kind.id_v1(),
                    title: group.kind.title_v1(),
                    suggestions: group
                        .suggestions
                        .into_iter()
                        .map(|suggestion| ZeroQuerySuggestionV1 {
                            label: suggestion.label,
                            root_kind: suggestion.root_kind.id_v1(),
                            query_scaffold: suggestion.query_scaffold,
                            behavior: suggestion.behavior.label_v1(),
                            reason_code: suggestion.reason.code.label_v1(),
                            reason_detail: suggestion.reason.detail,
                        })
                        .collect(),
                })
                .collect(),
        }
    }
}
