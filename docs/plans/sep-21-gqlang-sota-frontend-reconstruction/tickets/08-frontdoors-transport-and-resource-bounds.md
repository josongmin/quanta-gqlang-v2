# 08 — Front Doors, Transport, And Resource Bounds

Status: `PLANNED`

Decision authority: [00 — Packet Scope And Pinned Decisions](./00-packet-scope-and-pinned-decisions.md)

## Findings Closed

F12's protocol-hardening portion, F24, F26, and F36.

## Goal

Route every public front door through the same document API and make all
user-controlled resource growth explicitly bounded.

## Current-Head Reuse/Delete/Build Audit

| Class | Assets | Concrete action |
|---|---|---|
| `KEEP` | stdio read/write loop, deterministic rendering helpers, one-byte/chunk/invalid-UTF-8 testing techniques, CLI argument/file handling | retain only behavior compatible with the pinned NDJSON/document API |
| `MIGRATE` | moved diagnostics crate, `run_stdio_session_v1`, CLI query/fmt, CLI-service parity tests | change framing/request carrier and facade calls in place |
| `DELETE` | object-concatenation extractor, handwritten request JSON scanner, glued-object/whitespace-frame expectations, whole-buffer clear, old LSP names, all inventoried V1 compile seams | delete rather than retain a legacy protocol mode |
| `NEW` | NDJSON frame state, duplicate-key parser hook, request IDs, byte-range diagnostics, ingress limits | use one maintained JSON parser and one framing implementation |

All nine old transport test files are tracked-moved with the crate, then each
case is classified as `retain`, `rewrite`, or `delete`. The three new transport
test files consolidate those cases; copying the old suite and leaving obsolete
object-concatenation tests is forbidden. Existing CLI front-door/parity tests
are similarly migrated, not duplicated.

## Write Set

1. `crates/quanta-gqlang-v2-cli/src/commands/v3_query.rs`
2. `crates/quanta-gqlang-v2-cli/src/commands/v3_format.rs`
3. `crates/quanta-gqlang-v2-cli/src/main.rs`
4. `crates/quanta-gqlang-v2-cli/Cargo.toml` — integrator only
5. `crates/quanta-gqlang-v2-conformance/Cargo.toml` — integrator only
6. `crates/quanta-gqlang-v2-pipeline/src/compile.rs` — integrator only
7. `crates/quanta-gqlang-v2-pipeline/src/lib.rs` — integrator only
8. `crates/quanta-gqlang-v2-frontend-contract/src/lib.rs` — integrator only
9. existing `crates/quanta-gqlang-v2-diagnostics-stdio/`
10. every crate path recorded in `LEGACY_API_INVENTORY.txt`; no path outside that
   inventory is implied
11. `/Cargo.toml` and `/Cargo.lock` — integrator only if dependency edges change
12. `/README.md` and `/docs/language/README.md`
13. the exact CLI/service/pipeline tests listed below
14. `docs/plans/sep-21-gqlang-sota-frontend-reconstruction/receipts/08.md`
15. `crates/quanta-gqlang-v2-cli/src/v3_plan.rs`
16. `crates/quanta-gqlang-v2-cli/tests/command_doc_gate.rs`
17. `crates/quanta-gqlang-v2-diagnostics-stdio/tests/method_allowlist_doc_gate.rs`
18. CLI and diagnostics owner-local test support modules

Surface, modules, sema, and logic budget implementations are read-only inputs
from tickets 02, 03, 04, and 06. Missing charge points reopen their owner ticket;
08 must not add cross-owner walkers.

## Pinned Transport

Ticket 01 has already renamed the crate and binary to
`quanta-gqlang-v2-diagnostics-stdio`. This ticket must not restore the old name.

1. Keep one read-only source-document diagnostics request.
2. Define explicit framing, request ID, size bounds, and recovery.
3. Remove all LSP claims, method documentation, crate names, binary names, and
   test names from this transport.
4. Do not add JSON-RPC, `initialize`, document lifecycle, or UTF-16 positions in
   this packet. Those belong to a future actual-LSP packet.

## Pinned Wire Contract

The diagnostics transport is UTF-8 NDJSON: one JSON object followed by one
`\n` byte per frame. JSON newlines inside source are escaped. `\r\n` is accepted
and normalized; blank lines reject as `transport.empty_frame`.

Request:

```json
{"id":1,"method":"v3_diagnostics","source":"where 1"}
```

`id` is required and is either a nonnegative JSON integer within `u64` or a
nonempty string of at most 128 UTF-8 bytes. `method` has exactly one accepted
value. `source` is required. Unknown fields reject; duplicate JSON object keys
reject rather than last-write-wins.

Response for a parsed ID:

```json
{"id":1,"ok":true,"analysis_status":"complete|incomplete|error","diagnostics":[]}
```

Malformed UTF-8/JSON returns `id:null`, `ok:false`, and one transport diagnostic.
No partial scan attempts to recover an ID from malformed JSON. A parsed JSON
object may echo its validated ID even when another request-field validation
fails. Codes are `transport.empty_frame`,
`transport.frame_too_large`, `transport.invalid_utf8`,
`transport.malformed_json`, `transport.invalid_request_id`,
`transport.unknown_field`, `transport.duplicate_key`, and
`transport.unknown_method`, plus `transport.incomplete_frame` for a nonempty
EOF tail. Serialization uses a maintained JSON parser with
duplicate-key detection; do not extend the hand-written JSON scanner.

`ok` reports request/framing validity, not semantic success. A well-formed
request with `analysis_status=error` therefore has `ok:true`. Transport
diagnostics use `TransportByteRangeV2 { start, end }` within the offending line;
frontend diagnostics retain source spans. An oversized frame reports range
`0..max_frame_bytes + 1` and never echoes buffered payload.

`max_frame_bytes` counts JSON payload bytes only: it excludes the required LF
and the optional CR immediately before LF. A payload exactly at the limit is
accepted. Observing payload byte `limit + 1` switches to discard-through-LF
without retaining later payload bytes. Source length is checked independently
after JSON decode and reports `frontend.resource_limit_exceeded` with dimension
`max_source_bytes`.

## Shared Front Door

1. CLI `query` and diagnostics stdio call
   `analyze_source_document_v2(source, OpenAnalysisAuthorityV2::standard_v2())`.
2. CLI `fmt` calls only facade `format_source_document_v2` with the standard
   frontend budget, which projects to the owner-local parse budget; it does not
   analyze or compile.
3. `compile_source_document_v2` remains the library seam for an embedding host
   that supplies `ClosedFrontendAuthorityV2`; CLI/service do not manufacture it.
4. Analysis with unresolved authority returns `AnalysisDocumentV2` plus stable
   diagnostics and never a compiled plan/artifact.
5. CLI exits 1 when analysis is incomplete or erroneous; diagnostics stdio
   returns a successful transport response containing the same ordered semantic
   codes/spans and an explicit `analysis_status`.
6. A fully resolved open analysis is still labeled `analysis`, not converted to
   `CompiledDocumentV2`.
7. Both front doors accept full source documents, not different
   single/multi-root parsers.
8. Both render the same ordered diagnostic code/span payload.

## Resource Budgets

Add one `FrontendBudgetV2` covering:

1. source bytes
2. frame bytes
3. token count
4. document item count
5. expression depth and width
6. list/record/projection width
7. module graph nodes/edges and rule graph nodes/edges as separate limits
8. witness depth/cap
9. semantic, module, and rule traversal steps as separate limits

Budget exhaustion returns typed `resource_limit_exceeded`; it does not panic,
truncate silently, or clear unrelated buffered requests.

## Budget Ownership And Projection

1. `quanta-gqlang-v2-sema::FrontendBudgetV2` is the sole public configuration
   and is created by the authority constructors from ticket 04.
2. Pipeline reads that value and uses the private owner-local views created by
   tickets 02-06. Owner-local types are not public knobs.
3. The diagnostics stdio service applies `max_frame_bytes` before deserialization
   and passes the same authority budget to document analysis.
4. CLI applies `max_source_bytes` while reading and passes the same open
   authority budget to document analysis.
5. Every cumulative field is charged once by the canonical owner table in
   `TARGET_ARCHITECTURE.md`. Pipeline passes identical immutable configuration
   and creates one counter per owner; it does not pass remaining counters across
   stages. CLI/service may repeat an absolute byte-cap check before allocation.

## Framing Recovery

1. consume only the offending frame/prefix
2. preserve later complete frames already buffered
3. define EOF handling for incomplete frames
4. bound resynchronization work

Recovery scans only to the next newline. Once a frame exceeds
`max_frame_bytes`, retain no more payload bytes, discard through the next
newline, emit one `transport.frame_too_large`, and continue. A nonempty tail at
EOF emits one `transport.incomplete_frame`; an empty tail emits nothing.

## Step-By-Step Procedure

1. Verify ticket 01's crate/binary rename and unchanged baseline tests before
   editing transport behavior.
2. Migrate CLI to frontend-contract only; remove direct HIR/surface/planning/
   pipeline normal dependencies unless a documented test-only edge remains.
3. Migrate the service to frontend-contract only.
4. Replace both public executable paths with source-document analysis. Closed
   compilation remains library-only for embedding hosts.
5. Share one diagnostic rendering DTO and ordering policy.
6. Construct one `OpenAnalysisAuthorityV2::standard_v2()` per CLI/service
   request path and use its same `FrontendBudgetV2` for ingress and analysis.
   Embedding hosts construct closed authority only through the library seam.
7. Verify the owner charge points from tickets 02-06; add only CLI source-read
   and service frame-accumulation prechecks in this ticket.
8. Implement typed budget diagnostics without truncating later frames.
9. Replace whole-buffer clear recovery with bounded prefix/frame consumption.
10. Migrate every code, test, and comment entry in
    `LEGACY_API_INVENTORY.txt` to V2 or delete obsolete V1-only coverage.
11. Delete every V1 compile-seam definition/re-export recorded in
    `LEGACY_API_INVENTORY.txt`, plus
    `crates/quanta-gqlang-v2-pipeline/tests/v3_legacy_api_ratchet.rs` and
    `LEGACY_API_INVENTORY.txt` after the final zero-match scan passes.
12. Mark tickets 02-08 `INTEGRATED` only after the whole workspace compiles with
    no V1 seam.

Prove the old identity did not return:

```bash
test -z "$(rg -l 'quanta[-_]gqlang[-_]v2[-_]lsp' \
  Cargo.toml Cargo.lock crates README.md docs/language)"
rg -n 'diagnostics-stdio|flat-JSON|not an LSP' README.md docs/language
```

The first command must find no old identity; the second must find the honest
transport description. Historical audit evidence under this Sep-21 packet may
retain the old name because it describes the audited base.

## Required Budget Fields

```text
max_source_bytes
max_frame_bytes
max_tokens
max_document_items
max_expression_depth
max_expression_nodes
max_collection_width
max_projection_width
max_semantic_steps
max_module_graph_nodes
max_module_graph_edges
max_module_steps
max_rule_graph_nodes
max_rule_graph_edges
max_rule_steps
max_witness_depth
max_witnesses
```

Each rejection identifies the exhausted dimension and configured limit without
echoing an unbounded user payload.

## Test Files

The existing nine transport test files are inputs to a tracked rewrite. Preserve
one-byte delivery, invalid UTF-8 recovery, deterministic diagnostics, method
allowlist, and parity coverage; delete glued-object and whitespace-separated
object expectations that contradict NDJSON. Migrate CLI
`v3_cli_front_door_contract.rs`, `v3_cli_lsp_parity.rs`, and
`v3_cli_fmt_idempotence.rs`; rename every remaining `lsp` test/helper token.

1. `[MOVE+REWRITE]` `crates/quanta-gqlang-v2-cli/tests/v3_cli_front_door_contract.rs` to `crates/quanta-gqlang-v2-cli/tests/v3_document_frontdoor_v2.rs`
2. `[MOVE+REWRITE]` `crates/quanta-gqlang-v2-cli/tests/v3_cli_lsp_parity.rs` to `crates/quanta-gqlang-v2-cli/tests/v3_cli_service_parity_v2.rs`
3. `[MOVE+REWRITE]` `crates/quanta-gqlang-v2-lsp/tests/stdio_frame_extractor_edges.rs` to `crates/quanta-gqlang-v2-diagnostics-stdio/tests/stdio_resynchronization.rs`
4. `[ADD]` `crates/quanta-gqlang-v2-diagnostics-stdio/tests/stdio_resource_limits.rs`
5. `[MOVE+REWRITE]` `crates/quanta-gqlang-v2-lsp/tests/v3_diagnostics_parity_hellgate.rs` to `crates/quanta-gqlang-v2-diagnostics-stdio/tests/document_diagnostics_parity.rs`
6. `[UPDATE AFTER CRATE MOVE]` remaining diagnostics byte/JSON/frame/determinism/allowlist tests in place
7. `[ADD]` `crates/quanta-gqlang-v2-pipeline/tests/v3_frontend_budget.rs`

Audited transport/front-door disposition after the NDJSON cut:

1. Re-derive the malformed-escape/rejected-frame/high-count cases against the
   new line-framing branches; do not copy object-concatenation expectations.
2. Collapse binary frame mirrors into one table-driven spawned-binary smoke that
   retains clean-run `stderr.is_empty()`, exit-status, and response-count
   assertions. Keep resynchronization cases for non-object input and brace depth
   only when still reachable under NDJSON; delete the whole-buffer-clear case.
3. Hoist repeated clean request constants into diagnostics-local test support.
   Keep the roughly 15-line CLI/diagnostics decoder overlap as documented
   intentional duplication; do not add a test-support crate or reverse dev
   dependency for it.
4. Add a wire fixture with at least two ordered diagnostics and assert serialized
   order. Single-diagnostic determinism is retained but is not ordering proof.
5. Add CLI and diagnostics fixtures for `modules.unknown_pack_manifest`,
   `binder.unresolved_symbol`, `types.bad_overload`, and
   `logic.unbounded_witness.zero_path_length_bound` while deleting weaker
   redundant front-door rows.
6. Consume ticket 07's ordered `SemanticCostV2` schema accessor in the CLI
   renderer; no second hardcoded five-field list remains.
7. Rename stale `lsp` comments/test identifiers and normalize touched lint
   headers only after the transport rewrite.

The receipt records source and destination for every tracked move plus every
deleted object-concatenation test function.

## Exact Byte-Stream Cases

1. malformed line + newline + valid object line in one write
2. malformed object line + valid object line in one write
3. invalid UTF-8 line + valid object line
4. one byte per read
5. multiple complete objects plus incomplete tail at EOF
6. frame exactly at limit and one byte over limit
7. oversized frame followed by a valid bounded frame

Every case specifies response count, stable code, exit status, and whether the
following valid request is processed.

## Exact Verification

```bash
cargo test -p quanta-gqlang-v2-cli
cargo test -p quanta-gqlang-v2-diagnostics-stdio
cargo test -p quanta-gqlang-v2-pipeline --test v3_frontend_budget
cargo clippy -p quanta-gqlang-v2-cli -p quanta-gqlang-v2-diagnostics-stdio --all-targets -- -D warnings
python3 tools/architecture/check_architecture.py
test -z "$(rg -l 'compile_(program_structural|program_with_schema|with_context|item_structural|item_with_schema|item_with_context|document_structural|document_with_schema|document_with_context)' crates README.md docs --glob '*.rs' --glob '*.md')"
git diff --check
```

## Forbidden Shortcuts

1. retaining CLI multi-root and service single-root paths
2. enforcing limits only at CLI while library/service remain unbounded
3. clearing the entire buffer after one malformed prefix
4. calling flat JSON an LSP
5. retaining any LSP naming or claims on the renamed flat-JSON service
6. leaving a frozen V1 compile-seam definition, export, call, test description,
   or doc comment listed by the inventory

## RED

1. CLI accepts multi-root while service reports trailing tokens
2. incomplete input can grow frame buffer without bound
3. malformed prefix clears a valid suffix request
4. CLI reads an unbounded source file
5. recursive catalog/module/logic traversal lacks a common budget

## Tests

1. identical CLI/service multi-root success and failure
2. malformed prefix plus valid suffix in one read
3. invalid UTF-8 frame plus valid frame
4. byte-fragmented delivery
5. oversized frame/source/token/item/width/graph cases
6. exact budget diagnostic code and span
7. no request after the offending frame is lost
8. unresolved external relation yields CLI exit 1 and service
   `analysis_status=incomplete` with identical diagnostic code/span
9. no CLI/service output labels an analysis result as compiled

## Definition Of Done

1. one facade is the only frontend entry point
2. transport identity matches actual protocol
3. all untrusted-input growth is bounded
4. recovery policy is deterministic and tested across chunk boundaries
5. the stacked 02-08 cut contains no V1 compile seam and the whole workspace
   compiles before integration

## Dependencies

Requires 07. Blocks 09.
