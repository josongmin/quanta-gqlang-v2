use quanta_gqlang_v2_syntax::roots::select::{
    SelectFieldRefAstV1, SelectFilterAstV1, SelectFilterOpV1, SelectLiteralAstV1,
    SelectProjectionAstV1, SelectQueryShapeV1, SelectRootAstV1, SelectRootErrorV1,
    SelectTableRefAstV1,
};
pub use quanta_gqlang_v2_syntax::roots::select::{
    SELECT_ACCEPTED_FORM_V1, SELECT_REJECTED_FORMS_V1,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedSelectRootV1 {
    // Shared sema ownership still forwards the raw SELECT root through this field.
    pub from_entity: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectSemaDiagnosticV1 {
    pub code: &'static str,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedSelectShapeV1 {
    pub table: TypedSelectTableRefV1,
    pub projection: TypedSelectProjectionV1,
    pub filter: Option<TypedSelectFilterV1>,
    pub limit: Option<u64>,
}

impl TypedSelectShapeV1 {
    #[must_use]
    pub fn canonical_query_v1(&self) -> String {
        let projection = match &self.projection {
            TypedSelectProjectionV1::Wildcard => "*".to_string(),
            TypedSelectProjectionV1::Fields(fields) => fields
                .iter()
                .map(TypedSelectFieldRefV1::dotted_name_v1)
                .collect::<Vec<_>>()
                .join(", "),
        };
        let mut query = format!("SELECT {projection} FROM {}", self.table.dotted_name_v1());
        if let Some(filter) = &self.filter {
            query.push_str(" WHERE ");
            query.push_str(&filter.field.dotted_name_v1());
            query.push(' ');
            query.push_str(filter.op.as_str_v1());
            query.push(' ');
            query.push_str(&filter.value.to_query_fragment_v1());
        }
        if let Some(limit) = self.limit {
            query.push_str(" LIMIT ");
            query.push_str(&limit.to_string());
        }
        query
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedSelectTableRefV1 {
    pub entity_path: Vec<String>,
}

impl TypedSelectTableRefV1 {
    #[must_use]
    pub fn dotted_name_v1(&self) -> String {
        self.entity_path.join(".")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedSelectFieldRefV1 {
    pub field_path: Vec<String>,
}

impl TypedSelectFieldRefV1 {
    #[must_use]
    pub fn dotted_name_v1(&self) -> String {
        self.field_path.join(".")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypedSelectProjectionV1 {
    Wildcard,
    Fields(Vec<TypedSelectFieldRefV1>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedSelectFilterV1 {
    pub field: TypedSelectFieldRefV1,
    pub op: TypedSelectFilterOpV1,
    pub value: TypedSelectLiteralV1,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypedSelectFilterOpV1 {
    Eq,
    NotEq,
    Lt,
    Lte,
    Gt,
    Gte,
}

impl TypedSelectFilterOpV1 {
    #[must_use]
    pub const fn as_str_v1(&self) -> &'static str {
        match self {
            Self::Eq => "=",
            Self::NotEq => "!=",
            Self::Lt => "<",
            Self::Lte => "<=",
            Self::Gt => ">",
            Self::Gte => ">=",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypedSelectLiteralV1 {
    String(String),
    Integer(i64),
    Boolean(bool),
}

impl TypedSelectLiteralV1 {
    #[must_use]
    pub fn to_query_fragment_v1(&self) -> String {
        match self {
            Self::String(value) => format!("'{}'", value.replace('\'', "''")),
            Self::Integer(value) => value.to_string(),
            Self::Boolean(value) => value.to_string(),
        }
    }
}

impl From<SelectRootErrorV1> for SelectSemaDiagnosticV1 {
    fn from(value: SelectRootErrorV1) -> Self {
        Self {
            code: value.code,
            message: value.message,
        }
    }
}

impl TypedSelectRootV1 {
    pub fn from_ast_v1(ast: &SelectRootAstV1) -> Result<Self, SelectSemaDiagnosticV1> {
        let shape = ast.shape_v1()?;
        Ok(Self {
            from_entity: shape.canonical_query_v1(),
        })
    }

    #[must_use]
    pub fn raw_query_v1(&self) -> &str {
        &self.from_entity
    }

    pub fn shape_v1(&self) -> Result<TypedSelectShapeV1, SelectSemaDiagnosticV1> {
        let ast = SelectRootAstV1 {
            from: self.from_entity.clone(),
        };
        let shape = ast.shape_v1()?;
        Ok(TypedSelectShapeV1::from_ast_shape_v1(shape))
    }
}

pub fn analyze_select_root_v1(
    ast: &SelectRootAstV1,
) -> Result<TypedSelectRootV1, SelectSemaDiagnosticV1> {
    TypedSelectRootV1::from_ast_v1(ast)
}

impl TypedSelectShapeV1 {
    fn from_ast_shape_v1(shape: SelectQueryShapeV1) -> Self {
        Self {
            table: TypedSelectTableRefV1::from_ast_v1(shape.table),
            projection: TypedSelectProjectionV1::from_ast_v1(shape.projection),
            filter: shape.filter.map(TypedSelectFilterV1::from_ast_v1),
            limit: shape.limit,
        }
    }
}

impl TypedSelectTableRefV1 {
    fn from_ast_v1(table: SelectTableRefAstV1) -> Self {
        Self {
            entity_path: table.path,
        }
    }
}

impl TypedSelectFieldRefV1 {
    fn from_ast_v1(field: SelectFieldRefAstV1) -> Self {
        Self {
            field_path: field.path,
        }
    }
}

impl TypedSelectProjectionV1 {
    fn from_ast_v1(projection: SelectProjectionAstV1) -> Self {
        match projection {
            SelectProjectionAstV1::Wildcard => Self::Wildcard,
            SelectProjectionAstV1::Fields(fields) => Self::Fields(
                fields
                    .into_iter()
                    .map(TypedSelectFieldRefV1::from_ast_v1)
                    .collect(),
            ),
        }
    }
}

impl TypedSelectFilterV1 {
    fn from_ast_v1(filter: SelectFilterAstV1) -> Self {
        Self {
            field: TypedSelectFieldRefV1::from_ast_v1(filter.field),
            op: TypedSelectFilterOpV1::from_ast_v1(filter.op),
            value: TypedSelectLiteralV1::from_ast_v1(filter.value),
        }
    }
}

impl TypedSelectFilterOpV1 {
    fn from_ast_v1(op: SelectFilterOpV1) -> Self {
        match op {
            SelectFilterOpV1::Eq => Self::Eq,
            SelectFilterOpV1::NotEq => Self::NotEq,
            SelectFilterOpV1::Lt => Self::Lt,
            SelectFilterOpV1::Lte => Self::Lte,
            SelectFilterOpV1::Gt => Self::Gt,
            SelectFilterOpV1::Gte => Self::Gte,
        }
    }
}

impl TypedSelectLiteralV1 {
    fn from_ast_v1(value: SelectLiteralAstV1) -> Self {
        match value {
            SelectLiteralAstV1::String(inner) => Self::String(inner),
            SelectLiteralAstV1::Integer(inner) => Self::Integer(inner),
            SelectLiteralAstV1::Boolean(inner) => Self::Boolean(inner),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        analyze_select_root_v1, TypedSelectFieldRefV1, TypedSelectFilterOpV1, TypedSelectLiteralV1,
        TypedSelectProjectionV1,
    };
    use crate::{analyze_document_v1, TypedRootV1};
    use quanta_gqlang_v2_syntax::roots::select::SelectRootAstV1;

    #[test]
    fn typed_select_shape_survives_shared_analyze_pipeline() {
        let doc = quanta_gqlang_v2_syntax::parse_document_v1(
            "SELECT id, profile.name FROM catalog.users WHERE active = true LIMIT 7",
        )
        .expect("parse should succeed");
        let typed = analyze_document_v1(&doc).expect("shared analyze should succeed");
        let Some(TypedRootV1::Select(select)) = typed.roots.first() else {
            panic!("expected a SELECT root");
        };
        let shape = select.shape_v1().expect("shape should parse");
        assert_eq!(shape.table.entity_path, vec!["catalog", "users"]);
        assert_eq!(
            shape.projection,
            TypedSelectProjectionV1::Fields(vec![
                TypedSelectFieldRefV1 {
                    field_path: vec!["id".to_string()],
                },
                TypedSelectFieldRefV1 {
                    field_path: vec!["profile".to_string(), "name".to_string()],
                },
            ]),
        );
        let filter = shape.filter.expect("filter should exist");
        assert_eq!(filter.field.field_path, vec!["active"]);
        assert_eq!(filter.op, TypedSelectFilterOpV1::Eq);
        assert_eq!(filter.value, TypedSelectLiteralV1::Boolean(true));
        assert_eq!(shape.limit, Some(7));
    }

    #[test]
    fn owner_local_select_analyzer_rejects_join_shapes() {
        let ast = SelectRootAstV1 {
            from: "SELECT id FROM users JOIN teams".to_string(),
        };
        let err = analyze_select_root_v1(&ast).expect_err("JOIN should be rejected");
        assert_eq!(err.code, "unsupported_table_shape");
    }
}
