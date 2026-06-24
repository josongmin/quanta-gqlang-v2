#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration-test assertions intentionally use expect/unwrap/indexing and exercise fail-closed panics"
)]

//! CLI/LSP diagnostic parity over the SAME failing advanced query (ticket 08
//! required rail 7, D22).
//!
//! Both surfaces consume the ONE shared `pipeline::compile_program` (D22), so for
//! one failing fixture the CLI binary's stage-tagged error output and the LSP
//! `v3_diagnostics` handler must carry the SAME stage tag, the SAME inner stable
//! code, and the SAME exact byte span. A drift here means a surface forked the
//! compiler path, which D22 forbids.
//!
//! syntax->sema->ir path pinned: source text -> `compile_program` -> first
//! failing stage's typed diagnostic -> {CLI binary stderr, LSP handler item}.

mod support;

use quanta_gqlang_v2_lsp::handlers::v3_diagnostics::handle_v3_diagnostics_v1;
use support::{cli_stderr_v1, run_cli_argv_v1};

/// Pull a `key=value` line's value out of the CLI stderr block.
fn cli_field_v1(stderr: &str, key: &str) -> String {
    stderr
        .lines()
        .find_map(|line| line.strip_prefix(&format!("{key}=")))
        .unwrap_or_else(|| panic!("CLI stderr missing `{key}=`:\n{stderr}"))
        .to_owned()
}

/// For each failing advanced fixture, the CLI binary and the LSP handler agree
/// on stage, inner code, and exact span — bit-for-bit aligned because they share
/// the compiler path.
#[test]
fn cli_and_lsp_agree_on_stage_code_and_span_for_same_failure_v1() {
    let fixtures = [
        "from users | where (a + 1",
        "from users | project 1 + \"x\" as bad",
        "from findings f | group by count(f.id)",
    ];

    for source in fixtures {
        // LSP side: the read-only handler over the shared pipeline.
        let lsp = handle_v3_diagnostics_v1(source);
        assert_eq!(
            lsp.len(),
            1,
            "LSP must surface exactly one diagnostic for `{source}`"
        );
        let lsp = &lsp[0];

        // CLI side: the real binary's stage-tagged error block.
        let output = run_cli_argv_v1(&["v3-query", source]);
        assert_eq!(
            output.status.code(),
            Some(1),
            "CLI must fail closed (exit 1) for `{source}`"
        );
        let stderr = cli_stderr_v1(&output);

        assert_eq!(
            cli_field_v1(&stderr, "pipeline.error.stage"),
            lsp.stage,
            "stage drift between CLI and LSP for `{source}`"
        );
        assert_eq!(
            cli_field_v1(&stderr, "pipeline.error.code"),
            lsp.code,
            "code drift between CLI and LSP for `{source}`"
        );
        assert_eq!(
            cli_field_v1(&stderr, "pipeline.error.span"),
            format!("{}..{}", lsp.span_start, lsp.span_end),
            "span drift between CLI and LSP for `{source}`"
        );
    }
}
