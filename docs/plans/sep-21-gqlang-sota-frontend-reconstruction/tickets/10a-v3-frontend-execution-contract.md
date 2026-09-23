# Ticket 10a — V3 frontend execution contract producer

Status: `V3 PRODUCER IMPLEMENTED AND LOCALLY QUALIFIED; SEMANTICA CONSUMER OPEN`

Inventory baseline: qglang `main@1a0761b007751de4f1811081d246326ccc4ce017`, clean worktree, 2026-09-23. This SHA is an audit input, **not** an implementation or qualification receipt. Recheck HEAD, status, and the inventories below before editing.

## Objective and ownership

Replace the public V2 execution DTO with one source-bound V3 semantic contract produced by the existing checked Pipeline transaction. Do not create a second parser, type checker, algebra lowerer, serving authority, or compatibility write path. This repo owns the frontend projection, canonical frontend digests, facade, and producer proof only. Semantica owns selected-item admission, generation/source closure, provider issuer, lease, runtime budget, physical plan, execution, and receipt.

The `V2` suffix on existing HIR/Sema/Types and analysis owners does not imply those owners must be copied or renamed. The breaking cutover concerns the public **execution** contract. Keep `analyze_source_document_v2` and `format_source_document_v2` unless a concrete API dependency requires a change.

## Implemented producer decisions

- [x] Published the normative `docs/contracts/frontend-execution-v3.md`: closed public DTO, ordering, supported/unsupported boundary, transaction-local IDs, and versioned canonical bytes for all three digests. This ticket does not duplicate the byte specification.
- [x] Split checked resolved compilation from source-bound execution. `compile_resolved_document_v2` returns `CompiledDocumentV2` without `execution()`; only `compile_source_document_v3` returns `CompiledSourceDocumentV3` and seals `ExecutionDocumentV3` with the actual source bytes. Both paths reuse one checked `run_document`/assembler.
- [x] Source digest covers exact UTF-8 bytes. Source-qualified IDs deliberately affect contract and qualification; neither is a cross-transaction semantic-equivalence key. The named vectors prove same bytes/different ID and whitespace-only byte mutation.
- [x] Projected typed literals directly from the checked type owner: signed `i128` including deferred minimum, normalized finite `f64`, bool, UTF-8 string, null. The V3 encoder has fixed tags and byte widths; no literal reparse or second numeric validator.
- [x] Kept family/policy distinctions without silently dropping checked plan-required items. Graph, CHECK, presentation, meta, and rule-program execution remain explicitly outside the complete V3 payload; Semantica must reject or separately specify them before effects. No runtime/provider/generation fields were added here.

## Current proof and remaining handoff

- Producer-local `just ci` passed on the working tree on 2026-09-23: 1,090/1,090 nextest tests, doc tests, frontdoor parity, architecture lint, external facade consumer (1 positive, 15/15 negative compile-fail, 89/89 public roots), cargo deny, and non-vacuous semgrep. The first run's five surface doc failures were caused by a stale binary compiled against an old temporary checkout; `cargo clean -p quanta-gqlang-v2-surface` and a focused rerun proved the source tests pass, then the entire `just ci` passed. No surface source or test was altered for this cache incident.
- The 14-row shared TSV corpus covers source-ID, byte-only, schema, source-authority, composition-limit, and typed-literal cases. Pipeline owner tests additionally cover deterministic projection, binding/body mutations, foreign item lookup, and the resolved/source result split. The external privacy gate includes the V3 document-constructor negative.
- **Producer pin closure:** the original V3 implementation commit `a5c2f8c5c5c50f2d30e400facdc0737e2f59a369` passed clean detached-worktree `just qualify-local` with tree `5d3e215a6d6011af74ef06a3f3043b5664478ecd`, `exit=0`, duration 329 seconds. The later current code candidate `3c8acbe21a1ce48022c12642881d1fafc190ddb1` / tree `173b19e1f5b3aabd23fa22a6d09e509c723316dd` separately passed clean `just qualify-local` (`just ci`, exit 0, 1,102/1,102 nextest, 257 seconds, pinned Rust 1.81.0). Its ignored receipt is `target/qualification/3c8acbe21a1ce48022c12642881d1fafc190ddb1/receipt.json`. Later source, manifest, or tool changes need a new exact-SHA run; a docs/receipt-only attestation commit may instead record `tested_sha=3c8acbe` and prove its diff contains no executable change.
- **Still required outside this repo:** Semantica approval and consumer cutover against the exact qglang SHA, with its own SHA/dependency pin and source-text-to-result/receipt proof. No Semantica build, admission, provider, generation, or product execution is claimed here.

## Exhaustive current-source file inventory

`EDIT` is the expected producer write set. `VERIFY` is a regression or scan target, not permission to modify it without a demonstrated defect. Paths are relative to repository root.

| Class | Path | Exact work / reason |
| --- | --- | --- |
| NEW | `docs/contracts/frontend-execution-v3.md` | Normative V3 schema, digest encoding and domain separators, order, identity scope, family/unsupported matrix, golden-vector interpretation. |
| EDIT | `crates/quanta-gqlang-v2-pipeline/src/execution.rs` | Replace all public `Execution*V2` roots with the single V3 execution closure; add closed typed literal; remove string-literal encoding and `new()` + `bind_source_bytes()` empty-source intermediate; version all three digest domains/encoder together. Preserve existing checked projections and private constructors. |
| EDIT | `crates/quanta-gqlang-v2-pipeline/src/compile.rs` | Separate resolved unqualified result from source-bound V3 result; call existing checked stages exactly once; source entry binds its actual bytes before exposing V3; remove `CompiledDocumentV2::execution()` access to an unqualified DTO. Retain exact item/membership diagnostics. |
| EDIT | `crates/quanta-gqlang-v2-pipeline/src/lib.rs` | Curate the V3 source entry and V3 DTO re-exports; retire V2 execution roots without aliasing. Analysis/format exports stay as needed. |
| EDIT | `crates/quanta-gqlang-v2-frontend-contract/src/lib.rs` | Expose only the source-bound V3 execution path and closed DTO roots; no owner-crate glob, public proof constructor, or V1 export. Keep unrelated curated analysis/authority roots. |
| EDIT | `crates/quanta-gqlang-v2-pipeline/tests/v3_compiled_document_assembly.rs` | Migrate source calls/DTO assertions. Replace `resolved_entry_shares_contract_but_not_source_digest`: resolved compilation must have no executable source/qualification digest. Keep composition identity, authority, foreign-item, deterministic and mutation oracles. |
| EDIT | `crates/quanta-gqlang-v2-pipeline/tests/v3_analysis_document_assembly.rs` | Update its direct closed-source calls/result type without changing the open-analysis truth contract. |
| EDIT | `crates/quanta-gqlang-v2-conformance/tests/self_validation.rs` | Update direct facade source-compile calls; maintain existing family tests rather than reimplementing owner digest tests. |
| EDIT | `tools/frontend_external_consumer/src/main.rs` | Facade-only V3 positive: nondefault authority, source compile, typed literal, composition identity/order, three digest projections, two `SourceId` values. |
| EDIT | `tools/test_frontend_external_consumer.py` | Pin the new exact root allowlist and expected privacy diagnostics; keep the manifest/topology and checkout-immutability gate. |
| EDIT | `tools/frontend_external_consumer/negatives/d13_read_execution_composition_bindings.rs` | Point the existing private-field probe at V3 composition; do not create duplicate probes for the same property. |
| EDIT | `tools/frontend_external_consumer/negatives/d14_import_retired_compiled_item_v1.rs` | Keep the V1 rejection, but replace the stale comment directing consumers to V2 execution. |
| NEW or EDIT | `crates/quanta-gqlang-v2-pipeline/tests/fixtures/execution-v3-vectors.tsv` and the existing `execution.rs` unit tests | One shareable named vector corpus plus exact owner assertions: source-ID change, byte change, literal boundaries, binding, authority/schema, foreign item. The fixture format must be consumable by Semantica without a second canonicalization implementation. No new parser/dependency solely for the fixture. |
| VERIFY | `crates/quanta-gqlang-v2-pipeline/tests/v3_composition_source_query.rs` | Sema/composition producer regression. It has no direct V2 execution API reference in the baseline; edit only if the semantic contract actually changes. |
| VERIFY | `crates/quanta-gqlang-v2-types/src/ty.rs`, `crates/quanta-gqlang-v2-types/src/check.rs`, Sema/HIR | Existing typed literal and finite-float authority. No engine change unless a reproducing semantic defect appears. |
| VERIFY | `crates/quanta-gqlang-v2-pipeline/src/diagnostic.rs`, Cargo manifests/lockfile, `justfile`, `tools/qualification/qualify-local.sh`, `.github/workflows/lint.yml` | No expected change or new dependency. Existing local qualification and facade gate should run unchanged. |
| VERIFY | `FINAL_AUDIT.md`, `IMPLEMENTATION_REUSE_AUDIT.md`, `tickets/04-sema-kernel-and-document-transaction.md` | Direct V2 compile/result mentions are historical ownership/audit statements, not current public-entry instructions. Preserve them unless a current-tense claim is demonstrated. |

Documentation synchronization after the API is implemented:

| Class | Path | Exact work |
| --- | --- | --- |
| EDIT | `tickets/10-downstream-serving-interop.md`, `tickets/README.md` | Route new V3 producer work here; label old V2 snapshots/hypotheses as historical. Do not reuse Ticket 10 as an implementation SSOT. |
| EDIT | `tickets/00-packet-scope-and-pinned-decisions.md`, `tickets/07-semantic-planning-and-public-facade.md` | Amend only public execution/facade decisions and the actual new function/result signatures; preserve completed W4 historical proof. |
| EDIT | `tickets/08-frontdoors-transport-and-resource-bounds.md`, `TARGET_ARCHITECTURE.md` | Fix current-tense `compile_source_document_v2` embedding-host claims if source entry is renamed; do not rewrite historical receipts. |
| EDIT | `tickets/09-conformance-truth-sync-and-closeout.md` | Add V3 producer/external-consumer/digest evidence and the two-repo source-pin handoff; local producer GREEN is not consumer or product GREEN. |
| VERIFY | `README.md`, `docs/language/`, June plans | Baseline scan found no direct V2 execution API claim in current public-language docs. Change only a demonstrated stale current-tense claim; retain historical provenance. |

Baseline direct-code reference scan for every `Execution*V2` root, `CompiledDocumentV2`, `compile_source_document_v2`, and `compile_resolved_document_v2` found exactly the two Pipeline source files, Pipeline public lib, facade lib, two Pipeline assembly tests, conformance `self_validation.rs`, external-consumer positive, its Python gate, and the D13/D14 privacy probes. The broader `.execution()`/`bind_source_bytes` scan found no additional code files. Repeat both scans after edits and require no retired public execution root or V1 facade dependency.

## Acceptance vectors and gates

1. Same source bytes + different `SourceId`: equal source digest, different contract/qualification when source-qualified IDs remain in V3 encoding. Same source/ID/authority: all DTO fields and digests equal.
2. Meaning-preserving byte edit: source/qualification differ; do **not** assert contract equality unless the actual canonical DTO is equal. Typed literal variants carry typed values, including `i128::MIN`, finite float and normalized zero, bool, non-ASCII string, and null.
3. Binding order/name/producer/body identity, relation source authority, output schema, and selected item identity each affect the applicable contract/admission boundary. A foreign item selection fails before effects. A different Semantica generation changes no frontend digest; downstream must prove admission changes.
4. Owner unit/integration tests, `cargo test -p quanta-gqlang-v2-conformance --test self_validation --locked`, and `python3 tools/test_frontend_external_consumer.py` pass on the implementation candidate. Run `just ci` locally. After committing a clean candidate, run `just qualify-local` and record exact HEAD/tree, toolchain, commands, exits and selected/executed counts. GitHub availability is not a substitute for this local proof.
5. Semantica must separately compile against the exact qglang source pin and execute source text → V3 document → admitted provider/generation → result/receipt. Do not mark this ticket consumer-complete from producer-local tests.

## Stop conditions

- A proposed V3 document can exist with an empty placeholder digest unrelated to the actual source bytes.
- V1/V2 execution compatibility exports or dual-write paths are requested without a separately approved scope change.
- The implementation reparses source/literals, reconstructs binding/name semantics downstream, or adds runtime handles/provider/generation to the frontend DTO.
- A stable cross-transaction semantic cache key is claimed from the transaction-local contract digest.
- A shared dirty Semantica checkout is modified as part of this producer ticket.
