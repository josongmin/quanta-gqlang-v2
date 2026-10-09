# Modules / Logic final audit

- Source: QGLang HEAD `25fad76cf5b4510244157c2e009b4dc37aa958ff`, tree `99501ba1aabfa37361fccfb7dee388cd91055ee1`. Relevant Rust paths have no working-tree diff. Other workers' dirty documentation was not edited.
- Scope: `modules` View topology, final catalog family-edge merge, `REQUIRE CERTIFIED` reachability, indexed SCC/DAG, Logic recursion and lockfile. This is a source-level audit. No new Cargo test, benchmark, `just ci`, or downstream runtime check was run in this final pass.

## Confirmed findings

| ID | Owner and reachable path | Counterexample and failure mode |
| --- | --- | --- |
| [M04-01](M04-01-eager-visible-scan.md) | `catalog/view_topology.rs:189-197`, `DocumentCatalogValidatorV2::validate_view_topology` via request owner | A local target already resolved by `predeclared.get` still triggers a full visible-binding scan. `E` local edges and `B` unrelated visible bindings cause up to `E×B` comparisons outside the edge charge. No latency/RSS measurement. |
| [M04-02](M04-02-sorted-vec-insert.md) | `catalog/view_topology.rs:238-271`, same request owner | Reverse-key unique direct View edges insert at index zero in a sorted Vec, shifting `E(E-1)/2` dependency values. The edge budget charges each edge once; it does not bound these shifts. |
| [MF-01](MF-01-family-edge-insert.md) | `catalog.rs:1064-1156`, `validate_predeclared_with_request_owner_v3` → `try_new_predeclared` → `merge_family_references` | With an empty direct topology and `F` valid View→Query family references authored in reverse target-ID order, the final merge independently shifts `F(F-1)/2` dependency values. This is a second phase/owner site of the M04-02 pattern; fixing topology alone leaves it open. |
| [MF-02](MF-02-certified-reachability-insert.md) | `catalog.rs:915-928,1159-1218`, final catalog with `REQUIRE CERTIFIED` | `N` sorted direct View edges targeting distinct external Views enter `queue` ascending; LIFO pop visits targets descending and `seen.insert(0, id)` shifts `N(N-1)/2` references. This work has no per-visit step charge. |

The counterexample arithmetic was checked independently by binary-search insertion for `N=4` (6 shifts) and `N=4096` (8,386,560 shifts); `N=50,000` gives 1,249,975,000 by formula only. These are operation counts, not measured performance or proof that every size is admitted by the complete parser/host. Small valid typed paths are established by the caller and existing family-reference fixtures; no new end-to-end fixture was executed here.

## Checked without a new confirmed correctness defect

- `graph/indexed_scc.rs:57-137` rejects absent/out-of-range edge indices before labeling and propagates first callback refusal. Logic `recursion.rs:729-775,789-823` maps malformed membership to typed errors and charges edge scans before selection. The earlier [lane report](02-modules-logic.md) records focused SCC/Logic tests and their limits.
- `graph/indexed_walk.rs:37-97` checks target/root indices and preserves root multiplicity; `manifest.rs:533-620` maps the count into the owner budget. Existing independent small-graph oracle is in `graph/indexed_walk/tests.rs`. No large-graph CPU/RSS measurement was run.
- `catalog/view_topology.rs:360-447` uses a min-heap for canonical dependency-first order. `lockfile.rs:174-264` parses at most four fields per line and checks canonical order after field validation. Existing focused checks are in the [lane report](02-modules-logic.md); this pass adds no new execution result.

## Repair and verification boundary

- Fix M04-02 and MF-01 together in their respective phases, preserving authored-first duplicate/kind error and final canonical dependency order. Fix MF-02 with indexed membership or a collect/deduplicate traversal that retains certification reachability and the same error precedence. M04-01 needs a lazy fallback lookup after local/external misses.
- For each, add a small independent semantic oracle plus a reverse-order work counter or comparable same-SHA workload measurement. Run affected Modules owner tests, then one repository `just ci` after the coupled edits. Installed product, downstream integration, and benchmark claims remain `NOT_RUN` in this audit.
