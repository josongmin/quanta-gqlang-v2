# Jun 24 DSL Advanced v3 Ticket Pack

Status: `Current Main Updated` + `11-15 Implemented With Green Rails` + `10 Downstream Umbrella Open` + `16 Repo-Local Closeout Narrowed` + `17 Packet Sync Blocked On D5`

## Current Main Receipt

As of `2026-06-24`, the packet's previously open semantic gaps are materially
implemented on current `main`:

1. `12`: explicit context-first semantic compile surface is live
2. `13`: source-schema lifting and local type closure are live
3. `14`: source horn-rule lowering into recursive predicate semantics is live
4. `15`: explicit non-relational artifact and no-serve contracts are live
5. `11`: module/import graph, packs, signatures, metadata, and declarative
   model-pack seams are live

Operational follow-up lanes still open on `2026-06-25`:

1. `10`: downstream semantica cutover umbrella and SSOT
2. `16`: repo-local closeout is now narrowed to the last current-main diff plus
   refreshed proof receipt
3. `17`: final packet sync remains blocked until semantica D5 proof closure

Local verification receipt:

1. `cargo fmt --all`
2. `cargo clippy --workspace --all-targets`
3. `cargo test --workspace`

Closeout refresh on `2026-06-25`:

1. `git status --short --branch` now shows one tracked modification only:
   `crates/quanta-gqlang-v2-surface/tests/v3_family_hellgate.rs`
2. `git diff --check`
3. `cargo test -p quanta-gqlang-v2-surface --test v3_family_hellgate --quiet`
4. `cargo test --workspace --quiet`
5. `cargo clippy --workspace --all-targets --quiet`

Live regression rails:

1. `crates/quanta-gqlang-v2-pipeline/tests/v3_frontend_context.rs`
2. `crates/quanta-gqlang-v2-pipeline/tests/v3_alias_type_closure_gap.rs`
3. `crates/quanta-gqlang-v2-pipeline/tests/v3_normalize_item_decl_rule.rs`
4. `crates/quanta-gqlang-v2-cli/tests/v3_cli_item_surfacing.rs`
5. `crates/quanta-gqlang-v2-frontend-contract/tests/downstream_contract.rs`
6. `crates/quanta-gqlang-v2-conformance/tests/self_validation.rs`
7. `crates/quanta-gqlang-v2-modules/tests/v3_modules_graph.rs`
8. `crates/quanta-gqlang-v2-modules/tests/v3_modules_packs.rs`
9. `crates/quanta-gqlang-v2-modules/tests/v3_modules_hellgate.rs`
10. `crates/quanta-gqlang-v2-modules/tests/v3_catalog_hellgate.rs`

## What This Packet Is

This is not a patch packet for the current `v2` shell.

It is the architectural rewrite packet for a breaking-change `v3` frontend:

1. generic expression language
2. typed HIR SSOT
3. relational and logic algebra
4. plan-level truth/evidence/effect semantics

The packet assumes deletion and replacement are acceptable outcomes.

## Source Anchors

Current local anchors:

1. [docs/language/README.md](../../../language/README.md)
2. [crates/quanta-gqlang-v2-surface/src/parser.rs](../../../../crates/quanta-gqlang-v2-surface/src/parser.rs)
3. [crates/quanta-gqlang-v2-hir/src/item.rs](../../../../crates/quanta-gqlang-v2-hir/src/item.rs)
4. [crates/quanta-gqlang-v2-algebra/src/normalize.rs](../../../../crates/quanta-gqlang-v2-algebra/src/normalize.rs)
5. [crates/quanta-gqlang-v2-pipeline/src/compile.rs](../../../../crates/quanta-gqlang-v2-pipeline/src/compile.rs)
6. [crates/quanta-gqlang-v2-frontend-contract/src/lib.rs](../../../../crates/quanta-gqlang-v2-frontend-contract/src/lib.rs)

Historical architecture warning:

1. [jun-10-gqlnag-v2-followup/F18b-MIGRATION-PLAN.md](../../jun-10-gqlnag-v2-followup/F18b-MIGRATION-PLAN.md)
2. [EXECUTION_PLAYBOOK.md](./EXECUTION_PLAYBOOK.md)

## Packet Truth

1. the current repo is already strong on deterministic frontend ownership
2. the current repo now has a live `v3` core stack; this packet records how to
   close and defend it rather than how to greenfield it
3. `v3` quality required a new core, not incremental `SELECT` patches
4. semantica integration is downstream work; this packet owns frontend
   contracts, typing, normalization, and tooling inside this repo only
5. the packet still needs an explicit downstream handoff because semantica
   still has residual compat/public-facade cleanup before the `v3` seam is the
   only default serving authority
6. current head now closes the old false-green semantic compile path through
   explicit frontend context and lifted local typing, but explicit
   structural/schema-only helpers remain and must not be reported as semantic
   proof
7. current head now lowers source horn rules through the public pipeline into
   recursive predicate semantics, with explicit unsupported-head rejects
8. current head now exposes explicit non-relational compiled artifacts and
   `no_serve_reason` boundaries for standalone `let` / horn-`rule`
9. this packet is delta-closure over live `v3` owner crates, not permission to
   mint a second HIR/binder/types/algebra/logic/planning stack
10. the former red rails are now live green regression rails and must stay
    strict:
    `v3_alias_type_closure_gap.rs` -> `12`/`13`;
    `v3_normalize_item_decl_rule.rs` -> `14`/`15`

## Canonical Tickets

1. [00-packet-scope-and-pinned-decisions](./00-packet-scope-and-pinned-decisions.md)
2. [01-hir-ssot-and-generic-expression-kernel](./01-hir-ssot-and-generic-expression-kernel.md)
3. [02-binder-scope-and-symbol-resolution](./02-binder-scope-and-symbol-resolution.md)
4. [03-type-universe-and-operator-function-registry](./03-type-universe-and-operator-function-registry.md)
5. [04-relational-algebra-and-normalization](./04-relational-algebra-and-normalization.md)
6. [05-logic-semantics-recursion-and-stratification](./05-logic-semantics-recursion-and-stratification.md)
7. [06-syntax-rewrite-and-surface-language-v3](./06-syntax-rewrite-and-surface-language-v3.md)
8. [07-planning-truth-evidence-and-effects-v3](./07-planning-truth-evidence-and-effects-v3.md)
9. [08-cli-lsp-formatter-and-proof-rails-v3](./08-cli-lsp-formatter-and-proof-rails-v3.md)
10. [09-hard-cutover-and-v2-surface-deletion](./09-hard-cutover-and-v2-surface-deletion.md)
11. [10-semantica-serving-contract-handoff](./10-semantica-serving-contract-handoff.md)
12. [11-module-system-packs-and-extension-models](./11-module-system-packs-and-extension-models.md)
13. [12-frontend-context-and-semantic-authority-contract](./12-frontend-context-and-semantic-authority-contract.md)
14. [13-source-schema-lifting-and-local-type-closure](./13-source-schema-lifting-and-local-type-closure.md)
15. [14-horn-rule-semantics-and-recursive-predicate-lowering](./14-horn-rule-semantics-and-recursive-predicate-lowering.md)
16. [15-non-relational-artifact-contract-and-servable-boundaries](./15-non-relational-artifact-contract-and-servable-boundaries.md)
17. [16-current-main-proof-and-artifact-hygiene](./16-current-main-proof-and-artifact-hygiene.md)
18. [17-downstream-cutover-closeout-and-packet-sync](./17-downstream-cutover-closeout-and-packet-sync.md)

## Worker Prompt

1. [REMAINING_WORKER_PROMPT.md](./REMAINING_WORKER_PROMPT.md)

## Worker Contract

1. architecture first, migration second
2. no surface-level syntax patch may bypass 01/02/03
3. no crate may invent its own expression mini-language once 01 lands
4. frontend-only boundary is non-negotiable
5. ticket 09 is the only lane allowed to claim repo-local packet closure
6. ticket 10 is a downstream sibling handoff; it does not expand this repo's
   ownership, but it is required before any end-to-end serving claim
7. ticket 11 is now materially implemented on current head; preserve its
   deterministic module/pack/extension rails before claiming extensibility or DX
   closure
8. no one may claim semantic closure while multiplicity/order, witness
   selection, or pack-conflict behavior remains implicit
9. no one may claim deterministic authoring while import precedence, re-export
   rules, nested result-shape ownership, or fixed cost-schema fields remain
   unspecified
10. semantic query typing claims must preserve the explicit context plus lifted
    local-env closure now guarded by the green regression rails
11. horn-rule support claims must preserve source-rule lowering and
    `logic.unsupported_rule_head` fail-closed behavior
12. non-relational closure claims must preserve explicit artifact/no-serve
    outputs and must not regress to placeholder algebra rejects
13. no one may fork a second owner implementation for a live lane; extend the
    current crate map or escalate the ownership conflict explicitly
14. every worker closeout must include the receipt fields from
    [EXECUTION_PLAYBOOK.md](./EXECUTION_PLAYBOOK.md)
15. ticket 16 must classify current-main proof state and artifact hygiene before
    ticket 17 closes the packet as synchronized SSOT
