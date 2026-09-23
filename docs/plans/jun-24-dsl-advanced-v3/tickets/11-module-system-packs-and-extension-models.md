# 11 Module System Packs And Extension Models

> Historical 2026-06-24 snapshot. “Current Main” below refers to that dated
> source, not today's repository. Use the [current language authority](../../../language/README.md).

Status: `Materially Implemented On Current Main`

## Current Main Receipt

As of `2026-06-24`, the owner-local module/pack/extension seam is live:

1. `quanta-gqlang-v2-modules` owns the canonical module/import graph, pack
   manifests, lockfiles, metadata, signatures, visibility, and declarative model
   packs
2. deterministic graph rules are pinned:
   local shadowing, direct-import precedence over re-export, ambiguity rejects,
   and import-cycle rejects
3. pack validation is lock-backed and fail-closed:
   compatibility windows, dependency ranges, lock mismatches, and overlapping
   exports all reject with typed diagnostics
4. declarative model packs are schema-validated, additive, and fail closed on
   conflicting facts or invalid attributes
5. catalog/meta seams are wired:
   `PACK`/`PROFILE`/`GATE`/`RULEDEF` registration, `USE PACK`, and
   `REQUIRE CERTIFIED` all resolve through typed modules diagnostics
6. proof rails:
   `v3_modules_graph.rs`,
   `v3_modules_packs.rs`,
   `v3_modules_hellgate.rs`,
   `v3_catalog_hellgate.rs`,
   `crates/quanta-gqlang-v2-lsp/tests/v3_diagnostics_parity_hellgate.rs`
7. verified with:
   `cargo test -p quanta-gqlang-v2-modules`,
   `cargo test --workspace`,
   `cargo clippy --workspace --all-targets`

Honest residual:

1. current LSP/CLI consume this lane mainly through diagnostics, plan summaries,
   and pack/catalog seams; there is no separate module-browser or pack-manager UX
   in this packet
2. runtime/provider-side pack activation and execution remain downstream and out
   of scope

## Goal

Make `v3` reusable and extensible without forcing core edits or query
copy-paste.

## Why

Core semantics alone are insufficient for a CodeQL-class authoring experience.
SOTA query systems also provide:

1. reusable modules and imports
2. parameterized interfaces/signatures
3. sharable query/library packs
4. explicit extension points for framework and dependency modeling
5. metadata and suite boundaries so queries can be tested, distributed, and run
   predictably

Without that layer, the language can be expressive on paper while remaining
operationally weak and painful to extend.

Current head already ships `quanta-gqlang-v2-modules` as the owner lane. This
ticket closes and broadens that lane; it does not authorize a second pack or
module graph implementation.

## Deliverables

1. module/import semantics over shared HIR and libraries
2. parameterized signatures/interfaces for reusable query abstractions
3. pack manifest format:
   - package identity
   - versioning
   - dependency ranges
   - language/feature compatibility range
   - suite membership
4. module visibility/export rules
5. query metadata model:
   - kind
   - severity/truth class
   - tags/categories
   - help/doc hooks
6. extension model seam for framework/domain modeling:
   - frontend-owned and host-agnostic
   - declarative data-extension friendly where possible
   - additive, versioned, and testable
7. declarative model-data format and validation rules for extension packs
8. lock-backed pack/suite resolution rails for CLI and LSP workflows
9. overlapping-pack conflict detection and diagnostic rules
10. canonical module/import graph rules:
    - import shadowing behavior
    - re-export precedence
    - ambiguous import diagnostics
    - module cycle policy

## Pinned Semantics

1. modules are the reuse boundary; copy-pasted query fragments are not an
   acceptable extension story
2. signatures/interfaces must type-check module reuse explicitly
3. extension models may add host-agnostic semantic facts or library models, but
   may not import runtime substrate DTOs or backend wiring
4. extension stories should prefer declarative model data over mandatory core
   code edits whenever semantics permit
5. query metadata is part of the authoring contract, not an optional comment
6. pack resolution and suite selection must be deterministic
7. package resolution must be reproducible offline from manifest plus lock data
8. extension packs cannot override closed core operators, types, or planning
   truth rules
9. overlapping model packs must either compose by an explicit rule or reject;
   silent last-wins behavior is forbidden
10. textual import order is never semantic authority
11. module dependency cycles reject unless a future ticket explicitly defines
    cycle semantics

## Non-Goals

1. this ticket does not move runtime/provider ownership into this repo
2. this ticket does not define semantica-native execution plans
3. this ticket does not weaken the closed builtin/operator/type core; it defines
   extension seams above that core

## Scenarios Unlocked

1. an org-local security pack imports shared auth/dataflow helpers instead of
   cloning predicates across queries
2. a framework model pack extends frontend semantics for a library family
   without changing the core language
3. CLI/LSP can run a stable suite and show metadata/help for a query or rule
4. two developers resolve the same pack graph and get identical query suite
   membership and diagnostics
5. two packs attempt to model the same framework symbol incompatibly and the
   resolver rejects with an actionable conflict
6. a module re-export and a direct import expose the same symbol and resolution
   still picks one deterministic rule or rejects

## Definition Of Done

1. reusable query/module boundaries exist in syntax, binder, and tooling
2. packs and suites can be resolved deterministically
3. extension models are explicit, host-agnostic, schema-validated, and
   test-covered
4. `v3` extensibility no longer depends on core edits for every new framework or
   query family
5. dependency resolution is lock-backed and reproducible across machines
6. conflicting pack graphs fail closed with deterministic diagnostics
7. import shadowing, re-export precedence, ambiguous imports, and module cycles
   are all pinned by tests and no longer left to implementation choice

## Dependencies

1. requires `00`
2. depends on `01`, `02`, and `03`
3. closes with `08` and `09`

## Likely Touch Points

1. `crates/quanta-gqlang-v2-modules/src/manifest.rs`
2. `crates/quanta-gqlang-v2-modules/src/graph.rs`
3. `crates/quanta-gqlang-v2-modules/src/lockfile.rs`
4. `crates/quanta-gqlang-v2-modules/src/metadata.rs`
5. `crates/quanta-gqlang-v2-modules/src/extension.rs`
6. `crates/quanta-gqlang-v2-modules/src/signature.rs`
7. `crates/quanta-gqlang-v2-modules/src/visibility.rs`
8. `crates/quanta-gqlang-v2-binder/src/module_graph.rs` only if binder-facing
   consumption of the resolved graph changes

## Mandatory Worker Outputs

1. deterministic module/import graph rules with no import-order authority
2. manifest plus lockfile story for reproducible pack resolution
3. explicit conflict diagnostics for overlapping imports and overlapping model
   packs

## Proof Rails

1. extend `crates/quanta-gqlang-v2-modules/tests/v3_modules_packs.rs`
2. extend `crates/quanta-gqlang-v2-modules/tests/v3_modules_graph.rs`
3. extend `crates/quanta-gqlang-v2-modules/tests/v3_modules_hellgate.rs`
4. extend `crates/quanta-gqlang-v2-modules/tests/v3_catalog_hellgate.rs`
5. add at least one negative rail for ambiguous import or re-export conflict

## Reject Or Escalate

1. silent last-wins behavior for imports or model packs
2. module cycle acceptance without an explicit new semantic decision
3. runtime/provider wiring hidden inside pack extension stories
