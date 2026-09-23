# Test hellgate ledger

This ledger maps the current v3 frontend contracts to repository-owned primary
proof rails. It is a contract/risk map, not a hand-maintained list of test
functions or a substitute for test discovery. It replaces the retired v2
syntax/sema/IR and LSP inventory; those crates and methods are not
compatibility promises.

## Inventory and execution

- `cargo nextest list --workspace --message-format json` is the generated
  inventory of Rust unit and integration tests. `cargo test --workspace --doc`
  covers doctests separately. A `tests/support/mod.rs` file is a helper module,
  not an independent test target. No checked-in per-test count is authoritative.
- `just ci` is the local repository-wide composition: Rust tests, doctests,
  CLI/stdio parity, external-consumer/privacy probes, and static/policy rails.
  `just qualify-local` records an exact clean committed HEAD receipt; a focused
  test or this ledger alone is not that qualification.
- A crate integration test can exercise an owner API without spawning a binary.
  The proof type below is determined by what the test actually invokes, not by
  its `tests/` location or filename.

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
| Surface rejection and declarations | `crates/quanta-gqlang-v2-surface/tests/v3_family_edges.rs`, `crates/quanta-gqlang-v2-surface/tests/v3_surface_hellgate.rs`, `crates/quanta-gqlang-v2-surface/tests/v3_surface_unit.rs` | Unsupported family clauses, malformed/trailing syntax and spans, shared declaration/rule expression grammar |
| Expression grammar | `crates/quanta-gqlang-v2-surface/tests/v3_surface_expr_kernel.rs`, `crates/quanta-gqlang-v2-surface/tests/v3_surface_expr_edges.rs`, `crates/quanta-gqlang-v2-surface/tests/v3_mutation_rails.rs` | Precedence, path/call/quantifier/existence shapes, mutation-sensitive branches |
| UTF-8 and formatting | `crates/quanta-gqlang-v2-surface/tests/v3_format_edges.rs`, `crates/quanta-gqlang-v2-surface/tests/v3_format_hellgate.rs`, `crates/quanta-gqlang-v2-surface/tests/v3_surface_proptest.rs`, `crates/quanta-gqlang-v2-surface/tests/v3_lexer_symbol_rails.rs` | Unicode and escape correctness, precedence-aware canonicalization, independent structural witness, property rails, and iterative formatting of 100,000-deep raw shapes |
| Parse resource budgets | `crates/quanta-gqlang-v2-surface/tests/v3_surface_budget.rs` | Exact-limit/one-over-limit token, item, recursive and structural expression depth, nested relation depth, collection, and projection dimensions |
| HIR identity, depth, and integrity | `crates/quanta-gqlang-v2-hir/tests/v3_item_identity.rs`, `crates/quanta-gqlang-v2-hir/tests/v3_hir_depth_boundaries.rs`, `crates/quanta-gqlang-v2-hir/tests/v3_hir_hellgate.rs`, `crates/quanta-gqlang-v2-hir/tests/v3_span_semantics.rs` | Unique IDs, source-qualified document integrity, stack-safe exact/over-depth and owned rejection, direct raw-root/document iterative destruction including mixed recursive kinds, node and auxiliary span bounds, invalid-state rejection |
| HIR literal builders and source spans | `crates/quanta-gqlang-v2-hir/tests/v3_builder_literals.rs`, `crates/quanta-gqlang-v2-hir/tests/v3_relation_source_name_span.rs` | Literal carriers, monotonic IDs, and source-name auxiliary span rejection |
| HIR semantic carriers and traversal | `crates/quanta-gqlang-v2-hir/tests/v3_hir_expr_kernel.rs`, `crates/quanta-gqlang-v2-hir/tests/v3_item_hellgate.rs`, `crates/quanta-gqlang-v2-hir/tests/v3_visit_traversal.rs` | Closed join/group/order/call/path/composition shapes and exhaustive traversal |
| Closed type universe and checking | `crates/quanta-gqlang-v2-types/tests/v3_types_universe.rs`, `crates/quanta-gqlang-v2-types/tests/v3_types_check_edges.rs`, `crates/quanta-gqlang-v2-types/tests/v3_types_hellgate.rs` | Non-forgeable type states, overload resolution, record projection, complete effects/facts, type budgets |
| Rule constraint solving | `crates/quanta-gqlang-v2-types/tests/v3_rule_type_solver.rs` | Anchored recursive rule types, unanchored/conflicting rejection, exact owner/ordinal quantifier binding, arity and budget boundaries |

## Semantic-stage authorities

| Authority | Primary rails | Contract pinned |
|---|---|---|
| Binding and scope | `crates/quanta-gqlang-v2-binder/tests/v3_binder_scope.rs`, `crates/quanta-gqlang-v2-binder/tests/v3_binder_hellgate.rs`, `crates/quanta-gqlang-v2-binder/tests/v3_binder_mutants.rs`, `crates/quanta-gqlang-v2-binder/tests/v3_composition_hellgate.rs` | Frame/capture barriers, ambiguity rejection, composition scope, negative mutants |
| Source use and inline views | `crates/quanta-gqlang-v2-binder/tests/v3_relation_source_use.rs`, `crates/quanta-gqlang-v2-binder/tests/v3_catalog_inline_view.rs` | Exact prior LET and graph source resolution, no host fallback, duplicate-use rejection, inline-view alias scope and symbol spans |
| Module/catalog authority | `crates/quanta-gqlang-v2-modules/tests/v3_modules_v2_authority.rs` | Source registry, lock/import/export closure, catalog coherence, cycle and ambiguity rejection, cheap forward/reverse exact-budget matrices, and separate 256-node 128-KiB-stack safety |
| Document predeclaration | `crates/quanta-gqlang-v2-modules/tests/v3_document_predeclaration.rs` | Cross-family host/local and forward references, kind mismatch, ambiguity, module identity, and shared authority budget |
| Semantic document transaction | `crates/quanta-gqlang-v2-sema/tests/v3_document_transaction.rs`, `crates/quanta-gqlang-v2-sema/tests/v3_composition_query.rs`, `crates/quanta-gqlang-v2-sema/tests/v3_semantic_budget.rs`, `crates/quanta-gqlang-v2-sema/tests/v3_reference_resolution.rs` | One checked source/document authority, relation and composition frames, complete-profile certification independent of authored item order, exact semantic budgets, and source-qualified reference facts |
| Closed authority and frame facts | `crates/quanta-gqlang-v2-sema/tests/v3_authority_builder.rs`, `crates/quanta-gqlang-v2-sema/tests/v3_incremental_relation_frames.rs`, `crates/quanta-gqlang-v2-sema/tests/v3_semantic_kernel.rs` | Canonical workspace and pack/lock sealing, mutation rejection, retained host relation identity, and closed-document boolean type facts |
| Relational algebra | `crates/quanta-gqlang-v2-algebra/tests/v3_algebra_normalization.rs`, `crates/quanta-gqlang-v2-algebra/tests/v3_algebra_hellgate.rs`, `crates/quanta-gqlang-v2-algebra/tests/v3_algebra_mutants.rs`, `crates/quanta-gqlang-v2-algebra/tests/v3_algebra_probe.rs` | Typed lowering, normalization, semantic mutants, operator reachability probes, and exact item/root ownership for direct and transitive nested relations |
| Logic and graph rules | `crates/quanta-gqlang-v2-logic/tests/v3_logic_recursion.rs`, `crates/quanta-gqlang-v2-logic/tests/v3_logic_source.rs`, `crates/quanta-gqlang-v2-logic/tests/v3_logic_witness.rs`, `crates/quanta-gqlang-v2-logic/tests/v3_logic_hellgate.rs`, `crates/quanta-gqlang-v2-logic/tests/v3_graph_query_hellgate.rs` | Multi-rule SCC/stratification, exact admitted positive/negated/aggregated dependency polarity, recursive aggregate rejection, source provenance, witness stability, and graph-query legality |
| SCC and aggregate rejection | `crates/quanta-gqlang-v2-logic/tests/v3_rule_scc_hellgate.rs`, `crates/quanta-gqlang-v2-logic/tests/v3_rule_aggregate_policy.rs` | Actual canonical cycle path, permutation-invariant offending edge, nonlocal-target rejection, and fixed recursive-aggregate policy |
| Graph-query conflict and witness policy | `crates/quanta-gqlang-v2-logic/tests/test_endpoint_conflict.rs`, `crates/quanta-gqlang-v2-logic/tests/v3_graph_query_depth_waypoint_conflict.rs`, `crates/quanta-gqlang-v2-logic/tests/v3_logic_mutation_rails.rs` | Endpoint conflicts, depth/waypoint coexistence and order, subkind-specific bounds, witness cap and tie-break retention |
| Planning IR | `crates/quanta-gqlang-v2-planning/tests/v3_planning_truth.rs`, `crates/quanta-gqlang-v2-planning/tests/v3_planning_hellgate.rs`, `crates/quanta-gqlang-v2-planning/tests/v3_planning_cost_schema.rs`, `crates/quanta-gqlang-v2-planning/tests/v3_item_hellgate.rs`, `crates/quanta-gqlang-v2-planning/tests/v3_capability_derivation.rs`, `crates/quanta-gqlang-v2-planning/tests/v3_explain_provenance.rs` | Current public plan schema; re-anchored Semi/Anti join admission; exact truth/capability/provenance derivation; meta, presentation, composition, CHECK identity, and sealed nested-payload matrices; canonical-one witness selection; fail-closed item planning |
| Qualitative cost classification | `crates/quanta-gqlang-v2-planning/tests/v3_cost_classification.rs` | Operator-to-cardinality/risk classifications and bounded recursive closure; these labels are not numeric estimates or physical-plan optimization |

## Composed pipeline

| Surface | Primary rails | Contract pinned |
|---|---|---|
| Compiled document assembly | `crates/quanta-gqlang-v2-pipeline/tests/v3_compiled_document_assembly.rs` | Closed source transaction, exact membership, duplicate authored projection alias rejection, authored FIND/CHECK/EXPLAIN and six presentation-verb fields retained in final plan policy, planning and typed non-plan outcomes, exact/over-limit source relation depth including nested EXISTS, admitted-depth formatting, typed rejection of overdeep expression trees |
| V3 execution producer | `crates/quanta-gqlang-v2-pipeline/tests/v3_compiled_document_assembly.rs`, `crates/quanta-gqlang-v2-pipeline/tests/fixtures/execution-v3-vectors.tsv`, `crates/quanta-gqlang-v2-pipeline/src/execution.rs` unit tests | Source-bound typed literals; source-ordered item identity; source/contract/qualification digest vectors, including projected host field-schema change; encoder-unit binding identity/order mutation sensitivity. Unequal raw item IDs miss on document-local lookup, but a reused raw ID is not provenance proof. This proves producer semantics, not downstream admission or execution |
| Composition source lowering | `crates/quanta-gqlang-v2-pipeline/tests/v3_composition_source_query.rs` | Source-to-HIR-to-sema path for SAMPLE and the composition body families, including mixed-document item variants |
| Analysis document assembly | `crates/quanta-gqlang-v2-pipeline/tests/v3_analysis_document_assembly.rs` | Partial open authority stays incomplete; fully populated open authority can complete analysis without issuing an execution document; blocked-stage retention, deterministic diagnostics, compilation parity, and over-limit relation rejection |

## Shipped front doors

| Front door | Binary rails | Contract pinned |
|---|---|---|
| CLI | `crates/quanta-gqlang-v2-cli/tests/open_analysis_contract.rs`, `crates/quanta-gqlang-v2-cli/tests/ingress_matrix.rs`, `crates/quanta-gqlang-v2-cli/tests/v3_cli_fmt_idempotence.rs`, `crates/quanta-gqlang-v2-cli/tests/command_doc_gate.rs` | Open-authority incomplete/error semantics, inline/file/non-UTF-8 ingress, exact source-size boundary, exit/channel contract, formatting fixed point, every shipped example through `fmt --file` and `query --file`, command/documentation allowlist |
| Diagnostics stdio | `crates/quanta-gqlang-v2-diagnostics-stdio/tests/ndjson_contract.rs`, `crates/quanta-gqlang-v2-diagnostics-stdio/tests/resource_boundaries.rs`, `crates/quanta-gqlang-v2-diagnostics-stdio/tests/server_rejection_matrix.rs`, `crates/quanta-gqlang-v2-diagnostics-stdio/tests/binary_contract.rs` | Strict NDJSON schema, deterministic canonical diagnostics, malformed-byte/JSON and interrupted-read recovery, per-response flushing and flush-error propagation, request/method/ID rejection, frame/source/diagnostic/response bounds, binary exit/channel behavior, and the closed read-only command plane |
| Front-door parity | `tools/test_frontdoor_parity.py` | Black-box CLI/diagnostics agreement without a Cargo dependency between shipped adapters |
| Shipped example corpus | `crates/quanta-gqlang-v2-surface/tests/v3_examples_corpus.rs` | Every shipped example parse/format round trip, exact single-index listing with README routing, and single-root entry-point agreement |

## Architecture and composition gates

| Gate | Rail | What it proves |
|---|---|---|
| Public downstream boundary | `tools/test_frontend_external_consumer.py` | Cargo-parsed fixture→facade→pipeline path-only dependency seam, external consumer build/run, selected V3 composition item and shared digest-vector replay, curated accessor return types and closed-enum variants, compile-fail private-boundary probes, and zero direct or globbed public leakage. No provider or query runtime executes here |
| Architecture boundary | `tools/architecture/tests/test_architecture.py`, `tools/architecture/check_architecture.py` | Workspace dependency DAG and forbidden/retired edge rejection, including dependency-kind distinctions |
| Policy and tooling integrity | `tools/lint/tests/test_semgrep_non_vacuity.py`, `tools/prompt-manager/tests/test_pm.py`, `tools/qualification/tests/test_qualify_local.py`, `tools/qualification/tests/test_external_consumer_cache.py`, `tools/qualification/tests/test_frontdoor_parity.py` | Semgrep positive-fixture non-vacuity, scoped prompt imports, managed checkout/lock/receipt behavior, external-consumer cache/privacy-probe custody, and Cargo-emitted frontdoor executable selection |
| Documentation path integrity | `crates/quanta-gqlang-v2-surface/tests/v3_doc_inventory.rs` | Named Rust rail paths in this ledger exist; it does not establish reverse coverage or assertion quality |
| Registry integrity | `crates/quanta-gqlang-v2-conformance/tests/self_validation.rs` | All 18 registered roots compile through the facade with exact V2 disposition and V3 item/family/payload-presence mapping; both supplemental sources retain exact no-plan or document-program membership and stay outside the V3 execution DTO; unsupported roots reject with an exact diagnostic/span; closed-source predicates and aggregate contexts retain exact positive/negative semantics; authored inner/CROSS joins retain mode, ON, inputs, and prior filter in the V3 relation tree; two authored mutually recursive rules form one checked program with two ordered members |
| Full local composition | `just ci` | Repository-configured format, lint, architecture, Semgrep, test, and conformance composition; it is local proof, not hosted-CI evidence |

## Maintenance rule

Add or revise a row when a contract boundary, failure mode, or its primary proof
changes. Rename or delete references in the same change as the rail; the doc
inventory test checks named Rust paths, not whether every test target appears.
Use generated nextest inventory for total counts and missing execution, not a
second hand-edited manifest. Owner tests, binary tests, and conformance are
distinct proof types; do not promote one to another.
`command_doc_gate.rs` pins the shipped CLI allowlist and diagnostics method
documentation. Test filenames are not authority for a retired protocol; each
table entry must state the actual current boundary.
