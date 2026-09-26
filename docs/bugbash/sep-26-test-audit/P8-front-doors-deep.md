# P8 — front doors deep fail-open audit (cli + diagnostics-stdio + conformance)

Final state. Base: committed HEAD `9040a8a` ("Close Sep 26 test audit proof
gaps"), plus this pass's supplements (uncommitted working tree at the time of
writing). Companion to the curated `P8-front-doors.md` (TA-01/TA-05 resolution
record); this file carries the full audit, the supplement pass, and the
remaining held items.

Scope:

- `crates/quanta-gqlang-v2-cli/tests/**` (`command_doc_gate.rs`,
  `ingress_matrix.rs`, `open_analysis_contract.rs`, `v3_cli_fmt_idempotence.rs`,
  `support/mod.rs`)
- `crates/quanta-gqlang-v2-diagnostics-stdio/tests/**` (`binary_contract.rs`,
  `ndjson_contract.rs`, `resource_boundaries.rs`, `server_rejection_matrix.rs`)
- `crates/quanta-gqlang-v2-conformance/tests/self_validation.rs`
- every `#[cfg(test)]` block in the three crates' `src/` (`cli/src/lib.rs`,
  `cli/src/line_value.rs`, `cli/src/main.rs`; the other two `src/` trees have
  zero test attributes)

Method: five independent read-only audits in parallel (per-crate deep pass ×3,
an adversarial "what one-line production mutation still leaves this green"
pass, and a ledger-vs-assertion cross-check), then a lead pass that re-read and
re-verified every claim in the implementation before editing, then a final
supplement pass on the committed HEAD that closes every remaining gap closable
by assertion rather than by production change. Classes A/B/C/D/D'/E/F/G/H plus
process class I. `docs/`, fixtures, and production code were not modified;
`just ci` / `just qualify-local` were not run (integrator-only).

## 1) Applied fixes

### 1a. Guard pass (fail-open → fail-closed; landed before `9040a8a`)

| file:line | class | note |
|---|---|---|
| `cli/tests/command_doc_gate.rs:47-51` | I | No-args invocation never asserted its exit status; usage-on-stderr + exit 0 passed. Added `Some(2)`. |
| `cli/tests/v3_cli_fmt_idempotence.rs` | E | String-literal rail compared only `first == second`; `""` is a fixed point. Added a non-empty guard. |
| `cli/src/lib.rs` | E | Alias parity `None == None` passed when both canonical token and alias stopped resolving. Added non-None guards (later superseded by exact kind identity, §1b). |
| `conformance/tests/self_validation.rs:234-239,412-417` | F | Empty-registry vacuity in two loops. Added exact `len == ROOT_FAMILIES_V2.len()` guards. |
| `conformance/tests/self_validation.rs:458-462` | F | `.all(BlockedByDiagnostics)` was vacuously true over an empty item list. Added `!first.items().is_empty()`. |
| `diagnostics-stdio/tests/ndjson_contract.rs:199-202,218-221` | E/F | Determinism and chunked-vs-baseline comparisons passed on two empty outputs. Added non-empty guards. |
| `cli/tests/ingress_matrix.rs:243-246` | E | Leading-empty-argv differential compared empty stderr to empty stderr. Added non-empty guard. |
| `diagnostics-stdio/tests/binary_contract.rs` | D' | `diagnostics: []` survived the stdio error branch (later superseded by TA-01 exact field pinning). |

TA-01 (integrator, `9040a8a`) then pinned one complete error-branch diagnostic
(stage/code/severity/message/source_id/span) in the shipped binary test, and
TA-05 added `messy_declaration_has_canonical_binary_output_v1` asserting the
exact canonical text for a messy declaration.

### 1b. Supplement pass (final, this session)

| file:line | closes what |
|---|---|
| `cli/src/lib.rs:95` | Tautological `from_specs == CLI_COMMAND_TOKENS_V1` (both sides derive from the same table) → literal `["query","fmt","v3-query","v3-fmt"]`, which can fail on reorder/rename. |
| `cli/src/lib.rs:101-115` | Alias parity alone accepted a full `query↔fmt` kind swap. Now pins `kind("query") == Some(Query)` and `kind("fmt") == Some(Fmt)` before parity. |
| `cli/src/main.rs:257,287,313` | `parse_invocation`'s `.command` was never asserted (a `Some(_) => Fmt` mutation passed). Now asserts `CliCommandKindV2::Query` on all three success paths. |
| `cli/tests/ingress_matrix.rs:135-136` | The `help`/`--help` arity gate had no negative case; removing `size_hint()==(1,Some(1))` would silently make `help x` exit 0. Added `help x` / `--help x` usage failures. |
| `cli/tests/ingress_matrix.rs:238-242` | Formatter-failure parse code was a `contains("...surface.parse.")` substring. Now exact `surface.parse.unexpected_eof`. |
| `cli/tests/ingress_matrix.rs:391-396` | Symlink positive case used `contains("analysis.status=incomplete")`. Now line-exact. |
| `cli/tests/open_analysis_contract.rs:69-90` | Hard-source-error rail used substrings. Now exact `analysis.diagnostic_count=1`, `diagnostic[0].code=surface.parse.unexpected_eof`, `diagnostic[0].span=25..25`. |
| `cli/tests/open_analysis_contract.rs:112-118` | Exact-budget rail used `contains` for the accepted status and never asserted stdout empty on the over-limit side. Now line-exact + `stdout.is_empty()`. |
| `diagnostics-stdio/tests/ndjson_contract.rs:153-172` | The incomplete-analysis response only asserted a non-empty diagnostics array. Now pins the exact three codes, order, `stage`/`severity`/`source_id`/`span_start`/`span_end`, and non-empty message. |
| `diagnostics-stdio/tests/ndjson_contract.rs:211,310` | Glued-framing and whitespace rails asserted only `ok == false` / echoed `id`. Now also `error.code == invalid_request` and `ok == true` on the recovered line. |
| `diagnostics-stdio/tests/server_rejection_matrix.rs:32-56` | Ledger L88 "closed read-only command plane" was asserted only via `shutdown`. New test pins `READ_ONLY_METHOD_NAMES_V2 == ["v3_diagnostics"]` and rejects seven other/retired method names. |
| `diagnostics-stdio/tests/server_rejection_matrix.rs:88` | TA-02: added a wrong-`source`-type case **without** `extra`, so the type rejection is isolated from the unknown-field rejection. |
| `conformance/tests/self_validation.rs:240-250` | `spec.family` was never validated and the keyword check was a prefix `starts_with`. Now asserts the registry maps one-to-one onto `ROOT_FAMILIES_V2` in order, and that each source's first token equals its declared keyword. |

## 2) Remaining held items (report-only)

| file:line | class | severity | why it remains |
|---|---|---|---|
| `diagnostics-stdio/tests/server_rejection_matrix.rs:95` | E | 낮음 | The schema matrix asserts `!error.1.is_empty()` only; exact messages are unpinned. Needs message goldens. |
| `conformance/tests/self_validation.rs:661` | E | 낮음 | `aggregate.expressions().len() == 1` is count-only; a `sum(f.score)`→`count(f.score)` swap keeps one expression and the field name `total`. Needs an expression-shape/argument assert. |
| `conformance/tests/self_validation.rs:761` | E | 낮음 | `value.item(*value_id) == value.items().first()` derives `value_id` from `first()`, so an id-ignoring `item()` passes. Needs an absent-id lookup expecting `None`. |
| `conformance/tests/self_validation.rs:320-347` | F | 낮음 | `selected_shipped_examples_...` iterates a test-local 4-entry literal with no count assert; only a test edit can empty it. |
| `diagnostics-stdio/tests/ndjson_contract.rs` (determinism / short-reads) | E | 낮음 | Differential rails compare two runs of the same implementation; an independent response golden lives only in the sibling rails. Inherent to a differential property. |
| `cli/tests/command_doc_gate.rs:34` | E (latent) | 낮음 | `usage_alternation_v1` takes the first `<`/`>`. An extra future bracket would truncate silently; current alternation has none. |
| `cli/tests/command_doc_gate.rs:86-134` | D' (latent) | 낮음 | The dispatch rail hardcodes the four tokens; all four are covered today, so only a future token addition escapes. |
| `conformance/tests/self_validation.rs:267` | A (latent) | 낮음 | `(NoPlan, _)` wildcards the item variant, but `assert_disposition` panics on a mismatch first. |

### Ledger cross-check (report-only; `docs/` not modified)

| ledger line | claim | reality |
|---|---|---|
| L111 | "`command_doc_gate.rs` pins the shipped CLI allowlist **and diagnostics method documentation**" | The file has no diagnostics/method content. Stale. |
| L88 header | diagnostics row under "Binary rails" | 3 of its 4 files are API-only (`run_stdio_session_v2`, `run_stdio_session_with_budget_v2`, `DiagnosticsServerV2::dispatch`); only `binary_contract.rs` spawns. Mislabeled per the ledger's own proof-boundary rule. |
| L100 | "V3 item/family/**payload-presence** mapping" | Only relation/composition presence is asserted. Partial. |
| L88 | "the closed read-only command plane" | Now substantively covered by the new `only_the_advertised_read_only_method_is_admitted` test; the ledger text can stay. |
| L88 | "Strict NDJSON schema" | Strictness is request-side only; no response-side closed-schema assertion. |

## 3) Refuted claims (verified, no action)

- `diagnostics-stdio/tests/resource_boundaries.rs:83-99` — a parallel audit
  reported the frame-limit fixture as a live D'. Re-verified against
  `src/lib.rs:43-79` and `src/server.rs:106-128`: overflow emits
  `frame_limit_exceeded`, schema failure emits `invalid_request`, so
  `responses[1] == invalid_request` (256 admitted) vs `responses[2] ==
  frame_limit_exceeded` (257 rejected) does pin the exact boundary; a `>=`
  off-by-one would flip response[1]. Not fail-open.
- `self_validation.rs` uniqueness — can fail if the const slice lists a variant
  twice; meaningful.
- `self_validation.rs` `zip` — both lengths are pinned to `2` before the zip;
  truncation is impossible.
- `parse_invocation` budget oracles — accepted boundary is `len == limit` and
  rejection is `len == limit + 1`, matching production `> limit`; no off-by-one.

## 4) Swept clean

- Class A: no `if let Some/Ok(..) { assert }` (only `let .. else { panic! }`).
- Class B: every `.get()/.find()/.first()/.nth()` is consumed by
  `.expect(...)` or `assert_eq!`; no silent skip.
- Class C: no caller-parameter gate.
- Class D: no proptest targets in these crates.
- Class F: every other loop is a fixed literal array or is guarded by an exact
  count/non-empty assert.
- Class G: every spawned process asserts exit status; every structured response
  asserts `id` + `ok`/`error.code`.
- Class I: shipped CLI and stdio children now share `tools/test_support/process.rs`
  with a 10 s watchdog, concurrent pipe drains, and direct-child kill/reap cleanup.
  Timeout custody and output beyond pipe capacity have independent controls.
  This helper does not promise descendant process-group cleanup.
- Hygiene: `#[ignore]` 0, `let _ =` 0, `unwrap_or*` as oracle 0, `filter_map`
  0, `todo!`/`unimplemented!` 0, tautological `assert_eq!(x, x)` 0.

## 5) Verification

| stage | command | result |
|---|---|---|
| committed-HEAD baseline `9040a8a` | `cargo test -p quanta-gqlang-v2-cli -p quanta-gqlang-v2-diagnostics-stdio -p quanta-gqlang-v2-conformance` | green, 59 passed, 0 failed, 0 ignored |
| final (guard + supplement pass) | same command | green, **60 passed**, 0 failed, 0 ignored (cli 28 + diagnostics-stdio 22 + conformance 10) |
| lint | `cargo clippy -p quanta-gqlang-v2-cli -p quanta-gqlang-v2-diagnostics-stdio -p quanta-gqlang-v2-conformance --tests` | Finished clean, exit 0 |
| empirical | shipped stdio binary for `shutdown/initialize/hover/completion/symbols/diagnostics/zero_query` | each `method_not_allowed` (grounds the new plane test) |

`just ci` / `just qualify-local` were not run (integrator-only).
