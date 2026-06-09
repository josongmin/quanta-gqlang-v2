//! Single authority for how the read-only front doors DISPLAY a parser
//! diagnostic code's class.
//!
//! The canonical parser is the only source of diagnostics. A handful of its
//! codes are semantic-value rejections (an unsupported enum value / activation
//! for an otherwise well-formed statement) which the read-only surface presents
//! as the `sema` class; every other shape/keyword error is presented as
//! `syntax`. Both the LSP error surface and the dev-only conformance registry
//! route through this one mapping so the classification can never drift between
//! the front door and the rails that pin it.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrontDoorDiagnosticClassV1 {
    Syntax,
    Sema,
}

impl FrontDoorDiagnosticClassV1 {
    #[must_use]
    pub fn as_str_v1(self) -> &'static str {
        match self {
            Self::Syntax => "syntax",
            Self::Sema => "sema",
        }
    }
}

/// The display class for a parser diagnostic code.
#[must_use]
pub fn front_door_diagnostic_class_v1(code: &str) -> FrontDoorDiagnosticClassV1 {
    match code {
        "unsupported_show_category"
        | "unsupported_find_subkind"
        | "unsupported_describe_kind"
        | "use_pack_invalid_activation" => FrontDoorDiagnosticClassV1::Sema,
        _ => FrontDoorDiagnosticClassV1::Syntax,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sema_display_codes_are_exactly_the_semantic_value_rejections_v1() {
        for code in [
            "unsupported_show_category",
            "unsupported_find_subkind",
            "unsupported_describe_kind",
            "use_pack_invalid_activation",
        ] {
            assert_eq!(
                front_door_diagnostic_class_v1(code),
                FrontDoorDiagnosticClassV1::Sema,
                "{code} must display as sema"
            );
        }
        for code in [
            "expected_from_keyword",
            "check_missing_rule_or_scope",
            "view_missing_name",
            "unsupported_root",
        ] {
            assert_eq!(
                front_door_diagnostic_class_v1(code),
                FrontDoorDiagnosticClassV1::Syntax,
                "{code} must display as syntax"
            );
        }
    }
}
