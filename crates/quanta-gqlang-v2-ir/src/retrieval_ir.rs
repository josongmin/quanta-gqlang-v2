#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RetrievalIrV1 {
    pub query: String,
    pub scope: Option<InteractiveScopeIrV1>,
    pub mode: RetrievalModeIrV1,
    pub limit: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InteractiveScopeIrV1 {
    pub kind: String,
    pub locator: String,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RetrievalModeIrV1 {
    Lexical,
    Semantic,
    Hybrid,
}
