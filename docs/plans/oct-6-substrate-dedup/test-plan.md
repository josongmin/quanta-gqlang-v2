# Owner and product test plan

Parent: [execution plan](readme.md). This is a request/oracle specification,
not an execution receipt. Old selectors below are inputs to re-resolution.
Latest caller/request audit: Semantica `871224b2059422a8f36eca91ffb0c9119238ac9c`.
`qv2/` abbreviates Semantica `packages/analysis/quanta-v2/crates/`.

## Execution contract

1. Use Semantica `scripts/quanta-build-cli owner resolve` then the matching
   `owner run`. Read the actual manifest/profile before choosing features.
2. Freeze committed source plus committed GQLang/Index sibling pins, source
   digest, toolchain, execution root and lane admission. Reviewed overlays may
   support a candidate proof; final integration proof uses committed source.
3. One exact recipe/logical lane runs serially. Independent read reviews run
   concurrently. Different compile classes may run concurrently only after
   actual lane/cache/resource admission; four agents do not imply four heavy
   Rust builds. A test filter does not reduce `lib(test)` compilation.
4. Record selected/executed/passed/failed/ignored/skipped/leaked counts, required
   witness names, command/features, source/dependency pins and archived receipt
   hashes. Zero selected tests, pending publication or compiler-only success
   does not discharge acceptance.
5. R38 common7/Java139, R44 445, R40 quarantine130 and historical D4/D5 evidence
   retain their different sources. Rebind unchanged owner inputs explicitly or
   rerun changed owners; never sum these as a current product verdict.

## Consumer regression oracles

S0-CALLBACK additionally needs a discriminating owner regression through the
**actual bridge**, not just existing finisher tests: callback `Err(E)` retains
its exact identity and triggers zero owner/Kernel post-polls; pre-recorded
original refusal retains priority; callback success still executes the required
postvalidation and returns its typed failure. Common protocol/ordinary, infra
controlled syntax and Kernel tests retain their own owners. Existing
`language_plugin_port/callback_finish_tests_v1.rs` checks the finisher alone and
does not prove this bridge path. A diagnostic poll-counter test may prove local
control flow; actual original-control proof remains separate and requires its
normal execution authority. The source-only follow-up did not execute tests.

| Owner scope | Positive result | Discriminating rejection / resource oracle |
| --- | --- | --- |
| Each changed plugin | Existing authored HIR/item/scope/span/bundle expected values and ordinary from-tree/facade parity; actual module assembly | Recoverable parse quarantine, malformed/empty authority, stable order/IDs; controlled carrier without current-capable API has exact typed refusal; no retained-tree substitution |
| Each changed DefUse | Reviewed explicit definitions/uses/assignments/block bindings/order; source-only vs retained parity is supplemental | Same-length foreign source and missing required authority on Native source-sealed input, foreign backend, stale CFG/callable binding; current/no-tree only if reachable; exact first refusal and function ID |
| C/Go/TypeScript snapshots | Exact language-specific scalar relationships preserved; one actual original-admission contract used by all three | Cap/overflow/physical-birth refusal before growth, original temporary grant/refund, low-budget refusal during post-snapshot calculation, cancellation before publication; no partial output |
| Kotlin current | Real checked walk and existing expected facts | First callback failure remains first; prune/leave failure priority, cancellation/work stop, exact source/backend rejection and output lifetime |
| Native facts ingress | Actual adapter sends original tree/source/current; facts projected through real extractor | Exact NativeRefused → RuntimeError type/category; unsupported/missing authority never reparsed into success |

Use existing real original-control issuers for resource proof. Diagnostic/fake
controls may test a local mapper but do not establish original-root custody.
For DFG facts, first trace the actual original issuer and final retainer:
the erased Native control and Java parser issuer do not establish that handoff.
Keep syntax/facts fixtures small except where depth/allocation mechanisms require
scale. No sleep-based timing oracle, self-equality or bare `is_ok` success claim.

The current dirty Kotlin candidate adds
`retained_kotlin_binding_rejects_same_length_foreign_source_and_backend_v1`
with an exact-source positive and two typed, function-bound refusals. It does
not discharge original current, output lifetime or all traversal controls.
The current dirty C/Go/TypeScript candidate moves the identical local
128 MiB/checked-add counter into `codegraph-lang-common/src/syntax_snapshot_budget_v1.rs`;
its exact-cap and overflow tests cover only that local counter. The helper does
not issue original grants or account post-snapshot calculation. Review the
complete producer/consumer cut and run its owners before accepting custody or
duplicate-removal completion.

### Changed language owners

- Resolve the owning `codegraph-lang-{rust,go,kotlin,typescript}` target under
  its current reviewed all-features closure, including existing bundle/dispatch
  and quarantine tests. C source is integrated, but its owner acceptance remains.
- Resolve `codegraph-defuse-extractor-{rust,java,python,typescript,c,kotlin,go}`
  affected targets. New tests live at these owners; no cheaper substitute crate.
- Fresh selected minima/witnesses come from source discovery and reviewed
  requests. Do not copy total counts from old language runs as new minima.
- Compile plugin prerequisites before dependent DefUse/runtime owner proof.
  A same-language coupled bundle must compile as a whole before integration.

## Original issuer traversal — exactly five existing controls

Package `quanta-adapters-parser`, target `lib`, module selector:

`language_host_adapter::original_native_tree_control_v1_tests::native_incremental_origin_v3::original_traversal_v3_tests`

Compile policy: `package-all-features`. Minimum executed/passed: 5, ignored: 0.
Preserve all literal witnesses from the existing R38 request:

1. `original_named_subtree_keeps_literal_method_boundary_without_backing_growth_v3`
2. `original_subtree_work_refusal_stops_after_first_callback_and_remains_first_v3`
3. `original_subtree_cancellation_inside_callback_stops_before_next_node_v3`
4. `original_phase_cancellation_while_pruning_stops_before_leave_callback_v3`
5. `original_phase_work_refusal_while_pruning_stops_before_leave_and_stays_first_v3`

Cursor three files are already on main as `dc2611b96e1`. Run these after the
actual adapter compile closure is repaired. Do not reimport those source files
or weaken the no-backing-growth/first-refusal oracles.

## F2 Native registry and Preview

All three requests use `quanta-cli-ingress-runtime` lib/module selectors.
Historical feature IDs must be resolved on the new source, not assumed stable.

| Selector | Historical feature class | Minimum / required witness suffixes |
| --- | --- | --- |
| `dispatch::operation_binding_registry_v1::native_mutation_binding_tests` | `no-default.01d5e8939f20` | 2; `every_declared_native_mutation_has_one_matching_runtime_binding`, `admitted_rename_preview_obeys_mutation_provider_availability` |
| `dispatch::operation_binding_registry_v1` | `no-default.9d4f950584d2` | at least 3 required witnesses: `runtime_operation_bindings_cover_every_canonical_service_operation_id_v1` and both above; old request minimum1 alone is insufficient |
| `dispatch::mutation::tests` | `no-default.9d4f950584d2` | 3; `typed_refactor_rename_reaches_backend_and_projects_owner_preview`, `refactor_rename_apply_without_issued_response_root_never_calls_backend_v1`, `refactor_rename_apply_with_host_root_still_refuses_before_backend_v2` |

Verify literal contract→operation ID→service bindings, duplicates, all six
Mutation IDs, feature/provider-off behavior and admitted context→worker→Preview
backend. Missing production terminal/replay issuer makes Apply refusal correct;
host limits and caller caps do not justify an Apply-success oracle.

## F7: separate mapper proof and actual deadline behavior

### Existing SDK mapper five

- Package `quanta-runtime`, lib/module
  `sdk::abstract_interpret_builder::non_query_error_tests`.
- Historical feature class `no-default.e8963e85866e`; minimum 5, ignored 0.
- These cover timeout/non-timeout error mapping, missing authority, invalid
  input and cancellation. They do **not** prove five real SDK deadline behaviors.

### Existing behavioral owners to resolve

- Engine: `packages/core/abstract/engine/src/solver/deadline_tests.rs`.
- Lowering: `qv2/quanta-abstract-interp-lowering/src/tests_deadline.rs`.
- Adapter: `qv2/quanta-adapters-abstract-interp/src/deadline_tests.rs`.
- Wire: `qv2/quanta-contract-types-core/tests/abstract_interpret_deadline_v1.rs`.
- Resolve package names/targets/features from those owners; source test counts
  are not executed counts. Assert exact deadline type versus iteration outcome,
  all supported domains and final publication refusal on the actual owner.

### Authored real SDK regressions; execution acceptance pending

Use `Code::verification().abstract_interpret().execute()` and the real adapter
in `quanta-runtime/tests/abstract_interpret_fluent_api_e2e_test.rs`; manifest
requires `abstract-interp,parser-python,test-utils`.

- Dirty source now authors `test_e2e_sdk_deadline_zero_is_typed_for_all_domains`:
  deterministic `.timeout_ms(0)` on Sign, Interval and Constant must return
  `RuntimeError::AbstractInterpretDeadlineExceeded` with the exact structured
  reason and execution phase, rather than success or a generic error. The SDK
  variant exposes no separate operation-identity field.
- Dirty source also authors
  `test_e2e_sdk_explicit_deadline_preserves_all_domain_values`: on real Python
  `i = 0`, an explicit 30_000 ms deadline must converge and return Sign(Zero),
  Interval `[0,0]` and Constant `Some(0)`. Confirm the default 30_000 ms
  parameter at its builder owner rather than sleeping for 30 seconds.
- These are authored assertions, not executed or qualified results. Resolve the
  current integration target and features, then run its owner proof on the
  frozen source tuple. Keep the existing mapper five separate.
- Iteration exhaustion and non-timeout failures preserve their distinct types/
  result posture. An overdue lowering/projection result cannot publish success;
  retain the deterministic engine/adapter final-check oracle.
- Deadline begins at adapter `analyze`; SDK source read and CFG construction
  are not currently covered. Do not silently change that compatibility contract
  or claim hard interruption of non-cooperative callbacks.

## Original output custody, JSON and quarantine

Re-resolve existing R31/R38 requests on the changed memory-owner source:

- `quanta-contract-ports` lib/module
  `cfg_dfg_pipeline_port::original_output_custody_v3::tests_v3`, all-features,
  minimum8: original-root birth/lifecycle, first failure, retention/refund.
- `quanta-runtime-execution` integration target `gqlang_relation_execution_v3`,
  module `json_projection`, historical feature `no-default.6aeb25727949`,
  minimum13: exact actual execution output and durable replay.
- `quanta-sdk-exact-repo-quarantine-owner` whole lib/all-features,
  historical minimum130 plus required preissuer-refund and last-clone witnesses.
  Verify current discovery before retaining that minimum; tests must not shrink
  merely to bypass a failing count.

## D1–D7 matrix and policies

`remaining-r34-substrate-owner-map45.json` is **45 historical resolve-only
requests**, not 45 passing runs. Reconcile the changed request definitions before
using `rebind-owner-map.py` on the successor tuple; retain the serial/archive
pattern of `run-substrate-matrix-r29.py`. Do not execute the old driver unchanged.

Final source `ff8f2ce935f` requires these SCC map repairs:

| Request change | Owning package / selector / reviewed oracle |
| --- | --- |
| Replace old `tarjan-graph-core` | `codegraph-graph-scc` lib/module `scc`, all-features; source defines20 tests. Require emission order/IDs, original checkpoint error and preallocation bounds/overflow witnesses |
| Add compatibility facade | `quanta-graph-utils-v1` lib/module `facade`, all-features, min1; `facade_root_and_module_paths_share_core_types_v1` |
| Add uncovered PTA adapter | `quanta-adapters-pta` lib/module `scc_collapsing::tests`; source defines8 tests. Resolve reviewed compile policy; require one-byte-short refusal before publication, membership without constraint materialization, reverse component order and exact rewrite result |
| Reuse existing PTA engine request | `historical-d5-pta` selects `codegraph-core-pta-engine` lib/module `solver::`; re-resolve changed dependencies and require actual SCC governance witnesses, including cancel/deadline, unlaundered failure and resource refusal. No duplicate extra engine run |

Core/facade split changes45→46; adding the missing adapter changes46→47 if all
other requests remain. At `5f76eca7812`, the Tarjan gate source has ten steps,
including the PTA adapter and Andersen owners. This closes the gate-source
omission; its execution and the historical map repair remain pending. Review
added SCC consumers, including concurrency deadlocks, before freezing the final
request inventory/cardinality/witnesses together;
do not lower old minima or retain the graph-utils `scc` selector (now zero cases).

### Added SCC caller coverage

[D2-CALLER-COVERAGE](tickets.md#d2-caller-coverage) records eleven additional
package/scope pairs and their actual source paths: current-target planner,
dominance, concurrency engine/schema, navigation, distinct `codegraph-import-graph`,
`codegraph-core-cfg` call-graph SCC, rule execution, metrics and runtime
community clustering, plus abstract worklist (`90e6bb0ea3d`). The old45 map and Tarjan10 filters omit these specific
scopes; the runtime-analysis PTA request is not a planner request, and runtime
Shape selectors do not cover community clustering. The call-graph schema owner
needs exact component order, 12k-chain, one-byte-short and inconsistent
reverse-index controls; taint Zipper's criticality result is a separate consumer
oracle, not proof from an SCC type assertion. The abstract worklist additionally
needs independent cyclic-edge/deep-cycle/resource/graph-rejection oracles and
actual solve deadline/widening results; its newly canonical limits deliberately
add bounded rejection. Preexisting HIR stratification has a dedicated gate and
is not counted as a new migration.

- Discover real test targets/witnesses under the owning feature closure. Reuse
  existing accepted source-equivalent receipts where present; add only uncovered
  owner requests. Report the final count after this reconciliation, not47 as a
  whole-campaign inventory. One concurrency-engine proof covers its two consumers.
- Preserve actual domain outputs: SCC membership/order and IDs are not enough
  to prove planner partition, virtual-exit dominance, real cycle edges, import
  grouping, rule execution, max depth or community projection.
- Require the relevant malformed/unregistered graph, one-byte-short resource
  refusal, deep iterative cycle and original Stop oracles. Controlled consumers
  retain current/refund/last-drop obligations; ordinary APIs retain their own
  supported limits and error contract. No new generic adapter is assigned.
- D1 owns Shape and D2 PTA/taint keep their current owners. Serialize shared
  compile classes and avoid duplicate owner runs for overlapping proof rows.

- PTA prerequisite requests precede PTA callers; retain actual Shape feature
  combinations, DFG/taint, SCV2 and isolated solver/API absence acceptance.
- After the reconciled owner requests, re-resolve/archive the three existing Shape
  feature-tree diagnostics: `shape-tree-runtime` (`quanta-runtime`,
  `gateway-ingress-runtime-core,shape-analyzer`), `shape-tree-session`
  (`quanta-sdk-session-backend-owner`, `session-shape-analyzer-runtime`) and
  `shape-tree-composition` (`quanta-sdk-runtime-composition-owner`,
  `session-shape-analyzer-runtime`). These prove dependency/feature closure,
  not three additional Rust behavioral executions.
- The `07093dbe72a` shared Shape/Cost allocation projection adds two
  `quanta-contract-pta-result-shared` no-default feature classes `399e8a289332`
  (features[]) and `3d4e77a3e18b` (`java-allocation-metadata`). Re-resolve both
  and the actual Shape/Cost output owners. The feature-policy hash changed;
  the historical45 hash is not current approval or dependency continuity.
- Preserve every reviewed minimum and literal witness. Explicitly ignored-only
  `deep_chain_50k` is its own request; other requests exclude ignored cases.
- Reuse unchanged inputs only with an explicit source/dependency continuity
  attestation. Changed production/features/manifests require affected reruns.
- Run affected feature/dependency, test honesty/compile topology, GOD/Honesty,
  FFI/input and generated-source policies after relevant source repair. Fix
  attributable findings; no global `honesty-bless`, ratchet reapproval, new allow
  or narrowed scope to obtain green.
- Catalog/generated artifacts are updated only through their owning tools when
  the source mapping or authoritative evidence changes; no manually invented
  execution verdicts.

## Product, external installation and frontend completion

- Semantica process entries: `just test-gqlang-v3-graph-process`,
  `just test-gqlang-v3-check-linux-process`,
  `just test-gqlang-v3-unsupported-host-process`. Reconcile exact selectors and
  counts from `tools/testing/suite-registry-v1.yaml` and
  `crates/quanta-cli-server/tests/server_socket_daemon_e2e.rs`.
- Bind built artifacts and daemon/SDK executions to the frozen tuple. Validate
  actual positive results, typed refusal controls and process cleanup counts.
- Installed positive CHECK needs the release-owned signed x86_64 Linux profile,
  root-owned immutable slots, independent approval/key and active generation.
  Keep FCP-INSTALL incomplete while unavailable. macOS refusal or generic
  aarch64 fixtures cannot substitute for that installed positive result.
- QGLang source changes require the smallest affected test, then `just ci`
  once for repository-wide local completion. Prose-only planning does not claim
  a new CI result. `just qualify-local` is only for clean committed HEAD.

Receipt acceptance distinguishes source integration, focused owner proof,
feature/policy proof, product process proof and installed qualification. Close
only the corresponding remaining-work row; unavailable prerequisites are not
successful executions.
