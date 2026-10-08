# Code map and change boundaries

Parent: [execution plan](readme.md). Baseline Semantica
`ff8f2ce935fc470351d1dbc6c819173a47f6d65e` (supplier paths unchanged since
`070f4f96770`; final SCC owner/PTA cut reconciled separately).
All paths below are relative to `semantica-codegraph-v2` unless marked QGLang.
`core/` abbreviates `packages/core/`; `qv2/` abbreviates
`packages/analysis/quanta-v2/crates/`. Subtrees indicate reachable files to
review, not authority to replace every file. Final claims enumerate exact paths.

Latest repaired boundaries are in [R79 onward execution](execution-r79.md).
Native allocation now has the WIP canonical physical owner
`core/codegraph-native-allocation-core/`; tree-sitter consumes/reexports that
owner. The retired `native_shared_birth_v3.rs` path below is historical.
The Java member handoff now borrows actual parser FileId instead of forwarding
an external `members_v3` slice. Neither source change proves public Original success.

R77 cause/source classification is [R77 RCA](rca.md): committed parser/module
closure, live capture rejection and three unfinished Original output boundaries.
The source paths below retain their historical observations; R77's per-file
hashes identify current readbacks separately from the fixed proof tuple.

Current scheduling overlay includes Python `27d9505015e`, Kotlin binding
`df315e6` and common local counter `f2d85a9`. All inventoried plugin/DefUse
read cuts and S0-CALLBACK source are integrated; the retired production carrier
getter count is0. These are owner acceptance boundaries, not new migrations.
TypeScript's original control/output scope joins C/Go below. Follow the
[current allocation](readme.md#maximum-parallel-scheduling) before assigning work.

Kotlin exact1/whole70 and local counter2 have accepted scoped R49 owner results.
Their actual tested files were checked for continuity; this does not qualify
original output custody. SDK deadline2 and community2 await execution acceptance.
R63 rule owner34/34 includes the same-IR connection regression, integrated as
`a23ced26e09`. Minimal compile repairs
remain in their coowners' source bundles; R52 semantic contract55 passed.
R54 also accepted common callback2 and Kernel finisher5. The
[current handoff](../../work/substrate-repair-checkpoints-2026-10-07.md#fixed-candidate-compile-and-ownership-checkpoints)
records later SourceWire/DFG compile repairs and preserves their coupled ownership.

## Before / after at the real caller

| Boundary | Boundary being closed; see current overlay | Required after |
| --- | --- | --- |
| Plugin facade | Facade→from-tree plugin→checked reads is integrated; ordinary owners accepted | Preserve that validated bundle; prove controlled-carrier refusal under the actual current-capable boundary |
| Native facts | `FactExtractionRequestV1` passes source/tree/current → `DefUseFactsParseContextV1::single_parse` → integrated checked consumers; output admission is absent | Same ingress/context → exact binding admission → real extraction under checked read and original output custody → ordered facts or first typed refusal |
| C/Go/TypeScript facts | Checked read builds scalar snapshot → callback ends → fact calculation without original current; one shared local128MiB ledger is integrated | Original admission covers temporary snapshot births, traversals and output lifetime; owned scalar grammar projections retained |
| Kotlin facts | Exact source/backend guard integrated `df315e6`, whole owner70 passed; original output custody remains separate | Preserve that guard and ordinary compatibility; prove original output births, work/cancel refusal and final lifetime |
| SDK deadline | Real builder passes timeout to adapter; five existing tests cover error mapping/authority/input/cancel | Same execution code with result-based real SDK deadline tests; existing mapping five remain distinct from domain/deadline behavioral proof |

## Reached parser feature-boundary repair

- This no-Java export correction is one prepared cut within FCP-COMPILE.
  [R77's diagnostic breakdown](rca.md#1-the-committed-parser-packet-is-incomplete)
  additionally maps the fixed all-features parser's 11 module lookup failures,
  32 missing/private names, four declaration/trait diagnostics and 58 denied
  unused imports. The two feature profiles must both compile on one complete packet.
- Owner file: `qv2/quanta-adapters-parser/src/lib.rs`.
- Before: the unconditional export group names
  `ExistingFileSemanticModuleTransitionFallbackV2`, whose definition and only
  consuming mapper are Java-gated. R63's existing no-Java community profile
  reaches `E0432` before tests.
- Authored after: move only that enum into the existing `parser-java` export
  group. Keep `ExactIncrementalParseConsumptionPortV1` and the materialized view
  unconditional. Java's public export and wire representation are unchanged.
- Claim: `OCT8-GQLANG-PARSER-NOJAVA-EXPORT-R64`, reconciled with the existing
  parser coowners. Its recorded diff is one symbol move; never stage the whole
  dirty `lib.rs` as this repair.
- DoD: complete the owning parser/dependency/module bundle and verify the same
  no-Java owner profile plus Java feature closure. The static hunk review is
  accepted; parser compile and community execution are still pending. The other
  R63 errors have existing concurrent repairs and do not authorize duplicate
  implementation.

## S0: existing common interfaces, one coordinated change

- Caller audit at committed Semantica `f574fa82bf1` plus separately observed dirty
  DFG source narrows the missing-authority statement to general facts production.
  `qv2/quanta-adapters-dfg/src/multi_lang_dfg_adapter/build_pipeline/fact_extraction.rs`
  still passes erased Native control into
  `DefUseFactsParseContextV1::single_parse`; its ordinary output keeper has no
  original output admission. The Java sidecar's
  `original_generated_callable_candidate_v3.rs` does prepare an admitted CFG/facts
  pair, but explicitly refuses public publication. The dirty
  `original_workspace_evidence_issue_v3.rs` now calls the existing keeper's
  `OriginalDfgFunctionEvidenceBackingV1::try_issue_member_original_v3` and
  `try_issue_original_members_v3` internally. R74 additionally finds the existing
  dirty runtime-analysis→PTA original Java evidence path calling
  `try_issue_original_java_workspace_evidence_v3`. This corrects the earlier
  "no actual ingress caller" statement; successful publication is still absent.
  At R74 the workspace producer's proposed `members_v3` argument was omitted by
  runtime and SDK fixture callers. R96 supersedes it with the genuine parser
  member loan; current callers pass `parser_handoff_v1`. The R74 PTA transaction
  and retained DFG publication refusals retain their historical source. Its retained
  workspace validator consumes existing evidence; it does not issue that evidence.
  The shared backing and original assignment-index constructors are co-owned
  precursors, not a demonstrated successful general issuer/final keeper. Finish
  the producer, storage births and final publication together; do not duplicate
  the active Python-original-port implementation.

### Existing Java producer callers and remaining refusal — R74

Semantica-relative paths:

1. `qv2/quanta-runtime-analysis/src/queries/intraprocedural/production_live_native_lowering_generation_parts_v1.rs`
   calls the exported PTA workspace producer under `live-native-java-generation`.
2. `qv2/quanta-adapters-pta/src/production_live_native_workspace_generation_v1/original_java_workspace_v1.rs`
   at R74 accepted the proposed `members_v3`; current source accepts the actual
   `parser_handoff_v1` and calls `original_java_evidence_v1.rs`;
   that helper invokes the existing DFG workspace issuer under the same cohort.
3. The runtime call and
   `qv2/quanta-sdk-session-backend-owner/src/production_normal_source_java_workspace_generation_port_v1_tests/original_native_parts_v1.rs`
   both now pass the actual parser handoff. The old argument-missing work item is
   superseded; complete coupled owner/caller validation remains required.
4. At R74 PTA `issue_workspace_transactions_v1` refused the nonempty workspace
   pending admitted formal/alias/catalog births and canonical extraction phases.
   `retained_workspace_issuance_v1.rs` separately refuses the original DFG source
   world before ordinary births. These source refusals must be completed before
   declaring live Original output success.

This is historical static inspection with the current handoff correction, not a
current build or product receipt.
General `FactExtractionRequestV1`/`single_parse` output admission remains the
separate missing seam inside the same coordinated change. Preserve missing-
authority refusal, operation binding and final-drop grant lifetime.

### Coupled shared-backing supplier closure

At observed main `f574fa82bf1`, the committed Java DFG generated-callable candidate
`semantic_identity_sidecar_v1/original_generated_callable_candidate_v3.rs` consumes
`OriginalSemanticSharedBackingV3`, but the canonical shared-backing definition
and export remain in other owners' dirty/untracked source. This is a committed
consumer versus uncommitted supplier gap, not a reason to build a second holder.
The separate `original_evidence_backing_v1.rs` migration to that holder is WIP;
its source presence is not committed consumer integration.
The existing source path is:

1. `third_party/tree-sitter/binding_rust/{lib.rs,native_shared_birth_v3.rs}`:
   actual shared native birth and fallible retain.
2. `core/codegraph-cfg-dfg-kernel/src/ports/language_plugin_port/`:
   `canonical_output_api_v1.rs`, `type_copy_admission_v1.rs` and
   `shared_value_admission_v3.rs` bind paid birth/retain to canonical admission.
3. `qv2/quanta-runtime-query-surface-foundation/`: its manifest, `src/lib.rs`,
   `native_allocation_refusal_v3.rs`, `original_semantic_output_admission_v3.rs`
   and that module's `shared_backing_v3.rs`, `typed_input_v3.rs`,
   `std_btree_consumer_v3.rs`, `operation_v3.rs` and same-output tests.
4. `operation_v3.rs` now consumes SourceWire's `validate_operation_v3` through
   `qv2/quanta-runtime-source-authority-wire/src/{lib.rs,original_operation_custody_v3.rs}`.
   Include other mounted module changes required by the selected owner closure.

R76's manifest forwarding repair uses the existing dependency alias
`quanta_runtime_query_floor/original-repository-fd-capture-v3`. R77 metadata passes;
include that owned hunk with this compatible supplier packet. Separately review
the three current hierarchy module mount pairs under
`java_workspace_resolution_syntax_v1/hierarchy_authority_v1/` described in the
RCA. Eight other historical `E0583` mounts have explicit corrected WIP paths.
None of these static observations replaces the actual no-Java/Java compiler proof.

R74 follow-up also observes SourceWire's first-refusal observer handoff in shared
WIP. Its observer factory is fallible and charges work before retaining the
existing root; `token_refusal_observer_v3.rs` propagates that result. Immediate
normalization and authority-copy error conversion now read the authenticated
admission's borrowed `source_first_refusal_v3()` instead of retaining an observer
while reporting failure. Keep these callers and the factory in the same compatible
packet and preserve the first recorded cause. This is an authored source readback,
not execution acceptance or main integration; it adds no separate ticket.
`r74-concurrent-observer-handoff-readback.json` records exact postimages.

These are dependency-review paths, not a whole-file staging authorization.
Freeze the complete compatible postimages, resolve the real owner profiles and
prove them before serial integration. Importing only the backing/export files
does not close the current compile graph. Supplier integration also does not
wire the missing general facts producer: that remains the separate S0 handoff
above, using the existing member/cohort keeper.

The current read-only supplier/parser inventory is
`supplier-foundation-parser-f574-readonly-inventory.json` at the evidence root.
It records existing postimages and overlaps, not a proven transitive Rust
closure. R63's56 parser errors remain an old-source result. Live source repairs
several symbols, but the unconditional `std::path::Path` import in
`quanta-adapters-parser/src/semantic_attribution_normal_edit_materialization.rs`
is consumed by a Java-gated production request; its test module has its own
import. This is a no-Java boundary risk, not evidence that an R63 all-features
error remains. Check the existing no-Java owner before marking it repaired;
do not gate that production import only with `cfg(test)` or stage partial
Foundation/SourceWire mounts to obtain a narrower pass.

### Existing S0 protocol owners

- S0-CALLBACK follow-up source `65036b7221a`: existing
  `core/codegraph-lang-common/src/tree_sitter_host/owned_syntax_v1.rs` protocol,
  typed bridge and ordinary implementation; controlled implementation in
  `packages/infra/parsers/src/engines/tree_sitter/parser/controlled_native_tree_v1.rs`;
  Kernel metadata postvalidation in `syntax_authority_v1.rs` below. Change the
  same canonical callback protocol and its two actual implementations together.
  Preserve the success-only checkpoint/box/backing checks. No duplicate bridge,
  invented native error, cleared sticky cause, new Root or issuer-phase capture.
- `core/codegraph-cfg-dfg-kernel/src/ports/cfg_parsed_tree.rs`: source/backend
  metadata projection.
- `core/codegraph-cfg-dfg-kernel/src/ports/cfg_parsed_tree/syntax_authority_v1.rs`:
  existing `with_read_context_v1`, original metadata/syntax custody.
- `core/codegraph-lang-common/src/tree_sitter_host/current_read_v1.rs`: checked
  nodes, metadata, child/field/text access and traversal. Already co-owned.
- `core/codegraph-defuse-extractor-contract/src/lib.rs`: context, required
  retained tree, exact assignment inventory and typed native errors.
- `core/codegraph-cfg-dfg-kernel/src/ports/language_plugin_port/output_admission.rs`
  and `temporary_backing_v1.rs`: existing output work/birth and backing permit.
- `qv2/quanta-adapters-dfg/src/multi_lang_dfg_adapter/build_pipeline/fact_extraction.rs`:
  actual original-current consumer and typed runtime error mapping.

Choose the smallest existing owner for common Native retained admission. If a
new helper is needed, keep it at this context/authority boundary and migrate its
callers in the same bundle. Function/CFG binding remains language-owned. Native
producers already attach source authority; prove same-length mismatch and
missing required authority there. Ordinary unsealed retained inputs retain
their documented compatibility; never fabricate source authority in a fixture.

Shared owner handoff must record signatures, current/output lifetime, precise
first-error precedence, source/backend requirements by lane and consumer list.
`CanonicalSemanticOutputAdmissionV1` is existing **kernel** authority, not a
grant that a language helper may issue. Common language code must not import
the kernel back into itself. `FactExtractionRequestV1` and
`DefUseFactsParseContextV1` currently have no output-admission field: S0 must
implement the actual issuer/adapter→context→extractor bridge, borrowing the
output/temporary authority only after its original issuer is traced. DFG's
erased `NativeAllocationControlV1` cannot project the source-operation/memory
lease required by the generic original output admission. The current Java
semantic wrapper does not prove DFG issuance. Include the actual upstream
ingress/source-control handoff and final-retainer caller closure in S0; the
write set can exceed these two structs. Required missing authority refuses.
The lower-level snapshot boundary
uses NativeAllocationControl or admitted backing callbacks, not a kernel import.

## Existing owners: integrated cuts and remaining acceptance

The earlier T1/D0/C65/C66 labels below record ownership observations at the
baseline. Current claims require refresh; integrated read cuts are acceptance
boundaries, not instructions to assign their implementation again.

### Rust plugin + DefUse — T1

- Plugin: `core/codegraph-lang-rust/src/language_parser_plugin/`, including
  `mod.rs`, `hir_lowering.rs`, `identity_extract.rs`, `item_extract.rs`,
  `scope_extract.rs`, `span_extract.rs`, dispatch/call helpers and the new
  checked modules integrated in `ed94af892c0`.
- DefUse: `core/codegraph-defuse-extractor-rust/src/extractor/`, including
  `mod.rs`, `projection.rs` and T1's new frontdoor/read-selection/body/branch/
  statement/pattern/value modules.
- Source read cut integrated `ed94af892c0`; verify the full owner closure.
  Do not create a second frontdoor or a partial replacement helper graph.

### Go plugin — T1

- `core/codegraph-lang-go/src/language_parser_plugin.rs` and
  `language_parser_plugin/{constructors,helpers,language_plugin_impl,parse_ports,types}.rs`.
- `language_parser_plugin/parse_ports/`: call authority/extraction, HIR,
  identity, item index/conversion, scope and span helpers.
- Source read cut integrated `ed94af892c0`; prove retained authority is never
  substituted by a ParseResult tree. No new fallback-removal assignment.
- Go DefUse migration is committed. Its control work is CONTROL-SNAPSHOTS.

### Java DefUse — D0 + C66 + C65

- Read cut is integrated `e14eeda8604`; the earlier shared claims below are
  historical handoff inputs. Preserve any remaining foreign lvalue/WIP and
  verify actual binding/assignment/output acceptance.
- D0: `core/codegraph-defuse-extractor-java/src/extractor/extraction_frontdoor/mod.rs`,
  `extract_ctx.rs`, `node_dispatch/`, owner projection and module selectors.
- C66: `extraction_frontdoor/{callable_binding,cfg_callable_bind}.rs`, related
  checked callee/type/switch helpers under its exact claims.
- C65: definition/argument/projection/event helpers and related tests under its
  exact claims. Some paths have multiple claims; reconcile by symbol/hunk.
- Required coupling: checked callable region → CFG binding → checked dispatch
  → exact semantic assignments → output materialization. One frontdoor change
  does not establish that the full Java calculation is checked.

### Python DefUse — D0

- `core/codegraph-defuse-extractor-python/src/extractor/mod.rs`,
  `retained_callable_index.rs`, `callable_owner_projection.rs`,
  `binding_predeclaration*`, `class_scope*`, `control_flow*`, `const_fold*`,
  `import_resolution*`, `parameters*`, `projection_semantics/`,
  `extraction_core/` and authored checked helpers.
- Preserve `PreserveMayBranches`, source-closed module policy, exact semantic
  inventory, scopes and closure/capture behavior. The complete checked walk is
  integrated `27d9505015e`; zero retired production calls remain. Its145 package
  files and manifest match the R49 frozen source; owner execution is still required.
- Refresh ownership before any further Python edits. No new migration writer or
  blanket crate staging; source integration alone does not discharge control proof.

## Integrated Kotlin/TypeScript acceptance boundaries

Kotlin plugin is integrated `ecc0c5784b4`; TypeScript plugin and DefUse share
the integrated scalar syntax owner in `b32361e9d83`. The baseline line numbers
below identify the repaired cuts. Only their actual owner acceptance and shared
CONTROL-SNAPSHOTS work remain; no new read-migration workers are assigned.

### P-KOTLIN

- `core/codegraph-lang-kotlin/src/language_parser_plugin/parse_ports.rs`:
  two entrypoints at baseline lines 304/354.
- `helpers.rs`, `helpers/{call,hir,identity,item,parse_error,rfc310,scope,span}_helpers.rs`.
- `constructors.rs` / `language_plugin_impl.rs` only where an affected signature
  is called; owner tests in `core/codegraph-lang-kotlin/src/tests.rs`.
- Preserve semantic bundle/module parity, stable IDs, recoverable error
  isolation and declaration/call ordering.

### P-TS

- `core/codegraph-lang-typescript/src/language_parser_plugin/mod.rs`:
  two entrypoints at baseline lines 1184/1221.
- `hir_lowering.rs`, `hir_lowering/{arguments,calls,declarations,imports,signatures}.rs`.
- `identity_extract.rs`, `item_extract.rs`, `scope_extract.rs`, `span_extract.rs`,
  `type_facts.rs`, `dispatch_facts.rs`, `dispatch_facts/seed.rs`,
  `error_collection.rs`, `v1_convert.rs`, and affected owner tests.
- TS, TSX and JavaScript syntax selection, duplicate/ordering behavior,
  identity and dispatch targets must retain independent expected results.

### D-TS

- `core/codegraph-defuse-extractor-typescript/src/extractor/mod.rs`:
  retained getter at baseline line 956.
- `traversal.rs`, `traversal/`, `extraction_session.rs`,
  `binding_identity/`, `statement_identity/`, `projection/` and affected tests.
- Preserve declaration/initialization execution order, destructuring,
  assignment/update use/defs, callable boundaries, block binding and
  semantic assignment identity. Consume original current throughout.
- Separate crate/write set from P-TS. Coordinate compilation and field
  semantics by contract, not by sharing raw nodes or copying its implementation.

## CONTROL-SNAPSHOTS: shared mechanics, distinct language projections

- `core/codegraph-lang-common/src/{c,go,typescript}_syntax_snapshot_v1.rs`: the
  committed baseline has three duplicate `SnapshotHeapBudgetV1::charge_v1`
  implementations, checked addition and 128 MiB local caps. Checkpoint `f2d85a9`
  extracts those mechanics into private `syntax_snapshot_budget_v1.rs` and
  registers it in `lib.rs`; both exact-cap/overflow tests passed. The5 tested
  files match the current source. Original grant custody remains open.
- Finish the original-admission change separately from this local dedup.
  Reuse an existing helper or extract shared birth/overflow mechanics when they
  have the same contract. Record a justified retention of different local caps;
  a standalone generic helper is not a prerequisite for fixing custody.
- Retain distinct C and Go node data: Go repeated fields/sibling index/error
  metadata differ from C grammar facts; TypeScript retains its binding/field
  projections. No generic semantic visitor.
- `core/codegraph-defuse-extractor-c/src/extractor/mod.rs`, its scalar traversal/
  projection helpers and tests.
- `core/codegraph-defuse-extractor-go/src/extractor/traversal/{fact_helpers,extract_core}.rs`,
  scalar traversal/projection helpers and tests.
- `core/codegraph-defuse-extractor-typescript/src/extractor/mod.rs`, shared scalar
  extraction/binding/assignment helpers and tests on the integrated `b32361e9d83` cut.
- S0 owns the missing original output context bridge. Couple that producer
  and all three consumers before controlled-success proof; a local cap alone cannot
  issue output authority. Missing required custody returns typed refusal.
- Account before storage growth, retain temporary permits until buffers are
  freed, account post-snapshot work, recheck before publication and test exact
  refunds/first refusal. Ordinary behavior retains the explicit local cap.

## CONTROL-KOTLIN

- `core/codegraph-defuse-extractor-kotlin/src/extractor/mod.rs`,
  `checked_node_v1.rs`, affected facts/output helpers and `tests.rs`.
- Retain the checked node wrapper and first-failure latch. Use S0's Native
  source/backend admission; root bounds do not prove exact binding.
- Checkpoint `df315e6` validates retained source/backend in the read callback
  before extracting facts, with an ordinary retained positive and same-length
  foreign-source/backend typed refusals. Exact1/whole70 owner tests passed;
  original-current and output-custody proof remains separate.
- Complete source-sealed Native rejection and
  cancellation/work refusal during traversal and publication tests. Current/no-tree
  uses existing test-support context/current mutation; no new constructor or
  invented production reachability is needed.
- Output birth/custody follows S0; do not create a Kotlin-specific issuer.

## SDK-DEADLINE / QG-DIAG

- SDK: `qv2/quanta-runtime/tests/abstract_interpret_fluent_api_e2e_test.rs`
  (`abstract-interp,parser-python,test-utils` required features), existing real
  harness in `tests/support/`. Dirty candidate tests now assert zero-timeout
  typed error/reason/phase and exact Sign/Interval/Constant values under an
  explicit live deadline. Owner execution and complete F7 acceptance are pending.
- Existing SDK mapping owner:
  `qv2/quanta-runtime/src/sdk/abstract_interpret_builder.rs::non_query_error_tests`.
  Do not rename these five mapping tests as end-to-end deadline proof.
- Behavior code changes only for a reached failing oracle, at builder/adapter/
  lowering/engine owner as appropriate. Deadline currently starts at adapter
  `analyze`, not before SDK source-read/CFG work.
- QG-DIAG is resolved as the existing primary-only private wire, not a new
  schema task. [ADR-0008](../../adr/0008-private-diagnostics-primary-location-contract.md)
  records the actual parity/NDJSON consumer requirement; the
  [language contract](../../language/README.md#cli-and-diagnostics-stdio)
  fixes the exact fields. Execution receipts remain separate.

## D1–D7: retain canonical owners; no duplicate redevelopment

| ID | Existing owner / actual consumer |
| --- | --- |
| D1 | `qv2/quanta-adapters-shape/src/shape_analyzer_adapter.rs` → `quanta-adapters` / `quanta-adapters-runtime-surface` factories/exports → SDK/converter |
| D2 | `core/codegraph-graph-scc/src/scc.rs` owns bounded traversal; `qv2/quanta-graph-utils-v1/src/lib.rs` reexports compatibility paths → runtime-analysis `precise_query_scc.rs` / taint `wire_recursion.rs`; PTA adapter `scc_collapsing.rs` / Andersen `scc.rs` use core directly |
| D3 | `qv2/quanta-adapters-dfg/src/memssa_adapter/` active PortDFG path; retired build-input entry remains absent |
| D4 | retrieval kernel `scv2_exact_fusion.rs` → `quanta-retrieval-algo-v1/src/rrf.rs` |
| D5 | `core/pta/engine/src/solver/union_find_base.rs` and active Steensgaard; retired Andersen export remains absent |
| F1/D6 | SDK `_product_search_projection_v1.py` → shared `current_search_projection.py` → API/MCP consumer policy |
| D7 | MCP `domain/query_surface_registry.py` → metadata/list/schema/dispatch and `main.py` preflight |

The recorded D1–D7 source checkpoints are ancestors of audited main. The final
`ff8f2ce935f` SCC/consumer cut is integrated; old graph-utils `src/scc.rs` was
renamed to the core owner and is not another deletion task. Its old test selector
is stale and the new PTA adapter proof is missing from that old request map.
Keep these concrete proof/product rows in remaining work.
If a current test exposes a failure, assign the smallest owning repair with its
caller and oracle; do not automatically move or rewrite these owners again.

## Source callback and checked caller cut — R169–R176

Paths below use Semantica `qv2/`. All changes reuse the existing owner.

| File / function | Before | After / remaining proof |
| --- | --- | --- |
| `quanta-adapters-pta/src/production_native_lowering_relation_output_v1/original_relation_output_v3.rs::produce_native_relation_lowering_outputs_from_original_v3` | DFG Source callback returned owned relation output | Callback returns unit; outer accumulators/final-output slots feed the opaque wrapper only after Source success. Parser/DFG member checks, census and order retained; genuine owner execution pending. |
| `quanta-adapters-pta/src/production_live_native_workspace_generation_v1/original_java_workspace_v1.rs::produce_original_java_relation_v3` | Parser callback returned owned relation | Local result slot; scratch publication follows successful Parser postvalidation. Existing nonempty UnsupportedOperation remains; no new solver. |
| `quanta-runtime-analysis/src/queries/intraprocedural/pre_native_function_build_authority_owner_v1/original_workspace_rows_v3.rs::axes_from_original_sources_v3` | Owned axes returned through Parser Source callback | Paid context/projection and Copy digests stay outside the loan; callback returns unit; existing Kernel nominal axes constructor runs afterwards. Checked validator work is paid explicitly. |
| Same file, `copy_axes_v3` request/result callers | Owned projection copy inside scoped read | Distinct outer projection/context slots; canonical axes constructor after read. Actual whole request/result construction is outside the Parser loan; it is not an additional lender defect. |
| `quanta-sdk-runtime-executor/src/client/product_java_exact_build_authority_owner_v1/original_java_build_input_v3/tests/genuine_original_pre_native_authority_v3.rs` | Independent one-row/body/digest oracle; refund only in preflight negatives | Positive also observes retained IntermediateBytes above baseline and exact baseline after last result drop. Source-ready, not executed. |
| `quanta-runtime-analysis/src/testing.rs::execute_typed_request_v1` | Calls removed raw-wire and infallible erasure APIs | Existing checked wire/erasure APIs use the actual RuntimeCtx and propagate typed failure. Source-ready, not executed. |

The QRA supplier cut also requires Kernel's existing axes-from-slots constructor,
Foundation's axes read/projection slots/reexports and SourceWire's canonical
member/projection sink APIs. Capturing the QRA file alone cannot establish it.
Kernel methods moved to child impls are not automatically removed APIs; compare
the actual definition, signature and consumer. Current source hash drift keeps
the complete carrier/context/SDK integration outside the accepted nominal9 proof.

## R180–R187 discovery and changed source cut

Semantica `.gitignore`'s broad `target*/` rule hid two referenced QRA Rust
modules under `quanta-runtime-analysis/src/queries/pta/domain/precise_query_types/target_authority/`.
R182 adds the exact source-directory exception; both files are visible as
untracked source, while four root/nested build-artifact probes remain ignored.
The R185 bounded `packages/**/src/**/*.rs` census found no other ignored Rust
source among 1,026 untracked Rust files on that observed tree. Discovery alone
does not put those files in an immutable capture or compile them.

R184's SDK capture guard stops before owner resolve: the PTA relation producer
and Java caller changed from the R179 expected postimages. R187 records their
new hashes. Its successor is the mounted 14-path PTA source cut, not just two
files. R191 later captures that cut with the six SDK/QRA required postimages;
R193 verifies 20 captured paths and reviews the changed producer statically.
The R184 guard is an input-identity refusal, not an SDK compiler or behavior result.
See the [R180–R187 checkpoint](../../work/substrate-repair-checkpoints-2026-10-07.md#r180r187-owner-proof-and-capture-checkpoint).

## R190–R193 mounted PTA and SDK capture

R187's current PTA relation producer uses unit/status callbacks and
`OriginalRelationBirthSlotsV3` to retain paid output and typed error until
Source postvalidation. Its Java caller parks output after the Parser callback.
The 14-path static review found no concrete defect; the source changed during
review, so R191 captures the later complete postimage. R193 verifies all 20
frozen paths and the producer's operation, group, census and error chronology
statically; no owner execution follows from this review. The producer can
project nonempty relations, while the current composer still refuses their
publication with typed `UnsupportedOperation`.

R190's frozen PTA error is a missing DFG `original_writes_v3` mount. The
current source already points to the existing child explicitly; do not assign
another mount repair without a fresh failure. R191's six required SDK postimages
include `.gitignore`, the two QRA child modules, QRA axes, the SDK positive test
and test executor. The new capture digest is `8537a14ae748…`; owner behavior
remains unexecuted. See the [successor checkpoint](../../work/substrate-repair-checkpoints-2026-10-07.md#r190r193-owner-and-sdk-capture-successor).

## R201–R206 compile owner map

R201's final R107 frozen postflight reaches four zero-test compile failures.
The shared DFG `storage_mode_v3.rs` child mount for `original_writes_v3`
blocks PTA retained-copy, Taint MemSSA and navigation import; the explicit
mount exists in R191 and current source. Metrics instead reaches the
HirArgs `try_retain_original_typed_input_v3` override in query-surface
symbol-owner under a feature class where Foundation's
`original_semantic_output_admission_v3` is disabled. The owning feature guard
exists live but is absent from R191. These are two frozen compile causes,
not four independent runtime defects.

R194's SDK target reaches `quanta-runtime-kernel-contract/src/domain/contracts/query_trait.rs`:
its `try_live_output_overlays_from_checked_output_v3` return type lacks the
`QueryLiveOutputOverlaysV3` alias in the R191 source. R205 verifies that the
current owner already supplies the exact pair of `ErasedLiveOutputOverlayV1`;
the alias successor has not passed compile. R206 then refuses a new canonical
capture because untracked overlay content changes between capture and copy.
No new manifest or owner result exists for that attempt. The SDK three-case
owner and R200's two resolved DFG selectors retain separate behavior oracles
after one complete source tuple is frozen. Nonempty PTA publication remains
explicitly unsupported by the current composer; general Original facts and
output publication require implementation. See the
[R201–R206 checkpoint](../../work/substrate-repair-checkpoints-2026-10-07.md#r201r206-final-batch-and-sdk-capture-checkpoint).

## QRA function-input custody co-cut — R212/R213

This is additional detail within `CHECKED-RETAINED-CONTROL`, observed on
Semantica `3b8f86dd` plus dirty source. Source ownership shows payload drop
before the outer Source finisher; it does not establish early budget refund,
wrong output, partial publication or a leak. The output group lease is retained.
The following changes must preserve the existing canonical encoder, field
order, digest preimage, work charges and typed Source/Portable causes.

| Existing owner / caller | Before, observed source | Required after |
| --- | --- | --- |
| `qv2/quanta-runtime-analysis/src/queries/intraprocedural/intraprocedural_artifact_digest_v1/original_components_copy_v3.rs` | Function ID/provenance and component copiers return owned values; a later fallible field can destroy earlier paid fields. | Extend the same copier owner with external field sinks/slots, using existing `copy_string_into_v3` and identity sink primitives. Preserve partial backing on failure. |
| Same crate, `intraprocedural_artifact_input_assembly_v1.rs::function_input_components_v3` and `intraprocedural_artifact_input_assembly_v1/original_v3.rs::visit_function_build_inputs_from_admitted_request_v3` | Struct-literal field copies use `?`; the visitor emits an owned digest. A single replacement Option would also destroy previous inputs. | Build through external component slots; visit borrowed input. Keep pending input plus every completed input in the QRA birth through Source finish, admitting any retained Vec growth at its existing owner. |
| Same crate, `intraprocedural_artifact_digest_v1/original_encode_v3.rs` | `try_new_with_original_v3` consumes components; portable bytes and encoder scratch are returned or dropped inside the call. | Borrow components through fallible canonical work, then transfer them into a completed input. Retain partial encode/output state through the existing encoder owner; do not introduce a serializer or a second digest calculation. |
| Same crate, `pre_native_function_build_authority_owner_v1/original_workspace_rows_v3/{birth_v3,result_v3}.rs` | Row construction/portable copying can fail after owned emit; state has no input/component/portable scratch slots. | Store partial row, portable backing, pending input and completed inputs in existing birth state; preserve full typed cause at the existing unit-status transaction boundary. |
| Same crate, `production_live_native_lowering_generation_parts_v1.rs`; `qv2/quanta-sdk-runtime-executor/src/client/production_live_native_lowering_generation_parts_authority_v1.rs`; `qv2/quanta-runtime-test-support-pta/src/exact_native_generation_fixture_v1/original_execution_v1.rs` | Analysis creates QRA birth inside the SDK or test-support Source callback, before context `finish_original_source_result_v3`. | Move that existing birth to each actual caller outside the Source loan, pass it by mutable borrow and consume the completed result by borrow until finish. Migrate both actual callers and the three direct SDK QRA test callers in one compatible cut. |

These are existing owner extensions, not separate engines or duplicate tasks.
The component/encode helper and upper caller have active overlapping source
claims; this audit makes no competing source edit. Revalidate their resulting
ABI before editing the visitor, and remove the superseded Original owned-return
path with its callers. Ordinary APIs retain their distinct existing contract.
See the [test obligations](test-plan.md#qra-function-input-custody-acceptance--r212r213).

### R220 physical QRA input producer ABI

The normal R90 claim is reconciled with the existing component-helper owner.
Four source files above now contain this producer cut; no encoder body or
highest SDK/QRA caller was edited by this lane:

```rust
// Allocation-free Default; field storage stays outside the Source loan.
OriginalFunctionBuildInputComponentsSlotsV3
OriginalFunctionBuildInputAssemblySlotsV3 {
    components_copy_v3: OriginalFunctionBuildInputComponentsSlotsV3,
    components_v3: Option<FunctionBuildInputComponentsV1>,
}
visit_function_build_inputs_from_admitted_request_v3(
    request, output, &mut assembly_slots,
    emit: FnMut(&StableDeclarationIdV1, &mut Option<Components>) -> Result<(), E>,
) -> Result<(), E>
```

Existing runtime-function/provenance copiers now take `&mut ComponentsSlots`
and return `Result<(), OriginalFunctionBuildInputErrorV3>`.
`copy_components_v3(source, output, &mut slots, &mut Option<Components>)`
does the same. `try_finish_components_v3` guards first, then moves backing
without a fallible step. The ordinary and admitted assembly paths share one
nine-field selection and one ten-field component mapping; optional fallback
presence remains distinct. Occupied or failed slots are never replaced.

The actual QRA emit callback must invoke the **same canonical constructor**
using external encoder/input slots, then retain every completed input and
partial/retired encoder or row backing until highest Source finish. It consumes
components only after successful canonical construction. No arbitrary builder
callback, serializer, issuer or guessed Core scratch type is added. The prior
encoder/QRA receiving signatures remain a pending compatible cut, not a
compiling or qualified result. Evidence:
`r220-qra-input-producer-source-packet.json` and
`r220-qra-input-producer-owned.diff` at the existing recovery root.
