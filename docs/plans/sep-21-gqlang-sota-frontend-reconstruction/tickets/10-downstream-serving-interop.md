# Ticket 10 — Downstream serving interop after the V2 facade cut

> Current routing, 2026-09-23: this ticket's V2 producer backlog and source pins are historical. The breaking V3 frontend producer implementation and exact file inventory are owned by [Ticket 10a](./10a-v3-frontend-execution-contract.md). This ticket remains cross-repository background; its older V2 acceptance text does not supersede Ticket 10a or current source.

Status: `PARTIAL — CLOSED AUTHORITY BUILDER GREEN; LOGICAL SERVING VIEW, CONSUMER CUTOVER, AND QUALIFICATION OPEN`
Snapshot: GQLang `9a23f937c8d83dfc83cb6ae59e531037d83c5072`, clean isolated worktree, 2026-09-22.
Consumer snapshot: Semantica qualification worktree `e850efb71a8` (re-freeze before work); its pre-existing `Cargo.lock` is dirty. This ticket is a cross-repository handoff, **not** a change to the frontend-only ownership rule in Ticket 00. It does not make runtime execution, physical planning, or provider wiring GQLang responsibilities.

## Failure and causal boundary

- Semantica's `packages/analysis/quanta-v2/crates/quanta-gqlang-v2-exec-contract/src/program_request.rs:30-34` imports deleted V1 facade symbols, including `CompiledProgramV1`, `AlgebraNodeV1`, raw HIR, `PublicPlanV1`, and `format_item`. The same V1 dependency spans its composition and host-adapter lowering owners. The immutable AG7 qualification receipt at `~/Library/Caches/semantica-codegraph-v2-target/quanta-build-cli/_state/execution-roots/semantica-codegraph-v2-0a0cfdc87cc4/lanes/suite-registry-runtime-envelope-digest-canonicality-owner/verification-results/20260922T084502.222703Z-c82b6c0ff710/receipt.json` records exit 101; sibling `stderr.log` records 27 unresolved-import/path errors and the `quanta-gqlang-v2-exec-contract` compile failure. This is an observed dependency mismatch, not evidence that AG7 product behavior failed at runtime.
- The V2 facade (`crates/quanta-gqlang-v2-frontend-contract/src/lib.rs:3-41`) intentionally exposes three source-document functions and a curated result allowlist. `tools/test_frontend_external_consumer.py:121-154` explicitly rejects the missing V1 names. Re-exporting V1 to silence the errors would violate that gate and create two language authorities.
- `CompiledDocumentV2`/`PlannedItemV2` (`pipeline/src/compile.rs:84-98,165-260`) carry source-ordered items and semantic plans, not a host-lowerable logical operator tree. `SemanticPlanV2` (`planning/src/plan.rs:126-215`) is a policy/result/cost/explain summary. Renaming V1 consumer types to these V2 names is not an implementation.
- Historical gap, now closed locally: the facade exports
  `ClosedFrontendAuthorityBuilderV2` plus host-agnostic source/location,
  relation, graph, dependency-free pack, and lockfile DTOs. The external
  fixture builds a non-default closed authority and actually compiles ordinary
  and `USE PACK` source. This does not provide the sealed logical serving view,
  canonical compiled identity, Semantica cutover, or hosted/service proof.

## Two separate outcomes; do not conflate them

### A. Restore the AG7 service build, if V3 serving is outside that service's requirement

**Owner: Semantica feature graph and AG7 qualification. No GQLang product edit.** The feature-isolation route is a hypothesis until a fresh build and dependency graph prove it. Semantica's `quanta-gqlang-v2-exec-contract` defaults to no `v3-serving`, but `quanta-runtime-public-ingress-surface/Cargo.toml:52` enables it unconditionally; `gqlang-analysis-execution` also reaches the host adapter. The AG7 service directly imports only `BudgetIntentV1` from the exec contract, but transitive host features may still require advanced serving. Owners must:

1. Re-freeze the exact Semantica checkout, dirty lockfile hash, service feature set, and GQLang sibling path/HEAD. Trace `cargo tree -p quanta-may-alias-at-public-service -e features -i quanta-gqlang-v2-exec-contract` and the reverse edge through `may-alias-product-host`, `gqlang-analysis-execution`, and host adapter. Identify the *owning* manifest/feature edge; do not hide the import errors with a local stub.
2. If AG7 does not exercise V3 serving, make that feature opt-in at its actual owner while preserving other public-service capabilities. Prove both the selected AG7 service graph excludes `v3-serving` and the explicit V3-serving graph still selects it (and remains RED until route B). Check no Cargo feature unification from another selected dependency re-enables it.
3. Re-run the immutable AG7 build/receipt at the exact changed Semantica SHA, with selected/executed counts, source digest, actual process exit, and log. A `cargo check` or feature graph alone is not AG7 qualification. If the service truly requires V3 serving, stop this route; no feature disable is allowed.

**Accept A:** fresh AG7 artifact build succeeds on the intended service feature graph, while a positive feature-selection test proves V3 serving was isolated, not silently removed. This says nothing about V3 serving migration.

### B. Historical V2 serving proposal (superseded by Ticket 10a)

**Producer owner: GQLang Pipeline/Planning/Algebra/Logic/Modules/Sema via facade. Consumer owner: Semantica exec-contract/ingress/host-adapter.** This is a new public-contract expansion requiring a Ticket 00/07 allowlist amendment before code work. Keep it separate from AG7-only isolation.

1. **External closed-authority construction (GQLang).** Define `ClosedFrontendAuthorityBuilderV2` as the sole curated public transaction over host-agnostic inputs: registered host source lengths, selected module and imports/packs/lockfile, relation names/schema/provenance, graph selectors, and one frontend budget. Reuse existing `ModuleAuthorityBuilderV2`, `RelationCatalogV2`, `GraphReferenceCatalogV2`, and `ClosedFrontendAuthorityV2::try_new` internally; never mint unchecked locations or an implicit empty/default authority. Preserve source registry identity and projected budget equality; reject duplicates, foreign locations, invalid schemas, absent lockfiles, and exact-limit/+1 input sizes with stable typed diagnostics. A facade-only fixture must construct the authority and call `compile_source_document_v2`, not merely pin its function pointer. Open analysis must not silently promote to closed compilation.
2. **Sealed logical serving view (GQLang).** Specify a bounded, immutable, host-agnostic read model derived from the *same* checked document transaction for each lowerable family: typed relational operator/expressions and relation-source identity; graph query/witness bounds; CHECK mode/evidence/confirm; presentation payload; composition producer IDs/binding/body identity; rule-program membership and stratification proof. Keep the existing `SemanticPlanV2` summary beside this view, not as a pretend execution tree. No physical operator, runtime handle, provider registry, repository scope, SDK type, or host callback enters the facade. Unsupported/non-plan families retain explicit checked reason; never drop an item or infer from source text. The view must not provide public constructors/remint paths for checked proofs. Decide owned logical DTO names and exhaustive enums before edits, then update the pinned facade allowlist and compile-fail cases together.
3. **Canonical compiled identity (GQLang).** Define a stable, versioned, length-delimited identity from sealed semantic payload *and* typed relation/graph/module authority origins, not `Debug`, HIR display, source whitespace, item ordinal alone, or a runtime request ID. Require equal semantics under formatting changes and different identity under authority/schema/binding changes. Bound traversal and bytes. If the frontend cannot promise cross-version identity, expose an explicit versioned content descriptor and make the Semantica owner hash that descriptor; do not duplicate canonicalization algorithms.
4. **Consumer cutover (Semantica).** Replace `compile_program_structural`/`compile_with_context`, `CompiledProgramV1`, raw HIR/Algebra/Plan V1 imports, and the V1 `format_item` identity path across exec-contract, ingress bridge, and host adapter in one coherent migration. Translate every V2 planned family exhaustively to a typed served arm; keep command/artifact lanes separate and unsupported lowerings fail closed with a family-specific reason. Runtime narrowing/physical execution remain Semantica-owned. Remove old V1 tests only after equivalent V2 positive/negative proof exists; never retain a dual-serving truth path.
5. **Qualification.** External facade-only positive and compile-fail fixture; producer owner tests for exact typed payload, foreign authority, tampered composition/rule membership, nested/multi-item budget; Semantica per-family lowering and rejection tests; then one source-to-service integration corpus and the exact AG7/serving build receipts. Record GQLang and Semantica SHAs and dependency path pair. A producer package-green run is not a consumer build or service activation.

**Accept B:** a sibling crate with only the facade dependency can build a non-default closed authority, compile source, and consume the sealed logical view; Semantica uses no deleted V1 facade names on the selected V3 path; positive and reject cases run through ingress and host adapter; exact-source hosted/service qualification is separately recorded. Do not claim this from an allowlist diff or a compile-only proof.

## Ordered ownership and stop rules

1. Decide whether AG7 requires V3 serving. If not, route A can proceed independently in Semantica; it does not wait for GQLang producer work. If yes, route A is invalid.
2. Amend Ticket 00/07's explicit facade contract and external-consumer allowlist before route B producer edits. Freeze the schema, constructor authority, identity version, and family matrix. This ticket does not itself authorize inventing a runtime-facing API.
3. GQLang producer implementation/receipt first, then Semantica consumer migration, then pair qualification. Re-freeze both repositories at each handoff. Do not modify the AG7 qualification worktree or its dirty lockfile from this ticket.
4. Stop on a requested V1 re-export, raw owner-crate dependency from Semantica, default/structural closed compilation, facade exposure of private checked constructors, or a `SemanticPlanV2` summary being treated as a logical execution tree.

## Suggested proof commands (from each repository root)

```sh
# GQLang producer; exact test command set must be frozen with the API amendment.
cargo test -p quanta-gqlang-v2-pipeline -p quanta-gqlang-v2-frontend-contract --all-targets --locked
python3 tools/test_frontend_external_consumer.py
cargo public-api -p quanta-gqlang-v2-frontend-contract --simplified

# Semantica consumer; use the qualified worktree and its own documented build wrapper.
cargo tree -p quanta-may-alias-at-public-service -e features -i quanta-gqlang-v2-exec-contract
rg -n 'CompiledProgramV1|AlgebraNodeV1|PublicPlanV1|compile_program_structural|format_item' packages/analysis/quanta-v2/crates/quanta-gqlang-v2-exec-contract packages/analysis/quanta-v2/crates/quanta-gqlang-v2-host-adapter
```

Record actual exits and selected/executed tests. The second scan is a migration inventory, not a semantic acceptance test.

## 2026-09-22 Producer Progress And Exact Remaining Backlog

Base: GQLang `main@d1206801bc7b4dd7ec6c2a8ecbfaee5c2d6c0364` plus an
uncommitted remediation patch. Re-freeze before any Semantica edit.

### Implemented producer slice

- `ClosedFrontendAuthorityBuilderV2` is exported only through the curated
  facade chain. External callers submit raw source lengths and locations;
  HIR/Modules/Types proof constructors remain hidden.
- Current-module selection, relation schemas/provenance, graph selectors,
  dependency-free pack manifests, canonical lockfile bytes, and the one
  frontend budget are sealed in one source-registry/budget transaction.
- The facade-only positive fixture performs two real closed compilations,
  including source-level `USE PACK`; it no longer pins an unusable function
  pointer. The public API allowlist contains 41 exact roots and all 51 negative
  imports/privacy probes still reject.

### Remaining producer work before Route B can start downstream

1. **Freeze the consumer-required logical schema.** Inventory every Semantica
   V1 field actually read by exec-contract and host-adapter. Define exhaustive,
   immutable facade DTOs for relational expressions/operators, graph witness
   bounds, CHECK/presentation policy, composition identity, and rule-program
   membership/stratification. Each DTO must be derived in the same checked
   Pipeline transaction and expose no constructor that remints proofs.
2. **Add bounded logical-view assembly.** Attach exactly one typed logical view
   to every plan-required `CompiledItemV2`; retain `NoPlanReasonV2` for catalog,
   value, and meta items. Enforce missing/extra/duplicate/foreign/wrong-kind
   rejection and charge the existing document receipt across all items.
3. **Define identity only after the view freezes.** Specify a versioned,
   domain-separated, length-delimited descriptor covering semantic payload,
   relation/graph/module authority origins, composition bindings, and rule
   membership. Test formatting invariance and authority/schema mutation.
4. **Decide multi-module authority requirements from consumer evidence.** The
   current builder intentionally supports one selected module and
   dependency-free pack manifests. If Semantica requires imports/reexports or
   pack dependencies/exports/suites, add explicit host-agnostic DTOs and owner
   conversions with duplicate/foreign-location/exact-budget negatives. Do not
   expose `ModuleIdV2`, `CatalogAssetRefInputV2`, or manifest proof types.
5. **Amend facade gate atomically.** Add the frozen logical DTO roots to the
   41-name allowlist, positive fixture, privacy negatives, strict rustdoc, and
   public-api comparison in the same change. An allowlist-only patch is RED.

### Remaining consumer and qualification work

1. Re-freeze Semantica HEAD, dirty lockfile hash, selected service features,
   and the exact sibling GQLang source. Decide Route A versus Route B from the
   actual feature graph; do not disable V3 serving if the selected service uses
   it.
2. For Route B, replace all deleted V1 imports and both compilation entry paths
   in exec-contract/ingress/host-adapter. Exhaustively lower the new facade view
   into Semantica-owned physical/runtime requests; unsupported families fail
   closed with typed reasons.
3. Run owner tests, source-to-service corpus, exact AG7/serving builds, and
   immutable receipts at the exact producer/consumer SHA pair. Hosted/service
   execution and activation are separate evidence from local package tests.

### Stop conditions

- Any need for raw HIR/Algebra/Planning owner types reopens the producer DTO
  design; it does not justify a direct dependency or V1 re-export.
- Any logical-view field sourced from reparsing text or `Debug` output rejects
  the patch.
- Any source or manifest change after qualification begins invalidates the
  receipt and requires a fresh candidate.
