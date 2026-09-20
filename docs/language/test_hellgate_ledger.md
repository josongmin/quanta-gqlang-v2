# Test hellgate ledger

This ledger maps the current v3 frontend authorities to repository-owned proof
rails. Every referenced path exists at the current source snapshot. It replaces
the retired v2 syntax/sema/IR and LSP inventory; those crates and methods are
not compatibility promises.

## Proof boundary

- Binary e2e means the shipped CLI or diagnostics-stdio process is spawned and
  its framing, channel, response, or exit behavior is asserted.
- Contract proof means a crate-local public or owner API is exercised. It does
  not prove a shipped process boundary.
- These rails prove frontend syntax, HIR, binding, types, modules, algebra,
  logic, planning IR, pipeline composition, CLI, and diagnostics transport.
- Runtime execution, provider routing, result materialization, backend DTOs,
  deployment, and activation remain out of scope.

## Source, HIR, and type authorities

| Authority | Primary rails | Contract pinned |
|---|---|---|
| Surface grammar and item families | `crates/quanta-gqlang-v2-surface/tests/v3_surface_query.rs`, `crates/quanta-gqlang-v2-surface/tests/v3_family_hellgate.rs`, `crates/quanta-gqlang-v2-surface/tests/v3_document_hellgate.rs` | Typed query stages, all root families, document/source identity, fail-closed grammar |
| Expression grammar | `crates/quanta-gqlang-v2-surface/tests/v3_surface_expr_kernel.rs`, `crates/quanta-gqlang-v2-surface/tests/v3_surface_expr_edges.rs`, `crates/quanta-gqlang-v2-surface/tests/v3_mutation_rails.rs` | Precedence, path/call/quantifier/existence shapes, mutation-sensitive branches |
| UTF-8 and formatting | `crates/quanta-gqlang-v2-surface/tests/v3_format_edges.rs`, `crates/quanta-gqlang-v2-surface/tests/v3_format_hellgate.rs`, `crates/quanta-gqlang-v2-surface/tests/v3_surface_proptest.rs`, `crates/quanta-gqlang-v2-surface/tests/v3_lexer_symbol_rails.rs` | Unicode and escape correctness, precedence-aware canonicalization, independent structural witness, property rails |
| Parse resource budgets | `crates/quanta-gqlang-v2-surface/tests/v3_surface_budget.rs` | Exact-limit/one-over-limit token, item, expression, collection, and projection dimensions |
| HIR identity and integrity | `crates/quanta-gqlang-v2-hir/tests/v3_item_identity.rs`, `crates/quanta-gqlang-v2-hir/tests/v3_hir_hellgate.rs`, `crates/quanta-gqlang-v2-hir/tests/v3_span_semantics.rs` | Unique IDs, source-qualified document integrity, node and auxiliary span bounds, invalid-state rejection |
| HIR semantic carriers and traversal | `crates/quanta-gqlang-v2-hir/tests/v3_hir_expr_kernel.rs`, `crates/quanta-gqlang-v2-hir/tests/v3_item_hellgate.rs`, `crates/quanta-gqlang-v2-hir/tests/v3_visit_traversal.rs` | Closed join/group/order/call/path/composition shapes and exhaustive traversal |
| Closed type universe and checking | `crates/quanta-gqlang-v2-types/tests/v3_types_universe.rs`, `crates/quanta-gqlang-v2-types/tests/v3_types_check_edges.rs`, `crates/quanta-gqlang-v2-types/tests/v3_types_hellgate.rs` | Non-forgeable type states, overload resolution, record projection, complete effects/facts, type budgets |

## Semantic-stage authorities

| Authority | Primary rails | Contract pinned |
|---|---|---|
| Binding and scope | `crates/quanta-gqlang-v2-binder/tests/v3_binder_scope.rs`, `crates/quanta-gqlang-v2-binder/tests/v3_binder_hellgate.rs`, `crates/quanta-gqlang-v2-binder/tests/v3_binder_mutants.rs`, `crates/quanta-gqlang-v2-binder/tests/v3_composition_hellgate.rs` | Frame/capture barriers, ambiguity rejection, composition scope, negative mutants |
| Module/catalog authority | `crates/quanta-gqlang-v2-modules/tests/v3_modules_v2_authority.rs` | Source registry, lock/import/export closure, catalog coherence, budget provenance, cycle and ambiguity rejection |
| Relational algebra | `crates/quanta-gqlang-v2-algebra/tests/v3_algebra_normalization.rs`, `crates/quanta-gqlang-v2-algebra/tests/v3_algebra_hellgate.rs`, `crates/quanta-gqlang-v2-algebra/tests/v3_algebra_mutants.rs`, `crates/quanta-gqlang-v2-algebra/tests/v3_algebra_probe.rs` | Typed lowering, normalization, semantic mutants, operator reachability probes |
| Logic and graph rules | `crates/quanta-gqlang-v2-logic/tests/v3_logic_recursion.rs`, `crates/quanta-gqlang-v2-logic/tests/v3_logic_source.rs`, `crates/quanta-gqlang-v2-logic/tests/v3_logic_witness.rs`, `crates/quanta-gqlang-v2-logic/tests/v3_logic_hellgate.rs`, `crates/quanta-gqlang-v2-logic/tests/v3_graph_query_hellgate.rs` | Multi-rule SCC/stratification, source provenance, witness stability, graph-query legality |
| Planning IR | `crates/quanta-gqlang-v2-planning/tests/v3_planning_truth.rs`, `crates/quanta-gqlang-v2-planning/tests/v3_planning_hellgate.rs`, `crates/quanta-gqlang-v2-planning/tests/v3_planning_cost_schema.rs`, `crates/quanta-gqlang-v2-planning/tests/v3_item_hellgate.rs`, `crates/quanta-gqlang-v2-planning/tests/v3_capability_derivation.rs`, `crates/quanta-gqlang-v2-planning/tests/v3_explain_provenance.rs` | Current public plan schema, truth/capability/provenance derivation, fail-closed item planning |

## Composed pipeline

| Surface | Primary rails | Contract pinned |
|---|---|---|
| All root families | `crates/quanta-gqlang-v2-pipeline/tests/audit_18_families.rs`, `crates/quanta-gqlang-v2-pipeline/tests/v3_pipeline_compile.rs`, `crates/quanta-gqlang-v2-pipeline/tests/v3_pipeline_hellgate.rs` | Frontend transaction across the complete declared root-family set |
| Context, catalog, and composition | `crates/quanta-gqlang-v2-pipeline/tests/v3_frontend_context.rs`, `crates/quanta-gqlang-v2-pipeline/tests/v3_composition_catalog_scope.rs`, `crates/quanta-gqlang-v2-pipeline/tests/v3_composition_type_env.rs`, `crates/quanta-gqlang-v2-pipeline/tests/v3_use_pack_authority.rs`, `crates/quanta-gqlang-v2-pipeline/tests/v3_relation_source_authority.rs` | Shared authority, catalog scope, type environment, pack lock, relation-source closure |
| Stage-shape gaps and normalization | `crates/quanta-gqlang-v2-pipeline/tests/v3_alias_type_closure_gap.rs`, `crates/quanta-gqlang-v2-pipeline/tests/v3_normalize_item_decl_rule.rs`, `crates/quanta-gqlang-v2-pipeline/tests/v3_surface_to_algebra_group_by.rs`, `crates/quanta-gqlang-v2-pipeline/tests/v3_surface_to_algebra_order_by.rs`, `crates/quanta-gqlang-v2-pipeline/tests/v3_quantifier_body_type_check.rs` | Alias/type closure, item lowering, group/order preservation, quantified body checking |
| Adversarial and generated coverage | `crates/quanta-gqlang-v2-pipeline/tests/audit_edge_cases.rs`, `crates/quanta-gqlang-v2-pipeline/tests/v3_uncovered_branches.rs`, `crates/quanta-gqlang-v2-pipeline/tests/v3_pipeline_proptest.rs`, `crates/quanta-gqlang-v2-pipeline/tests/v3_diagnostic_surface.rs` | Edge branches, property behavior, stable diagnostics |

## Shipped front doors

| Front door | Binary rails | Contract pinned |
|---|---|---|
| CLI | `crates/quanta-gqlang-v2-cli/tests/v3_cli_front_door_contract.rs`, `crates/quanta-gqlang-v2-cli/tests/v3_cli_pipeline_hellgate.rs`, `crates/quanta-gqlang-v2-cli/tests/v3_cli_fmt_idempotence.rs`, `crates/quanta-gqlang-v2-cli/tests/v3_cli_item_surfacing.rs`, `crates/quanta-gqlang-v2-cli/tests/v3_cli_plan_summary.rs`, `crates/quanta-gqlang-v2-cli/tests/command_doc_gate.rs` | Command allowlist, source/file parity, exit/channel contract, formatting fixed point, item and plan surfacing |
| CLI/diagnostics parity | `crates/quanta-gqlang-v2-cli/tests/v3_cli_lsp_parity.rs`, `crates/quanta-gqlang-v2-diagnostics-stdio/tests/v3_diagnostics_parity_hellgate.rs` | Historical filename retained; assertions target the current diagnostics-stdio binary and shared compiler result |
| Diagnostics stdio | `crates/quanta-gqlang-v2-diagnostics-stdio/tests/v3_diagnostics_determinism.rs`, `crates/quanta-gqlang-v2-diagnostics-stdio/tests/stdio_frame_edges.rs`, `crates/quanta-gqlang-v2-diagnostics-stdio/tests/stdio_frame_extractor_edges.rs`, `crates/quanta-gqlang-v2-diagnostics-stdio/tests/stdio_json_escape_negative.rs`, `crates/quanta-gqlang-v2-diagnostics-stdio/tests/stdio_json_object_negative.rs`, `crates/quanta-gqlang-v2-diagnostics-stdio/tests/stdio_byte_robustness_hellgate.rs`, `crates/quanta-gqlang-v2-diagnostics-stdio/tests/stdio_command_plane_hellgate.rs` | Private flat-JSON framing, deterministic diagnostics, malformed-byte/JSON recovery, closed read-only command plane |
| Documentation/allowlist drift | `crates/quanta-gqlang-v2-diagnostics-stdio/tests/method_allowlist_doc_gate.rs` | Only `v3_diagnostics` is advertised and dispatched; transport is explicitly not JSON-RPC or LSP; ledger paths must exist |

## Architecture and composition gates

| Gate | Rail | What it proves |
|---|---|---|
| Public downstream boundary | `crates/quanta-gqlang-v2-frontend-contract/tests/downstream_contract.rs` | Host-agnostic frontend contract remains consumable without runtime substrate types |
| Registry and artifact integrity | `crates/quanta-gqlang-v2-conformance/tests/registry_non_vacuous.rs`, `crates/quanta-gqlang-v2-conformance/tests/self_validation.rs`, `crates/quanta-gqlang-v2-conformance/tests/current_head_artifact_hygiene.rs` | Conformance targets are non-vacuous, registered, and current-source artifacts are clean |
| Full local composition | `just ci` | Repository-configured format, lint, architecture, Semgrep, test, and conformance composition; it is local proof, not hosted-CI evidence |

## Maintenance rule

When a rail is renamed or deleted, update this ledger and its owning ticket in
the same change. `method_allowlist_doc_gate.rs` scans every backticked Rust test
path under the crates directory in this file and fails if the target does not
exist. A
test filename containing legacy terminology is not authority for a retired
protocol; its table entry must state the actual current boundary.
