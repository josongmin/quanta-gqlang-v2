use std::collections::BTreeSet;

const DECLARATION_CLAUSE_DELIMITER_V1: &str = ";;";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackDeclAstV1 {
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProfileDeclAstV1 {
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GateDeclAstV1 {
    pub name: String,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PackActivationModeV1 {
    Manual,
    Session,
    Workspace,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ProfileAdmissionPolicyV1 {
    Allowlist,
    ReviewRequired,
    CertifiedOnly,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum GateCheckBindingModeV1 {
    All,
    Any,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackDeclShapeAstV1 {
    pub name: String,
    pub source: String,
    pub activation: PackActivationModeV1,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProfileDeclShapeAstV1 {
    pub name: String,
    pub capabilities: Vec<String>,
    pub admission: ProfileAdmissionPolicyV1,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GateDeclShapeAstV1 {
    pub name: String,
    pub checks: Vec<String>,
    pub binding: GateCheckBindingModeV1,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeclarationShapeErrorV1 {
    pub message: String,
}

impl PackDeclAstV1 {
    pub fn shape(&self) -> Result<PackDeclShapeAstV1, DeclarationShapeErrorV1> {
        decode_pack_decl_shape_v1(&self.name)
    }
}

impl ProfileDeclAstV1 {
    pub fn shape(&self) -> Result<ProfileDeclShapeAstV1, DeclarationShapeErrorV1> {
        decode_profile_decl_shape_v1(&self.name)
    }
}

impl GateDeclAstV1 {
    pub fn shape(&self) -> Result<GateDeclShapeAstV1, DeclarationShapeErrorV1> {
        decode_gate_decl_shape_v1(&self.name)
    }
}

pub fn decode_pack_decl_shape_v1(
    raw_carrier: &str,
) -> Result<PackDeclShapeAstV1, DeclarationShapeErrorV1> {
    let carrier = decode_declaration_carrier_v1("PACK", raw_carrier)?;
    let mut source: Option<String> = None;
    let mut activation: Option<PackActivationModeV1> = None;

    for (key, value) in carrier.clauses {
        match key.as_str() {
            "source" | "source-ref" => {
                set_optional_string_field_v1("PACK source", &mut source, value)?;
            }
            "activation" => {
                set_optional_enum_field_v1(
                    "PACK activation",
                    &mut activation,
                    parse_pack_activation_mode_v1(&value)?,
                )?;
            }
            _ => {
                return Err(declaration_shape_error_v1(format!(
                    "PACK clause '{key}' is unsupported"
                )));
            }
        }
    }

    let source = source.ok_or_else(|| {
        declaration_shape_error_v1("PACK declaration missing `source` clause".to_string())
    })?;
    let activation = activation.ok_or_else(|| {
        declaration_shape_error_v1("PACK declaration missing `activation` clause".to_string())
    })?;

    Ok(PackDeclShapeAstV1 {
        name: carrier.name,
        source,
        activation,
    })
}

pub fn decode_profile_decl_shape_v1(
    raw_carrier: &str,
) -> Result<ProfileDeclShapeAstV1, DeclarationShapeErrorV1> {
    let carrier = decode_declaration_carrier_v1("PROFILE", raw_carrier)?;
    let mut capabilities: Option<Vec<String>> = None;
    let mut admission: Option<ProfileAdmissionPolicyV1> = None;

    for (key, value) in carrier.clauses {
        match key.as_str() {
            "capabilities" => {
                set_optional_vec_field_v1(
                    "PROFILE capabilities",
                    &mut capabilities,
                    parse_identifier_list_v1("PROFILE capabilities", &value)?,
                )?;
            }
            "admission" => {
                set_optional_enum_field_v1(
                    "PROFILE admission",
                    &mut admission,
                    parse_profile_admission_policy_v1(&value)?,
                )?;
            }
            _ => {
                return Err(declaration_shape_error_v1(format!(
                    "PROFILE clause '{key}' is unsupported"
                )));
            }
        }
    }

    let capabilities = capabilities.ok_or_else(|| {
        declaration_shape_error_v1("PROFILE declaration missing `capabilities` clause".to_string())
    })?;
    let admission = admission.ok_or_else(|| {
        declaration_shape_error_v1("PROFILE declaration missing `admission` clause".to_string())
    })?;

    Ok(ProfileDeclShapeAstV1 {
        name: carrier.name,
        capabilities,
        admission,
    })
}

pub fn decode_gate_decl_shape_v1(
    raw_carrier: &str,
) -> Result<GateDeclShapeAstV1, DeclarationShapeErrorV1> {
    let carrier = decode_declaration_carrier_v1("GATE", raw_carrier)?;
    let mut checks: Option<Vec<String>> = None;
    let mut binding: Option<GateCheckBindingModeV1> = None;

    for (key, value) in carrier.clauses {
        match key.as_str() {
            "checks" => {
                set_optional_vec_field_v1(
                    "GATE checks",
                    &mut checks,
                    parse_identifier_list_v1("GATE checks", &value)?,
                )?;
            }
            "binding" => {
                set_optional_enum_field_v1(
                    "GATE binding",
                    &mut binding,
                    parse_gate_check_binding_mode_v1(&value)?,
                )?;
            }
            _ => {
                return Err(declaration_shape_error_v1(format!(
                    "GATE clause '{key}' is unsupported"
                )));
            }
        }
    }

    let checks = checks.ok_or_else(|| {
        declaration_shape_error_v1("GATE declaration missing `checks` clause".to_string())
    })?;
    let binding = binding.ok_or_else(|| {
        declaration_shape_error_v1("GATE declaration missing `binding` clause".to_string())
    })?;

    Ok(GateDeclShapeAstV1 {
        name: carrier.name,
        checks,
        binding,
    })
}

struct DeclarationCarrierV1 {
    name: String,
    clauses: Vec<(String, String)>,
}

fn decode_declaration_carrier_v1(
    declaration_kind: &str,
    raw_carrier: &str,
) -> Result<DeclarationCarrierV1, DeclarationShapeErrorV1> {
    let trimmed = raw_carrier.trim();
    if trimmed.is_empty() {
        return Err(declaration_shape_error_v1(format!(
            "{declaration_kind} declaration carrier is empty"
        )));
    }

    let mut name: Option<String> = None;
    let mut clauses = Vec::new();

    for (index, segment) in trimmed
        .split(DECLARATION_CLAUSE_DELIMITER_V1)
        .map(str::trim)
        .enumerate()
    {
        let segment_index = index + 1;
        if segment.is_empty() {
            return Err(declaration_shape_error_v1(format!(
                "{declaration_kind} clause {segment_index} is empty"
            )));
        }

        if index == 0 && !segment.contains('=') {
            name = Some(validate_identifier_v1(
                format!("{declaration_kind} name").as_str(),
                segment,
            )?);
            continue;
        }

        let Some((raw_key, raw_value)) = segment.split_once('=') else {
            return Err(declaration_shape_error_v1(format!(
                "{declaration_kind} clause {segment_index} must be key=value"
            )));
        };

        let key = normalize_clause_symbol_v1(raw_key);
        let value = raw_value.trim();
        if value.is_empty() {
            return Err(declaration_shape_error_v1(format!(
                "{declaration_kind} clause '{key}' has an empty value"
            )));
        }

        if key == "name" {
            let parsed_name =
                validate_identifier_v1(format!("{declaration_kind} name").as_str(), value)?;
            if name.replace(parsed_name).is_some() {
                return Err(declaration_shape_error_v1(format!(
                    "{declaration_kind} name clause is duplicated"
                )));
            }
            continue;
        }

        clauses.push((key, value.to_string()));
    }

    let name = name.ok_or_else(|| {
        declaration_shape_error_v1(format!("{declaration_kind} declaration missing name"))
    })?;

    Ok(DeclarationCarrierV1 { name, clauses })
}

fn parse_pack_activation_mode_v1(
    raw_value: &str,
) -> Result<PackActivationModeV1, DeclarationShapeErrorV1> {
    match normalize_clause_symbol_v1(raw_value).as_str() {
        "manual" => Ok(PackActivationModeV1::Manual),
        "session" => Ok(PackActivationModeV1::Session),
        "workspace" => Ok(PackActivationModeV1::Workspace),
        normalized => Err(declaration_shape_error_v1(format!(
            "PACK activation mode '{normalized}' is unsupported"
        ))),
    }
}

fn parse_profile_admission_policy_v1(
    raw_value: &str,
) -> Result<ProfileAdmissionPolicyV1, DeclarationShapeErrorV1> {
    match normalize_clause_symbol_v1(raw_value).as_str() {
        "allowlist" => Ok(ProfileAdmissionPolicyV1::Allowlist),
        "review-required" => Ok(ProfileAdmissionPolicyV1::ReviewRequired),
        "certified-only" => Ok(ProfileAdmissionPolicyV1::CertifiedOnly),
        normalized => Err(declaration_shape_error_v1(format!(
            "PROFILE admission policy '{normalized}' is unsupported"
        ))),
    }
}

fn parse_gate_check_binding_mode_v1(
    raw_value: &str,
) -> Result<GateCheckBindingModeV1, DeclarationShapeErrorV1> {
    match normalize_clause_symbol_v1(raw_value).as_str() {
        "all" => Ok(GateCheckBindingModeV1::All),
        "any" => Ok(GateCheckBindingModeV1::Any),
        normalized => Err(declaration_shape_error_v1(format!(
            "GATE binding mode '{normalized}' is unsupported"
        ))),
    }
}

fn parse_identifier_list_v1(
    field_name: &str,
    raw_value: &str,
) -> Result<Vec<String>, DeclarationShapeErrorV1> {
    let mut parsed = Vec::new();
    let mut seen = BTreeSet::new();

    for item in raw_value.split(',').map(str::trim) {
        let item = validate_identifier_v1(field_name, item)?;
        let normalized = normalize_identifier_entry_v1(&item);
        if !seen.insert(normalized) {
            return Err(declaration_shape_error_v1(format!(
                "{field_name} contains duplicate entry '{item}'"
            )));
        }
        parsed.push(item);
    }

    if parsed.is_empty() {
        return Err(declaration_shape_error_v1(format!(
            "{field_name} must contain at least one entry"
        )));
    }

    Ok(parsed)
}

fn validate_identifier_v1(
    field_name: &str,
    raw_value: &str,
) -> Result<String, DeclarationShapeErrorV1> {
    let trimmed = raw_value.trim();
    if trimmed.is_empty() {
        return Err(declaration_shape_error_v1(format!(
            "{field_name} must not be empty"
        )));
    }
    if trimmed.chars().any(char::is_whitespace) {
        return Err(declaration_shape_error_v1(format!(
            "{field_name} must not contain whitespace"
        )));
    }
    Ok(trimmed.to_string())
}

fn set_optional_string_field_v1(
    field_name: &str,
    slot: &mut Option<String>,
    raw_value: String,
) -> Result<(), DeclarationShapeErrorV1> {
    let value = validate_identifier_v1(field_name, &raw_value)?;
    if slot.replace(value).is_some() {
        return Err(declaration_shape_error_v1(format!(
            "{field_name} clause is duplicated"
        )));
    }
    Ok(())
}

fn set_optional_enum_field_v1<T>(
    field_name: &str,
    slot: &mut Option<T>,
    value: T,
) -> Result<(), DeclarationShapeErrorV1> {
    if slot.replace(value).is_some() {
        return Err(declaration_shape_error_v1(format!(
            "{field_name} clause is duplicated"
        )));
    }
    Ok(())
}

fn set_optional_vec_field_v1(
    field_name: &str,
    slot: &mut Option<Vec<String>>,
    value: Vec<String>,
) -> Result<(), DeclarationShapeErrorV1> {
    if slot.replace(value).is_some() {
        return Err(declaration_shape_error_v1(format!(
            "{field_name} clause is duplicated"
        )));
    }
    Ok(())
}

fn normalize_clause_symbol_v1(raw_value: &str) -> String {
    raw_value
        .trim()
        .to_ascii_lowercase()
        .replace(['_', ' '], "-")
}

fn normalize_identifier_entry_v1(raw_value: &str) -> String {
    normalize_clause_symbol_v1(raw_value).replace('.', "-")
}

fn declaration_shape_error_v1(message: impl Into<String>) -> DeclarationShapeErrorV1 {
    DeclarationShapeErrorV1 {
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::expect_used,
        reason = "declaration tests assert parse and shape decoding success for validated fixtures"
    )]
    #![expect(
        clippy::indexing_slicing,
        reason = "pack tests assert the single dispatched root directly"
    )]
    #![expect(
        clippy::panic,
        reason = "unexpected declaration variants must fail closed in unit tests"
    )]

    use super::{
        DeclarationShapeErrorV1, GateCheckBindingModeV1, GateDeclAstV1, GateDeclShapeAstV1,
        PackActivationModeV1, PackDeclAstV1, PackDeclShapeAstV1, ProfileAdmissionPolicyV1,
        ProfileDeclAstV1, ProfileDeclShapeAstV1,
    };
    use crate::ast::AstRootV1;
    use crate::parse_document_v1;

    #[test]
    fn pack_shape_decodes_source_and_activation() {
        let doc = parse_document_v1(
            "PACK secure-pack ;; source=registry::security ;; activation=workspace",
        )
        .expect("pack parse");

        let AstRootV1::Pack(spec) = &doc.roots[0] else {
            panic!("expected pack root");
        };

        assert_eq!(
            spec.shape(),
            Ok(PackDeclShapeAstV1 {
                name: "secure-pack".to_string(),
                source: "registry::security".to_string(),
                activation: PackActivationModeV1::Workspace,
            })
        );
    }

    #[test]
    fn pack_shape_rejects_missing_activation() {
        let spec = PackDeclAstV1 {
            name: "secure-pack ;; source=registry::security".to_string(),
        };

        assert_eq!(
            spec.shape(),
            Err(DeclarationShapeErrorV1 {
                message: "PACK declaration missing `activation` clause".to_string(),
            })
        );
    }

    #[test]
    fn profile_shape_decodes_capabilities_and_admission() {
        let spec = ProfileDeclAstV1 {
            name:
                "strict-review ;; capabilities=graph.read,graph.trace ;; admission=review_required"
                    .to_string(),
        };

        assert_eq!(
            spec.shape(),
            Ok(ProfileDeclShapeAstV1 {
                name: "strict-review".to_string(),
                capabilities: vec!["graph.read".to_string(), "graph.trace".to_string()],
                admission: ProfileAdmissionPolicyV1::ReviewRequired,
            })
        );
    }

    #[test]
    fn profile_shape_rejects_duplicate_capabilities() {
        let spec = ProfileDeclAstV1 {
            name: "strict-review ;; capabilities=graph.read,graph_read ;; admission=allowlist"
                .to_string(),
        };

        assert_eq!(
            spec.shape(),
            Err(DeclarationShapeErrorV1 {
                message: "PROFILE capabilities contains duplicate entry 'graph_read'".to_string(),
            })
        );
    }

    #[test]
    fn gate_shape_decodes_checks_and_binding() {
        let spec = GateDeclAstV1 {
            name: "release-gate ;; checks=owasp.sql,authz.path ;; binding=all".to_string(),
        };

        assert_eq!(
            spec.shape(),
            Ok(GateDeclShapeAstV1 {
                name: "release-gate".to_string(),
                checks: vec!["owasp.sql".to_string(), "authz.path".to_string()],
                binding: GateCheckBindingModeV1::All,
            })
        );
    }

    #[test]
    fn gate_shape_rejects_incomplete_binding_surface() {
        let spec = GateDeclAstV1 {
            name: "release-gate ;; checks=owasp.sql".to_string(),
        };

        assert_eq!(
            spec.shape(),
            Err(DeclarationShapeErrorV1 {
                message: "GATE declaration missing `binding` clause".to_string(),
            })
        );
    }
}
