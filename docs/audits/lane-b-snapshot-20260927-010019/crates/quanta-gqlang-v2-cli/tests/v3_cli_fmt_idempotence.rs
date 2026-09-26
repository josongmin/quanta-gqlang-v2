#![expect(
    clippy::expect_used,
    reason = "integration tests fail with exact shipped-example path context"
)]

//! v3 `v3-fmt` front-door idempotence rail (ticket 08, D22).
//!
//! Mirrors the `fmt_idempotence_frontdoor` pattern but for the v3 surface: the
//! formatter consumes the ONE shared v3 parser (`surface::parse_program`, D22)
//! and the asserted property is IDEMPOTENCE — `fmt(fmt(x)) == fmt(x)`. The
//! canonical form is DISCOVERED by running the binary, not guessed; we then feed
//! it back through the front door and assert a fixed point.
//!
//! syntax->sema->ir path pinned: source text -> `surface::parse_program` ->
//! shared HIR -> canonical render -> re-parse -> equal canonical render.

mod support;

use std::path::{Path, PathBuf};

use support::{cli_stderr_v1, cli_stdout_v1, run_cli_argv_v1};

struct FormatFixture(PathBuf);

impl FormatFixture {
    fn new(name: &str, bytes: &[u8]) -> Self {
        let path =
            std::env::temp_dir().join(format!("qglang-fmt-{}-{name}.qglang", std::process::id()));
        std::fs::write(&path, bytes).expect("write format fixture");
        Self(path)
    }

    fn format(&self) -> std::process::Output {
        run_cli_argv_v1(&[
            "fmt",
            "--file",
            self.0.to_str().expect("UTF-8 fixture path"),
        ])
    }
}

impl Drop for FormatFixture {
    fn drop(&mut self) {
        let _removed = std::fs::remove_file(&self.0);
    }
}

#[test]
fn cli_preserves_explicit_reserved_alias() {
    let first = run_cli_argv_v1(&["fmt", "from t as where"]);
    assert_eq!(first.status.code(), Some(0));
    assert!(first.stderr.is_empty());
    assert_eq!(cli_stdout_v1(&first), "from t as where\n");
    let second = run_cli_argv_v1(&["fmt", &cli_stdout_v1(&first)]);
    assert_eq!(second.status.code(), Some(0));
    assert!(second.stderr.is_empty());
    assert_eq!(second.stdout, first.stdout);
}

#[test]
fn cli_rejects_output_token_and_byte_overflow_before_writing_stdout() {
    let keys = vec!["a"; 1_024].join(", ");
    let token_source = format!("from t{}", format!(" | order by {keys}").repeat(44));
    let byte_source = format!("let x = \"{}\"", "\n".repeat(600_000));
    for (name, source, expected_message, expected_code) in [
        (
            "tokens",
            token_source,
            "canonical output: tokens budget exhausted: limit 131072, observed 131073",
            "surface.resource_exhausted",
        ),
        (
            "bytes",
            byte_source,
            "canonical output exceeds the frontend budget's max_source_bytes",
            "frontend.resource_limit_exceeded",
        ),
    ] {
        let output = FormatFixture::new(name, source.as_bytes()).format();
        assert_eq!(output.status.code(), Some(1));
        assert!(
            output.stdout.is_empty(),
            "failed formatting must not emit a success artifact"
        );
        assert_eq!(cli_stderr_v1(&output), format!(
            "cli.error.class=command\nanalysis.status=error\nanalysis.diagnostic_count=1\ndiagnostic[0].stage=resource\ndiagnostic[0].code={expected_code}\ndiagnostic[0].severity=error\ndiagnostic[0].source_id=0\ndiagnostic[0].span=0..{}\ndiagnostic[0].message={expected_message}\n", source.len()
        ));
    }
}

#[test]
fn cli_exact_source_byte_cap_is_a_refeedable_fixed_point() {
    let cap = 1_048_576;
    let canonical = format!("let x = \"{}\"", "x".repeat(cap - 10));
    assert_eq!(canonical.len(), cap);
    let first = FormatFixture::new("exact-bytes", canonical.as_bytes()).format();
    assert_eq!(first.status.code(), Some(0));
    assert!(first.stderr.is_empty());
    assert_eq!(first.stdout, canonical.as_bytes());
    let second = FormatFixture::new("exact-bytes-refeed", &first.stdout).format();
    assert_eq!(second.status.code(), Some(0));
    assert!(second.stderr.is_empty());
    assert_eq!(second.stdout, first.stdout);
}

fn shipped_examples_v1() -> Vec<PathBuf> {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("CLI crate lives at <repo>/crates/<crate>");
    let mut pending = vec![repo_root.join("examples")];
    let mut examples = Vec::new();
    while let Some(dir) = pending.pop() {
        for entry in std::fs::read_dir(&dir).expect("read shipped examples directory") {
            let path = entry.expect("read shipped example entry").path();
            if path.is_dir() {
                pending.push(path);
            } else if path
                .extension()
                .is_some_and(|extension| extension == "qglang")
            {
                examples.push(path);
            }
        }
    }
    examples.sort();
    assert!(
        !examples.is_empty(),
        "shipped example corpus must not be empty"
    );
    examples
}

#[test]
fn shipped_examples_pass_cli_file_frontdoors_v1() {
    for path in shipped_examples_v1() {
        let source_path = path.to_str().expect("shipped path is UTF-8");
        let formatted = run_cli_argv_v1(&["fmt", "--file", source_path]);
        assert!(
            formatted.status.success(),
            "fmt --file failed for {source_path}: {}",
            cli_stderr_v1(&formatted)
        );
        assert!(
            !cli_stdout_v1(&formatted).trim().is_empty(),
            "fmt --file returned empty text for {source_path}"
        );
        assert!(
            formatted.stderr.is_empty(),
            "fmt --file wrote stderr for {source_path}: {}",
            cli_stderr_v1(&formatted)
        );

        let queried = run_cli_argv_v1(&["query", "--file", source_path]);
        assert_eq!(
            queried.status.code(),
            Some(1),
            "open-authority query must be incomplete for {source_path}: {}",
            cli_stderr_v1(&queried)
        );
        let stdout = cli_stdout_v1(&queried);
        assert!(
            stdout.starts_with("analysis.status=incomplete\n"),
            "query --file did not report incomplete analysis for {source_path}: {stdout}"
        );
        assert!(
            stdout.contains("analysis.item_count=1\n"),
            "query --file lost the example item for {source_path}: {stdout}"
        );
        assert!(
            queried.stderr.is_empty(),
            "query --file wrote stderr for {source_path}: {}",
            cli_stderr_v1(&queried)
        );
    }
}

/// Run `v3-fmt <inline>` through the real binary; assert exit 0; return stdout
/// with the single trailing newline `println!` appends trimmed off, so the
/// result is a clean inline-re-feedable document.
fn v3_fmt_inline_v1(inline: &str) -> String {
    let output = run_cli_argv_v1(&["v3-fmt", inline]);
    assert!(
        output.status.success(),
        "v3-fmt `{inline}` failed (status={:?})\nstderr={}",
        output.status.code(),
        cli_stderr_v1(&output)
    );
    cli_stdout_v1(&output).trim_end_matches('\n').to_owned()
}

#[test]
fn messy_declaration_has_canonical_binary_output_v1() {
    let source = "let  x =  1 +  2*3";
    let formatted = v3_fmt_inline_v1(source);
    assert_eq!(formatted, "let x = 1 + 2 * 3");
    assert_ne!(formatted, source);
}

/// A messy-but-valid v3 query/decl/rule paired with a descriptive label, used as
/// a non-canonical control to prove `v3-fmt` converges to a fixed point.
struct MessyV3InputV1 {
    label: &'static str,
    source: &'static str,
}

/// For a spread of advanced v3 forms: prove `v3-fmt` is an idempotent fixed
/// point. We assert the *property* (`fmt(fmt(x)) == fmt(x)`) plus non-empty
/// output and exit 0 — the canonical text itself is discovered, not guessed.
#[test]
fn v3_fmt_is_a_fixed_point_on_messy_advanced_inputs_v1() {
    let messy_inputs = [
        MessyV3InputV1 {
            label: "messy-pipeline",
            source: "from   users   u  |  where  u.id>1  | project u.id  as  id ,u.name as name",
        },
        MessyV3InputV1 {
            label: "join-group-order",
            source: "from findings f | join repos r on f.repo = r.id \
                     | group by f.repo | order by f.score desc",
        },
        MessyV3InputV1 {
            label: "nested-result-shape",
            source: "from users u | project u.id as id => { id: u.id, tags: [ u.t1 ] }",
        },
        MessyV3InputV1 {
            label: "decl",
            source: "let  x =  1 +  2*3",
        },
        MessyV3InputV1 {
            label: "rule-quantifier",
            source: "rule reachable : x  if exists y in ( edge(x,y) and y > 0 )",
        },
    ];

    for messy in messy_inputs {
        let first = v3_fmt_inline_v1(messy.source);
        assert!(
            !first.trim().is_empty(),
            "v3-fmt produced empty output for `{}`",
            messy.label
        );

        let second = v3_fmt_inline_v1(&first);
        assert_eq!(
            first, second,
            "v3-fmt is not a fixed point for `{}`:\n--- first ---\n{first}\n--- second ---\n{second}",
            messy.label
        );
    }
}

/// The bare `fmt` alias (not only `v3-fmt`) dispatches to the formatter at
/// the REAL binary boundary: it exits 0, reaches the same fixed point as
/// `v3-fmt` on identical input, and re-feeding its output is stable.
#[test]
fn bare_fmt_alias_matches_v3_fmt_fixed_point_v1() {
    let source = "from   users   u  |  where  u.id>1  | project u.id  as  id";
    let output = run_cli_argv_v1(&["fmt", source]);
    assert!(
        output.status.success(),
        "bare `fmt` failed (status={:?})\nstderr={}",
        output.status.code(),
        cli_stderr_v1(&output)
    );
    let bare = cli_stdout_v1(&output).trim_end_matches('\n').to_owned();
    assert!(!bare.trim().is_empty(), "bare `fmt` produced empty output");
    assert_eq!(
        bare,
        v3_fmt_inline_v1(source),
        "bare `fmt` and `v3-fmt` disagree:\n--- fmt ---\n{bare}"
    );
    let output = run_cli_argv_v1(&["fmt", &bare]);
    assert!(
        output.status.success(),
        "re-fed bare `fmt` failed: {}",
        cli_stderr_v1(&output)
    );
    assert_eq!(
        cli_stdout_v1(&output).trim_end_matches('\n'),
        bare,
        "bare `fmt` is not a fixed point"
    );
}

/// String literals re-quote so the formatted output re-lexes to the same content
/// and stays a fixed point even with embedded quote/escape characters.
#[test]
fn v3_fmt_preserves_string_literals_idempotently_v1() {
    let source = "let label = \"a \\\"quoted\\\" b\"";
    let first = v3_fmt_inline_v1(source);
    assert!(
        !first.trim().is_empty(),
        "v3-fmt produced empty output for `{source}`; an empty fixed point is vacuous"
    );
    let second = v3_fmt_inline_v1(&first);
    assert_eq!(
        first, second,
        "v3-fmt string re-quote is not a fixed point:\n--- first ---\n{first}\n--- second ---\n{second}"
    );
}
