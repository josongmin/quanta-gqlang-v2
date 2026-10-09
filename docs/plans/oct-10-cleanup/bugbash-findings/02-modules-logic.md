# 02 Modules / Logic: SCC, DAG, View topology, lock

Source: QGLang `25fad76cf5b4510244157c2e009b4dc37aa958ff` (tree `99501ba1aabfa37361fccfb7dee388cd91055ee1`). The shared checkout had unrelated dirty ADR/plan documents; this file is the only edit in this lane. Rust source was read and focused tests were run at the pinned HEAD. Hosted, installed, allocation, and latency proof remain `NOT_RUN`.

## Confirmed source defects

| ID | Classification and owner | Reachable path, counterexample, expected / actual | Impact and repair criterion |
| --- | --- | --- | --- |
| M04-01 | `CONFIRMED_DEFECT` (unnecessary quadratic work); `modules/catalog/view_topology.rs` | `DocumentCatalogValidatorV2::validate_view_topology_with_request_owner_v3` → `DocumentViewTopologyProofV2::try_new` (`authority/request_catalog.rs:208-252`, `catalog/view_topology.rs:150-174,188-201`). One edge from local View A to local View B plus `B` unrelated visible bindings is enough: the local lookup already resolves B, yet `visible.visible_bindings().iter().find(...)` scans all `B` bindings before `local_target.or_else(...)` uses the local result. With `E` local edges this performs `E × B` binding comparisons. Independent expectation: a resolved local/external target requires zero visible lookup; actual: every edge performs it. `charge_edge` is once per edge and does not charge these comparisons. | Admitted graph work can grow quadratically despite a linear edge budget. Make visible lookup lazy after local and external miss, ideally use indexed identity lookup for visible misses. Preserve local > external > visible precedence and wrong-kind diagnostic location. Verify local-hit scan count stays zero and missing-local lookup returns the same diagnostic. |
| M04-02 | `CONFIRMED_DEFECT` (unnecessary quadratic work); `modules/catalog/view_topology.rs` | Same caller → `try_new` (`catalog/view_topology.rs:243-271`). Submit unique valid dependencies in reverse `(source_asset_id, target_asset_id)` order. For `E` edges, `dependencies.binary_search_by` finds insertion index 0 each time and `Vec::insert` shifts `0 + 1 + ... + (E-1) = E(E-1)/2` `DocumentCatalogDependencyV2` values. Independent expectation: canonical output order with work proportional to input plus sorting, and the first authored duplicate/kind error retained. Actual: ordered insertion performs quadratic copies; only `charge_edge` runs once per edge. | At 50,000 edges the path entails about 1.25 billion element shifts before cycle analysis, within the standard 200,000-edge limit (`sema/authority/budget.rs:303-326`). Collect edges, validate duplicates in authored order with an index, then sort once; keep earliest-location replacement and duplicate-kind precedence. Add a reverse-order scaling check with exact source/profile and a small semantic oracle. |

These are code-structure findings. Their operation counts follow directly from the loop and container operations. An isolated Python enumeration for `E=4, B=3` independently produced 6 shifts and 12 comparisons, matching `E(E-1)/2` and `E×B`. The same formulas give 8,386,560 shifts and 16,777,216 comparisons at `E=B=4096`, or 1,249,975,000 shifts and 2,500,000,000 comparisons at `E=B=50,000`. These are **operation counts for constructed inputs**, not measured latency, RSS, or proof that those sizes arise in a parsed document. Neither fix changes public DTO bytes by itself, but it must preserve diagnostic and dependency ordering.

## Checked paths without a confirmed semantic defect

| ID | Classification | Source evidence and independent expectation | Remaining proof |
| --- | --- | --- | --- |
| M04-03 | `PROOF_GAP` only for measured peak | `graph/indexed_scc.rs:55-140` validates target indices before indexing and propagates the first callback refusal; `graph/indexed_scc/tests.rs:17-52` has an independent 3-vertex mutual-reachability oracle across all 512 graphs and reversed edge order. `logic/recursion.rs:733-775` maps malformed edges to typed membership errors with authored edge location; `:786-833` scans all projected edges before negation/aggregate selection; `:345-382` gives negation precedence. Focused SCC/Logic tests passed, including 50k-deep chain and exact step charge. | No measured 50k-node peak RSS or hosted/product qualification; capture only if making that claim. |
| M04-04 | `PROOF_GAP` only for measured performance | `graph/indexed_walk.rs:45-106` validates dangling targets and roots, retains root multiplicity, and returns `None` for a cycle; `graph/indexed_walk/tests.rs:31-75` compares every 3-vertex graph, dangling-edge subset, and selected root multisets against independent path enumeration. `manifest.rs:533-620` and `graph.rs:742-795` convert the count to the caller's exact/one-short step policy. Focused walk and caller budget tests passed. | No measured large-graph CPU/RSS or hosted/product qualification. |
| M04-05 | `PROOF_GAP` only for measured allocation | `catalog/view_topology.rs:358-448` uses a min-heap of local View IDs and emits each only after all local View dependencies; the permutation and reverse-authored fixtures passed. `lockfile.rs:95-103,165-262` serializes sorted `BTreeMap` pins and parses no more than four fields per line; byte, late-error, and span fixtures passed. | Large all-space-line allocation/RSS trace remains `NOT_RUN`; existing focused tests do not establish measured peak allocation. |

## Executed focused checks

All commands used `--locked` at the pinned HEAD. Counts below are selected/executed/passed/skipped; `0` failures for every row. The 62 executions include overlapping filters and are **not** 62 independent test cases. The initial `--lib manifest::tests` filter selected 0 tests; it was corrected to `cycle_walk_tests` and is excluded from the total.

| Command suffix after `cargo test --locked` | Selected / executed / passed / skipped |
| --- | ---: |
| `-p quanta-gqlang-v2-modules --lib graph::indexed_scc::tests` | 4 / 4 / 4 / 0 |
| `-p quanta-gqlang-v2-modules --lib graph::indexed_walk::tests` | 3 / 3 / 3 / 0 |
| `-p quanta-gqlang-v2-modules --lib cycle_walk_tests` | 7 / 7 / 7 / 0 |
| `-p quanta-gqlang-v2-modules --test v3_modules_v2_authority lock_` | 18 / 18 / 18 / 0 |
| `-p quanta-gqlang-v2-modules --test v3_modules_v2_authority cycle` | 7 / 7 / 7 / 0 |
| `-p quanta-gqlang-v2-modules --test v3_modules_v2_authority document_view` | 2 / 2 / 2 / 0 |
| `-p quanta-gqlang-v2-modules --test v3_document_predeclaration topology_` | 2 / 2 / 2 / 0 |
| `-p quanta-gqlang-v2-modules --test v3_document_predeclaration reverse_authored_views_materialize_in_stable_dependency_first_order` | 1 / 1 / 1 / 0 |
| `-p quanta-gqlang-v2-logic --lib partition_tests` | 1 / 1 / 1 / 0 |
| `-p quanta-gqlang-v2-logic --test v3_logic_recursion` | 17 / 17 / 17 / 0 |
| **Total executions** | **62 / 62 / 62 / 0** |

## Integration boundary

- Fix M04-01/02 in `catalog/view_topology.rs` with one owner change; coordinate with the Modules and request-budget owners before editing shared `graph.rs` or public budget profiles.
- No behavior change, source fix, benchmark, or downstream Semantica qualification was performed in this audit lane. Focused tests do not close repository-wide `just ci` or the two View topology findings.
