# Code map and change boundaries

Parent: [execution plan](readme.md). Baseline Semantica
`ff8f2ce935fc470351d1dbc6c819173a47f6d65e` (supplier paths unchanged since
`070f4f96770`; final SCC owner/PTA cut reconciled separately).
All paths below are relative to `semantica-codegraph-v2` unless marked QGLang.
`core/` abbreviates `packages/core/`; `qv2/` abbreviates
`packages/analysis/quanta-v2/crates/`. Subtrees indicate reachable files to
review, not authority to replace every file. Final claims enumerate exact paths.

Current scheduling overlay at `871224b2059422a8f36eca91ffb0c9119238ac9c`:
Rust/Go/Kotlin/TypeScript plugins, Rust/Java/TypeScript DefUse and the coupled S0-CALLBACK repair are
integrated. Their before/after cuts below retain the audit baseline and acceptance
requirements; they are no longer new implementation assignments. The remaining
getter migration is Python DefUse. TypeScript's shared scalar control/output
scope joins C/Go below. Follow the
[current allocation](readme.md#maximum-parallel-scheduling) before assigning work.

Current Semantica dirty candidates, excluded from committed-source conclusions:
Kotlin exact retained source/backend admission and its one regression;
two SDK fluent deadline/domain-value regressions; and a shared local
`syntax_snapshot_budget_v1.rs` cap/overflow ledger replacing three copied
language counters. None has an accepted owner execution receipt. The shared
local ledger explicitly does not issue original allocation grants.

## Before / after at the real caller

| Boundary | Boundary being closed; see current overlay | Required after |
| --- | --- | --- |
| Plugin facade | `build_semantic_surface_bundle_with_profile_v1` / `assemble_semantic_module_v1` → from-tree plugin → retired carrier getter → raw helper calculations | Same facade → ordinary scoped read → checked HIR/item/scope/span/dispatch calculation → same validated bundle. Controlled carrier without current-capable contract returns typed refusal |
| Native facts | `FactExtractionRequestV1` passes source/tree/current → `DefUseFactsParseContextV1::single_parse` → historical raw getter consumers; output admission is absent | Same ingress/context → exact binding admission → real extraction under checked read and original output custody → ordered facts or first typed refusal |
| C/Go/TypeScript facts | Checked read builds scalar snapshot → callback ends → fact calculation without original current; three local 128 MiB snapshot ledgers | Original admission covers temporary snapshot births, traversals and output lifetime; owned scalar grammar projections retained; shared accounting mechanics |
| Kotlin facts | Checked node wrapper and first-refusal latch; root bounds but no exact retained source comparison in frontdoor | Exact source/backend admission before calculation; original current tests, first refusal retained, no controlled no-tree reparse |
| SDK deadline | Real builder passes timeout to adapter; five existing tests cover error mapping/authority/input/cancel | Same execution code with result-based real SDK deadline tests; existing mapping five remain distinct from domain/deadline behavioral proof |

## S0: existing common interfaces, one coordinated change

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
  implementations, checked addition and 128 MiB local caps. The dirty candidate
  extracts the identical local mechanics into `syntax_snapshot_budget_v1.rs`
  and registers it in `lib.rs`, with exact-cap and overflow tests; it remains
  uncommitted and unqualified.
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
- A dirty candidate validates retained source/backend in the read callback
  before extracting facts and adds an ordinary retained positive plus same-length
  foreign-source and foreign-backend typed refusals. Its current source/test
  status is authored, not accepted original-current or output-custody proof.
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
| D2 | `core/codegraph-graph-scc/src/scc.rs` owns bounded traversal; graph-utils `src/lib.rs` reexports compatibility paths → runtime-analysis `precise_query_scc.rs` / taint `wire_recursion.rs`; PTA adapter `scc_collapsing.rs` / Andersen `scc.rs` use core directly |
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
