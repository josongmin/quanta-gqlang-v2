#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShowRootAstV1 {
    pub category: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DescribeRootAstV1 {
    pub target: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShowDescribeSyntaxErrorV1 {
    pub code: &'static str,
    pub message: String,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ShowCategoryAstV1 {
    Capabilities,
    Models,
    Views,
    Rules,
    Packs,
    Profiles,
    Gates,
}

impl ShowCategoryAstV1 {
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DescribeTargetKindAstV1 {
    Capability,
    Model,
    View,
    Rule,
    Pack,
    Profile,
    Gate,
}

impl DescribeTargetKindAstV1 {
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DescribeTargetAstV1 {
    pub kind: DescribeTargetKindAstV1,
    pub path: Vec<String>,
}

impl DescribeTargetAstV1 {
    #[must_use]
    pub fn canonical_name_v1(&self) -> String {
        format!("{}::{}", self.kind.canonical_name_v1(), self.path.join("."))
    }
}

impl ShowRootAstV1 {
    pub fn category_spec(&self) -> Result<ShowCategoryAstV1, ShowDescribeSyntaxErrorV1> {
        parse_show_category_v1(&self.category)
    }

    #[must_use]
    pub fn from_category_spec_v1(category: ShowCategoryAstV1) -> Self {
        Self {
            category: category.canonical_name_v1().to_string(),
        }
    }
}

impl DescribeRootAstV1 {
    pub fn target_spec(&self) -> Result<DescribeTargetAstV1, ShowDescribeSyntaxErrorV1> {
        parse_describe_target_v1(&self.target)
    }

    #[must_use]
    pub fn from_target_spec_v1(target: DescribeTargetAstV1) -> Self {
        Self {
            target: target.canonical_name_v1(),
        }
    }
}

pub fn parse_show_root_v1(raw_category: &str) -> Result<ShowRootAstV1, ShowDescribeSyntaxErrorV1> {
    let category = parse_show_category_v1(raw_category)?;
    Ok(ShowRootAstV1::from_category_spec_v1(category))
}

pub fn parse_describe_root_v1(
    raw_target: &str,
) -> Result<DescribeRootAstV1, ShowDescribeSyntaxErrorV1> {
    let target = parse_describe_target_v1(raw_target)?;
    Ok(DescribeRootAstV1::from_target_spec_v1(target))
}

pub fn parse_show_category_v1(
    raw_category: &str,
) -> Result<ShowCategoryAstV1, ShowDescribeSyntaxErrorV1> {
    let trimmed = raw_category.trim();
    if trimmed.is_empty() {
        return Err(show_describe_error_v1(
            "show_category_missing",
            "SHOW category is missing",
        ));
    }
    if trimmed.contains(char::is_whitespace) {
        return Err(show_describe_error_v1(
            "show_category_invalid_shape",
            "SHOW category must be a single token",
        ));
    }

    match normalize_symbol_v1(trimmed).as_str() {
        "capability" | "capabilities" => Ok(ShowCategoryAstV1::Capabilities),
        "model" | "models" => Ok(ShowCategoryAstV1::Models),
        "view" | "views" => Ok(ShowCategoryAstV1::Views),
        "rule" | "rules" => Ok(ShowCategoryAstV1::Rules),
        "pack" | "packs" => Ok(ShowCategoryAstV1::Packs),
        "profile" | "profiles" => Ok(ShowCategoryAstV1::Profiles),
        "gate" | "gates" => Ok(ShowCategoryAstV1::Gates),
        _ => Err(show_describe_error_v1(
            "unsupported_show_category",
            format!(
                "SHOW category `{trimmed}` is unsupported; expected one of capabilities, models, views, rules, packs, profiles, gates"
            ),
        )),
    }
}

pub fn parse_describe_target_v1(
    raw_target: &str,
) -> Result<DescribeTargetAstV1, ShowDescribeSyntaxErrorV1> {
    let trimmed = raw_target.trim();
    if trimmed.is_empty() {
        return Err(show_describe_error_v1(
            "describe_target_missing",
            "DESCRIBE target is missing",
        ));
    }
    if trimmed.contains(char::is_whitespace) {
        return Err(show_describe_error_v1(
            "describe_target_invalid_shape",
            "DESCRIBE target must not contain whitespace",
        ));
    }

    let Some((raw_kind, raw_path)) = trimmed.split_once("::") else {
        return Err(show_describe_error_v1(
            "describe_target_missing_kind",
            "DESCRIBE target must use `<kind>::<identifier>`",
        ));
    };
    if raw_kind.trim().is_empty() || raw_path.trim().is_empty() {
        return Err(show_describe_error_v1(
            "describe_target_invalid_shape",
            "DESCRIBE target must use `<kind>::<identifier>`",
        ));
    }

    let kind = parse_describe_target_kind_v1(raw_kind)?;
    let path = parse_identifier_path_v1(raw_path)?;
    Ok(DescribeTargetAstV1 { kind, path })
}

fn parse_describe_target_kind_v1(
    raw_kind: &str,
) -> Result<DescribeTargetKindAstV1, ShowDescribeSyntaxErrorV1> {
    match normalize_symbol_v1(raw_kind).as_str() {
        "capability" | "capabilities" => Ok(DescribeTargetKindAstV1::Capability),
        "model" | "models" => Ok(DescribeTargetKindAstV1::Model),
        "view" | "views" => Ok(DescribeTargetKindAstV1::View),
        "rule" | "rules" => Ok(DescribeTargetKindAstV1::Rule),
        "pack" | "packs" => Ok(DescribeTargetKindAstV1::Pack),
        "profile" | "profiles" => Ok(DescribeTargetKindAstV1::Profile),
        "gate" | "gates" => Ok(DescribeTargetKindAstV1::Gate),
        _ => Err(show_describe_error_v1(
            "unsupported_describe_kind",
            format!(
                "DESCRIBE kind `{}` is unsupported; expected one of capability, model, view, rule, pack, profile, gate",
                raw_kind.trim()
            ),
        )),
    }
}

fn parse_identifier_path_v1(raw_path: &str) -> Result<Vec<String>, ShowDescribeSyntaxErrorV1> {
    let mut path = Vec::new();
    for segment in raw_path.split('.') {
        if segment.is_empty() {
            return Err(show_describe_error_v1(
                "describe_target_invalid_path",
                "DESCRIBE identifier path contains an empty segment",
            ));
        }
        if !is_identifier_segment_v1(segment) {
            return Err(show_describe_error_v1(
                "describe_target_invalid_path",
                format!(
                    "DESCRIBE identifier segment `{segment}` must match [A-Za-z_][A-Za-z0-9_-]*"
                ),
            ));
        }
        path.push(segment.to_string());
    }
    Ok(path)
}

fn is_identifier_segment_v1(segment: &str) -> bool {
    let mut chars = segment.chars();
    match chars.next() {
        Some(first) if first == '_' || first.is_ascii_alphabetic() => {
            chars.all(|ch| ch == '_' || ch == '-' || ch.is_ascii_alphanumeric())
        }
        _ => false,
    }
}

fn normalize_symbol_v1(raw_value: &str) -> String {
    raw_value.trim().to_ascii_lowercase().replace('_', "-")
}

fn show_describe_error_v1(
    code: &'static str,
    message: impl Into<String>,
) -> ShowDescribeSyntaxErrorV1 {
    ShowDescribeSyntaxErrorV1 {
        code,
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        parse_describe_root_v1, parse_show_root_v1, DescribeRootAstV1, DescribeTargetAstV1,
        DescribeTargetKindAstV1, ShowCategoryAstV1, ShowDescribeSyntaxErrorV1, ShowRootAstV1,
    };

    #[test]
    fn show_root_normalizes_supported_categories() {
        let root = parse_show_root_v1("Rule").expect("SHOW rule should parse");
        assert_eq!(
            root,
            ShowRootAstV1 {
                category: "rules".to_string(),
            }
        );
        assert_eq!(root.category_spec(), Ok(ShowCategoryAstV1::Rules));
    }

    #[test]
    fn show_root_rejects_generic_catalog_fallback() {
        assert_eq!(
            parse_show_root_v1("catalog"),
            Err(ShowDescribeSyntaxErrorV1 {
                code: "unsupported_show_category",
                message: "SHOW category `catalog` is unsupported; expected one of capabilities, models, views, rules, packs, profiles, gates".to_string(),
            })
        );
    }

    #[test]
    fn describe_root_decodes_concrete_identifier_surface() {
        let root = parse_describe_root_v1("rule::owasp.sql-injection")
            .expect("DESCRIBE target should parse");
        assert_eq!(
            root,
            DescribeRootAstV1 {
                target: "rule::owasp.sql-injection".to_string(),
            }
        );
        assert_eq!(
            root.target_spec(),
            Ok(DescribeTargetAstV1 {
                kind: DescribeTargetKindAstV1::Rule,
                path: vec!["owasp".to_string(), "sql-injection".to_string()],
            })
        );
    }

    #[test]
    fn describe_root_rejects_whitespace_and_missing_namespace() {
        assert_eq!(
            parse_describe_root_v1("rule::owasp sql-injection"),
            Err(ShowDescribeSyntaxErrorV1 {
                code: "describe_target_invalid_shape",
                message: "DESCRIBE target must not contain whitespace".to_string(),
            })
        );
        assert_eq!(
            parse_describe_root_v1("owasp.sql-injection"),
            Err(ShowDescribeSyntaxErrorV1 {
                code: "describe_target_missing_kind",
                message: "DESCRIBE target must use `<kind>::<identifier>`".to_string(),
            })
        );
    }
}
