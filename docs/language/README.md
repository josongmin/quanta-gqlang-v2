# QGLang Language Surface

## Current Status

`quanta-gqlang-v2` is now a `v3` frontend-only repo.

The old direct `syntax -> sema -> ir` crate stack is gone:

1. `quanta-gqlang-v2-syntax`: deleted
2. `quanta-gqlang-v2-sema`: deleted
3. `quanta-gqlang-v2-ir`: deleted

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

The semantic entrypoint is explicit:

1. `quanta_gqlang_v2_pipeline::compile_with_context`
2. `FrontendContextV1`

Non-semantic helpers also exist, but they are explicit and must not be treated
as semantic proof:

1. `compile_program_structural`
2. `compile_item_structural`
3. `compile_document_structural`
4. `compile_program_with_schema`
5. `compile_item_with_schema`
6. `compile_document_with_schema`

Interpretation rules:

1. `compile_with_context` is the semantic API
2. `compile_*_structural` is structural-only authority
3. `compile_*_with_schema` is schema-only authority
4. no caller may report structural or schema-only success as semantic closure

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
3. `query` parses multi-item documents and compiles through the shared pipeline
4. per-item output renders compiled kind first, then plan or artifact detail

Diagnostics stdio:

1. the read-only analysis method set is:

<!-- supported-diagnostics-methods:start -->
- `v3_diagnostics`
<!-- supported-diagnostics-methods:end -->

2. it runs over `compile_program_structural`
3. it surfaces the first failing stage's typed diagnostic verbatim
4. no legacy `hover` / `completion` / `symbols` / `diagnostics` / `zero_query`
   method remains
5. it is a private flat-JSON protocol, not JSON-RPC or LSP; it does not implement
   `initialize` or document lifecycle methods

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
2. `crates/quanta-gqlang-v2-cli/tests/v3_cli_item_surfacing.rs`
3. `crates/quanta-gqlang-v2-cli/tests/v3_cli_lsp_parity.rs`
4. `crates/quanta-gqlang-v2-cli/tests/v3_cli_pipeline_hellgate.rs`
5. `crates/quanta-gqlang-v2-cli/tests/v3_cli_plan_summary.rs`
6. `crates/quanta-gqlang-v2-cli/tests/command_doc_gate.rs`

Diagnostics stdio rails:

1. `crates/quanta-gqlang-v2-diagnostics-stdio/tests/v3_diagnostics_determinism.rs`
2. `crates/quanta-gqlang-v2-diagnostics-stdio/tests/v3_diagnostics_parity_hellgate.rs`
3. `crates/quanta-gqlang-v2-diagnostics-stdio/tests/stdio_frame_edges.rs`
4. `crates/quanta-gqlang-v2-diagnostics-stdio/tests/stdio_json_escape_negative.rs`
5. `crates/quanta-gqlang-v2-diagnostics-stdio/tests/stdio_byte_robustness_hellgate.rs`
6. `crates/quanta-gqlang-v2-diagnostics-stdio/tests/stdio_command_plane_hellgate.rs`
7. `crates/quanta-gqlang-v2-diagnostics-stdio/tests/method_allowlist_doc_gate.rs`

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

Logic / modules rails:

1. `crates/quanta-gqlang-v2-logic/tests/v3_logic_recursion.rs`
2. `crates/quanta-gqlang-v2-logic/tests/v3_logic_source.rs`
3. `crates/quanta-gqlang-v2-logic/tests/v3_logic_witness.rs`
4. `crates/quanta-gqlang-v2-logic/tests/v3_logic_hellgate.rs`
5. `crates/quanta-gqlang-v2-logic/tests/v3_graph_query_hellgate.rs`
6. `crates/quanta-gqlang-v2-modules/tests/v3_modules_v2_authority.rs`

Conformance rails:

1. `crates/quanta-gqlang-v2-conformance/tests/self_validation.rs`
2. `crates/quanta-gqlang-v2-conformance/tests/registry_non_vacuous.rs`
3. `crates/quanta-gqlang-v2-conformance/tests/current_head_artifact_hygiene.rs`
4. `tools/test_frontend_external_consumer.py`

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
