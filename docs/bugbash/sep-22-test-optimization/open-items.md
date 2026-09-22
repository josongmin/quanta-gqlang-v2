# Sep 22 Test Optimization — Open Items Only

Audit base: `b6d52b9c49208cadb177e246fc72faf79f4e20cc`.

This is the only action list for this bugbash. It contains only issues reproduced against this
fixed V2 frontend snapshot. Historical blockers, completed work, deleted V1 seams, point-in-time
counts, and speculative duplicate candidates are intentionally absent. Main advanced after the
audit; re-run the baseline after the current pipeline work stabilizes before closing any item.

Implementation authority: [`tickets/README.md`](./tickets/README.md). The ticket packet maps every
ID below exactly once, defines non-overlapping writer leases, and reserves qualification for one
immutable integrated candidate.

## Verification baseline

- `cargo check --workspace --all-targets` — pass.
- `cargo clippy --workspace --all-targets -- -D warnings` — pass.
- `cargo nextest run --workspace --no-tests pass` — 1,052 passed, 0 skipped.
- `cargo test --workspace --doc` — 5 passed.
- `python3 tools/test_frontdoor_parity.py` — pass, 2 cases.

The retired TSV is not a baseline. It contained 979 tests while nextest now lists 1,052. Its
scanner missed macro-generated tests, invented a truncated function name through fixed-width
lookahead, and treated heuristic body hashes as duplicate verdicts.

## P1 — behavior or test-authority defects

| ID | Evidence | Actual gap | Required change | Verification |
|---|---|---|---|---|
| REF-01 | `crates/quanta-gqlang-v2-surface/src/family.rs:289-321` | `parse_ref` documents `::` but accepts a single `:` because the second colon is optional. Existing `describe rule::...` coverage consumes `::` before `parse_ref`, so this branch is untested. | Require exactly two colons. Add a positive `check repo::auth \| confirm=skip`, a negative `check repo:auth \| confirm=skip`, and a missing-segment case. | `cargo test -p quanta-gqlang-v2-surface --test v3_family_hellgate --test v3_family_edges` |
| TEST-01 | `Justfile:25-27`; `.github/workflows/lint.yml:38-45` | CI runs the unique shipped-binary CLI↔stdio parity gate, but `just test` runs only nextest and doctests. Local test authority can be green without the parity rail. | Make `test` invoke `frontdoor-parity`; remove the separate `frontdoor-parity` dependency from `lint` so there is one composition path. Keep `tools/test_frontdoor_parity.py`: no Rust test replaces its cross-binary payload comparison. | `just test` and `just lint` |

## P2 — reachable oracle gaps

| ID | Evidence | Actual gap | Required change | Verification |
|---|---|---|---|---|
| DOC-01 | `crates/quanta-gqlang-v2-surface/tests/v3_document_hellgate.rs:61-75` | `malformed_middle_item_rejects_with_exact_span` only checks `span.start >= prefix.len()`. A diagnostic at a later token or the third item passes. | Assert the exact offending `db` start and end. | `cargo test -p quanta-gqlang-v2-surface --test v3_document_hellgate malformed_middle_item_rejects_with_exact_span` |
| DOC-02 | `crates/quanta-gqlang-v2-surface/tests/v3_document_hellgate.rs:79-90,101-105`; `crates/quanta-gqlang-v2-surface/src/parser.rs:190-218` | Empty statement, leading separator, and missing separator tests assert only `UnexpectedToken`, while production has distinct message/span branches. One generic reject can satisfy all three. | Table-drive exact message and span assertions for each branch. | `cargo test -p quanta-gqlang-v2-surface --test v3_document_hellgate` |
| VIEW-01 | `crates/quanta-gqlang-v2-surface/tests/v3_family_hellgate.rs:360-370` | The inline VIEW test matches `ViewBodyV1::Inline(_)` and discards the query. A parser that drops or truncates `where u.active` can remain self-consistent in round-trip coverage. | Assert the inline query's independent structural witness or exact canonical body. | `cargo test -p quanta-gqlang-v2-surface --test v3_family_hellgate view_inline_select_parses_to_catalog_decl` |
| EXPLAIN-01 | `crates/quanta-gqlang-v2-planning/tests/v3_explain_provenance.rs:117-148`; `crates/quanta-gqlang-v2-planning/src/plan.rs:679-681,795-799` | The contract is left-before-right preorder, but both join tests use symmetric sibling family sequences. Swapping sibling traversal stays green. | Add one fixture whose left and right subtrees have different family shapes and assert the exact sequence. | `cargo test -p quanta-gqlang-v2-planning --test v3_explain_provenance` |
| CLI-01 | `crates/quanta-gqlang-v2-cli/tests/command_doc_gate.rs:25-44`; `crates/quanta-gqlang-v2-cli/src/main.rs:121-128` | `usage.contains("fmt")` is satisfied by `v3-fmt`; the test neither proves the bare alias exists nor rejects extra usage tokens. No test invokes bare `fmt`. | Parse the usage alternation and set-compare it with `CLI_COMMAND_TOKENS_V1`; invoke every command token at the binary boundary. | `cargo test -p quanta-gqlang-v2-cli --test command_doc_gate --test v3_cli_fmt_idempotence` |
| PRODUCER-01 | `crates/quanta-gqlang-v2-surface/tests/v3_family_hellgate.rs:950-960`; `crates/quanta-gqlang-v2-surface/src/family.rs:129-141,228-252` | `composition_explain_producer_rejects` is rejected by composition lookahead and never reaches `parse_producer`, so it does not pin the producer-column allowlist it names. | Use a valid first binding followed by an invalid second `let bad = explain ...` so the actual producer parser rejects it; assert exact code/message/span. | `cargo test -p quanta-gqlang-v2-surface --test v3_family_hellgate composition_explain_producer_rejects` |
| CLI-02 | `crates/quanta-gqlang-v2-cli/src/main.rs:147-149`; `crates/quanta-gqlang-v2-cli/tests/open_analysis_contract.rs:41-62` | Valid and over-budget file ingress are covered; the shared trim-empty guard is not. | Add empty inline and empty-file cases with exit 2 and the exact usage error. | `cargo test -p quanta-gqlang-v2-cli --test open_analysis_contract` |
| STAGE-01 | `crates/quanta-gqlang-v2-surface/src/query.rs:133-151`; `crates/quanta-gqlang-v2-surface/tests/v3_surface_query.rs:156-187` | `cross` is accepted but omitted from the closed-stage diagnostic. The test checks only a substring, so the drift is invisible. | Include `cross` in the message and assert the complete expected message for retired stages. | `cargo test -p quanta-gqlang-v2-surface --test v3_surface_query relation_pipeline_stage_set_is_closed` |

## P3 — measured deletion

| ID | Evidence | Actual gap | Required change | Verification |
|---|---|---|---|---|
| DEDUPE-01 | `crates/quanta-gqlang-v2-surface/tests/v3_family_hellgate.rs:426-436,455-461`; `crates/quanta-gqlang-v2-surface/tests/v3_family_edges.rs:196-222` | The weak profile/gate missing-clause tests use the same inputs as stronger exact-message rails. They add executions without an independent oracle. | Delete `profile_missing_capabilities_rejects` and `gate_missing_checks_rejects`; keep the `family_edges` rails. | Run both surface test targets. |

## Execution order

1. Fix `REF-01` and `TEST-01`.
2. Add the P2 oracles.
3. Delete `DEDUPE-01` only after the stronger rails remain green.
4. Re-run the verification baseline on one fixed commit.
