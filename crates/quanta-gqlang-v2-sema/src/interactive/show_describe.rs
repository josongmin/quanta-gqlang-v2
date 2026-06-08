use quanta_gqlang_v2_syntax::interactive::show::{
    parse_describe_target_v1, parse_show_category_v1, DescribeRootAstV1, DescribeTargetAstV1,
    DescribeTargetKindAstV1, ShowCategoryAstV1, ShowDescribeSyntaxErrorV1, ShowRootAstV1,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypedShowDescribeRootV1 {
    Show { category: String },
    Describe { target: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShowDescribeSemaErrorV1 {
    pub code: &'static str,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypedShowDescribeSpecV1 {
    Show(TypedShowRootV1),
    Describe(TypedDescribeRootV1),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedShowRootV1 {
    pub category: TypedShowCategoryV1,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TypedShowCategoryV1 {
    Capabilities,
    Models,
    Views,
    Rules,
    Packs,
    Profiles,
    Gates,
}

impl TypedShowCategoryV1 {
    #[must_use]
    pub const fn canonical_name_v1(&self) -> &'static str {
        match self {
            Self::Capabilities => "capabilities",
            Self::Models => "models",
            Self::Views => "views",
            Self::Rules => "rules",
            Self::Packs => "packs",
            Self::Profiles => "profiles",
            Self::Gates => "gates",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedDescribeRootV1 {
    pub target: TypedDescribeTargetV1,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedDescribeTargetV1 {
    pub kind: TypedDescribeTargetKindV1,
    pub path: Vec<String>,
}

impl TypedDescribeTargetV1 {
    #[must_use]
    pub fn canonical_name_v1(&self) -> String {
        format!("{}::{}", self.kind.canonical_name_v1(), self.path.join("."))
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TypedDescribeTargetKindV1 {
    Capability,
    Model,
    View,
    Rule,
    Pack,
    Profile,
    Gate,
}

impl TypedDescribeTargetKindV1 {
    #[must_use]
    pub const fn canonical_name_v1(&self) -> &'static str {
        match self {
            Self::Capability => "capability",
            Self::Model => "model",
            Self::View => "view",
            Self::Rule => "rule",
            Self::Pack => "pack",
            Self::Profile => "profile",
            Self::Gate => "gate",
        }
    }
}

impl From<ShowDescribeSyntaxErrorV1> for ShowDescribeSemaErrorV1 {
    fn from(value: ShowDescribeSyntaxErrorV1) -> Self {
        Self {
            code: value.code,
            message: value.message,
        }
    }
}

impl TypedShowDescribeRootV1 {
    pub fn from_show_ast_v1(ast: &ShowRootAstV1) -> Result<Self, ShowDescribeSemaErrorV1> {
        let category = ast.category_spec()?;
        Ok(Self::show(TypedShowCategoryV1::from_ast_v1(category)))
    }

    pub fn from_describe_ast_v1(ast: &DescribeRootAstV1) -> Result<Self, ShowDescribeSemaErrorV1> {
        let target = ast.target_spec()?;
        Ok(Self::describe(TypedDescribeTargetV1::from_ast_v1(target)))
    }

    #[must_use]
    pub fn show(category: TypedShowCategoryV1) -> Self {
        Self::Show {
            category: category.canonical_name_v1().to_string(),
        }
    }

    #[must_use]
    pub fn describe(target: TypedDescribeTargetV1) -> Self {
        Self::Describe {
            target: target.canonical_name_v1(),
        }
    }

    pub fn typed_spec(&self) -> Result<TypedShowDescribeSpecV1, ShowDescribeSemaErrorV1> {
        match self {
            Self::Show { category } => {
                let category = parse_show_category_v1(category)?;
                Ok(TypedShowDescribeSpecV1::Show(TypedShowRootV1 {
                    category: TypedShowCategoryV1::from_ast_v1(category),
                }))
            }
            Self::Describe { target } => {
                let target = parse_describe_target_v1(target)?;
                Ok(TypedShowDescribeSpecV1::Describe(TypedDescribeRootV1 {
                    target: TypedDescribeTargetV1::from_ast_v1(target),
                }))
            }
        }
    }
}

impl TypedShowCategoryV1 {
    fn from_ast_v1(category: ShowCategoryAstV1) -> Self {
        match category {
            ShowCategoryAstV1::Capabilities => Self::Capabilities,
            ShowCategoryAstV1::Models => Self::Models,
            ShowCategoryAstV1::Views => Self::Views,
            ShowCategoryAstV1::Rules => Self::Rules,
            ShowCategoryAstV1::Packs => Self::Packs,
            ShowCategoryAstV1::Profiles => Self::Profiles,
            ShowCategoryAstV1::Gates => Self::Gates,
        }
    }
}

impl TypedDescribeTargetV1 {
    fn from_ast_v1(target: DescribeTargetAstV1) -> Self {
        Self {
            kind: TypedDescribeTargetKindV1::from_ast_v1(target.kind),
            path: target.path,
        }
    }
}

impl TypedDescribeTargetKindV1 {
    fn from_ast_v1(kind: DescribeTargetKindAstV1) -> Self {
        match kind {
            DescribeTargetKindAstV1::Capability => Self::Capability,
            DescribeTargetKindAstV1::Model => Self::Model,
            DescribeTargetKindAstV1::View => Self::View,
            DescribeTargetKindAstV1::Rule => Self::Rule,
            DescribeTargetKindAstV1::Pack => Self::Pack,
            DescribeTargetKindAstV1::Profile => Self::Profile,
            DescribeTargetKindAstV1::Gate => Self::Gate,
        }
    }
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::expect_used,
        reason = "SHOW and DESCRIBE sema tests assert validated decoding success"
    )]
    #![expect(
        clippy::panic,
        reason = "unexpected typed variants must fail closed in unit tests"
    )]

    use super::{
        TypedDescribeTargetKindV1, TypedShowCategoryV1, TypedShowDescribeRootV1,
        TypedShowDescribeSpecV1,
    };
    use quanta_gqlang_v2_syntax::interactive::show::{DescribeRootAstV1, ShowRootAstV1};

    #[test]
    fn typed_show_root_normalizes_supported_categories() {
        let typed = TypedShowDescribeRootV1::from_show_ast_v1(&ShowRootAstV1 {
            category: "Rule".to_string(),
        })
        .expect("SHOW rule should typecheck");

        assert_eq!(
            typed,
            TypedShowDescribeRootV1::Show {
                category: "rules".to_string(),
            }
        );
        assert_eq!(
            typed.typed_spec(),
            Ok(TypedShowDescribeSpecV1::Show(super::TypedShowRootV1 {
                category: TypedShowCategoryV1::Rules,
            }))
        );
    }

    #[test]
    fn typed_show_root_rejects_generic_catalog_fallback() {
        let typed = TypedShowDescribeRootV1::Show {
            category: "catalog".to_string(),
        };

        let err = typed.typed_spec().expect_err("catalog must stay invalid");
        assert_eq!(err.code, "unsupported_show_category");
    }

    #[test]
    fn typed_describe_root_decodes_concrete_identifier_surface() {
        let typed = TypedShowDescribeRootV1::from_describe_ast_v1(&DescribeRootAstV1 {
            target: "rule::owasp.sql-injection".to_string(),
        })
        .expect("DESCRIBE rule target should typecheck");

        assert_eq!(
            typed,
            TypedShowDescribeRootV1::Describe {
                target: "rule::owasp.sql-injection".to_string(),
            }
        );

        let spec = typed.typed_spec().expect("typed spec");
        let TypedShowDescribeSpecV1::Describe(spec) = spec else {
            panic!("expected DESCRIBE typed spec");
        };
        assert_eq!(spec.target.kind, TypedDescribeTargetKindV1::Rule);
        assert_eq!(
            spec.target.path,
            vec!["owasp".to_string(), "sql-injection".to_string()]
        );
    }
}
