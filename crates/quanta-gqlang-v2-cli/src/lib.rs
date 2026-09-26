pub mod commands;
pub mod line_value;
pub mod v3_plan;

/// Canonical CLI command kinds. Compatibility aliases (`v3-query`, `v3-fmt`)
/// map to the same kind as their canonical token; no new commands exist.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CliCommandKindV2 {
    Query,
    Fmt,
}

/// One advertised command token and the canonical kind it dispatches to.
#[derive(Copy, Clone, Debug)]
pub struct CliCommandSpecV1 {
    pub token: &'static str,
    pub kind: CliCommandKindV2,
}

/// The single command authority (CLI-01): every advertised token appears
/// exactly once here. Token enumeration, usage rendering, and binary dispatch
/// all derive from this table; no second handwritten token list exists.
pub const CLI_COMMAND_SPECS_V1: [CliCommandSpecV1; 4] = [
    CliCommandSpecV1 {
        token: "query",
        kind: CliCommandKindV2::Query,
    },
    CliCommandSpecV1 {
        token: "fmt",
        kind: CliCommandKindV2::Fmt,
    },
    CliCommandSpecV1 {
        token: "v3-query",
        kind: CliCommandKindV2::Query,
    },
    CliCommandSpecV1 {
        token: "v3-fmt",
        kind: CliCommandKindV2::Fmt,
    },
];

/// The complete set of CLI subcommand tokens, in usage order. Derived from
/// [`CLI_COMMAND_SPECS_V1`]: the doc gate checks the README and the `--help`
/// usage string against this list, so a shipped command can never go
/// undocumented. A binary test pins that each token is a real, accepted
/// command.
///
/// HARD CUTOVER: the CLI is v3-ONLY. `query` / `fmt` are the v3 path; `v3-query`
/// / `v3-fmt` are retained as explicit aliases for the same behavior.
pub const CLI_COMMAND_TOKENS_V1: [&str; 4] = [
    CLI_COMMAND_SPECS_V1[0].token,
    CLI_COMMAND_SPECS_V1[1].token,
    CLI_COMMAND_SPECS_V1[2].token,
    CLI_COMMAND_SPECS_V1[3].token,
];

/// Resolve an argv command token to its canonical kind through the single
/// spec table. Unknown tokens (including extra usage tokens) resolve to
/// `None` and fail closed at the usage error.
#[must_use]
pub fn cli_command_kind_v1(token: &str) -> Option<CliCommandKindV2> {
    CLI_COMMAND_SPECS_V1
        .iter()
        .find(|spec| spec.token == token)
        .map(|spec| spec.kind)
}

/// Render the `<a|b|c|d>` usage alternation from the spec table in usage
/// order, so the emitted usage string can never drift behind (or ahead of)
/// the dispatched token set.
#[must_use]
pub fn cli_command_alternation_v1() -> String {
    let mut alternation = String::from("<");
    for (index, spec) in CLI_COMMAND_SPECS_V1.iter().enumerate() {
        if index > 0 {
            alternation.push('|');
        }
        alternation.push_str(spec.token);
    }
    alternation.push('>');
    alternation
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The authority is single and closed: tokens enumerate the spec table in
    /// order, the alternation renders from the same table, alias pairs share
    /// kinds, tokens are unique, and unknown tokens (including would-be extra
    /// usage tokens) resolve to `None`.
    #[test]
    fn command_authority_is_single_and_closed() {
        let from_specs: Vec<&str> = CLI_COMMAND_SPECS_V1.iter().map(|spec| spec.token).collect();
        assert_eq!(from_specs, CLI_COMMAND_TOKENS_V1);
        assert_eq!(cli_command_alternation_v1(), "<query|fmt|v3-query|v3-fmt>");
        let mut seen = std::collections::HashSet::new();
        for spec in CLI_COMMAND_SPECS_V1 {
            assert!(seen.insert(spec.token), "duplicate token");
        }
        assert!(
            cli_command_kind_v1("query").is_some(),
            "canonical `query` must resolve to a kind"
        );
        assert_eq!(
            cli_command_kind_v1("query"),
            cli_command_kind_v1("v3-query")
        );
        assert!(
            cli_command_kind_v1("fmt").is_some(),
            "canonical `fmt` must resolve to a kind"
        );
        assert_eq!(cli_command_kind_v1("fmt"), cli_command_kind_v1("v3-fmt"));
        assert_ne!(cli_command_kind_v1("query"), cli_command_kind_v1("fmt"));
        for unknown in ["", "explain", "QUERY", "v3_fmt", "query "] {
            assert_eq!(
                cli_command_kind_v1(unknown),
                None,
                "{unknown:?} must not resolve"
            );
        }
    }
}
