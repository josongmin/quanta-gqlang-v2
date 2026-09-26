# Semantic honesty policy

`just semantic-honesty` runs the policy fixtures and scans production Rust under
`crates/**/src/*.rs`. It is part of `just policy` and `just ci`.

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
