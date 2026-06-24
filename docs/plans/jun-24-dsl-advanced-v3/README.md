# Jun 24 — DSL Advanced v3

Status: `Current Main Audit Refreshed` + `Tickets 09, 11-15 Materially Implemented`

## Current Verified State

As of `2026-06-24`, current `main` materially closes the repo-local gaps owned
by tickets `09`, `11`, `12`, `13`, `14`, and `15`.

Verified locally with:

1. `cargo fmt --all`
2. `cargo clippy --workspace --all-targets`
3. `cargo test --workspace`

Honest remaining boundary:

1. ticket `10` still owns downstream semantica serving truth
2. explicit structural/schema-only helper entrypoints still exist, but they are
   partial-authority helpers rather than repo-local closure blockers and must
   not be reported as semantic proof
3. ticket `15` now has explicit artifact/no-serve contracts, but no separate
   LSP artifact renderer was introduced; current LSP remains diagnostics-first

## Closure Scopes And DoD

Repo-local closure is the end of ticket `09` plus the live owner lanes from
`11` through `15`:

1. CLI, LSP, tests, conformance, and downstream facade use the one pipeline
   seam; no shadow parser/sema path exists
2. `compile_with_context` is the semantic compile API and structural/schema-only
   helpers remain explicitly non-semantic
3. module/import graph, packs, signatures, metadata, and declarative model-pack
   seams are deterministic, typed, and fail closed
4. alias/local/query closure is fail-closed: binder-visible locals are typed or
   rejected, with no env-open skip path
5. horn `rule` lowers through the public pipeline into recursive-predicate
   semantics with typed unsupported-head rejects
6. standalone `let` / horn `rule` compile into explicit artifact/no-serve
   outcomes instead of placeholder algebra failures
7. `quanta-gqlang-v2-frontend-contract` exposes the stable downstream bundle,
   including item/document structural helpers and family inspection
8. `cargo fmt --all`, `cargo clippy --workspace --all-targets`, and
   `cargo test --workspace` are green
9. packet docs describe current-head truth rather than deleted `v2` crate
   assumptions

Full sibling cutover closure adds ticket `10` on top of that repo-local DoD:

1. `semantica` depends on this repo only through the allowed `v3` frontend seam,
   preferably `quanta-gqlang-v2-frontend-contract`
2. `semantica` does not absorb this repo as a workspace member
3. no semantica manifest or bridge path depends on deleted `syntax` / `sema` /
   `ir` crates
4. no ingress authority starts from `PublicIrRootV1` or widens
   `RootSpecV1::Select` to fake `v3` support
5. semantica-owned runtime DTOs, provider routing, SDK/backend wiring, and
   host-adapter lowering start from `CompiledProgramV1`
6. semantica's default public SDK/query facades stop exporting legacy
   request/root helpers and default authority starts from `ServingProgramRequestV1`
7. downstream bridge/conformance rails are rewritten to the `v3` bundle seam
8. only after those downstream rails are green may anyone claim end-to-end
   advanced-serving closure

Out of scope for this repo even after full packet closure:

1. runtime substrate implementation
2. provider registry ownership
3. semantica-native request/response DTO ownership
4. SDK backend wiring or materialization logic

## Goal

Define the breaking-change packet that replaces the current root-carrier,
read-only-shell frontend with a generic expression, relation, and logic
language core suitable for a `v3` line.

This packet assumes:

1. breaking changes are allowed
2. ROI is irrelevant
3. architecture quality dominates migration convenience

## Why This Exists

When this packet was first written, `quanta-gqlang-v2` still shipped a
deterministic frontend shell without the now-live `v3` crate stack.

The load-bearing limitations of that older shell were:

1. parser dispatch is root-keyword first, not expression first
2. AST has no generic `Expr` / `BoolExpr` / `RelationExpr` nodes
3. sema has no binder/scope model beyond narrow root-local carriers
4. IR has no relational algebra or logic algebra layer
5. planning is root-kind based, not expression/query-plan based
6. `SELECT` is intentionally restricted to one-table, one-predicate, no-join,
   no-subquery, no-aggregate shape

This packet treated those as architectural blockers, not backlog items. On
current `main`, the architectural stack now exists; this document remains the
closure and ownership record for that stack.

## End State

Default language architecture:

1. syntax AST
2. typed HIR SSOT
3. normalized algebra IR
4. planning / truth / evidence IR

Default surface capabilities:

1. generic value expressions
2. boolean formulas with grouping
3. alias/scope-aware relational composition
4. aggregates, grouping, sorting, subqueries
5. quantifier-style existence checks
6. typed parametric declarations and reusable rules/views
7. explicit truth/evidence/effect semantics

## SOTA Alignment Criteria

This packet is only considered directionally aligned with CodeQL-class query
systems if it closes all of the following:

1. declarative logic semantics with recursion, fixed-point reasoning, and
   aggregates
2. statically typed expression/query layers with structural as well as domain
   types
3. reusable modules, signatures, query packs, and extension points instead of
   copy-pasted query fragments
4. first-class result kinds:
   table, alert, path/witness, proof/explain
5. IDE-first authoring:
   locations, hover, completions, explainability, tests, and profile/debug
   support
6. scalable evaluation ergonomics:
   cost visibility, incremental workflows, and fail-closed unsupported-shape
   handling

## SOTA Reference Set

This packet is intentionally borrowing strengths from multiple production-grade
DSL families, not only CodeQL:

1. Souffle:
   components, subtypes/union/ADT modeling, provenance proof trees, and
   schedule/profile awareness
2. Flix:
   strong typing, effect-aware semantics, row/record expressivity, and
   compiler-backed IDE behavior
3. Differential Datalog:
   incremental update model, library embedding, replay debugging, and module
   system discipline
4. Cypher / GQL:
   quantified path patterns, path values, and first-class `EXPLAIN` / `PROFILE`
5. EdgeQL:
   path scoping, nested shapes, and compositional result structures
6. PRQL:
   readable transform pipelines and compositional relational surface ergonomics
7. Malloy:
   reusable semantic models, reusable aggregates, and named query libraries

The goal is not to clone any one of these systems. The goal is to combine their
strongest semantics and DX properties while preserving this repo's frontend-only
boundary.

Hard constraint:

1. frontend-only ownership remains intact
2. runtime substrate, provider registry, and backend wiring stay out of the
   public language core

## Adversarial Gap Closure

The plan is still insufficient unless it closes the following failure modes:

1. no shadow compiler path:
   CLI, LSP, tests, and formatter must consume the same frontend pipeline
2. no stringly advanced semantics:
   path, witness, explain, and nested result structures must be typed values
3. no correlated-subquery handwave:
   normalized algebra must own correlated/lateral shapes explicitly
4. no pack story without determinism:
   module and pack resolution must be locked, reproducible, and schema-checked
5. no explain/profile confusion:
   logical explain/provenance belongs here; runtime physical execution detail
   stays downstream in semantica
6. no empty-authority semantic green:
   a compile running with an empty module graph or empty source-schema context
   must not be treated as semantic closure
7. no alias-type skip path:
   alias-dependent expressions must not compile because the type stage silently
   skipped env-open terms
8. no horn-rule half-implementation:
   parser/HIR support without recursive-predicate lowering is not rule support
9. no non-relational placeholder contract:
   standalone decl/rule units may not permanently bottom out in
   `algebra.unsupported_composition`

## Current Head Audit Verdict

The four structural gaps called out by the earlier adversarial draft are now
materially closed on current `main`:

1. explicit semantic authority lane:
   `crates/quanta-gqlang-v2-pipeline/src/compile.rs` now exposes
   `FrontendContextV1`, `compile_with_context`,
   `compile_item_with_context`, and `compile_document_with_context`, and
   `frontend-contract` re-exports that surface
2. alias/local type closure:
   source row schemas are lifted into a typed local env, result shapes are
   checked under merged local authority, and the former alias-gap rails are now
   green
3. horn-rule public lowering:
   source horn `rule` now lowers through the public pipeline into
   `RecursiveProgramV1`, with explicit fail-closed
   `logic.unsupported_rule_head` rejects
4. explicit non-relational artifact contract:
   standalone `let` and horn `rule` compile into explicit frontend artifacts
   with `no_serve_reason`, rather than bottoming out in placeholder algebra
   rejects

Honest residuals:

1. current semantic closure uses the explicit context-first API; legacy
   structural/schema-only helper entrypoints remain and must not be treated as
   semantic proof
2. current artifact surfacing is explicit in pipeline, CLI, conformance, and
   downstream facade tests; no separate LSP artifact-rendering surface exists

## Current Head Authority Map

Current head is not a greenfield packet anymore. Most ownership lanes already
exist as live crates, and tickets `12` through `15` have now landed on those
crates. Remaining packet work is closure/cutover work, not permission to mint a
parallel implementation stack.

1. `surface`: lexer, parser, formatter, family/unit/query surface forms
2. `hir`: shared item, expression, relation, and result-shape carriers
3. `binder`: lexical/relation scope, symbol resolution, capture
4. `types`: closed type universe, operator/builtin registry, type env
5. `modules`: pack/manifest/module-graph/signature/model metadata
6. `algebra`: normalized relational semantics
7. `logic`: recursive predicate, graph, aggregate, witness semantics
8. `planning`: capability, truth, evidence, effect, result-kind, cost
9. `pipeline`: one public cross-layer compile seam
10. `frontend-contract`: downstream facade for sibling consumers

## Non-Negotiable Semantic Properties

The packet is not implementation-ready until the language contract is explicit
on the following:

1. lexical/simple scoping only:
   nested/path forms cannot depend on hidden path-scoping magic
2. explicit multiplicity:
   set/bag/cardinality behavior must be defined per operator family
3. deterministic witness semantics:
   canonical-one, top-k, or all-witness modes must be typed and explicit
4. fail-closed recursive legality:
   recursive aggregates or unbounded path shapes reject unless explicitly owned
5. explicit pack conflict behavior:
   overlapping extension/model packs must diagnose, not silently override

## SOTA Capability Ownership

To avoid duplicate implementation across tickets, ownership is fixed as follows:

1. shared expression and result-shape HIR nodes:
   `01`
2. deterministic semantic-authority contract for public compile surfaces:
   `12`
3. structural and nested value typing:
   `03`
4. binder->types local closure, alias/source row typing, and typed local-env
   synthesis:
   `13`
5. correlated/lateral relational normalization:
   `04`
6. recursive/path semantics and witness legality:
   `05`
7. horn-rule source semantics and recursive-predicate lowering:
   `14`
8. logical explain, provenance, cost, and result-kind derivation:
   `07`
9. explicit non-relational compiled-artifact and no-serve contract:
   `15`
10. single-compiler tooling, profile surfaces, and authoring loop:
   `08`
11. reusable modules, import graph semantics, packs, manifests, and extension
   models:
   `11`
12. semantica serving contract and host-adapter cutover:
   `10`

## Semantica Integration Model

Public frontend path:

1. `syntax AST`
2. `typed HIR`
3. `normalized algebra / logic IR`
4. `planning / truth / evidence IR`
5. host-agnostic logical request bundle

Semantica-owned serving path:

1. ingress bridge consumes the host-agnostic bundle
2. exec-contract / host-spi define the serving boundary
3. host-adapter lowers the serving request into runtime-native plans
4. semantica runtime executes against engine substrates

Implications:

1. this repo does not mint semantica-native runtime DTOs directly
2. advanced language constructs must not be lossy-downgraded into the current
   narrow `RootSpecV1::Select` shell
3. if current serving contracts cannot represent a `v3` query shape, that shape
   stays non-servable until sibling exec-contract evolution lands

## Canonical Waves

1. Wave 0
   - [tickets/00-packet-scope-and-pinned-decisions.md](./tickets/00-packet-scope-and-pinned-decisions.md)
2. Wave 1
   - [tickets/01-hir-ssot-and-generic-expression-kernel.md](./tickets/01-hir-ssot-and-generic-expression-kernel.md)
   - [tickets/02-binder-scope-and-symbol-resolution.md](./tickets/02-binder-scope-and-symbol-resolution.md)
   - [tickets/03-type-universe-and-operator-function-registry.md](./tickets/03-type-universe-and-operator-function-registry.md)
3. Wave 1.5
   - [tickets/12-frontend-context-and-semantic-authority-contract.md](./tickets/12-frontend-context-and-semantic-authority-contract.md)
   - [tickets/13-source-schema-lifting-and-local-type-closure.md](./tickets/13-source-schema-lifting-and-local-type-closure.md)
4. Wave 2
   - [tickets/04-relational-algebra-and-normalization.md](./tickets/04-relational-algebra-and-normalization.md)
   - [tickets/05-logic-semantics-recursion-and-stratification.md](./tickets/05-logic-semantics-recursion-and-stratification.md)
   - [tickets/06-syntax-rewrite-and-surface-language-v3.md](./tickets/06-syntax-rewrite-and-surface-language-v3.md)
   - [tickets/14-horn-rule-semantics-and-recursive-predicate-lowering.md](./tickets/14-horn-rule-semantics-and-recursive-predicate-lowering.md)
   - [tickets/11-module-system-packs-and-extension-models.md](./tickets/11-module-system-packs-and-extension-models.md)
5. Wave 3
   - [tickets/07-planning-truth-evidence-and-effects-v3.md](./tickets/07-planning-truth-evidence-and-effects-v3.md)
   - [tickets/08-cli-lsp-formatter-and-proof-rails-v3.md](./tickets/08-cli-lsp-formatter-and-proof-rails-v3.md)
   - [tickets/15-non-relational-artifact-contract-and-servable-boundaries.md](./tickets/15-non-relational-artifact-contract-and-servable-boundaries.md)
6. Wave 4
   - [tickets/09-hard-cutover-and-v2-surface-deletion.md](./tickets/09-hard-cutover-and-v2-surface-deletion.md)
7. Wave 5
   - [tickets/10-semantica-serving-contract-handoff.md](./tickets/10-semantica-serving-contract-handoff.md)

## Ticket Order

Current-main note:

1. tickets `11`, `12`, `13`, `14`, and `15` are implemented locally
2. ticket `09` is now materially closed on current head
3. ticket `10` remains the downstream serving handoff lane

| # | Ticket | Lane |
|---|--------|------|
| 00 | packet scope, pinned decisions, non-goals | serial gate |
| 01 | HIR SSOT + generic expression kernel | worker A |
| 02 | binder / scope / symbol resolution | worker A (after 01) |
| 03 | type universe + operator/function registry | worker A (after 01, parallel with 02 if interfaces are frozen) |
| 12 | frontend context + semantic authority contract | worker A2 (after 01/02/03; before semantic closure claims) |
| 13 | source-schema lifting + local type closure | worker A3 (after 02/03/12) |
| 04 | relational algebra + normalization | worker B (after 01/02/03/13) |
| 05 | logic semantics + recursion / stratification | worker C (after 01/02/03; closes with 14) |
| 06 | syntax rewrite + surface language v3 | worker D (after 01/02/03, closes with 04/05/14) |
| 14 | horn-rule semantics + recursive-predicate lowering | worker C2 (after 02/03/05/06/11/12) |
| 11 | module system, packs, extension models | worker G (after 01/02/03, closes with 08/09/14) |
| 07 | planning / truth / evidence / effects | worker E (after 04/05/06/13) |
| 15 | non-relational artifact + servable boundary contract | worker E2 (after 07/12/14) |
| 08 | CLI / LSP / formatter / proof rails v3 | worker F (after 04/05/06/07/12/13/14/15) |
| 09 | hard cutover + v2 surface deletion | serial closeout |
| 10 | semantica serving contract handoff | downstream sibling lane (after 07, finalized with 09) |

## Breaking Rules

1. no compatibility shim is mandatory for packet completion
2. no root-local ad-hoc extension if the same feature belongs in HIR
3. no new public syntax that bypasses binder or type checking
4. no runtime/provider nouns in public expression or type systems
5. no heuristic lowering; unsupported constructs reject with typed diagnostics
6. no verbatim lift-and-shift of legacy `quanta-gqlang-*` crates; semantics may
   be re-derived, but old module boundaries are not authoritative
7. syntax, logic, algebra, and planning ownership boundaries must stay
   single-owner; no duplicate feature classification across layers

## Success Criteria

1. direct `syntax -> root-local typed carrier -> IR` paths are no longer the
   default language architecture
2. generic expression nodes exist and are shared across query/declaration/rule
   surfaces
3. relational composition is typed and scope-aware
4. planning works on normalized query shape, not only root kind
5. v2 root-carrier surface is deleted or explicitly quarantined behind
   non-default compat crates
6. advanced `v3` shapes are either mapped through a sibling serving contract or
   rejected explicitly as non-servable on the current semantica seam
7. reusable module/pack boundaries exist so advanced queries and framework
   models do not require core edits or query copy-paste
8. DX includes path/witness rendering, source locations, cost visibility, and
   incremental authoring support
9. CLI, LSP, formatter, and tests share one frontend compiler path with no
   shadow parser/sema implementation
10. public compile surfaces consume explicit semantic authority rather than
    silently treating empty context as semantic truth
11. alias/local expressions are either typed under a lifted local environment or
    reject; they do not compile through unchecked
12. horn `rule` lowers through an explicit recursive-predicate contract rather
    than stopping at parser/HIR presence
13. standalone non-relational units compile into explicit host-agnostic
    artifacts or explicit no-serve outcomes rather than permanent algebra
    placeholders

Current-main receipt:

1. criteria `10` through `13` are now satisfied locally
2. criteria `5` and `6` still depend on tickets `09` and `10`
