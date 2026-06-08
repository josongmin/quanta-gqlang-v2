use core::fmt;

const CHECK_SCOPE_PAYLOAD_PREFIX_V1: &str = "check:v1|";
const NONE_SENTINEL_V1: &str = "~";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckRootAstV1 {
    pub rule_id: String,
    pub scope: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CheckConfirmIntentV1 {
    Skip,
    Prompt,
    Require,
}

impl CheckConfirmIntentV1 {
    fn as_str(self) -> &'static str {
        match self {
            Self::Skip => "skip",
            Self::Prompt => "prompt",
            Self::Require => "require",
        }
    }

    fn decode(raw: &str) -> Result<Self, CheckPayloadErrorV1> {
        match raw {
            "skip" => Ok(Self::Skip),
            "prompt" => Ok(Self::Prompt),
            "require" => Ok(Self::Require),
            other => Err(CheckPayloadErrorV1::new(format!(
                "unknown CHECK confirm intent `{other}`"
            ))),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CheckExecutionModeV1 {
    PatternOnly,
    SemanticOnly,
    Hybrid,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckRootIntentV1 {
    pub rule_id: String,
    pub class: String,
    pub scope: Option<String>,
    pub pattern: Option<String>,
    pub confirm: CheckConfirmIntentV1,
}

impl CheckRootIntentV1 {
    pub fn new(
        rule_id: impl Into<String>,
        class: impl Into<String>,
        scope: Option<String>,
        pattern: Option<String>,
        confirm: CheckConfirmIntentV1,
    ) -> Result<Self, CheckPayloadErrorV1> {
        let rule_id = rule_id.into();
        let class = class.into();
        let scope = normalize_optional(scope, "scope")?;
        let pattern = normalize_optional(pattern, "pattern")?;

        if rule_id.trim().is_empty() {
            return Err(CheckPayloadErrorV1::new(
                "CHECK rule_id must not be empty".to_string(),
            ));
        }
        if class.trim().is_empty() {
            return Err(CheckPayloadErrorV1::new(
                "CHECK class must not be empty".to_string(),
            ));
        }
        if scope.is_none() && pattern.is_none() {
            return Err(CheckPayloadErrorV1::new(
                "CHECK intent requires semantic scope, pattern, or both".to_string(),
            ));
        }

        Ok(Self {
            rule_id,
            class,
            scope,
            pattern,
            confirm,
        })
    }

    pub fn execution_mode(&self) -> CheckExecutionModeV1 {
        match (self.scope.is_some(), self.pattern.is_some()) {
            (false, true) => CheckExecutionModeV1::PatternOnly,
            (true, false) => CheckExecutionModeV1::SemanticOnly,
            (true, true) => CheckExecutionModeV1::Hybrid,
            (false, false) => unreachable!("validated in CheckRootIntentV1::new"),
        }
    }

    pub fn encode_scope_payload(&self) -> String {
        let class = encode_component(&self.class);
        let scope = encode_optional_component(self.scope.as_deref());
        let pattern = encode_optional_component(self.pattern.as_deref());
        format!(
            "{CHECK_SCOPE_PAYLOAD_PREFIX_V1}class={class}|scope={scope}|pattern={pattern}|confirm={}",
            self.confirm.as_str(),
        )
    }

    pub fn decode(rule_id: &str, scope_field: &str) -> Result<Self, CheckPayloadErrorV1> {
        if !scope_field.starts_with(CHECK_SCOPE_PAYLOAD_PREFIX_V1) {
            return Self::decode_legacy(rule_id, scope_field);
        }

        let payload = &scope_field[CHECK_SCOPE_PAYLOAD_PREFIX_V1.len()..];
        let mut class = None;
        let mut scope = None;
        let mut pattern = None;
        let mut confirm = None;

        for segment in payload.split('|') {
            let (key, value) = segment.split_once('=').ok_or_else(|| {
                CheckPayloadErrorV1::new(format!("malformed CHECK payload segment `{segment}`"))
            })?;
            match key {
                "class" => {
                    reject_duplicate_field_v1(class.is_some(), "class")?;
                    class = Some(decode_required_component(value, "class")?);
                }
                "scope" => {
                    reject_duplicate_field_v1(scope.is_some(), "scope")?;
                    scope = Some(decode_optional_component(value, "scope")?);
                }
                "pattern" => {
                    reject_duplicate_field_v1(pattern.is_some(), "pattern")?;
                    pattern = Some(decode_optional_component(value, "pattern")?);
                }
                "confirm" => {
                    reject_duplicate_field_v1(confirm.is_some(), "confirm")?;
                    confirm = Some(CheckConfirmIntentV1::decode(value)?);
                }
                other => {
                    return Err(CheckPayloadErrorV1::new(format!(
                        "unknown CHECK payload field `{other}`"
                    )));
                }
            }
        }

        let class = class
            .ok_or_else(|| CheckPayloadErrorV1::new("CHECK payload missing `class`".to_string()))?;
        let scope = scope
            .ok_or_else(|| CheckPayloadErrorV1::new("CHECK payload missing `scope`".to_string()))?;
        let pattern = pattern.ok_or_else(|| {
            CheckPayloadErrorV1::new("CHECK payload missing `pattern`".to_string())
        })?;
        let confirm = confirm.ok_or_else(|| {
            CheckPayloadErrorV1::new("CHECK payload missing `confirm`".to_string())
        })?;

        Self::new(rule_id.to_string(), class, scope, pattern, confirm)
    }

    fn decode_legacy(rule_id: &str, scope_field: &str) -> Result<Self, CheckPayloadErrorV1> {
        let legacy_scope = normalize_optional(Some(scope_field.to_string()), "scope")?;
        Self::new(
            rule_id.to_string(),
            "legacy".to_string(),
            legacy_scope,
            None,
            CheckConfirmIntentV1::Skip,
        )
    }
}

impl CheckRootAstV1 {
    pub fn from_intent(intent: CheckRootIntentV1) -> Self {
        Self {
            scope: intent.encode_scope_payload(),
            rule_id: intent.rule_id,
        }
    }

    pub fn intent(&self) -> Result<CheckRootIntentV1, CheckPayloadErrorV1> {
        CheckRootIntentV1::decode(&self.rule_id, &self.scope)
    }

    pub fn execution_mode(&self) -> Result<CheckExecutionModeV1, CheckPayloadErrorV1> {
        Ok(self.intent()?.execution_mode())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckPayloadErrorV1 {
    pub message: String,
}

impl CheckPayloadErrorV1 {
    fn new(message: String) -> Self {
        Self { message }
    }
}

impl fmt::Display for CheckPayloadErrorV1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for CheckPayloadErrorV1 {}

fn reject_duplicate_field_v1(
    already_set: bool,
    field_name: &str,
) -> Result<(), CheckPayloadErrorV1> {
    if already_set {
        return Err(CheckPayloadErrorV1::new(format!(
            "CHECK payload field `{field_name}` is duplicated"
        )));
    }
    Ok(())
}

fn normalize_optional(
    value: Option<String>,
    field_name: &str,
) -> Result<Option<String>, CheckPayloadErrorV1> {
    match value {
        Some(value) if value.trim().is_empty() => Err(CheckPayloadErrorV1::new(format!(
            "CHECK {field_name} must not be blank"
        ))),
        Some(value) => Ok(Some(value)),
        None => Ok(None),
    }
}

fn encode_optional_component(value: Option<&str>) -> String {
    value
        .map(encode_component)
        .unwrap_or_else(|| NONE_SENTINEL_V1.to_string())
}

fn decode_required_component(
    encoded: &str,
    field_name: &str,
) -> Result<String, CheckPayloadErrorV1> {
    let decoded = decode_component(encoded, field_name)?;
    if decoded.trim().is_empty() {
        return Err(CheckPayloadErrorV1::new(format!(
            "CHECK payload field `{field_name}` must not be blank"
        )));
    }
    Ok(decoded)
}

fn decode_optional_component(
    encoded: &str,
    field_name: &str,
) -> Result<Option<String>, CheckPayloadErrorV1> {
    if encoded == NONE_SENTINEL_V1 {
        return Ok(None);
    }
    let decoded = decode_component(encoded, field_name)?;
    if decoded.trim().is_empty() {
        return Err(CheckPayloadErrorV1::new(format!(
            "CHECK payload field `{field_name}` must not be blank"
        )));
    }
    Ok(Some(decoded))
}

fn encode_component(raw: &str) -> String {
    let mut encoded = String::with_capacity(raw.len());
    for byte in raw.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'/' | b':' => {
                encoded.push(char::from(byte))
            }
            _ => {
                encoded.push('%');
                encoded.push(hex_digit(byte >> 4));
                encoded.push(hex_digit(byte & 0x0F));
            }
        }
    }
    encoded
}

fn decode_component(encoded: &str, field_name: &str) -> Result<String, CheckPayloadErrorV1> {
    let mut bytes = Vec::with_capacity(encoded.len());
    let mut cursor = 0usize;
    let raw = encoded.as_bytes();

    while let Some(current) = raw.get(cursor).copied() {
        match current {
            b'%' => {
                if cursor + 2 >= raw.len() {
                    return Err(CheckPayloadErrorV1::new(format!(
                        "CHECK payload field `{field_name}` has truncated percent encoding"
                    )));
                }
                let high = raw
                    .get(cursor + 1)
                    .copied()
                    .and_then(decode_hex)
                    .ok_or_else(|| {
                        CheckPayloadErrorV1::new(format!(
                            "CHECK payload field `{field_name}` has invalid hex escape"
                        ))
                    })?;
                let low = raw
                    .get(cursor + 2)
                    .copied()
                    .and_then(decode_hex)
                    .ok_or_else(|| {
                        CheckPayloadErrorV1::new(format!(
                            "CHECK payload field `{field_name}` has invalid hex escape"
                        ))
                    })?;
                bytes.push((high << 4) | low);
                cursor += 3;
            }
            other => {
                bytes.push(other);
                cursor += 1;
            }
        }
    }

    String::from_utf8(bytes).map_err(|_| {
        CheckPayloadErrorV1::new(format!(
            "CHECK payload field `{field_name}` is not valid UTF-8"
        ))
    })
}

fn hex_digit(value: u8) -> char {
    match value {
        0..=9 => char::from(b'0' + value),
        10..=15 => char::from(b'A' + (value - 10)),
        _ => unreachable!("hex digit is always <= 15"),
    }
}

fn decode_hex(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(10 + (value - b'a')),
        b'A'..=b'F' => Some(10 + (value - b'A')),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{CheckConfirmIntentV1, CheckExecutionModeV1, CheckRootAstV1, CheckRootIntentV1};

    #[test]
    fn encoded_check_payload_round_trips_reserved_text() {
        let intent = CheckRootIntentV1::new(
            "rule.security/no-shell",
            "security.review",
            Some("repo:semantica|crate=runtime".to_string()),
            Some("path ~= \"src/**/*.rs\" && call = \"std::process::Command\"".to_string()),
            CheckConfirmIntentV1::Require,
        )
        .expect("intent");
        let root = CheckRootAstV1::from_intent(intent.clone());

        assert_eq!(root.intent().expect("decode"), intent);
        assert_eq!(
            root.execution_mode().expect("mode"),
            CheckExecutionModeV1::Hybrid,
        );
    }

    #[test]
    fn legacy_scope_decodes_as_semantic_only_check() {
        let root = CheckRootAstV1 {
            rule_id: "rule.audit/stale".to_string(),
            scope: "repo://frontend".to_string(),
        };
        let intent = root.intent().expect("decode");

        assert_eq!(intent.rule_id, "rule.audit/stale");
        assert_eq!(intent.class, "legacy");
        assert_eq!(intent.scope.as_deref(), Some("repo://frontend"));
        assert_eq!(intent.pattern, None);
        assert_eq!(intent.confirm, CheckConfirmIntentV1::Skip);
        assert_eq!(intent.execution_mode(), CheckExecutionModeV1::SemanticOnly);
    }

    #[test]
    fn intent_requires_scope_or_pattern() {
        let err = CheckRootIntentV1::new(
            "rule.empty",
            "policy",
            None,
            None,
            CheckConfirmIntentV1::Prompt,
        )
        .expect_err("missing scope and pattern must fail");

        assert_eq!(
            err.message,
            "CHECK intent requires semantic scope, pattern, or both"
        );
    }

    #[test]
    fn duplicate_check_payload_field_is_rejected() {
        let root = CheckRootAstV1 {
            rule_id: "rule.x".to_string(),
            scope: "check:v1|class=a|class=b|scope=~|pattern=p|confirm=skip".to_string(),
        };
        let result = root.intent();
        assert!(
            matches!(&result, Err(error) if error.message.contains("duplicated")),
            "duplicate `class` key must fail closed, got {result:?}"
        );
    }

    #[test]
    fn duplicate_confirm_key_does_not_silently_flip_policy() {
        let root = CheckRootAstV1 {
            rule_id: "rule.y".to_string(),
            scope: "check:v1|class=a|scope=~|pattern=p|confirm=skip|confirm=require".to_string(),
        };
        let result = root.intent();
        assert!(
            matches!(&result, Err(error) if error.message.contains("duplicated")),
            "duplicate `confirm` key must fail closed, got {result:?}"
        );
    }
}
