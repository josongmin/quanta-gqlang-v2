#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "integration tests assert with expect/unwrap/indexing/panic and match exhaustively; allowed for test targets only"
)]

//! Doc gate: the language README's CLI command list, the `--help`/usage string
//! the binary emits, and the `CLI_COMMAND_TOKENS_V1` const cannot drift apart.
//! Structural guard for the `explain`-omission the audit caught.

mod support;

use std::fs;
use std::path::PathBuf;

use quanta_gqlang_v2_cli::CLI_COMMAND_TOKENS_V1;
use support::{cli_stderr_v1, cli_stdout_v1, run_cli_argv_v1};

fn language_readme_v1() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("docs/language/README.md");
    fs::read_to_string(&path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
}

/// Extract the `<a|b|...>` command alternation from the emitted `cli.usage=`
/// line. Parsing the alternation (not substring search) is what rejects a
/// missing bare alias that a longer alias textually contains.
fn usage_alternation_v1(usage: &str) -> Vec<&str> {
    let line = usage
        .lines()
        .find(|line| line.starts_with("cli.usage="))
        .expect("usage block carries a cli.usage= line");
    let field = line
        .strip_prefix("cli.usage=quanta-gqlang-v2-cli ")
        .expect("usage names the shipped binary")
        .split_whitespace()
        .next()
        .expect("usage carries a command field");
    let inner = field
        .strip_prefix('<')
        .and_then(|value| value.strip_suffix('>'))
        .expect("command field is one closed <a|b|...> alternation");
    assert!(
        !inner.contains(['<', '>']),
        "command alternation contains no nested or trailing brackets"
    );
    inner.split('|').collect()
}

#[test]
fn every_command_token_is_in_usage_and_documented_v1() {
    let readme = language_readme_v1();
    // No-args invocation fails closed with the usage string on stderr; that
    // string is the binary's own command list, so it ties the const to the
    // shipped binary.
    let output = run_cli_argv_v1(&[]);
    assert_eq!(
        output.status.code(),
        Some(2),
        "no-args invocation must fail closed as a usage error"
    );
    assert!(cli_stdout_v1(&output).is_empty());
    let usage = cli_stderr_v1(&output);

    // CLI-01: exact set comparison between the parsed binary alternation and
    // the advertised token set. Cardinality + membership + uniqueness reject
    // omissions, duplicates, and extras. `contains("fmt")` alone would accept
    // `v3-fmt` as proof for a missing bare `fmt`; the alternation parse does
    // not.
    let alternation = usage_alternation_v1(&usage);
    assert_eq!(
        alternation.len(),
        CLI_COMMAND_TOKENS_V1.len(),
        "usage alternation cardinality drift:\n{usage}"
    );
    let mut seen = std::collections::HashSet::new();
    for token in &alternation {
        assert!(seen.insert(*token), "duplicate usage token `{token}`");
    }
    for token in CLI_COMMAND_TOKENS_V1 {
        assert!(
            alternation.contains(&token),
            "const command `{token}` is missing from the binary usage alternation:\n{usage}"
        );
        // README coverage matches code-delimited command entries, not
        // arbitrary substring presence.
        let code_entry = format!("`{token}`");
        assert!(
            readme.contains(&code_entry),
            "language README does not list the CLI command as {code_entry}"
        );
    }
}

#[test]
fn every_command_token_dispatches_at_the_binary_boundary_v1() {
    // CLI-01: every advertised token is invoked through the compiled binary
    // with a minimal valid input. Alias pairs must be behaviorally identical:
    // equal exit code and byte-equal normalized stdout/stderr.
    let mut covered = ["query", "v3-query", "fmt", "v3-fmt"];
    let mut advertised = CLI_COMMAND_TOKENS_V1;
    covered.sort_unstable();
    advertised.sort_unstable();
    assert_eq!(
        advertised, covered,
        "every advertised token needs a binary dispatch case"
    );
    let query_pair = [
        run_cli_argv_v1(&["query", "show capabilities"]),
        run_cli_argv_v1(&["v3-query", "show capabilities"]),
    ];
    assert_eq!(query_pair[0].status.code(), Some(1));
    assert!(cli_stderr_v1(&query_pair[0]).is_empty());
    assert!(cli_stdout_v1(&query_pair[0])
        .lines()
        .any(|line| line == "analysis.status=incomplete"));
    assert_eq!(
        query_pair[0].status.code(),
        query_pair[1].status.code(),
        "query alias exit drift"
    );
    assert_eq!(
        query_pair[0].stdout, query_pair[1].stdout,
        "query alias stdout drift"
    );
    assert_eq!(
        query_pair[0].stderr, query_pair[1].stderr,
        "query alias stderr drift"
    );

    let fmt_pair = [
        run_cli_argv_v1(&["fmt", "let x = 1"]),
        run_cli_argv_v1(&["v3-fmt", "let x = 1"]),
    ];
    for output in &fmt_pair {
        assert!(
            output.status.success(),
            "fmt alias failed: {:?}\n{}",
            output.status.code(),
            cli_stderr_v1(output)
        );
        assert_eq!(cli_stdout_v1(output), "let x = 1\n");
        assert!(cli_stderr_v1(output).is_empty());
    }
    assert_eq!(
        fmt_pair[0].stdout, fmt_pair[1].stdout,
        "fmt alias stdout drift"
    );
    assert_eq!(
        fmt_pair[0].stderr, fmt_pair[1].stderr,
        "fmt alias stderr drift"
    );
}
