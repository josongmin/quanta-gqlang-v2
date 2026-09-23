# 08 — Front Doors, Transport, And Resource Bounds

> 2026-09-23 embedding-host API supersession: historical `compile_source_document_v2`/`CompiledDocumentV2` references below are not the current execution facade. The source-bound execution entry is `compile_source_document_v3` returning `CompiledSourceDocumentV3`; analyze/format retain their V2 names. See [Ticket 10a](./10a-v3-frontend-execution-contract.md). This API change does not itself change CLI/stdio protocol ownership or prove a front-door qualification receipt.

Status: `CODE INTEGRATED; EXACT-SHA REPO-LOCAL QUALIFICATION AT 08e5ce5 (RECEIPT 09)`

This header supersedes the older blocked/unbounded-V1 checkpoint. The CLI and
private diagnostics NDJSON front doors are bounded; clean `08e5ce5`
qualification passed frontdoor parity 8/8 and owner tests (Receipt 09). This
stdio protocol is not JSON-RPC/LSP.

Decision authority: [00 — Packet Scope And Pinned Decisions](./00-packet-scope-and-pinned-decisions.md)

## Findings Owned / To Close

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
10. every active path in the regenerated candidate-scoped legacy inventory;
    the frozen `f9ec954` `LEGACY_API_INVENTORY.txt` is historical input only
11. `/Cargo.toml` and `/Cargo.lock` — integrator only if dependency edges change
12. `/README.md` and `/docs/language/README.md`
13. the exact CLI/service/pipeline tests listed below
14. `docs/plans/sep-21-gqlang-sota-frontend-reconstruction/receipts/08.md`
15. `crates/quanta-gqlang-v2-cli/src/v3_plan.rs`
16. `crates/quanta-gqlang-v2-cli/tests/command_doc_gate.rs`
17. `crates/quanta-gqlang-v2-diagnostics-stdio/tests/method_allowlist_doc_gate.rs`
18. CLI and diagnostics owner-local test support modules
19. `tools/test_frontdoor_parity.py` — dependency-neutral two-binary
    build/discovery/execution harness, replacing the CLI-owned LSP parity test

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
{"id":1,"method":"v3_diagnostics","source_id":7,"source":"where 1"}
```

`id` is required and is either a nonnegative JSON integer within `u64` or a
nonempty string of at most 128 UTF-8 bytes. `method` has exactly one accepted
value. `source_id` is a required JSON integer in `0..=u32::MAX`; `source` is
required. Unknown fields reject; duplicate JSON object keys reject rather than
last-write-wins. The transport passes the exact source ID through the facade and
never derives or renumbers it.

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
`transport.invalid_source_id`,
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
accepted. Keep one pending trailing CR byte outside the payload counter: if
the next byte is LF, drop that CR as framing; otherwise commit it as payload
before processing the next byte. Only when the committed payload count reaches
`limit + 1` switch to discard-through-LF without retaining later payload
bytes. At EOF, a pending CR is payload of an incomplete frame, not a free
terminator. Test exact-limit+CRLF (accept), exact-limit+CR+non-LF (oversize),
fragmented CR/LF at the cap, and limit+1 without CR (discard/recover). Source
length is checked independently
after JSON decode and reports `frontend.resource_limit_exceeded` with dimension
`max_source_bytes`.

## Shared Front Door

1. CLI `query` and diagnostics stdio call
   `analyze_source_document_v2(source_id, source,
   OpenAnalysisAuthorityV2::try_standard_v2()?)`. The CLI is a single-document
   standard-authority caller and uses explicit source ID `0`; stdio uses the
   request's required `source_id`.
2. CLI `fmt` calls only facade `format_source_document_v2` with the standard
   frontend budget and explicit source ID `0`, which projects to the owner-local
   parse budget; it does not analyze or compile.
3. `compile_source_document_v3` is the source-bound library seam for an embedding
   host that supplies `ClosedFrontendAuthorityV2`; it returns
   `CompiledSourceDocumentV3`. CLI/service do not manufacture closed authority.
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

Consume and verify ticket 04's one `FrontendBudgetV2`; ticket 08 does not
create a second budget type or add missing owner charge points. The canonical
configuration covers:

1. source bytes
2. frame bytes
3. token count
4. document item count
5. expression depth and width
6. list/record/projection width
7. module graph nodes/edges and rule graph nodes/edges as separate limits
8. witness depth/cap
9. semantic, module, and rule traversal steps as separate limits
10. per-admission-session authority input bytes and per-authority-string bytes

Budget exhaustion returns typed `resource_limit_exceeded`; it does not panic,
truncate silently, or clear unrelated buffered requests.

## Budget Ownership And Projection

1. `quanta-gqlang-v2-sema::FrontendBudgetV2` is the sole public configuration
   created and validated by ticket 04 authority constructors. Ticket 08 only
   consumes it and verifies identical projection at each front door.
2. Pipeline reads that value and uses the private owner-local views created by
   tickets 02-06. Owner-local types are not public knobs.
3. The diagnostics stdio service applies `max_frame_bytes` before deserialization
   and passes the same authority budget to document analysis.
4. CLI applies `max_source_bytes` while reading and passes the same open
   authority budget to document analysis. Both `--file` and inline argv are
   ingress paths: check a regular file's metadata when available, but read
   every file/pipe through a bounded `limit + 1` stream before constructing
   a `String`; incrementally sum each inline argument's UTF-8 byte length
   plus join spaces with checked arithmetic before cloning/joining. Reject
   over-limit input without echoing its bytes in an error. A precheck only
   on the file path leaves inline argv and non-regular files unbounded.
5. Every cumulative field is charged once by the canonical owner table in
   `TARGET_ARCHITECTURE.md`. Pipeline passes identical immutable configuration
   and creates one counter per owner; it does not pass remaining counters across
   stages. CLI/file ingress repeats an absolute byte-cap check before a second
   frontend allocation. Service ingress bounds the raw frame first and enforces
   decoded source during a bounded visitor or immediately after bounded decode,
   as pinned below.

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
6. Construct one `OpenAnalysisAuthorityV2::try_standard_v2()` per CLI/service
   request path, propagate construction failure, and use its same
   `FrontendBudgetV2` for ingress and analysis.
   Embedding hosts construct closed authority only through the library seam.
7. Verify the owner charge points from tickets 02-06; add only CLI source-read
   and service frame-accumulation prechecks in this ticket.
8. Implement typed budget diagnostics without truncating later frames.
9. Replace whole-buffer clear recovery with bounded prefix/frame consumption.
10. Regenerate the inventory on the exact candidate scope, then migrate every
    code, test, and current-doc entry to V2 or delete obsolete V1-only coverage.
11. Delete every V1 compile-seam definition/re-export recorded in
    the candidate-scoped inventory, plus
    `crates/quanta-gqlang-v2-pipeline/tests/v3_legacy_api_ratchet.rs` and
    its current inventory after the final zero-match scan passes. Retain the
    `f9ec954` snapshot only as historical receipt evidence.
12. Mark tickets 02-08 `INTEGRATED` only after the whole workspace compiles with
    no V1 seam.

Prove the old identity did not return:

```bash
test -z "$(rg -l 'quanta[-_]gqlang[-_]v2[-_]lsp' \
  Cargo.toml Cargo.lock crates README.md docs/language)"
rg -n 'UTF-8 NDJSON, not JSON-RPC/LSP' README.md docs/language
test -z "$(rg -l '\b(Lsp|lsp)\b|Lsp[A-Z]|_[Ll][Ss][Pp]_' \
  crates --glob '*.rs' --glob 'Cargo.toml')"
```

The first command must find no old identity; the second must find the honest
transport description. Historical audit evidence under this Sep-21 packet may
retain the old name because it describes the audited base.
The third command covers public and test symbols such as live
`LspServerV1`, `StdioLspErrorV1`, and `v3_cli_lsp_parity.rs`; renaming only
the package string is insufficient. Keep any honest “not JSON-RPC/LSP” prose
outside the active `crates` scan and document an exact allowlist if a future
source comment must retain it.

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
max_authority_bytes
max_authority_string_bytes
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
2. `[MOVE+REWRITE]` `crates/quanta-gqlang-v2-cli/tests/v3_cli_lsp_parity.rs` to `tools/test_frontdoor_parity.py`; this black-box harness owns parity without a CLI-to-service or conformance-to-binaries Cargo edge
3. `[RENAME+REWRITE CURRENT]` `crates/quanta-gqlang-v2-diagnostics-stdio/tests/stdio_frame_extractor_edges.rs` to `crates/quanta-gqlang-v2-diagnostics-stdio/tests/stdio_resynchronization.rs`
4. `[ADD]` `crates/quanta-gqlang-v2-diagnostics-stdio/tests/stdio_resource_limits.rs`
5. `[RENAME+REWRITE CURRENT]` `crates/quanta-gqlang-v2-diagnostics-stdio/tests/v3_diagnostics_parity_hellgate.rs` to `crates/quanta-gqlang-v2-diagnostics-stdio/tests/document_diagnostics_parity.rs`
6. `[UPDATE AFTER CRATE MOVE]` remaining diagnostics byte/JSON/frame/determinism/allowlist tests in place
7. `[ADD]` `crates/quanta-gqlang-v2-pipeline/tests/v3_frontend_budget.rs`
8. `[UPDATE]` `crates/quanta-gqlang-v2-cli/tests/v3_cli_plan_summary.rs`
   and `v3_cli_item_surfacing.rs` — consume Ticket 07's ordered four-field
   `SemanticCostV2` accessor; remove the dead V1
   `cost.unsupported_reason_code=none` success line while retaining typed
   unsupported reason on error output

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
python3 tools/test_frontdoor_parity.py
cargo test -p quanta-gqlang-v2-pipeline --test v3_frontend_budget
cargo clippy -p quanta-gqlang-v2-cli -p quanta-gqlang-v2-diagnostics-stdio --all-targets -- -D warnings
python3 tools/architecture/check_architecture.py
test -z "$({ rg -l 'compile_(program_structural|program_with_schema|with_context|item_structural|item_with_schema|item_with_context|document_structural|document_with_schema|document_with_context)' crates --glob '*.rs'; rg -l 'compile_(program_structural|program_with_schema|with_context|item_structural|item_with_schema|item_with_context|document_structural|document_with_schema|document_with_context)' README.md docs/language --glob '*.md'; } | sort -u)"
git diff --check
```

The legacy zero-match scope is active Rust source/tests plus current root and
language docs. It intentionally excludes `docs/plans/**`; ticket 09 preserves
and classifies historical plan mentions instead of falsifying old evidence.

## Forbidden Shortcuts

1. retaining CLI multi-root and service single-root paths
2. enforcing limits only at CLI while library/service remain unbounded
3. clearing the entire buffer after one malformed prefix
4. calling NDJSON an LSP or JSON-RPC
5. retaining any LSP naming or stale flat-JSON claims on the NDJSON service
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

## 2026-09-21 Final Adversarial Audit Corrections

This section is normative and supersedes conflicting earlier wording.

### Current paths and protocol truth

The current files are:

- `crates/quanta-gqlang-v2-diagnostics-stdio/tests/stdio_frame_extractor_edges.rs`
- `crates/quanta-gqlang-v2-diagnostics-stdio/tests/v3_diagnostics_parity_hellgate.rs`

Treat them as `[RENAME+REWRITE CURRENT]`; no deleted `quanta-gqlang-v2-lsp`
path is a write target. Update root README, language README, and
`method_allowlist_doc_gate.rs` to the exact phrase `UTF-8 NDJSON, not
JSON-RPC/LSP`; remove the stale `private flat-JSON protocol` oracle.

Every response/diagnostic has an exact schema with unknown fields denied.
User-derived identifier/method excerpts are truncated on UTF-8 boundaries to
128 bytes before formatting; final messages are capped at 1,024 bytes. Unknown
method/field errors must not echo an unbounded frame.

### Front-door authority and errors

Both executable front doors call `OpenAnalysisAuthorityV2::try_standard_v2()`
or receive a checked open authority and propagate construction failure. The
library compile entry accepts the facade builder's closed authority. Wire
`analysis_status=error` is produced only from the public `Err` branch; internal
successful analysis states map exhaustively to complete/blocked/not-applicable.
Whole-document wire mapping is exact: `error` only for the public `Err` branch;
`incomplete` iff at least one applicable item is `BlockedByDiagnostics`; and
`complete` otherwise, including a valid document whose items are all
`NotApplicable`.

The authority byte cap is per sealed admission session: one module-authority
session, one relation-authority admission, and one per-document catalog
session, each independently bounded by the projected value. It is not a sum
across sessions. Document this maximum resident-input model and test every
domain at exact limit and one over; profile equality alone is not described as
a total-byte proof.

### Parity and final dependency edge

Preserve CLI/service parity using `tools/test_frontdoor_parity.py`, a
dependency-neutral black-box harness. It builds both exact named bins with
`cargo build --message-format=json`, selects the two `compiler-artifact`
records whose target kind is `bin` and package/target names match, rejects a
missing or duplicate executable path, then invokes both with the same bounded
fixtures and compares canonical stage/code/source byte spans and analysis
status. It reports selected/executed fixture counts and nonzero exit on drift.
Do not rely on `CARGO_BIN_EXE_*` for binaries owned by another package: Cargo
does not supply those to conformance tests. Do not retain
`CLI -> diagnostics-stdio` even as a dev dependency or add conformance dev
edges to the binaries. The receipt records the old test deletion and new
script path; `just ci` and Ticket 09's final recipe invoke this script, while
workspace Cargo tests remain independently runnable.

Migrate old compiled-item field consumers in CLI `v3_query.rs` and
`v3_format.rs` to facade accessors. Record zero matches for the removed fields
and V1 compile/analyze names.

Requires 07. Blocks 09.

## 2026-09-21 Live-Tree Final Reconciliation (HEAD `f9ec954`)

The current diagnostics crate is already named `diagnostics-stdio`; do not
repeat Ticket 01's rename. Its live manifest still depends on pipeline, and
the CLI still has a `v3_cli_lsp_parity.rs` test. Preserve the byte-fragment,
UTF-8, deterministic-output, and process assertions; replace the framing and
request parser once, then migrate the parity harness to conformance. The
direct `CLI -> diagnostics-stdio` edge, if present during migration, is
transitional only and must be absent in final architecture mode.

The NDJSON decoder must first bound bytes through LF, then decode UTF-8, then
parse a JSON object with duplicate-key rejection *during* deserialization.
Parsing into a last-write-wins generic object and checking duplicates later
cannot prove the wire contract. Assert the decoded `source` byte length
separately; escaped JSON may be shorter or longer than the source it encodes.
An over-limit line must discard only through its LF and leave following
complete lines intact. Test the exact-limit payload, `limit+1` discard,
invalid UTF-8/JSON/duplicate key, incomplete EOF tail, and a subsequent valid
frame under one-byte and coalesced writes. Include exact-limit+CRLF,
exact-limit+CR+non-LF, and split CR/LF boundary cases so CR lookbehind cannot
silently exceed the payload cap. For CLI, test exact/+1 inline joined-argv
bytes, exact/+1 regular file, and a non-regular input stream whose metadata
cannot prove its eventual length; no path may allocate the full source before
rejecting.

Only the facade document APIs are allowed on executable paths. The receipt
must show the direct pipeline dependency removed from diagnostics, direct
internal frontend dependencies removed from CLI, the cost-schema accessor
used by CLI, every tracked test move, and the frozen 244-line V1 seam
inventory reduced to zero active definitions/usages before deleting that
temporary inventory.

## Final Execution Audit — `main@ecc38f59` + Dirty Overlay

Verdict: `PLANNED`, blocked by 07. The crate rename already happened; current
`LspServerV1`/`StdioLspErrorV1` names, direct Pipeline dependency, custom flat
JSON parser, and CLI parity dev edge are active V1 seams, not a reason to add a
second transport.

Single implementation path:

1. Rename in place to transport-neutral server/error types and expose exact
   UTF-8 NDJSON request/response structs with unknown fields denied. Do not add
   JSON-RPC envelopes or `initialize`.
2. Implement one streaming decoder: bound bytes through LF, handle one-byte CR
   lookbehind, decode UTF-8, reject duplicate JSON keys during deserialization,
   then validate decoded source bytes. Over-limit frames discard only through
   their LF and recovery continues.
3. Enforce the frame limit before decode/allocation growth; enforce decoded
   source during a bounded visitor or immediately after the already bounded
   frame decode; enforce diagnostic count/response bytes before accumulation or
   serialization. CLI joined argv, regular files, stdin/non-regular streams,
   and stdio frames each need exact-limit/+1 tests.
4. Route CLI and diagnostics only through the Ticket 07 facade and standard
   open authority. Remove direct HIR/Surface/Types/Planning/Pipeline imports
   from executable paths and remove the CLI dev edge to diagnostics-stdio.
5. Move parity to `tools/test_frontdoor_parity.py`. Discover exact binary
   artifacts from Cargo JSON, run both as black boxes, and compare canonical
   stage/code/source-byte-span/status. Delete the old cross-package parity test
   only after fixture equivalence is recorded.
6. Update README and doc-gate wording to `UTF-8 NDJSON, not JSON-RPC/LSP`, then
   run the previously blocked `method_allowlist_doc_gate`.

Acceptance includes split/coalesced frames, CRLF boundary cases, invalid UTF-8,
duplicate keys, incomplete EOF, over-limit recovery, bounded error excerpts,
CLI/stdio parity, strict Clippy, and workspace compile. The frozen V1 seam
inventory must reach zero before it is deleted.

## Final Independent RR — Wire, CLI, And Inventory Corrections

### Refresh the migration authority first

`LEGACY_API_INVENTORY.txt` is historical (`f9ec954`, 244 lines), not an
authority for `ecc38f59` plus the dirty overlay. Regenerate a candidate-scoped
inventory from active source/tests/docs, retain the old snapshot as historical
or add base/current columns, hash it, and require final set-difference zero.
Do not delete an unlisted live consumer merely because the stale list omitted
it.

### Allocation and amplification bounds

- `max_frame_bytes` is the pre-decode boundary. Ordinary `String`
  deserialization may allocate up to that cap, then rejects decoded source over
  `max_source_bytes`. If source must be bounded during construction, use a
  custom bounded Serde visitor; do not claim this with derive decoding.
- Pin whether JSON escaping/envelope may make a CLI-admissible source fail the
  service frame limit, or validate a conservative worst-case encoded size plus
  envelope. `frame >= source` alone is not parity.
- Add canonical `max_diagnostics` and `max_response_bytes` (or a proven
  streaming serializer with a derived hard bound) to `FrontendBudgetV2` and
  authority-profile equality. Charge before push/serialization and emit one
  bounded resource failure without echoing discarded payload. These are shared
  frontend caps, not hidden transport defaults.
- List exact maintained JSON dependencies and the duplicate-key visitor in
  `diagnostics-stdio/Cargo.toml`.
- Rewrite CLI inline argv handling to append into one checked bounded buffer,
  charging separators and stopping at limit+1. Cap lossy invalid-UTF8 and path
  excerpts on UTF-8 boundaries. Test many empty/one-byte args,
  separator-only overflow, huge invalid OS arg, and huge file-path errors.

### Total request/response schema

Validation precedence is frame/UTF-8; duplicate keys before trusting ID;
top-level object; unknown fields; required fields; field types/ranges; method;
decoded source size; facade call. Define stable codes and ID echo/null behavior
for every branch. Cover `{}`, array/scalar roots, duplicate ID, each missing
field, every wrong field type, negative/fractional/overflow integer IDs,
empty/oversize string IDs, unknown+valid request, invalid method, and invalid
source value. `malformed_json` is syntax-only.

Wire diagnostics are discriminated:

- transport diagnostics carry raw frame/line byte range and no fabricated
  source identity;
- frontend diagnostics carry stage, code, severity, bounded message, primary
  `{source_id,start,end}`, and ordered secondary locations;
- canonical field order and omission versus `null` are pinned;
- multi-source and CLI projection parity are fixtures.

Raw-payload span rules: empty `0..0`; UTF-8 offending bytes; parser token/key/
value range; missing field at end-of-object; incomplete tail `0..tail_len`;
duplicate at the second key; validation error at the field value. Multibyte and
escape tests prove transport ranges use raw bytes while frontend spans use
decoded source bytes.

### Session status

Process exit is 1 iff any transport/request/I/O failure occurred. Semantic
incomplete/error for a well-formed request is response data and does not set
process failure. Bad+good frames continue processing and exit 1; a new process
starts with a clear latch. Add mixed-frame latch/reset mutations.

Receipt 08 records refreshed inventory hash, dependency versions, exact
byte/request/response/budget case counts, selected/executed tests, and
CLI/service parity.
## 2026-09-21 Final Current-Tree Audit

Status remains `PLANNED`. No Ticket 08 receipt exists. Current front doors still
depend on the compiler-RED Pipeline, so their local tests cannot establish
semantic parity.

Required implementation sequence:

1. Wait for the Ticket 07 explicit facade; CLI and diagnostics-stdio import
   only that facade.
2. Keep the service named diagnostics-stdio. Do not reintroduce LSP naming or
   methods unless real JSON-RPC initialization, framing, lifecycle, and UTF-16
   coordinates are implemented as a separately accepted scope change.
3. Implement the pinned bounded NDJSON request/response schema, stable status
   mapping, one-response-per-frame recovery, EOF/truncation behavior, and
   canonical diagnostics ordering.
4. Charge source bytes, frame bytes, item/expression/semantic steps,
   diagnostics count, response bytes, and recursive traversal before growth.
   Exact-limit and +1 tests are required at owner and front-door layers.
5. Run the same semantic corpus through facade, CLI, and diagnostics-stdio and
   compare canonical results, diagnostics, and exit/status codes.
6. Remove legacy binaries/modules/docs and their final-DAG edges only after
   parity is green; record receipt 08 with bounded malformed-input tests.

No runtime substrate, provider registry, network server, or SDK wiring belongs
in this ticket.

## SS-08A Current-Tree Frontdoor Audit — 2026-09-21

Status remains `PLANNED`; no Receipt 08 exists. Current executable paths are
the prohibited V1 path: CLI directly parses/compiles with Pipeline, and
diagnostics-stdio calls `compile_program_structural`. The frontend-contract is
also a V1 re-export surface, so no final facade exists for either consumer.

The transport is not an incomplete NDJSON implementation; it is a different,
unbounded protocol and must be replaced in place:

- buffered reads append to an unbounded `Vec`;
- framing is brace balancing rather than CRLF-aware NDJSON;
- malformed frames clear the whole buffer and can lose a valid suffix;
- JSON is handcrafted and flat-string-only;
- wire request/response lack request ID, source identity/text, analysis status,
  and typed source/secondary ranges.

Current tests pin glued objects and whole-buffer-clear recovery, so they are
negative migration inputs, not reusable acceptance coverage. The one-byte
reader, invalid UTF-8 recovery technique, and process-status harness are the
only reusable test assets.

`FrontendBudgetV2` has no diagnostic-count or response-byte caps. Add those
canonical caps and authority-profile equality before allocating transport
buffers or serializing output. The required sequence is: Ticket 07 facade and
Pipeline repair; budget extension; one bounded NDJSON state machine and typed
schema; CLI bounded ingress/facade migration; docs/tests/parity; then
candidate-scoped inventory and architecture proof. No LSP/JSON-RPC claim or
runtime/network substrate is authorized.

## RR-08B Authoritative Current-State Reconciliation — 2026-09-21

Frozen observation: `main@ecc38f59e9fdcd62b5e030445188ffa8b01e100a`
plus the 178-entry dirty overlay. Receipt 08 created with this audit is
**audit-only**; it is not an implementation or qualification receipt.

### Actual current front doors

| Path | Current implementation | Required disposition |
|---|---|---|
| CLI query | `commands/v3_query.rs::{parse_items_v1,run}` directly uses HIR + Surface + Pipeline V1 | replace with facade `analyze_source_document_v2` |
| CLI fmt | `commands/v3_format.rs::format_source_v1` directly uses HIR + Surface | replace with facade `format_source_document_v2` |
| CLI file ingress | `main.rs::read_input_file_v1` calls unbounded `fs::read_to_string` | bounded `limit + 1` streaming read |
| CLI argv ingress | `main.rs::inline_input_v1` clones every argument, echoes invalid bytes lossily, then unbounded `join` | checked byte sum before allocation; bounded/non-echoing error |
| stdio accumulation | `lib.rs::run_stdio_session_v1` appends every read to an unbounded `Vec` | bounded line state machine |
| stdio framing | `server.rs::extract_next_request_frame_v1` balances braces and accepts glued objects | replace with CRLF-aware NDJSON |
| recovery | `drain_request_frames_v1` clears the whole buffer on extractor error | discard only offending line/prefix; preserve suffix |
| JSON | `json.rs::JsonCursorV1` is a handwritten flat string-only parser | maintained JSON parser plus duplicate-key visitor |
| request schema | `{method, query}` strings only | required `id`, `method`, `source_id`, `source`; reject unknown/duplicate fields |
| response schema | `{ok,method,result}` with no request ID/status | pinned ID + `ok` + `analysis_status` + ordered diagnostics |
| naming | `LspServerV1`, `StdioLspErrorV1`, test file/symbol/comment LSP names remain | remove active LSP identity in the cut |
| dependencies | CLI has five internal normal edges; stdio depends directly on Pipeline | both normal-depend only on frontend-contract |

The root and language READMEs now honestly say the protocol is not JSON-RPC or
LSP, but `docs/language/release_discipline.md:18` still calls it LSP and active
Rust/test identifiers retain LSP terminology. Documentation is therefore only
partially corrected.

`FrontendBudgetV2` currently has 19 fields, including source/frame bytes and
owner traversal limits. It has no `max_diagnostics` or `max_response_bytes`.
Those two caps must be added by the Sema budget owner (Ticket 04 amendment) and
then consumed here; Ticket 08 must not define a parallel transport budget.

### Exact implementation worklist

1. **Freeze Ticket 07 facade first.** CLI and service consume the same
   `AnalysisDocumentV2` and canonical diagnostic DTO/order. Do not temporarily
   depend on Sema, Algebra, Planning, or repaired Pipeline internals.
2. **Amend canonical resource authority.** Add positive, checked
   `max_diagnostics` and `max_response_bytes` to `FrontendBudgetV2`, standard
   profiles, equality/profile conversion, and exact-limit/+1 owner tests. Every
   emitted diagnostic and response byte is charged before growth. Under the
   canonical wave schedule this is W1's SS-04W producer obligation. W5 must
   reopen W1 if it is missing; the Ticket 08 worker must not patch Sema.
3. **Bound CLI ingress in `main.rs`.** Replace `read_input_file_v1` with a
   `Read::take(limit + 1)`-style checked stream for regular files, pipes, and
   special files; metadata is only a precheck. Replace `inline_input_v1` with
   checked UTF-8 byte accumulation including join spaces before any clone/join.
   Errors expose dimension/limit, never source bytes.
4. **Replace transport state.** Replace `run_stdio_session_v1`,
   `drain_request_frames_v1`, and `extract_next_request_frame_v1` with one
   byte-oriented NDJSON state that tracks payload length, pending CR,
   discard-through-LF, request-local failure, and process failure latch. Never
   retain bytes after `limit + 1`.
5. **Replace request parsing.** Remove `JsonCursorV1` and the handwritten JSON
   emitters. Parse exactly the pinned object with duplicate-key detection,
   integer range checks, bounded string ID, unknown-field reject, required
   source ID, and exact source text. Malformed input has `id:null`; parsed IDs
   may be echoed on other validation failures.
6. **Unify semantic status.** Query and stdio call the facade with one standard
   open authority. CLI exit 1 on incomplete/error analysis. A valid stdio frame
   remains `ok:true` even when analysis is incomplete/error; transport/request
   errors set the process latch while later frames continue.
7. **Migrate tests in place.** Rewrite the nine current stdio suites into the
   pinned NDJSON/request/recovery/budget suites. Rename
   `v3_cli_lsp_parity.rs` and all active LSP symbols/comments. Glued-object and
   whole-buffer-clear expectations become negative mutations, not preserved
   compatibility behavior.
8. **Truth and dependency cleanup.** Update README/language/release docs from
   the final schema, generate method/command gates from source, remove direct
   Pipeline/internal Cargo edges, and implement
   `tools/test_frontdoor_parity.py`. Wire it into `just ci` and hosted CI.

### Required byte, request, and mutation proof

In addition to the existing exact byte-stream table, the owner suite must cover
exact limit and `+1` for payload/source/diagnostics/response; CRLF split at
every byte; pending CR at EOF; blank line; invalid UTF-8; duplicate/unknown
field; numeric/string ID bounds; source ID bounds; malformed+valid and
oversize+valid suffix recovery; one-byte reader; short writer; I/O failure;
mixed failure latch and fresh-process reset.

```bash
cargo test -p quanta-gqlang-v2-cli --all-targets
cargo test -p quanta-gqlang-v2-diagnostics-stdio --all-targets
cargo test -p quanta-gqlang-v2-conformance --test v3_frontdoor_parity
cargo clippy -p quanta-gqlang-v2-cli -p quanta-gqlang-v2-diagnostics-stdio --all-targets -- -D warnings
python3 tools/test_frontdoor_parity.py
test -z "$(rg -l '\b(Lsp|lsp)\b|Lsp[A-Z]|_[Ll][Ss][Pp]_' crates --glob '*.rs' --glob 'Cargo.toml')"
```

Kill mutations that remove each `+1` check, count LF/CR as payload
incorrectly, clear the complete suffix, reset the process latch per frame,
last-write-win a duplicate key, renumber `source_id`, echo malformed payload,
convert fully resolved open analysis into compiled output, or let CLI and stdio
sort diagnostics differently.

### Parallelization constraint

After Ticket 07 freezes facade DTOs, the bounded CLI ingress/adapter and NDJSON
transport can be implemented in parallel in disjoint crates. The canonical
budget extension is an upstream owner change and must land first. Shared parity
fixtures, Cargo-edge deletion, docs, inventory, and receipt generation are one
serial integration step. Ticket 09 cannot generate final truth until that step
is complete.
