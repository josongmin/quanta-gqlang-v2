# Worker Start Here

Current main status at `2026-06-24`:

1. tickets `09`, `12`, `13`, `14`, and `15` are materially implemented and
   backed by green owner-local rails
2. ticket `11` is also materially implemented and backed by green owner-local
   rails
3. downstream serving closure is still ticket `10`, now treated as the umbrella
   lane for the semantica D5 execution packet
4. tickets `16` and `17` are the remaining qglang-side closeout lanes:
   current-main proof/artifact hygiene first, then final packet sync
5. structural/schema-only helpers remain explicit partial-authority helpers and
   are not semantic proof

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
   - 09 is the hard-cutover closure lane
   - 10 is the downstream semantica handoff lane; it cannot pull runtime
     substrate logic back into this repo
   - 11 now owns the live reusable module/pack/extension ergonomics lane
   - 16 owns current-main proof, dirty-tree classification, and artifact
     hygiene before final packet closure claims
   - 17 is the terminal packet-sync lane; it may close only after the semantica
     D5 execution packet is green
6. Closure scopes:
   - repo-local closure is ticket `09` plus the live owner lanes from `11`-`15`
   - full sibling cutover closure is repo-local closure plus downstream ticket
     `10`, qglang ticket `16`, and qglang ticket `17`
   - ticket `11` no longer sits as an undefined future lane; its current rails
     must stay green for any extensibility/DX claim
7. Repo-local DoD:
   - one pipeline seam only across CLI/LSP/tests/conformance/facade
   - module/import graph, packs, metadata, signatures, and model packs are
     deterministic and fail closed
   - `compile_with_context` is the semantic API; structural/schema-only helpers
     are not semantic proof
   - alias/local expressions are typed or rejected; no env-open skip path
   - horn `rule` lowers to recursive-predicate semantics with fail-closed
     unsupported-head rails
   - standalone `let` / horn-rule compile to explicit artifact/no-serve
     outcomes
8. Downstream ticket `10` DoD:
   - semantica consumes this repo only as a downstream frontend seam, preferably
     through `quanta-gqlang-v2-frontend-contract`
   - semantica does not absorb this repo as a workspace member
   - no semantica manifest or ingress path depends on deleted `syntax` /
     `sema` / `ir` crates or `PublicIrRootV1`
   - semantica-owned runtime DTOs and host-adapter lowering start from
     `CompiledProgramV1`
   - semantica default public SDK/query facades stop exporting legacy
     `ServingQueryRequestV1` / deleted root-helper authority
   - end-to-end advanced-serving claims require downstream bridge/conformance
     rails green on that seam
   - qglang packet closeout claims require ticket `16` proof closure and ticket
     `17` packet sync after semantica D5 closes
9. Semantic-closure guardrails that remain mandatory:
   - multiplicity/order semantics, witness mode, and pack-conflict behavior
     must be explicit before anyone claims semantic closure
   - import precedence, module-cycle policy, nested result-shape ownership, and
     fixed semantic-cost fields must be explicit before anyone claims
     deterministic authoring or explainability closure
