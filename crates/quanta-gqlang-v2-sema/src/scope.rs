#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SemanticScopeV1 {
    pub repositories: Vec<String>,
    pub files: Vec<String>,
}
