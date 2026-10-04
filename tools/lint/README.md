# Local Rust lint policies

`just size-lint` uses locked Tokei 14.0.0 to count code lines in every Rust
file under `crates/*/src/` and `crates/*/tests/`, including untracked source.
`size_policy.json` limits production files to 1,200 code lines and test
files to 2,500. Its legacy exception list is empty. `test_support.rs` and
extracted owner unit-test modules receive the test limit only while their
declarations remain test guarded.
The gate fails on tool-version drift, missing or inaccurate Tokei results,
unknown files, stale exceptions, and exceeded ceilings. It measures source
size, not algorithmic complexity or behavioral correctness.

The same gate uses the pinned Rust AST parser to reject production
`allow`/`expect` overrides of `clippy::too_many_lines` and the covering
`clippy::pedantic`, `clippy::all`, and `warnings` groups. Module, impl, crate,
and conditional attributes are included. Literal macro-emitted overrides and
dynamic macro lint arguments are rejected. Comments and string literals are
not attributes. Real test-only scopes can retain test fixture exceptions.
Clippy enforces the actual 200-line function limit; this AST check prevents
local suppressions from bypassing it. The independent policy fixtures run
through the existing `ci-rail-test` command in local CI.

## Semantic honesty policy

`just semantic-honesty` runs the policy fixtures and scans production Rust under
`crates/**/src/*.rs`. It is part of `just policy` and `just ci`.

Out-of-line unit-test files are excluded only when the shared exact module
inventory proves a real `cfg(test)` and matching `path` on their owner. Missing,
commented, or mismatched guards fail the scan; a test-like filename is insufficient.

The scanner uses `tree-sitter-language-pack` pinned in `requirements.txt`.
Install it into the same Python environment used by `just` before running the
gate. Missing parser support fails the gate.

`semantic_honesty_policy.json` lists the owned enum families, their polarity,
and the one permitted `ItemRunV2::NoPlan` → `AnalysisItemV2::NotApplicable`
projection. The enum inventory must match current production declarations
exactly. The scanner skips test-only Rust and fails when a parse error overlaps
a governed expression. No suppressions or baseline file are used.

The policy enumerates the syntactic exits of each governed match arm, including
`if`, nested `match`, blocks, and early `return` paths. Closure-local returns
are separate. It classifies the returned constructor, not constructors inside
arguments. It checks negative-to-positive transitions, empty or untyped
success, forbidden scalar exits, payload binding use, successful wildcards,
`Err` to `Ok` conversions, lossy `.ok()` in semantic fields, and semantic wire
defaults. Unsupported control exits fail the scan. A direct macro exit is
accepted only inside a function whose return type AST is exactly `Vec<String>`;
it cannot construct a governed enum or `Result` at that return position.

The open-analysis status priority and document-error stdio projection have
separate checks. Each successful document-error exit must construct a payload
with error status and retained diagnostics. The local AST check does not prove
interprocedural flow, arbitrary reexports or name shadowing inside one scope,
side-effect collection semantics, or
the content of `Vec<String>` renderers. Owner and binary tests cover those
behaviors. In stdio, `ok: true` means transport success; `analysis_status`
reports analysis success or failure.
