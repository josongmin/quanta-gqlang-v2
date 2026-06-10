#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration-test assertions intentionally use expect/unwrap/indexing and exercise fail-closed panics"
)]

//! Hellgate: CLI negative-DSL fail-closed contract at the binary front door.
//!
//! The library already pins each malformed DSL root to its parser diagnostic
//! code (see `quanta-gqlang-v2-syntax/tests/parser_conformance.rs`). This rail
//! promotes that negative contract to the shipped `quanta-gqlang-v2-cli`
//! binary: for every malformed root the binary must fail closed with exit code
//! 1, render `cli.error.class=command` to **stderr**, carry the parser error
//! code in that diagnostic, and leave stdout empty.
//!
//! `doctor`'s three-way stdout classification is intentionally NOT covered
//! here — it is owned by `doctor_frontdoor_hellgate.rs`. This file owns the
//! `parse`/`sema`/`query` command-error stderr channel for malformed roots.

mod support;

use quanta_gqlang_v2_conformance::root_surface_specs_v1;
use support::{cli_stderr_v1, cli_stdout_v1, run_cli_argv_v1};

/// One malformed-DSL front-door case. `argv` is passed verbatim to the binary
/// (there is no shell, so `;;`, `->`, `|`, and `=` are literal argv tokens).
/// `expect_code_fragment` is the parser error code, which the binary surfaces
/// verbatim as `syntax.diagnostic[0].code=<code>` inside the rendered command
/// diagnostic.
struct NegativeDslCaseV1 {
    label: &'static str,
    argv: &'static [&'static str],
    expect_code_fragment: &'static str,
}

const NEGATIVE_DSL_CASES_V1: &[NegativeDslCaseV1] = &[
    NegativeDslCaseV1 {
        label: "check-missing-rule-or-scope",
        argv: &["parse", "CHECK", "rule.hybrid"],
        expect_code_fragment: "check_missing_rule_or_scope",
    },
    NegativeDslCaseV1 {
        label: "describe-missing-target",
        argv: &["parse", "DESCRIBE"],
        expect_code_fragment: "describe_missing_target",
    },
    NegativeDslCaseV1 {
        label: "invalid-explain-shape",
        argv: &[
            "parse",
            "EXPLAIN",
            "rule::owasp.sql-injection",
            ";;",
            "truth=maybe",
        ],
        expect_code_fragment: "invalid_explain_shape",
    },
    NegativeDslCaseV1 {
        label: "invalid-find-seed",
        argv: &[
            "parse",
            "FIND",
            "taint",
            "api.entry",
            "->",
            "db.write",
            "|",
            "depth=3",
            "|",
            "depth=5",
        ],
        expect_code_fragment: "invalid_find_seed",
    },
    NegativeDslCaseV1 {
        label: "invalid-sample-payload",
        argv: &[
            "parse",
            "SAMPLE",
            "query=callgraph",
            ";;",
            "mode=semantic",
            ";;",
            "mode=hybrid",
        ],
        expect_code_fragment: "invalid_sample_payload",
    },
    NegativeDslCaseV1 {
        label: "invalid-export-payload",
        argv: &[
            "parse",
            "EXPORT",
            "source=result::latest",
            ";;",
            "kind=sarif",
            ";;",
            "format=pretty",
        ],
        expect_code_fragment: "invalid_export_payload",
    },
    NegativeDslCaseV1 {
        label: "unsupported-root",
        argv: &["parse", "FROBNICATE", "x"],
        expect_code_fragment: "unsupported_root",
    },
    NegativeDslCaseV1 {
        label: "unsupported-use-meta",
        argv: &["parse", "USE", "PROFILE", "strict-review"],
        expect_code_fragment: "unsupported_use_meta",
    },
    NegativeDslCaseV1 {
        label: "unsupported-require-meta",
        argv: &["parse", "REQUIRE", "TRUSTED"],
        expect_code_fragment: "unsupported_require_meta",
    },
    NegativeDslCaseV1 {
        label: "use-pack-missing-lockfile",
        argv: &[
            "parse",
            "USE",
            "PACK",
            "secure-pack",
            ";;",
            "activation=session",
        ],
        expect_code_fragment: "use_pack_missing_lockfile",
    },
    NegativeDslCaseV1 {
        label: "use-pack-invalid-activation",
        argv: &[
            "parse",
            "USE",
            "PACK",
            "secure-pack",
            ";;",
            "activation=manual",
            ";;",
            "lockfile=packs/secure-pack.lock",
        ],
        expect_code_fragment: "use_pack_invalid_activation",
    },
    NegativeDslCaseV1 {
        label: "unsupported-use-pack-clause",
        argv: &[
            "parse",
            "USE",
            "PACK",
            "secure-pack",
            ";;",
            "activation=session",
            ";;",
            "lockfile=packs/secure-pack.lock",
            ";;",
            "source=ambient",
        ],
        expect_code_fragment: "unsupported_use_pack_clause",
    },
    NegativeDslCaseV1 {
        label: "use-pack-invalid-lockfile",
        argv: &[
            "parse",
            "USE",
            "PACK",
            "secure-pack",
            ";;",
            "activation=session",
            ";;",
            "lockfile=packs/secure-pack.txt",
        ],
        expect_code_fragment: "use_pack_invalid_lockfile",
    },
];

#[test]
fn cli_binary_fails_closed_for_malformed_dsl_roots_v1() {
    for case in NEGATIVE_DSL_CASES_V1 {
        let out = run_cli_argv_v1(case.argv);

        assert_eq!(
            out.status.code(),
            Some(1),
            "case `{}`: malformed root must exit 1, got {:?}; stderr=\n{}",
            case.label,
            out.status.code(),
            cli_stderr_v1(&out)
        );

        let stderr = cli_stderr_v1(&out);
        assert!(
            stderr.contains("cli.error.class=command"),
            "case `{}`: missing `cli.error.class=command` in stderr:\n{stderr}",
            case.label
        );
        assert!(
            stderr.contains(case.expect_code_fragment),
            "case `{}`: missing error code `{}` in stderr:\n{stderr}",
            case.label,
            case.expect_code_fragment
        );

        let stdout = cli_stdout_v1(&out);
        assert!(
            stdout.is_empty(),
            "case `{}`: malformed root must not write stdout, got:\n{stdout}",
            case.label
        );
    }
}

#[test]
fn cli_binary_fails_closed_for_every_registry_family_negative_v1() {
    // The conformance registry is the single source of truth for each family's
    // canonical fail-closed negative. Driving ALL of them through the real binary
    // makes this hellgate consume that one table — every front-door family
    // (the 17 roots + the composition family) is pinned at the binary, not just
    // the skewed hand-written subset above. Each negative source is a single
    // statement, passed as one argv tail (inline argv ≡ document).
    for spec in root_surface_specs_v1() {
        let out = run_cli_argv_v1(&["parse", spec.negative.source]);

        assert_eq!(
            out.status.code(),
            Some(1),
            "{:?}: registry negative `{}` must exit 1; stderr=\n{}",
            spec.family,
            spec.negative.source,
            cli_stderr_v1(&out)
        );
        let stderr = cli_stderr_v1(&out);
        assert!(
            stderr.contains("cli.error.class=command"),
            "{:?}: missing `cli.error.class=command`:\n{stderr}",
            spec.family
        );
        assert!(
            stderr.contains(spec.negative.code),
            "{:?}: missing registry code `{}`:\n{stderr}",
            spec.family,
            spec.negative.code
        );
        assert!(
            cli_stdout_v1(&out).is_empty(),
            "{:?}: malformed root must not write stdout",
            spec.family
        );
    }
}

#[test]
fn malformed_parse_uses_stderr_channel_not_stdout_v1() {
    // Channel discipline: a command failure is a fail-closed fault, so the
    // diagnostic rides stderr while stdout stays empty — never the reverse.
    let out = run_cli_argv_v1(&["parse", "CHECK", "rule.hybrid"]);

    assert_eq!(
        out.status.code(),
        Some(1),
        "channel-discipline case: expected exit 1, got {:?}",
        out.status.code()
    );

    let stdout = cli_stdout_v1(&out);
    assert!(
        stdout.is_empty(),
        "channel-discipline case: stdout must be empty on a command failure, got:\n{stdout}"
    );

    let stderr = cli_stderr_v1(&out);
    assert!(
        stderr.contains("cli.error.class=command"),
        "channel-discipline case: stderr must carry the command diagnostic, got:\n{stderr}"
    );
    assert!(
        stderr.contains("check_missing_rule_or_scope"),
        "channel-discipline case: stderr must carry the parser error code, got:\n{stderr}"
    );
}
