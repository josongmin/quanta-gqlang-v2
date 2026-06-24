# DSL Advanced v3 Execution Playbook

## Purpose

Turn each ticket in this packet into an executable worker order with consistent
proof and handoff discipline.

Current-main note:

1. tickets `12` through `15` are now implemented on current `main`
2. the old packet red rails for those seams have been promoted to green lock
   rails

## Read Order

1. [00-packet-scope-and-pinned-decisions](./00-packet-scope-and-pinned-decisions.md)
2. this playbook
3. the assigned ticket
4. that ticket's dependency tickets
5. [docs/language/README.md](../../../language/README.md)

## Mandatory Worker Receipt

Every worker closeout for this packet must report:

1. ticket id and dependency tickets read
2. exact files touched
3. exact tests added or extended
4. exact commands run, including the canonical command set below
5. reject paths that still fail closed, each with the actual typed diagnostic
   text/code pasted from the run (not a claim that it rejected)
6. unresolved follow-ups that remain out of lane

### Canonical Command Set

D22 requires one shared compiler pipeline, so "green" is only credible against a
fixed command set. Every closeout runs and reports:

1. `cargo build --workspace` clean
2. `cargo test -p <owning-crate>` for each crate the ticket touched
3. `cargo test --workspace` for any change crossing crate boundaries
4. `cargo fmt --all -- --check` clean
5. `cargo clippy --workspace --all-targets` with no new warnings

A closeout that claims green without this set is not accepted.

## Default Proof Ladder

Choose the narrowest rails that prove the owning seam, then add cross-layer
rails if the change crosses crates.

1. crate-local contract rail in the owning crate
2. cross-layer surface->hir->binder/types->algebra/logic/planning rail when
   semantics cross boundaries
3. CLI/LSP parity rail if user-visible output changes
4. docs/examples/conformance rail if public examples or docs change

### New Rail Convention

This packet closes the live `v3` language core, so workers normally extend the
current rail map or add narrowly-owned new rails. New rails obey:

1. location: `crates/<owning-crate>/tests/` for the crate in the ownership
   heuristic; never in a sibling crate's tests dir
2. name: `v3_<seam>_<facet>.rs`, e.g.
   `v3_hir_expr_kernel.rs`, `v3_binder_scope.rs`, `v3_algebra_normalization.rs`,
   `v3_logic_recursion.rs`, `v3_planning_truth.rs`
3. hellgate (negative/fail-closed) rails name the reject explicitly:
   `v3_<seam>_hellgate.rs`, and each test asserts the actual typed diagnostic
   text/code, not just that an error occurred
4. cross-layer rails live in the lowest crate that owns the asserted seam, and
   state the surface->hir->binder/types->algebra/logic/planning path they pin
   in a module-level comment
5. one seam per file; do not append v3 cases into a v2 inventory file that
   ticket 09 is scheduled to delete

## Current Repo Rail Inventory

Current head already runs on the `v3` crate map. Reuse these live rails before
inventing new files. Rail tags:

- `EXTEND`: live owner-local semantic rail; extend in place unless ownership moves
- `LOCK`: previously open seam now closed; keep the rail green and strict
- `SUPPORT`: supporting transport/conformance/doc rail; useful but not semantic closure alone

1. `surface`
   `v3_surface_query.rs` (EXTEND),
   `v3_surface_expr_kernel.rs` (EXTEND),
   `v3_surface_hellgate.rs` (EXTEND),
   `v3_document_hellgate.rs` (EXTEND),
   `v3_format_hellgate.rs` (EXTEND),
   `v3_family_hellgate.rs` (SUPPORT)
2. `hir`
   `v3_hir_expr_kernel.rs` (EXTEND),
   `v3_hir_hellgate.rs` (EXTEND),
   `v3_item_hellgate.rs` (EXTEND)
3. `binder`
   `v3_binder_scope.rs` (EXTEND),
   `v3_binder_hellgate.rs` (EXTEND),
   `v3_composition_hellgate.rs` (EXTEND)
4. `types`
   `v3_types_universe.rs` (EXTEND),
   `v3_types_hellgate.rs` (EXTEND)
5. `modules`
   `v3_modules_packs.rs` (EXTEND),
   `v3_modules_graph.rs` (EXTEND),
   `v3_modules_hellgate.rs` (EXTEND),
   `v3_catalog_hellgate.rs` (EXTEND)
6. `algebra`
   `v3_algebra_normalization.rs` (EXTEND),
   `v3_algebra_hellgate.rs` (EXTEND),
   `v3_algebra_probe.rs` (SUPPORT)
7. `logic`
   `v3_logic_recursion.rs` (EXTEND),
   `v3_logic_source.rs` (EXTEND),
   `v3_logic_witness.rs` (EXTEND),
   `v3_logic_hellgate.rs` (EXTEND),
   `v3_graph_query_hellgate.rs` (EXTEND)
8. `planning`
   `v3_planning_truth.rs` (EXTEND),
   `v3_planning_hellgate.rs` (EXTEND),
   `v3_planning_cost_schema.rs` (EXTEND),
   `v3_item_hellgate.rs` (EXTEND)
9. `pipeline`
   `v3_pipeline_compile.rs` (EXTEND),
   `v3_pipeline_hellgate.rs` (EXTEND),
   `v3_surface_to_algebra_group_by.rs` (EXTEND),
   `v3_surface_to_algebra_order_by.rs` (EXTEND),
   `v3_composition_catalog_scope.rs` (EXTEND),
   `audit_18_families.rs` (SUPPORT),
   `v3_alias_type_closure_gap.rs` (LOCK, tickets `12`/`13`),
   `v3_normalize_item_decl_rule.rs` (LOCK, tickets `14`/`15`)
10. `cli`
    `v3_cli_fmt_idempotence.rs` (EXTEND),
    `v3_cli_plan_summary.rs` (EXTEND),
    `v3_cli_lsp_parity.rs` (EXTEND),
    `v3_cli_pipeline_hellgate.rs` (EXTEND),
    `v3_cli_item_surfacing.rs` (EXTEND),
    `command_doc_gate.rs` (SUPPORT)
11. `lsp`
    `v3_diagnostics_parity_hellgate.rs` (EXTEND),
    `v3_diagnostics_determinism.rs` (EXTEND),
    `stdio_frame_edges.rs` (SUPPORT),
    `stdio_json_escape_negative.rs` (SUPPORT),
    `stdio_byte_robustness_hellgate.rs` (SUPPORT),
    `stdio_command_plane_hellgate.rs` (SUPPORT)
12. `conformance`
    `self_validation.rs` (SUPPORT)

Rules:

1. never delete a live `v3` rail to make a lane look green
2. once a seam closes, its lock rails stay green and strict
3. `audit_18_families.rs` is breadth proof only; it is not semantic-closure proof
4. transport/doc gates do not replace owner-local semantic rails

## Ownership Heuristic

1. grammar, tokens, spans, formatter surface:
   `quanta-gqlang-v2-surface`
2. shared items, expressions, relations, result-shape carriers:
   `quanta-gqlang-v2-hir`
3. binding and scope resolution:
   `quanta-gqlang-v2-binder`
4. typing and operator legality:
   `quanta-gqlang-v2-types`
5. module, pack, manifest, signature, and model metadata:
   `quanta-gqlang-v2-modules`
6. normalized relational semantics:
   `quanta-gqlang-v2-algebra`
7. recursive predicate, graph, and witness semantics:
   `quanta-gqlang-v2-logic`
8. plan, truth, evidence, effect, and cost semantics:
   `quanta-gqlang-v2-planning`
9. cross-layer compile and public diagnostic seam:
   `quanta-gqlang-v2-pipeline`
10. command/output surfaces:
   `quanta-gqlang-v2-cli`
11. editor/read-only analysis surfaces:
   `quanta-gqlang-v2-lsp`
12. downstream facade and public contract checks:
   `quanta-gqlang-v2-frontend-contract`
13. docs-only guards and conformance checks:
   `quanta-gqlang-v2-conformance`

## Escalate Instead Of Guessing

Stop and document the blocker if any of the following becomes necessary:

1. runtime substrate nouns or backend wiring inside this repo
2. a second parser, binder, or type authority
3. a second nested-result shape tree outside `01`
4. import-order-sensitive resolution or silent pack override
5. serving claims that need semantica runtime proof instead of local frontend
   proof
