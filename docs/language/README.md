# QGLang Language Surface

## Current Status

`quanta-gqlang-v2` is now a `v3` frontend-only repo.

The legacy monolithic `syntax -> sema -> ir` path is gone. The current
`quanta-gqlang-v2-sema` crate is a stage owner inside the live pipeline, not a
public downstream entrypoint; the deleted legacy syntax/IR crate contracts are
not compatibility surfaces.

The live owner stack is:

1. `surface`: parser, document parser, formatter
2. `hir`: shared item / expr / relation / result-shape carriers
3. `binder`: lexical scope, capture, module-graph-aware binding
4. `types`: closed type universe, overload legality, type env
5. `modules`: pack, manifest, lockfile, catalog, import graph
6. `algebra`: normalized relational semantics
7. `logic`: recursive predicate, graph, witness, recursion legality
8. `planning`: truth, evidence, effect, result-kind, cost
9. `pipeline`: one shared compiler service
10. `frontend-contract`: downstream facade

## Public Compile Surface

`quanta-gqlang-v2-frontend-contract` is the only public downstream boundary.
It exposes two explicit authorities:

1. `FrontendAuthorityV2`: closed compilation with caller-supplied schema,
   catalog, module, pack, and lockfile authority
2. `OpenAnalysisAuthorityV2`: syntax-preserving analysis without fabricated
   external authority

Interpretation rules:

1. only closed authority can produce `AnalysisStatusV2::Complete`
2. unresolved external names under open authority produce
   `AnalysisStatusV2::Incomplete`, never a fabricated plan
3. parse or semantic rejection produces `AnalysisStatusV2::Error`
4. CLI, diagnostics stdio, conformance, and downstream consumers import only
   the frontend contract; pipeline internals are not a public seam
5. source parsing rejects relation paths deeper than the checked
   `FrontendBudgetV2::max_relation_depth` before HIR integrity traversal;
   the HIR owner also checks programmatically assembled trees before its
   recursive integrity walk, and Sema checks the configured budget before
   semantic traversal. The standard and maximum configurable value is 128
6. source parsing rejects expression trees deeper than
   `FrontendBudgetV2::max_expression_depth`, including left-associated operator
   chains, before formatting or semantic traversal; the maximum configurable
   value is 64; programmatic HIR also bounds nested result shapes to 64

## Surface Families

Current public breadth covered by the shared pipeline:

1. relational query
2. graph query / `find` subkinds:
   `reachability`, `flow`, `slice`, `taint`, `path`, `points_to`, `summary`
3. `check`
4. presentation families:
   `explain`, `show`, `describe`, `sample`, `compare`, `export`
5. catalog registration families:
   `view`, `model`, `pack`, `profile`, `gate`, `ruledef`
6. meta families:
   `use pack`, `require certified`
7. composition:
   `let ... with ...`
8. supplemental non-relational units:
   standalone `let` -> `ValueArtifact`
   standalone horn `rule` -> `RuleArtifact`

Current non-servable truth:

1. standalone `let` compiles to `ValueArtifact` with `NoServeReasonV1::ValueDeclaration`
2. standalone horn `rule` compiles to `RuleArtifact` with
   `NoServeReasonV1::RuleArtifact`
3. unsupported mixes reject fail-closed with typed planning or logic codes
4. nothing is down-cast into legacy `RootSpecV1`

## CLI And Diagnostics Stdio

CLI:

1. `query` and `v3-query` are the same command
2. `fmt` and `v3-fmt` are the same command
3. `query` parses multi-item documents through open analysis
4. complete analysis renders plan/artifact detail; incomplete or rejected
   analysis emits canonical diagnostics and exits non-zero

Diagnostics stdio:

1. the read-only analysis method set is:

<!-- supported-diagnostics-methods:start -->
- `v3_diagnostics`
<!-- supported-diagnostics-methods:end -->

2. it runs through `OpenAnalysisAuthorityV2`
3. it returns canonical status plus ordered typed diagnostics
4. no legacy `hover` / `completion` / `symbols` / `diagnostics` / `zero_query`
   method remains
5. it is bounded NDJSON with exact request fields, not JSON-RPC or LSP; it does
   not implement `initialize` or document lifecycle methods

## Examples

Repo-owned examples:

1. `examples/select/basic.qglang`
2. `examples/find/taint.qglang`
3. `examples/check/hybrid_rule.qglang`
4. `examples/explain/why_no_result.qglang`
5. `examples/show/capabilities.qglang`
6. `examples/describe/rule_detail.qglang`
7. `examples/sample/retrieval.qglang`
8. `examples/compare/semantic_delta.qglang`
9. `examples/export/sarif.qglang`
10. `examples/view/catalog_users.qglang`
11. `examples/rule/shell_exec.qglang`
12. `examples/model/embeddings_code.qglang`
13. `examples/pack/security_default.qglang`
14. `examples/profile/strict_review.qglang`
15. `examples/gate/release_gate.qglang`
16. `examples/use_pack/secure_pack.qglang`
17. `examples/require_certified/strict_only.qglang`
18. `examples/composition/let_with_find.qglang`

## Proof Rails

Front-door and compiler proof is pinned by current `v3` rails, not deleted v2
tests.

CLI rails:

1. `crates/quanta-gqlang-v2-cli/tests/v3_cli_fmt_idempotence.rs`
2. `crates/quanta-gqlang-v2-cli/tests/open_analysis_contract.rs`
3. `crates/quanta-gqlang-v2-cli/tests/command_doc_gate.rs`

Diagnostics stdio rails:

1. `crates/quanta-gqlang-v2-diagnostics-stdio/tests/ndjson_contract.rs`
2. `crates/quanta-gqlang-v2-diagnostics-stdio/tests/resource_boundaries.rs`

Surface / pipeline / planning rails:

1. `crates/quanta-gqlang-v2-surface/tests/v3_surface_query.rs`
2. `crates/quanta-gqlang-v2-surface/tests/v3_surface_expr_kernel.rs`
3. `crates/quanta-gqlang-v2-surface/tests/v3_surface_unit.rs`
4. `crates/quanta-gqlang-v2-surface/tests/v3_family_hellgate.rs`
5. `crates/quanta-gqlang-v2-surface/tests/v3_format_hellgate.rs`
6. `crates/quanta-gqlang-v2-pipeline/tests/v3_analysis_document_assembly.rs`
7. `crates/quanta-gqlang-v2-pipeline/tests/v3_compiled_document_assembly.rs`
8. `crates/quanta-gqlang-v2-planning/tests/v3_planning_truth.rs`
9. `crates/quanta-gqlang-v2-planning/tests/v3_planning_hellgate.rs`
10. `crates/quanta-gqlang-v2-planning/tests/v3_planning_cost_schema.rs`
11. `crates/quanta-gqlang-v2-planning/tests/v3_item_hellgate.rs`

Binder / types / sema / algebra rails:

1. `crates/quanta-gqlang-v2-binder/tests/v3_binder_scope.rs`
2. `crates/quanta-gqlang-v2-binder/tests/v3_composition_hellgate.rs`
3. `crates/quanta-gqlang-v2-types/tests/v3_rule_type_solver.rs`
4. `crates/quanta-gqlang-v2-types/tests/v3_types_hellgate.rs`
5. `crates/quanta-gqlang-v2-sema/tests/v3_authority_builder.rs`
6. `crates/quanta-gqlang-v2-sema/tests/v3_semantic_budget.rs`
7. `crates/quanta-gqlang-v2-algebra/tests/v3_algebra_normalization.rs`
8. `crates/quanta-gqlang-v2-algebra/tests/v3_algebra_hellgate.rs`

Logic / modules rails:

1. `crates/quanta-gqlang-v2-logic/tests/v3_logic_recursion.rs`
2. `crates/quanta-gqlang-v2-logic/tests/v3_logic_source.rs`
3. `crates/quanta-gqlang-v2-logic/tests/v3_logic_witness.rs`
4. `crates/quanta-gqlang-v2-logic/tests/v3_logic_hellgate.rs`
5. `crates/quanta-gqlang-v2-logic/tests/v3_graph_query_hellgate.rs`
6. `crates/quanta-gqlang-v2-modules/tests/v3_modules_v2_authority.rs`

Conformance rails:

1. `crates/quanta-gqlang-v2-conformance/tests/self_validation.rs`
2. `tools/test_frontend_external_consumer.py`

## Boundary

This repo owns:

1. syntax
2. sema/binder/types/modules
3. HIR
4. normalized frontend semantics
5. planning IR
6. CLI/diagnostics-stdio/formatter
7. frontend-facing downstream contract

This repo does not own:

1. runtime substrate
2. provider registry routing
3. backend DTOs
4. semantica-native serving request contracts
5. execution/materialization claims
