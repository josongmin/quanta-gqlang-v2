# 08 CLI LSP Formatter And Proof Rails v3

## Goal

Harden tooling and proof rails on top of the `v3` core.

## Why

Current tooling assumes root-carrier summaries and narrow read-only shell
surfaces. A new core without new tooling would leave the language unverifiable
and operationally opaque.

Current head already ships live `cli`, `lsp`, and shared `surface` formatting
lanes. This ticket unifies and hardens those lanes; it does not authorize
parallel parser/diagnostic/rendering stacks.

## Deliverables

1. formatter over `v3` syntax/HIR
2. CLI summaries over typed HIR/algebra/planning
3. LSP hover/completion/diagnostics over binder/type information
4. one shared frontend compiler/service used by CLI, LSP, formatter, and tests
5. result rendering for:
   - table outputs
   - alert outputs with source locations
   - path/witness outputs
   - explain/proof outputs
6. query metadata/help surfaces for CLI/LSP/test tooling
7. cost/profile diagnostics and incremental authoring rails
8. explain/provenance explorer surfaces over frontend planning output
9. stable surfacing of the fixed semantic-cost schema from `07`
10. new proof rails:
   - parse precedence
   - binder/scope
   - typing
   - normalization
   - recursion legality
   - planner truth/evidence/effect
   - result-kind and source-location rendering
   - deterministic output canonicalization
   - cost/incremental stability
   - cross-layer integration cases
   - deterministic binary front door

## Scenarios Unlocked

1. hover explains typed subexpressions and resolved symbols
2. completion is scope-aware instead of keyword-table-only
3. CLI can show algebra/plan summaries for advanced queries
4. path/proof queries render actionable witness steps instead of opaque blobs
5. query authors can inspect cost/profile data before a query becomes unusable
6. CLI and editor diagnostics stay bit-for-bit aligned because they use the
   same compiler path

## Required Integration Rails

1. correlated subquery + alias capture + type checking
2. aggregate + grouping + having normalization
3. recursive or negated rule rejection paths through planner diagnostics
4. formatter and LSP span stability on nested advanced expressions
5. source-location and path/witness rendering on advanced results
6. incremental rerun or cache-preserving authoring loop for changed queries
7. CLI/LSP/test diagnostics parity over the same failing query fixture
8. unordered result sets and canonical-one witness mode render deterministically
9. fixed semantic-cost fields render identically across CLI/LSP/tests

## Definition Of Done

1. formatter is canonical on the new grammar
2. LSP diagnostics are binder/type aware
3. CLI, LSP, formatter, and proof tooling do not fork parser or sema logic
4. proof rails exist for every new semantic layer
5. packet truth does not rely on syntax-only or root-only tests
6. cross-layer scenario rails cover at least one case from binder to planning
7. query authors have stable metadata, location, and cost/profile feedback loops
8. deterministic rendering rules exist for unordered sets and witness outputs
9. cost/profile output exposes the fixed semantic fields without surface drift

## Dependencies

1. requires `00`
2. depends on `04`, `05`, `06`, and `07`
3. `11` must be respected if module/pack metadata is surfaced

## Likely Touch Points

1. `crates/quanta-gqlang-v2-surface/src/format.rs`
2. `crates/quanta-gqlang-v2-cli/src/commands/v3_format.rs`
3. `crates/quanta-gqlang-v2-cli/src/commands/v3_query.rs`
4. `crates/quanta-gqlang-v2-cli/src/v3_plan.rs`
5. `crates/quanta-gqlang-v2-lsp/src/handlers/v3_diagnostics.rs`
6. `crates/quanta-gqlang-v2-lsp/src/server.rs`

## Mandatory Worker Outputs

1. one shared frontend compiler/service used by CLI, LSP, formatter, and tests
2. deterministic rendering for unordered sets, path witnesses, and explain
   payloads
3. stable surfacing of the fixed semantic-cost schema from `07`

## Proof Rails

1. extend `crates/quanta-gqlang-v2-cli/tests/v3_cli_fmt_idempotence.rs`
2. extend `crates/quanta-gqlang-v2-cli/tests/v3_cli_plan_summary.rs`
3. extend `crates/quanta-gqlang-v2-cli/tests/v3_cli_lsp_parity.rs`
4. extend `crates/quanta-gqlang-v2-lsp/tests/v3_diagnostics_determinism.rs`
5. extend `crates/quanta-gqlang-v2-lsp/tests/v3_diagnostics_parity_hellgate.rs`
6. add at least one CLI/LSP parity rail for the same failing advanced query

## Reject Or Escalate

1. parser, binder, or type logic copied into CLI/LSP handlers
2. renderer invents witness selection or multiplicity semantics instead of
   consuming `07`
3. frontend output drifts across CLI/LSP/tests for the same semantic payload
