use quanta_gqlang_v2_sema::roots::select::{
    SelectSemaDiagnosticV1, TypedSelectFieldRefV1, TypedSelectFilterOpV1, TypedSelectLiteralV1,
    TypedSelectProjectionV1, TypedSelectRootV1, TypedSelectShapeV1, TypedSelectTableRefV1,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectIrV1 {
    // The SELECT shape is parsed + validated ONCE, at lowering, and stored
    // structured here alongside its canonical text. `shape_v1()` returns the
    // structure without re-parsing the query string on every access (the audit's
    // SMELL-1).
    shape: SelectQueryIrV1,
    canonical_query: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectIrLoweringErrorV1 {
    pub code: &'static str,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectQueryIrV1 {
    pub table: SelectTableRefIrV1,
    pub projection: SelectProjectionIrV1,
    pub filter: Option<SelectFilterIrV1>,
    pub limit: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectTableRefIrV1 {
    pub entity_path: Vec<String>,
}

impl SelectTableRefIrV1 {
    #[must_use]
    pub fn dotted_name_v1(&self) -> String {
        self.entity_path.join(".")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectFieldRefIrV1 {
    pub field_path: Vec<String>,
}

impl SelectFieldRefIrV1 {
    #[must_use]
    pub fn dotted_name_v1(&self) -> String {
        self.field_path.join(".")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SelectProjectionIrV1 {
    Wildcard,
    Fields(Vec<SelectFieldRefIrV1>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectFilterIrV1 {
    pub field: SelectFieldRefIrV1,
    pub op: SelectFilterOpIrV1,
    pub value: SelectLiteralIrV1,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SelectFilterOpIrV1 {
    Eq,
    NotEq,
    Lt,
    Lte,
    Gt,
    Gte,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SelectLiteralIrV1 {
    String(String),
    Integer(i64),
    Boolean(bool),
}

impl From<SelectSemaDiagnosticV1> for SelectIrLoweringErrorV1 {
    fn from(value: SelectSemaDiagnosticV1) -> Self {
        Self {
            code: value.code,
            message: value.message,
        }
    }
}

impl SelectIrV1 {
    pub fn from_typed_v1(spec: &TypedSelectRootV1) -> Result<Self, SelectIrLoweringErrorV1> {
        let validated = spec.shape_v1()?;
        let canonical_query = validated.canonical_query_v1();
        Ok(Self {
            shape: SelectQueryIrV1::from_typed_shape_v1(validated),
            canonical_query,
        })
    }

    #[must_use]
    pub fn raw_query_v1(&self) -> &str {
        &self.canonical_query
    }

    /// The validated SELECT query shape. Infallible — validated at lowering, never
    /// re-parsed on access.
    #[must_use]
    pub fn shape_v1(&self) -> &SelectQueryIrV1 {
        &self.shape
    }
}

pub fn lower_select_root_to_ir_v1(
    spec: &TypedSelectRootV1,
) -> Result<SelectIrV1, SelectIrLoweringErrorV1> {
    SelectIrV1::from_typed_v1(spec)
}

impl SelectQueryIrV1 {
    fn from_typed_shape_v1(shape: TypedSelectShapeV1) -> Self {
        Self {
            table: SelectTableRefIrV1::from_typed_v1(shape.table),
            projection: SelectProjectionIrV1::from_typed_v1(shape.projection),
            filter: shape.filter.map(SelectFilterIrV1::from_typed_v1),
            limit: shape.limit,
        }
    }
}

impl SelectTableRefIrV1 {
    fn from_typed_v1(table: TypedSelectTableRefV1) -> Self {
        Self {
            entity_path: table.entity_path,
        }
    }
}

impl SelectFieldRefIrV1 {
    fn from_typed_v1(field: TypedSelectFieldRefV1) -> Self {
        Self {
            field_path: field.field_path,
        }
    }
}

impl SelectProjectionIrV1 {
    fn from_typed_v1(projection: TypedSelectProjectionV1) -> Self {
        match projection {
            TypedSelectProjectionV1::Wildcard => Self::Wildcard,
            TypedSelectProjectionV1::Fields(fields) => Self::Fields(
                fields
                    .into_iter()
                    .map(SelectFieldRefIrV1::from_typed_v1)
                    .collect(),
            ),
        }
    }
}

impl SelectFilterIrV1 {
    fn from_typed_v1(filter: quanta_gqlang_v2_sema::roots::select::TypedSelectFilterV1) -> Self {
        Self {
            field: SelectFieldRefIrV1::from_typed_v1(filter.field),
            op: SelectFilterOpIrV1::from_typed_v1(filter.op),
            value: SelectLiteralIrV1::from_typed_v1(filter.value),
        }
    }
}

impl SelectFilterOpIrV1 {
    fn from_typed_v1(op: TypedSelectFilterOpV1) -> Self {
        match op {
            TypedSelectFilterOpV1::Eq => Self::Eq,
            TypedSelectFilterOpV1::NotEq => Self::NotEq,
            TypedSelectFilterOpV1::Lt => Self::Lt,
            TypedSelectFilterOpV1::Lte => Self::Lte,
            TypedSelectFilterOpV1::Gt => Self::Gt,
            TypedSelectFilterOpV1::Gte => Self::Gte,
        }
    }
}

impl SelectLiteralIrV1 {
    fn from_typed_v1(value: TypedSelectLiteralV1) -> Self {
        match value {
            TypedSelectLiteralV1::String(inner) => Self::String(inner),
            TypedSelectLiteralV1::Integer(inner) => Self::Integer(inner),
            TypedSelectLiteralV1::Boolean(inner) => Self::Boolean(inner),
        }
    }
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::expect_used,
        reason = "SELECT IR tests assert validated parse, sema, and lowering success"
    )]
    #![expect(
        clippy::panic,
        reason = "unexpected lowered variants must fail closed in unit tests"
    )]

    use super::{
        lower_select_root_to_ir_v1, SelectFieldRefIrV1, SelectFilterOpIrV1, SelectLiteralIrV1,
        SelectProjectionIrV1,
    };
    use crate::{lower_document_to_ir_v1, root_ir::PublicIrRootV1};

    #[test]
    fn select_ir_shape_survives_shared_lowering_pipeline() {
        let doc = quanta_gqlang_v2_syntax::parse_document_v1(
            "SELECT id, profile.name FROM catalog.users WHERE active = true LIMIT 4",
        )
        .expect("parse should succeed");
        let typed =
            quanta_gqlang_v2_sema::analyze_document_v1(&doc).expect("shared sema should succeed");
        let ir_roots = lower_document_to_ir_v1(&typed).expect("shared lowering should succeed");
        let Some(PublicIrRootV1::Select(select)) = ir_roots.first() else {
            panic!("expected a SELECT IR root");
        };
        let shape = select.shape_v1();
        assert_eq!(shape.table.entity_path, vec!["catalog", "users"]);
        assert_eq!(
            shape.projection,
            SelectProjectionIrV1::Fields(vec![
                SelectFieldRefIrV1 {
                    field_path: vec!["id".to_string()],
                },
                SelectFieldRefIrV1 {
                    field_path: vec!["profile".to_string(), "name".to_string()],
                },
            ]),
        );
        let filter = shape.filter.as_ref().expect("filter should exist");
        assert_eq!(filter.field.field_path, vec!["active"]);
        assert_eq!(filter.op, SelectFilterOpIrV1::Eq);
        assert_eq!(filter.value, SelectLiteralIrV1::Boolean(true));
        assert_eq!(shape.limit, Some(4));
    }

    #[test]
    fn owner_local_ir_lowering_rejects_invalid_select_shape() {
        let typed = quanta_gqlang_v2_sema::roots::select::TypedSelectRootV1 {
            from_entity: "SELECT id FROM users JOIN teams".to_string(),
        };
        let err = lower_select_root_to_ir_v1(&typed).expect_err("JOIN should be rejected");
        assert_eq!(err.code, "unsupported_table_shape");
    }
}
