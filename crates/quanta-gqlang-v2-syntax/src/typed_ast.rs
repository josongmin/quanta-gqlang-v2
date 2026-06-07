use crate::ast::AstRootV1;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TypedDocumentAstV1 {
    pub roots: Vec<TypedRootAstV1>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedRootAstV1 {
    pub root: AstRootV1,
    pub profile: Option<String>,
}
