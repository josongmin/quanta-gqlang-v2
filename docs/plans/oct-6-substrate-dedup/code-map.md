# Code map and change boundaries

Parent: [execution plan](readme.md). Baseline Semantica
`ff8f2ce935fc470351d1dbc6c819173a47f6d65e` (supplier paths unchanged since
`070f4f96770`; final SCC owner/PTA cut reconciled separately).
All paths below are relative to `semantica-codegraph-v2` unless marked QGLang.
`core/` abbreviates `packages/core/`; `qv2/` abbreviates
`packages/analysis/quanta-v2/crates/`. Subtrees indicate reachable files to
review, not authority to replace every file. Final claims enumerate exact paths.

## Before / after at the real caller

| Boundary | Before on audited source | Required after |
| --- | --- | --- |
| Plugin facade | `build_semantic_surface_bundle_with_profile_v1` / `assemble_semantic_module_v1` → from-tree plugin → retired carrier getter → raw helper calculations | Same facade → ordinary scoped read → checked HIR/item/scope/span/dispatch calculation → same validated bundle. Controlled carrier without current-capable contract returns typed refusal |
| Native facts | `FactExtractionRequestV1` already passes source/tree/current → `DefUseFactsParseContextV1::single_parse` → four remaining raw getter consumers | Same ingress/context → exact binding admission → real extraction under checked read and original output custody → ordered facts or first typed refusal |
| C/Go facts | Checked read builds scalar snapshot → callback ends → uncontrolled scalar fact calculation; local 128 MiB snapshot ledger | Original admission covers temporary snapshot births, traversals and output lifetime; owned scalar grammar projections retained; one shared local cap/overflow mechanism |
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
already-issued output/temporary authority. The lower-level snapshot boundary
uses NativeAllocationControl or admitted backing callbacks, not a kernel import.

## Existing WIP: retain actual writers

### Rust plugin + DefUse — T1

- Plugin: `core/codegraph-lang-rust/src/language_parser_plugin/`, including
  `mod.rs`, `hir_lowering.rs`, `identity_extract.rs`, `item_extract.rs`,
  `scope_extract.rs`, `span_extract.rs`, dispatch/call helpers and the new
  checked modules already present in T1's working set.
- DefUse: `core/codegraph-defuse-extractor-rust/src/extractor/`, including
  `mod.rs`, `projection.rs` and T1's new frontdoor/read-selection/body/branch/
  statement/pattern/value modules.
- Complete the authored WIP. Do not create a second frontdoor or stage a
  partial subset of the dependent helper files.

### Go plugin — T1

- `core/codegraph-lang-go/src/language_parser_plugin.rs` and
  `language_parser_plugin/{constructors,helpers,language_plugin_impl,parse_ports,types}.rs`.
- `language_parser_plugin/parse_ports/`: call authority/extraction, HIR,
  identity, item index/conversion, scope and span helpers.
- Remove `parse_ports.rs`'s retained-tree → ParseResult-tree fallback when it
  would substitute another tree for the caller's retained authority.
- Go DefUse migration is already committed. Its new control work is CONTROL-CGO.

### Java DefUse — D0 + C66 + C65

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
  inventory, scopes and closure/capture behavior. Migrate the full walk;
  current getter removal alone is insufficient.
- Python `tests.rs` also has a separate generated-statement claim. No blanket
  Python crate staging; tests require that owner's handoff.

## New independent consumers

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

## CONTROL-CGO: shared mechanics, distinct language projections

- `core/codegraph-lang-common/src/{c,go}_syntax_snapshot_v1.rs`: duplicate
  `SnapshotHeapBudgetV1::charge_v1`, checked addition and 128 MiB local cap.
- Resolve these small duplicate ledgers inside the original-admission change.
  Reuse an existing helper or extract shared birth/overflow mechanics when they
  have the same contract. Record a justified retention of different local caps;
  a standalone generic helper is not a prerequisite for fixing custody.
- Retain distinct C and Go node data: Go repeated fields/sibling index/error
  metadata differ from C grammar facts. No generic semantic visitor.
- `core/codegraph-defuse-extractor-c/src/extractor/mod.rs`, its scalar traversal/
  projection helpers and tests.
- `core/codegraph-defuse-extractor-go/src/extractor/traversal/{fact_helpers,extract_core}.rs`,
  scalar traversal/projection helpers and tests.
- S0 owns the missing original output context bridge. Couple that producer
  and both consumers before controlled-success proof; a local cap alone cannot
  issue output authority. Missing required custody returns typed refusal.
- Account before storage growth, retain temporary permits until buffers are
  freed, account post-snapshot work, recheck before publication and test exact
  refunds/first refusal. Ordinary behavior retains the explicit local cap.

## CONTROL-KOTLIN

- `core/codegraph-defuse-extractor-kotlin/src/extractor/mod.rs`,
  `checked_node_v1.rs`, affected facts/output helpers and `tests.rs`.
- Retain the checked node wrapper and first-failure latch. Use S0's Native
  source/backend admission; root bounds do not prove exact binding.
- Add same-length foreign source and foreign backend on source-sealed input,
  cancellation/work refusal during traversal and publication tests. Current/no-tree
  uses existing test-support context/current mutation; no new constructor or
  invented production reachability is needed.
- Output birth/custody follows S0; do not create a Kotlin-specific issuer.

## SDK-DEADLINE / QG-DIAG

- SDK: `qv2/quanta-runtime/tests/abstract_interpret_fluent_api_e2e_test.rs`
  (`abstract-interp,parser-python,test-utils` required features), existing real
  harness in `tests/support/`. Add result tests here or the smallest existing
  owning target that exercises this **real** builder/adapter path.
- Existing SDK mapping owner:
  `qv2/quanta-runtime/src/sdk/abstract_interpret_builder.rs::non_query_error_tests`.
  Do not rename these five mapping tests as end-to-end deadline proof.
- Behavior code changes only for a reached failing oracle, at builder/adapter/
  lowering/engine owner as appropriate. Deadline currently starts at adapter
  `analyze`, not before SDK source-read/CFG work.
- QGLang: `crates/quanta-gqlang-v2-diagnostics-stdio/src/server.rs`,
  `src/handlers/v3_diagnostics.rs`, process contract tests and actual consuming
  client if discovered. Resolve the required wire fields before versioning it.

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
