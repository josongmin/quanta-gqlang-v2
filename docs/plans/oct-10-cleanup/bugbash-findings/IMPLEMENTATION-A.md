# Bundle A: Modules cost bounds

## Source and owner

- Source baseline: QGLang `25fad76cf5b4510244157c2e009b4dc37aa958ff`; edits are uncommitted in the shared checkout. This file records only the Modules write set.
- Consumer path: `DocumentCatalogValidatorV2::validate_view_topology_with_request_owner_v3` → Modules topology; `validate_predeclared_with_request_owner_v3` → Modules final catalog. Legacy V2 entry points use the same owners.
- Intended behavior: no change to accepted/rejected catalog facts, public dependency order, or first authored diagnostic. Internal work for the four confirmed findings changes from repeated linear scan/Vec shift to one sort plus logarithmic membership lookup. No public contract version change.

## Changes

| Finding | Owner change | Bound and invariant |
| --- | --- | --- |
| M04-01 | `catalog/view_topology.rs`: visible bindings are queried only after both predeclared and external target miss. | Local/external hit executes zero visible scans by the explicit branch guard. Local → external → visible precedence and wrong-kind target location remain as before. No runtime scan counter was instrumented. |
| M04-02 | Same file: build a fixed-length, sorted borrowed input-key index, validate authored edges in input order, append unique sealed edges, then sort retained output once. | `O(E log E)` key comparison plus append and sort; no ordered `Vec::insert`. Idempotent duplicate keeps minimum source location; conflict is diagnosed at the authored conflicting edge. |
| MF-01 | `catalog.rs::merge_family_references`: fixed-length sorted lookup includes direct topology and family keys, with direct edge first for equal keys; process references in authored order and sort output once. | `O((D+F) log(D+F))`; same direct-vs-family conflict and canonical final dependency order. |
| MF-02 | `catalog.rs::reachable_external_profiles`: pre-index all possible initial/external-edge target IDs, sort/deduplicate once, then binary search and flip one visited bit per pop. | `O(Q log Q)` index build and `O(log Q)` membership per visit, where `Q` is the already bounded queue-slot count. Profile result still sorted by ID before certification checks. |

`request_topology_bytes_v3` adds scratch for the edge index and retained capacity for cloned source/target identity strings held concurrently with the borrowed input lookup. `request_final_catalog_bytes_v3` adds scratch for the family index and widens visited slots from one ID reference to `(ID reference, bool)`. Checked arithmetic retains typed demand overflow. These are conservative capacity charges; no allocator-level peak RSS claim is made.

## Oracles and verification

- Added `reverse_unique_catalog_edges_seal_in_id_order_and_keep_earliest_duplicate`: 48 reverse-key unique direct edges plus a late duplicate with an earlier source location; expected 48 canonical targets and the independent literal span `70`.
- Added `reverse_unique_family_edges_seal_in_id_order_and_keep_earliest_duplicate`: same independent target-order and duplicate-location oracle for final family merge.
- Extended `require_certified_traverses_activated_external_pack_contracts`: an activated pack links 48 distinct certified profiles in sorted ID order, so LIFO traversal visits descending IDs. Existing Allowlist refusal and CertifiedOnly acceptance remain the expected outcomes.
- Existing local View edge oracle: `reverse_authored_views_materialize_in_stable_dependency_first_order`; existing external direct edge oracle: new `reverse_unique_catalog_edges_seal_in_id_order_and_keep_earliest_duplicate`. Existing visible reference semantics: `imported_path_is_visible_and_local_collision_is_ambiguous` and `optional_catalog_probe_keeps_absent_host_names_open_but_rejects_wrong_kind`. The latter do not measure direct-topology visible-lookup calls. Zero scans on local/external hits follow from the source branch guard, not a measured count.
- `cargo fmt -p quanta-gqlang-v2-modules`: PASS. `git diff --check -- crates/quanta-gqlang-v2-modules`: PASS.
- Focused Modules tests: PASS after fixture correction. `v3_document_predeclaration reverse_unique`: 2/2 passed; `v3_modules_v2_authority require_certified_traverses_activated_external_pack_contracts`: 1/1 passed; `v3_request_topology_final_storage topology_acyclic_and_scc_witness_admit_exact_peak_and_refuse_one_byte_short`: 1/1 passed. These are four executions across three focused filters. Initial run compiled but both reverse-unique fixtures failed at setup with `modules.invalid_catalog_name` because `query.000` is not a valid catalog local name. The fixture was corrected to `query_000`…`query_047` (and `policy_000`…`policy_047` for the activated-pack path), then the same focused filters passed. This was test-input repair; production source was unchanged.
- First combined `just ci` stopped at `size-lint` because `catalog.rs` reached 1221 code lines against the 1200-line cap; it ran zero tests. `merge_family_references` was moved intact into `catalog/family_merge.rs`; `just size-lint` then passed (292 Rust source files). A subsequent CI retry stopped at Clippy with explicit-import, `map_err`, comparison, and indexing findings; `just clippy-current` passed after the targeted repair.
- Final combined `just ci`: PASS at source snapshot `47e3fcf41a4f54f5704ecb1bb8fce35c2261dffc6d679bfae4e8c30c8cb7e9e8`, Nextest 1531/1531 selected/executed/passed with zero skip/leak. Representative CPU/peak RSS comparison is recorded below. Product/runtime and downstream qualification: NOT_RUN.

## Remaining risk

- Index setup sorts the complete authored input before the per-edge budget loop. It is bounded by the admitted input vector length; an over-budget legacy caller incurs this sort before the first budget refusal. Result code and exact/one-short counters are unchanged, but rejection CPU work has not been measured.
- Borrowing all topology inputs through the lookup keeps their original strings alive until sealing. The extra retained string copy is charged by the request census; the process-wide RSS measurement below does not isolate allocator peak in this owner.

## Isolated representative measurement

Baseline and patched sources were placed in separate detached temporary worktrees at commit `25fad76cf5b4510244157c2e009b4dc37aa958ff` (tree `99501ba1aabfa37361fccfb7dee388cd91055ee1`). The patched worktree applied the Modules diff with SHA-256 `549315102b8d9f87e29d08109efa39978ffabb1a3fa425c8a1b27acc6d922435` plus new `catalog/family_merge.rs` with SHA-256 `77e8b903f81ec9ade993ae13bf553e242c0192514bae092f704a231e1b86f0ea`. The two worktrees used byte-identical benchmark test sources (SHA-256: topology `e84a220ac03044c812f9fb4907cbfed384d1eb1972710bdce5288e2c7f71cbe4`, family `d320956dfacaeb84f8f943b6916d1215fd23dbf7f941addebc7c2364e7d56ece`, certified `c6b12fdc9e5ad0a10dffbb2fc1bfa5a6dc81592b229ddf8c07789b3832bfaa67`). All measured invocations ran one test and passed the same acceptance/rejection and dependency-count/order assertions. Temporary harnesses were not added to the shared checkout.

Host: macOS arm64, Rust/Cargo 1.95.0; `cargo test --locked --no-run` debug test binaries built serially with `CARGO_BUILD_JOBS=2`. Each binary ran directly with `--exact ... --nocapture --test-threads=1` under `/usr/bin/time -l`, alternating baseline/patched. One warm-up per source/case was excluded. `owner ms` is `Instant` around the named Modules API; wall/user CPU and peak RSS describe the **whole test process**, including fixture setup. Reported values are medians; wall/user resolution is 0.01 s.

| Input / timed API | Repeats per source | Baseline owner ms | Patched owner ms | Baseline whole-process wall / user s | Patched whole-process wall / user s | Baseline / patched peak RSS MiB |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 4,096 reverse-key local Query edges; V3 request-owned `validate_view_topology_with_request_owner_v3` | 5 | 39.007 | 17.251 | 0.05 / 0.04 | 0.02 / 0.02 | 8.28 / 8.47 |
| 4,096 reverse-key external Query family references; V2 `validate_predeclared` | 5 | 36.462 | 19.607 | 1.76 / 1.74 | 1.75 / 1.71 | 15.25 / 14.86 |
| Activated pack → 4,096 external Profiles; V2 `validate_predeclared`, Allowlist refusal | 3 | 7.572 | 7.847 | 0.16 / 0.16 | 0.17 / 0.16 | 19.19 / 19.03 |
| Same pack, CertifiedOnly acceptance | 3 | 7.881 | 7.712 | 0.16 / 0.16 | 0.17 / 0.16 | 19.19 / 19.03 |

The 4,096-edge topology fixture combines M04-01 local-hit and M04-02 reverse insertion; it does **not** contain a large visible-binding inventory, so it cannot isolate the removed M04-01 scan. Topology and family timed owner calls are faster in this workload; the certified path has no clear latency improvement at 4,096 profiles even though its quadratic insertion has been removed. Peak RSS is process-wide, not owner allocation. The first temporary family run used three-digit IDs, which invalidated its expected lexical order at 1,000; the fixture was corrected to four digits before these measurements. An initial certified command selected zero tests because its module-qualified test name was omitted; those zero-test outputs were excluded.

Timed owner-call samples in ms, in run order after warm-up: topology baseline `[40.996, 38.771, 41.474, 39.007, 38.754]`, patched `[17.199, 17.329, 17.251, 17.532, 17.113]`; family baseline `[36.000, 36.462, 42.019, 36.438, 61.137]`, patched `[17.853, 19.959, 17.947, 24.489, 19.607]`; certified refusal baseline `[7.572, 7.473, 7.684]`, patched `[7.847, 7.753, 7.940]`; certified acceptance baseline `[7.654, 8.195, 7.881]`, patched `[7.324, 8.142, 7.712]`.
