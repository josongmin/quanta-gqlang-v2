use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LetBindingAstV1 {
    pub name: String,
    pub value: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WithClauseAstV1 {
    pub bindings: Vec<LetBindingAstV1>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MetaCommandKindV1 {
    UsePack,
    RequireCertified,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PackActivationModeAstV1 {
    Session,
    Workspace,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UsePackMetaAstV1<'a> {
    pub pack_id: &'a str,
    pub activation_mode: PackActivationModeAstV1,
    pub lockfile_path: &'a str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CertifiedTruthRequirementAstV1 {
    CertifiedOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CertifiedAdmissionPolicyAstV1 {
    FailClosed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RequireCertifiedMetaAstV1 {
    pub truth_requirement: CertifiedTruthRequirementAstV1,
    pub admission_policy: CertifiedAdmissionPolicyAstV1,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MetaCommandShapeV1<'a> {
    UsePack(UsePackMetaAstV1<'a>),
    RequireCertified(RequireCertifiedMetaAstV1),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MetaCommandShapeErrorV1 {
    pub code: &'static str,
    pub message: String,
}

#[derive(Clone, PartialEq, Eq)]
pub enum MetaCommandAstV1 {
    UsePack {
        pack_id: String,
        activation_mode: Option<String>,
        lockfile_path: Option<String>,
    },
    RequireCertified,
}

impl MetaCommandAstV1 {
    pub fn kind(&self) -> MetaCommandKindV1 {
        match self {
            Self::UsePack { .. } => MetaCommandKindV1::UsePack,
            Self::RequireCertified => MetaCommandKindV1::RequireCertified,
        }
    }

    pub fn shape(&self) -> Result<MetaCommandShapeV1<'_>, MetaCommandShapeErrorV1> {
        match self {
            Self::UsePack {
                pack_id,
                activation_mode,
                lockfile_path,
            } => Ok(MetaCommandShapeV1::UsePack(UsePackMetaAstV1 {
                pack_id: validate_pack_id_v1(pack_id)?,
                activation_mode: parse_pack_activation_mode_v1(
                    activation_mode
                        .as_deref()
                        .ok_or_else(|| {
                            meta_shape_error_v1(
                                "use_pack_missing_activation",
                                "USE PACK requires explicit `activation` clause",
                            )
                        })?
                        .trim(),
                )?,
                lockfile_path: validate_lockfile_path_v1(
                    lockfile_path
                        .as_deref()
                        .ok_or_else(|| {
                            meta_shape_error_v1(
                                "use_pack_missing_lockfile",
                                "USE PACK requires explicit `lockfile` clause",
                            )
                        })?
                        .trim(),
                )?,
            })),
            Self::RequireCertified => Ok(MetaCommandShapeV1::RequireCertified(
                require_certified_shape_v1(),
            )),
        }
    }

    pub fn use_pack_shape(&self) -> Result<Option<UsePackMetaAstV1<'_>>, MetaCommandShapeErrorV1> {
        match self.shape()? {
            MetaCommandShapeV1::UsePack(spec) => Ok(Some(spec)),
            MetaCommandShapeV1::RequireCertified(_) => Ok(None),
        }
    }

    pub fn require_certified_shape(&self) -> Option<RequireCertifiedMetaAstV1> {
        match self {
            Self::RequireCertified => Some(require_certified_shape_v1()),
            Self::UsePack { .. } => None,
        }
    }
}

impl fmt::Debug for MetaCommandAstV1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.shape() {
            Ok(MetaCommandShapeV1::UsePack(spec)) => f
                .debug_struct("UsePack")
                .field("pack_id", &spec.pack_id)
                .field("activation_mode", &spec.activation_mode)
                .field("lockfile_path", &spec.lockfile_path)
                .finish(),
            Ok(MetaCommandShapeV1::RequireCertified(spec)) => f
                .debug_struct("RequireCertified")
                .field("truth_requirement", &spec.truth_requirement)
                .field("admission_policy", &spec.admission_policy)
                .finish(),
            Err(err) => f
                .debug_struct("InvalidMetaCommand")
                .field("kind", &self.kind())
                .field("code", &err.code)
                .field("message", &err.message)
                .finish(),
        }
    }
}

fn require_certified_shape_v1() -> RequireCertifiedMetaAstV1 {
    RequireCertifiedMetaAstV1 {
        truth_requirement: CertifiedTruthRequirementAstV1::CertifiedOnly,
        admission_policy: CertifiedAdmissionPolicyAstV1::FailClosed,
    }
}

fn parse_pack_activation_mode_v1(
    raw: &str,
) -> Result<PackActivationModeAstV1, MetaCommandShapeErrorV1> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "session" => Ok(PackActivationModeAstV1::Session),
        "workspace" => Ok(PackActivationModeAstV1::Workspace),
        normalized => Err(meta_shape_error_v1(
            "use_pack_invalid_activation",
            format!("USE PACK activation `{normalized}` is unsupported"),
        )),
    }
}

fn validate_pack_id_v1(pack_id: &str) -> Result<&str, MetaCommandShapeErrorV1> {
    let trimmed = pack_id.trim();
    if trimmed.is_empty() {
        return Err(meta_shape_error_v1(
            "use_pack_missing_pack_id",
            "USE PACK requires non-empty pack id",
        ));
    }

    if trimmed.chars().any(char::is_whitespace) {
        return Err(meta_shape_error_v1(
            "use_pack_invalid_pack_id",
            format!("USE PACK pack id must be a single token, got `{trimmed}`"),
        ));
    }

    if !trimmed
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.'))
    {
        return Err(meta_shape_error_v1(
            "use_pack_invalid_pack_id",
            format!(
                "USE PACK pack id `{trimmed}` contains unsupported characters; allowed set is [A-Za-z0-9._-]"
            ),
        ));
    }

    let starts_ok = trimmed
        .chars()
        .next()
        .is_some_and(|ch| ch.is_ascii_alphanumeric());
    let ends_ok = trimmed
        .chars()
        .last()
        .is_some_and(|ch| ch.is_ascii_alphanumeric());
    if !starts_ok || !ends_ok {
        return Err(meta_shape_error_v1(
            "use_pack_invalid_pack_id",
            format!(
                "USE PACK pack id `{trimmed}` must start and end with an ASCII alphanumeric character"
            ),
        ));
    }

    Ok(trimmed)
}

fn validate_lockfile_path_v1(lockfile_path: &str) -> Result<&str, MetaCommandShapeErrorV1> {
    let trimmed = lockfile_path.trim();
    if trimmed.is_empty() {
        return Err(meta_shape_error_v1(
            "use_pack_missing_lockfile",
            "USE PACK requires non-empty `lockfile` clause",
        ));
    }

    if trimmed.chars().any(char::is_whitespace) {
        return Err(meta_shape_error_v1(
            "use_pack_invalid_lockfile",
            format!("USE PACK lockfile must be a single token, got `{trimmed}`"),
        ));
    }

    if trimmed.starts_with('/') {
        return Err(meta_shape_error_v1(
            "use_pack_invalid_lockfile",
            format!("USE PACK lockfile `{trimmed}` must be relative"),
        ));
    }

    if !trimmed
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | '/'))
    {
        return Err(meta_shape_error_v1(
            "use_pack_invalid_lockfile",
            format!(
                "USE PACK lockfile `{trimmed}` contains unsupported characters; allowed set is [A-Za-z0-9._/-]"
            ),
        ));
    }

    let mut segments = trimmed.split('/');
    if segments.any(|segment| segment.is_empty() || matches!(segment, "." | "..")) {
        return Err(meta_shape_error_v1(
            "use_pack_invalid_lockfile",
            format!(
                "USE PACK lockfile `{trimmed}` must not contain empty, `.` or `..` path segments"
            ),
        ));
    }

    let file_name = trimmed.rsplit('/').next().unwrap_or(trimmed);
    if !file_name.ends_with(".lock") || file_name.len() <= ".lock".len() {
        return Err(meta_shape_error_v1(
            "use_pack_invalid_lockfile",
            format!(
                "USE PACK lockfile `{trimmed}` must end with `.lock` and include a non-empty file stem"
            ),
        ));
    }

    Ok(trimmed)
}

fn meta_shape_error_v1(code: &'static str, message: impl Into<String>) -> MetaCommandShapeErrorV1 {
    MetaCommandShapeErrorV1 {
        code,
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CertifiedAdmissionPolicyAstV1, CertifiedTruthRequirementAstV1, MetaCommandAstV1,
        MetaCommandShapeV1, PackActivationModeAstV1, RequireCertifiedMetaAstV1, UsePackMetaAstV1,
    };

    #[test]
    fn use_pack_shape_requires_explicit_activation_and_lockfile() {
        let command = MetaCommandAstV1::UsePack {
            pack_id: "secure-pack".to_string(),
            activation_mode: Some("session".to_string()),
            lockfile_path: Some("packs/secure-pack.lock".to_string()),
        };

        assert_eq!(
            command.shape().expect("valid use-pack shape"),
            MetaCommandShapeV1::UsePack(UsePackMetaAstV1 {
                pack_id: "secure-pack",
                activation_mode: PackActivationModeAstV1::Session,
                lockfile_path: "packs/secure-pack.lock",
            })
        );
        assert_eq!(
            command.use_pack_shape().expect("use-pack helper"),
            Some(UsePackMetaAstV1 {
                pack_id: "secure-pack",
                activation_mode: PackActivationModeAstV1::Session,
                lockfile_path: "packs/secure-pack.lock",
            })
        );
    }

    #[test]
    fn use_pack_shape_rejects_incomplete_or_malformed_payloads() {
        let missing = MetaCommandAstV1::UsePack {
            pack_id: String::new(),
            activation_mode: Some("session".to_string()),
            lockfile_path: Some("packs/secure-pack.lock".to_string()),
        };
        let missing_err = missing.shape().expect_err("empty pack id must fail closed");
        assert_eq!(missing_err.code, "use_pack_missing_pack_id");

        let malformed = MetaCommandAstV1::UsePack {
            pack_id: "secure pack".to_string(),
            activation_mode: Some("session".to_string()),
            lockfile_path: Some("packs/secure-pack.lock".to_string()),
        };
        let malformed_err = malformed
            .shape()
            .expect_err("multi-token pack id must fail closed");
        assert_eq!(malformed_err.code, "use_pack_invalid_pack_id");

        let invalid_chars = MetaCommandAstV1::UsePack {
            pack_id: "secure/pack".to_string(),
            activation_mode: Some("session".to_string()),
            lockfile_path: Some("packs/secure-pack.lock".to_string()),
        };
        let invalid_chars_err = invalid_chars
            .shape()
            .expect_err("slash-containing pack id must fail closed");
        assert_eq!(invalid_chars_err.code, "use_pack_invalid_pack_id");

        let missing_activation = MetaCommandAstV1::UsePack {
            pack_id: "secure-pack".to_string(),
            activation_mode: None,
            lockfile_path: Some("packs/secure-pack.lock".to_string()),
        };
        let missing_activation_err = missing_activation
            .shape()
            .expect_err("missing activation must fail closed");
        assert_eq!(missing_activation_err.code, "use_pack_missing_activation");

        let invalid_activation = MetaCommandAstV1::UsePack {
            pack_id: "secure-pack".to_string(),
            activation_mode: Some("manual".to_string()),
            lockfile_path: Some("packs/secure-pack.lock".to_string()),
        };
        let invalid_activation_err = invalid_activation
            .shape()
            .expect_err("unsupported activation must fail closed");
        assert_eq!(invalid_activation_err.code, "use_pack_invalid_activation");

        let missing_lockfile = MetaCommandAstV1::UsePack {
            pack_id: "secure-pack".to_string(),
            activation_mode: Some("workspace".to_string()),
            lockfile_path: None,
        };
        let missing_lockfile_err = missing_lockfile
            .shape()
            .expect_err("missing lockfile must fail closed");
        assert_eq!(missing_lockfile_err.code, "use_pack_missing_lockfile");

        let invalid_lockfile = MetaCommandAstV1::UsePack {
            pack_id: "secure-pack".to_string(),
            activation_mode: Some("workspace".to_string()),
            lockfile_path: Some("../secure-pack.lock".to_string()),
        };
        let invalid_lockfile_err = invalid_lockfile
            .shape()
            .expect_err("traversal lockfile must fail closed");
        assert_eq!(invalid_lockfile_err.code, "use_pack_invalid_lockfile");

        let wrong_suffix = MetaCommandAstV1::UsePack {
            pack_id: "secure-pack".to_string(),
            activation_mode: Some("workspace".to_string()),
            lockfile_path: Some("packs/secure-pack.txt".to_string()),
        };
        let wrong_suffix_err = wrong_suffix
            .shape()
            .expect_err("non-lockfile suffix must fail closed");
        assert_eq!(wrong_suffix_err.code, "use_pack_invalid_lockfile");
    }

    #[test]
    fn require_certified_shape_carries_explicit_truth_and_admission_intent() {
        let command = MetaCommandAstV1::RequireCertified;

        assert_eq!(
            command.shape().expect("require-certified shape"),
            MetaCommandShapeV1::RequireCertified(RequireCertifiedMetaAstV1 {
                truth_requirement: CertifiedTruthRequirementAstV1::CertifiedOnly,
                admission_policy: CertifiedAdmissionPolicyAstV1::FailClosed,
            })
        );
        assert_eq!(
            command.require_certified_shape(),
            Some(RequireCertifiedMetaAstV1 {
                truth_requirement: CertifiedTruthRequirementAstV1::CertifiedOnly,
                admission_policy: CertifiedAdmissionPolicyAstV1::FailClosed,
            })
        );
    }

    #[test]
    fn debug_output_stays_structured_for_main_thread_string_fallback() {
        let require_certified = format!("{:?}", MetaCommandAstV1::RequireCertified);
        assert!(require_certified.contains("truth_requirement"));
        assert!(require_certified.contains("admission_policy"));

        let use_pack = format!(
            "{:?}",
            MetaCommandAstV1::UsePack {
                pack_id: "secure-pack".to_string(),
                activation_mode: Some("workspace".to_string()),
                lockfile_path: Some("packs/secure-pack.lock".to_string()),
            }
        );
        assert!(use_pack.contains("pack_id"));
        assert!(use_pack.contains("activation_mode"));
        assert!(use_pack.contains("lockfile_path"));

        let invalid = format!(
            "{:?}",
            MetaCommandAstV1::UsePack {
                pack_id: "secure pack".to_string(),
                activation_mode: Some("workspace".to_string()),
                lockfile_path: Some("packs/secure-pack.lock".to_string()),
            }
        );
        assert!(invalid.contains("InvalidMetaCommand"));
    }

    #[test]
    fn activation_and_lockfile_shape_are_explicit_and_normalized() {
        let command = MetaCommandAstV1::UsePack {
            pack_id: "secure-pack".to_string(),
            activation_mode: Some("WORKSPACE".to_string()),
            lockfile_path: Some("packs/security/strict.lock".to_string()),
        };
        let Some(spec) = command.use_pack_shape().expect("use-pack helper") else {
            panic!("use-pack helper must surface typed spec");
        };

        assert_eq!(spec.activation_mode, PackActivationModeAstV1::Workspace);
        assert_eq!(spec.lockfile_path, "packs/security/strict.lock");
        let _session_shape = PackActivationModeAstV1::Session;
        let _workspace_shape = PackActivationModeAstV1::Workspace;
    }
}
