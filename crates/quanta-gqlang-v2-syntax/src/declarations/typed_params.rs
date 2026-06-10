//! E04C (syntax half): the typed parameter schema node for declarations and
//! reusable products.
//!
//! A `param <name>: <type>[?] [= <default>]` declaration decodes into a
//! [`TypedParamSchemaV1`] carrying the four properties E04C requires — name,
//! type, cardinality, and defaultability — with fail-closed coercion. Every
//! validation surfaces a typed [`ParamSchemaErrorV1`] with a stable code; an
//! unknown type, empty/duplicate enum domain, malformed cardinality, or a
//! default outside the declared enum domain is rejected at decode time. Nothing
//! is left for runtime-local coercion or provider-side default injection.
//!
//! The SEMA half of E04C (argument binding, sema reject fixtures) and the IR
//! product lowering are owned by a separate ticket and are NOT in this module.

/// The scalar base types a parameter may declare. Closed set: a new scalar is a
/// grammar change, not a runtime lookup, so an unknown scalar fails closed.
const SCALAR_TYPES_V1: &[&str] = &["u32", "i64", "f64", "bool", "string"];

/// The named semantic domains a parameter may declare. Also a CLOSED set: a
/// typo'd type (e.g. `bigint`) must fail closed as `param_unknown_type`, never be
/// silently accepted as an arbitrary "domain" identifier.
const DOMAIN_TYPES_V1: &[&str] = &["source_scope", "symbol_ref", "rule_ref", "capability_ref"];

/// A fully-decoded typed parameter schema.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedParamSchemaV1 {
    pub name: String,
    pub param_type: ParamTypeV1,
    pub cardinality: ParamCardinalityV1,
    /// The declared default value token, if any. Presence makes the parameter
    /// defaultable (see [`TypedParamSchemaV1::is_defaultable_v1`]).
    pub default: Option<String>,
}

/// The declared type of a parameter.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParamTypeV1 {
    /// A closed enum domain, e.g. `enum<low|medium|high>`. Members are ordered
    /// and de-duplicated at decode time.
    Enum(Vec<String>),
    /// A scalar base type from [`SCALAR_TYPES_V1`], e.g. `u32`.
    Scalar(String),
    /// A named semantic domain, e.g. `source_scope`.
    Domain(String),
}

/// Cardinality: whether an argument must be supplied. `?` marks a parameter
/// optional; its absence makes it required.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParamCardinalityV1 {
    Required,
    Optional,
}

/// A typed parameter-schema decode error with a stable machine-readable code.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParamSchemaErrorV1 {
    pub code: &'static str,
    pub message: String,
}

impl TypedParamSchemaV1 {
    /// Whether this parameter is defaultable. A parameter is defaultable iff it
    /// declares an explicit default — defaultability is never implicit.
    #[must_use]
    pub const fn is_defaultable_v1(&self) -> bool {
        self.default.is_some()
    }

    /// Decode a `param <name>: <type>[?] [= <default>]` declaration. Fail-closed
    /// throughout: see the module docs for the rejected forms.
    pub fn parse_decl_v1(input: &str) -> Result<Self, ParamSchemaErrorV1> {
        let trimmed = input.trim();
        let Some(rest) = strip_keyword_v1(trimmed) else {
            return Err(error_v1(
                "param_missing_keyword",
                "parameter declaration must start with `param`",
            ));
        };

        let Some((name_part, type_part)) = rest.split_once(':') else {
            return Err(error_v1(
                "param_missing_type",
                "parameter declaration requires `: <type>`",
            ));
        };

        let name = name_part.trim();
        validate_param_name_v1(name)?;

        // Split an optional ` = <default>` tail off the type spec. The `=` that
        // separates the default is the first top-level `=`; enum domains do not
        // contain `=`, so a plain `split_once('=')` is unambiguous here.
        let (type_spec_raw, default) = match type_part.split_once('=') {
            Some((type_spec, default_raw)) => {
                let default = default_raw.trim();
                if default.is_empty() {
                    return Err(error_v1(
                        "param_empty_default",
                        "parameter default after `=` must not be empty",
                    ));
                }
                (type_spec.trim(), Some(default.to_string()))
            }
            None => (type_part.trim(), None),
        };

        // A trailing `?` (before any default) marks the parameter optional.
        let (type_spec, cardinality) = match type_spec_raw.strip_suffix('?') {
            Some(stripped) => (stripped.trim(), ParamCardinalityV1::Optional),
            None => (type_spec_raw, ParamCardinalityV1::Required),
        };

        let param_type = parse_param_type_v1(type_spec)?;

        if let (ParamTypeV1::Enum(members), Some(default_value)) = (&param_type, &default) {
            if !members.iter().any(|member| member == default_value) {
                return Err(error_v1(
                    "param_default_not_in_domain",
                    format!(
                        "default `{default_value}` is not a member of the declared enum domain"
                    ),
                ));
            }
        }

        Ok(Self {
            name: name.to_string(),
            param_type,
            cardinality,
            default,
        })
    }
}

fn strip_keyword_v1(input: &str) -> Option<&str> {
    let rest = input.strip_prefix("param")?;
    // The keyword must be a whole token: `param` followed by whitespace.
    match rest.chars().next() {
        Some(ch) if ch.is_whitespace() => Some(rest.trim_start()),
        _ => None,
    }
}

fn validate_param_name_v1(name: &str) -> Result<(), ParamSchemaErrorV1> {
    if name.is_empty() {
        return Err(error_v1(
            "param_missing_name",
            "parameter declaration requires a name",
        ));
    }
    let mut chars = name.chars();
    let starts_ok = chars
        .next()
        .is_some_and(|ch| ch == '_' || ch.is_ascii_alphabetic());
    let rest_ok = chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric());
    if !starts_ok || !rest_ok {
        return Err(error_v1(
            "param_invalid_name",
            format!("parameter name `{name}` must match `[A-Za-z_][A-Za-z0-9_]*`"),
        ));
    }
    Ok(())
}

fn parse_param_type_v1(type_spec: &str) -> Result<ParamTypeV1, ParamSchemaErrorV1> {
    if type_spec.is_empty() {
        return Err(error_v1(
            "param_missing_type",
            "parameter declaration requires a non-empty `: <type>`",
        ));
    }

    if let Some(domain) = type_spec
        .strip_prefix("enum<")
        .and_then(|rest| rest.strip_suffix('>'))
    {
        return parse_enum_domain_v1(domain);
    }
    // A dangling `enum<...` without a closing `>` is malformed, not a scalar.
    if type_spec.starts_with("enum<") || type_spec == "enum" {
        return Err(error_v1(
            "param_malformed_enum",
            format!("enum type `{type_spec}` must be `enum<a|b|c>`"),
        ));
    }

    if SCALAR_TYPES_V1.contains(&type_spec) {
        return Ok(ParamTypeV1::Scalar(type_spec.to_string()));
    }

    // A named semantic domain is from the closed [`DOMAIN_TYPES_V1`] set.
    // Anything else (e.g. `bigint`) is an unknown type and fails closed.
    if DOMAIN_TYPES_V1.contains(&type_spec) {
        return Ok(ParamTypeV1::Domain(type_spec.to_string()));
    }

    Err(error_v1(
        "param_unknown_type",
        format!("parameter type `{type_spec}` is unsupported"),
    ))
}

fn parse_enum_domain_v1(domain: &str) -> Result<ParamTypeV1, ParamSchemaErrorV1> {
    let trimmed = domain.trim();
    if trimmed.is_empty() {
        return Err(error_v1(
            "param_empty_enum_domain",
            "enum type must declare at least one member",
        ));
    }

    let mut members = Vec::new();
    for raw_member in trimmed.split('|') {
        let member = raw_member.trim();
        if member.is_empty() {
            return Err(error_v1(
                "param_empty_enum_member",
                "enum domain must not contain an empty member",
            ));
        }
        if !is_domain_identifier_v1(member) {
            return Err(error_v1(
                "param_invalid_enum_member",
                format!("enum member `{member}` must match `[a-z][a-z0-9_-]*`"),
            ));
        }
        if members.iter().any(|existing| existing == member) {
            return Err(error_v1(
                "param_duplicate_enum_member",
                format!("enum member `{member}` is declared more than once"),
            ));
        }
        members.push(member.to_string());
    }

    Ok(ParamTypeV1::Enum(members))
}

fn is_domain_identifier_v1(value: &str) -> bool {
    let mut chars = value.chars();
    match chars.next() {
        Some(first) if first.is_ascii_lowercase() => {
            chars.all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_' || ch == '-')
        }
        _ => false,
    }
}

fn error_v1(code: &'static str, message: impl Into<String>) -> ParamSchemaErrorV1 {
    ParamSchemaErrorV1 {
        code,
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::expect_used,
        reason = "typed-param unit tests assert decode success for validated fixtures"
    )]

    use super::{ParamCardinalityV1, ParamTypeV1, TypedParamSchemaV1};

    #[test]
    fn required_enum_with_default_is_defaultable() {
        let schema =
            TypedParamSchemaV1::parse_decl_v1("param severity: enum<low|medium|high> = low")
                .expect("decodes");
        assert_eq!(schema.cardinality, ParamCardinalityV1::Required);
        assert!(schema.is_defaultable_v1());
        assert_eq!(
            schema.param_type,
            ParamTypeV1::Enum(vec![
                "low".to_string(),
                "medium".to_string(),
                "high".to_string()
            ])
        );
    }

    #[test]
    fn optional_scalar_without_default_is_not_defaultable() {
        let schema = TypedParamSchemaV1::parse_decl_v1("param max_depth: u32?").expect("decodes");
        assert_eq!(schema.cardinality, ParamCardinalityV1::Optional);
        assert!(!schema.is_defaultable_v1());
    }

    #[test]
    fn default_outside_domain_is_rejected() {
        let err = TypedParamSchemaV1::parse_decl_v1("param s: enum<a|b> = c")
            .expect_err("default not in domain");
        assert_eq!(err.code, "param_default_not_in_domain");
    }
}
