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

The policy checks local syntax: negative-to-positive match arms, empty success,
loss of named semantic bindings, successful wildcard arms, selected `Err` to
`Ok` conversions, lossy `.ok()` in semantic fields, and semantic wire defaults.
It also checks the open-analysis status priority and document-error stdio
projection. Owner and binary tests remain responsible for reachability,
interprocedural flow, and actual front-door behavior. In stdio, `ok: true`
means transport success; `analysis_status` reports analysis success or failure.
