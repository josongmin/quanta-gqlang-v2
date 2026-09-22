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

use support::{cli_stderr_v1, cli_stdout_v1, run_cli_argv_v1};

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
    let second = v3_fmt_inline_v1(&first);
    assert_eq!(
        first, second,
        "v3-fmt string re-quote is not a fixed point:\n--- first ---\n{first}\n--- second ---\n{second}"
    );
}
