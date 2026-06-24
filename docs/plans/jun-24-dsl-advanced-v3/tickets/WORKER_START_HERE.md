# Worker Start Here

Current main status at `2026-06-24`:

1. tickets `12`, `13`, `14`, and `15` are materially implemented and backed by
   green owner-local rails
2. repo-local packet closure is still ticket `09`
3. downstream serving closure is still ticket `10`

1. Read [00-packet-scope-and-pinned-decisions](./00-packet-scope-and-pinned-decisions.md) first.
   Cite decisions by `D#`.
   Then read [EXECUTION_PLAYBOOK.md](./EXECUTION_PLAYBOOK.md).
2. Do not add new query power directly to `v2` root-local carriers unless the
   ticket explicitly says it is a temporary compat shim.
3. Freeze these invariants in every lane:
   - generic expression nodes are shared, not root-local
   - binder/type/algebra sit between syntax and planning
   - frontend-only ownership stays intact
   - unsupported constructs fail closed with typed diagnostics
   - nested/path scoping stays lexical and binder-visible
   - CLI/LSP/tests do not fork their own parser or sema
   - semantic green requires explicit compile authority, not empty defaults
   - alias/local expressions must not slip past the type stage unchecked
   - live owner crates are authoritative; do not create a second parallel lane
4. Current owner crate map:
   - `surface`: lexer/parser/formatter/family parsing
   - `hir`: shared item/expression/relation/result-shape carriers
   - `binder`: lexical scope/symbol resolution/capture
   - `types`: type universe/operator registry/type env
   - `modules`: manifest/lock/pack/signature/model graph authority
   - `algebra`: normalized relational semantics
   - `logic`: recursive predicate/graph/witness semantics
   - `planning`: truth/evidence/effect/result-kind/cost semantics
   - `pipeline`: one public cross-layer compile seam
   - `frontend-contract`: downstream sibling facade
5. Lane map:
   - 01 is the architectural root
   - 02 and 03 define binding/type semantics
   - 12 now freezes the public semantic-authority contract; do not reopen it by
     adding new ambient defaults
   - 13 now wires binder-visible locals into the type stage; treat the alias
     closure rails as regression guards
   - 04/05/06 consume 01/02/03 and produce the new language core
   - 14 now owns the live horn-rule lowering lane; parser/HIR presence alone is
     still not rule support
   - 07 consumes normalized algebra, not syntax AST
   - 15 now owns the live non-relational compiled-artifact and no-serve
     boundary contract
   - 08 is tooling and proof only
   - 09 is the only repo-local closure lane
   - 10 is the downstream semantica handoff lane; it cannot pull runtime
     substrate logic back into this repo
   - 11 owns reusable module/pack/extension ergonomics and closes with 08/09
6. Definition of done is ticket 09's checklist, ending in default `v3` path
   green and `v2` direct path deleted or quarantined. Ticket 10 must still be
   ready before anyone claims end-to-end advanced serving integration. Ticket 11
   must still be ready before anyone claims SOTA-level DX/extensibility.
   Multiplicity/order semantics, witness mode, and pack-conflict behavior must
   also be explicit before anyone claims semantic closure.
   Import precedence, module-cycle policy, nested result-shape ownership, and
   fixed semantic-cost fields must also be explicit before anyone claims
   deterministic authoring or explainability closure.
   Public semantic compile APIs now have an explicit context-first surface;
   structural/compat shims are not semantic proof. Horn `rule` now reaches
   recursive-predicate lowering with fail-closed unsupported-head rails.
   Standalone `let` / horn-rule now compile to explicit artifact/no-serve
   outcomes; any regression reopens ticket `15`.
