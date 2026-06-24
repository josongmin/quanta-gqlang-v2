# 02 Binder Scope And Symbol Resolution

## Goal

Close the binder and scope resolver lane for `v3`.

## Why

Without a binder there is no safe support for:

1. table aliases
2. declaration params
3. local `let`
4. correlated subqueries
5. symbol shadowing/visibility rules

Current head already ships `quanta-gqlang-v2-binder` as the owner lane. This
ticket closes its semantics and tooling handoff; it does not authorize a second
resolver path.

## Deliverables

1. lexical scope model
2. relation scope model
3. declaration/rule/view symbol tables
4. alias binding and lookup
5. local let-binding and capture rules
6. parameter binding model
7. import/use binding over the frozen module graph from `11`
8. diagnostics for unresolved, duplicate, shadowed, ambiguous-import, or
   illegal-capture symbols

## Pinned Semantics

1. name resolution is deterministic and explicit
2. correlated capture requires an explicit allowed scope edge
3. unresolved symbols fail closed
4. ambiguous bindings fail closed
5. import order is not semantic authority
6. binder consumes a resolved module/import graph; re-export precedence and
   module cycles remain owned by `11`
7. binder owns names and capture, not final type closure; lifting resolved
   locals into a typed local environment is ticket `13`

## Scenarios Unlocked

1. alias-aware projection/filter
   - `SELECT u.id FROM catalog.users AS u WHERE u.active = true`
2. correlated existence checks
   - `WHERE EXISTS (SELECT 1 FROM findings f WHERE f.repo = r.id)`
3. parametric views/rules
   - `VIEW risky(repo: source_scope) ...`
4. explicit imported symbol resolution
   - conflicting imported names reject instead of silently shadowing

## Definition Of Done

1. binder sits between syntax/HIR construction and typing
2. scope-aware tests pin visibility, capture, and duplicate-name rules
3. no alias or param support lands without binder coverage
4. imported symbol lookup is deterministic and independent of textual import
   order

## Dependencies

1. requires `00`
2. depends on `01`
3. must consume, not redefine, module/import graph rules from `11`
4. feeds local-name resolution inputs into `13`; it does not close query typing
   by itself

## Likely Touch Points

1. `crates/quanta-gqlang-v2-binder/src/resolve.rs`
2. `crates/quanta-gqlang-v2-binder/src/scope.rs`
3. `crates/quanta-gqlang-v2-binder/src/diagnostic.rs`
4. `crates/quanta-gqlang-v2-binder/src/module_graph.rs`
5. `crates/quanta-gqlang-v2-lsp/src/handlers/v3_diagnostics.rs` only if new reject
   codes need surfacing

## Mandatory Worker Outputs

1. one deterministic lexical/relation scope model with explicit capture edges
2. imported-symbol resolution rules that consume the frozen graph from `11`
3. stable typed diagnostics for unresolved, duplicate, shadowed, ambiguous, and
   illegal-capture cases

## Proof Rails

1. extend `crates/quanta-gqlang-v2-binder/tests/v3_binder_scope.rs`
2. extend `crates/quanta-gqlang-v2-binder/tests/v3_binder_hellgate.rs`
3. extend `crates/quanta-gqlang-v2-binder/tests/v3_composition_hellgate.rs`
4. extend `crates/quanta-gqlang-v2-pipeline/tests/v3_composition_catalog_scope.rs`
   only if the bound cross-layer seam changes
5. add LSP diagnostics coverage only if user-visible error payloads change

## Reject Or Escalate

1. any import precedence invention inside binder:
   belongs to `11`
2. any overload/type legality implementation:
   belongs to `03`
3. any parser-side hidden scope rule that bypasses binder ownership
4. any attempt to claim alias/local type closure from name resolution alone:
   belongs to `13`
