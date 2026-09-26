# Lane E-1 — Pipeline assembly / V3 projection audit

## 1. Source boundary

- Snapshot: `docs/audits/lane-b-snapshot-20260927-010019` (shared with lane B; verified fresh for all lane-E files via sha256 manifest comparison against the live tree — zero drift at audit start).
- HEAD: `eaeb68ea2711d411616511713fb016d828cd80be`, branch `main`.
- Dirty boundary (tracked): cli/src/main.rs, cli/tests/v3_cli_fmt_idempotence.rs, pipeline/src/compile.rs, surface/src/{format,lib}.rs, surface/tests/{v3_format_hellgate,v3_surface_budget}.rs, docs/language/{README,test_hellgate_ledger}.md. Untracked: pipeline/tests/v3_format_budget.rs, docs/audits/, docs/bugbash/.../P3-format-closure-2026-09-27.md. Note: **pipeline/src/compile.rs is dirty** — conclusions below are snapshot-scoped.
- Note: subagent delegation for this half failed twice with no output; this report was produced inline by the lane-E coordinator. Static audit only; no cargo build/test executed (limitation).

## 2. Coverage

Read in full or in targeted depth: `pipeline/src/lib.rs` (70 ln, full), `pipeline/src/compile.rs` (2144 ln — entry points, orchestration, assembly validation, projections glue: lines 592–1758), `pipeline/src/execution.rs` (4381 ln — document/DTO core 1–380, plan/relation/composition projection 577–716, 1270–1430, literal projection 1724–1740, digest encoding 1900–2009; remaining encode helpers and graph witness projection skimmed via symbol map).
Not covered in depth: `pipeline/src/diagnostic.rs` (921 ln — code vocabulary/sort keys only skimmed), full graph-query witness projection internals, `run_item`'s `PlanningSessionV2`/`AlgebraSessionV2` internals (owned by lanes C/D), test files.

## 3. Traced paths

| Entry (compile.rs) | Path | Output |
|---|---|---|
| `compile_source_document_v3` (618) | `parse_source` → `CheckedSyntaxDocumentV2::try_from_hir` → `resolve_checked_document_v2` → `compile_resolved_parts` → `ExecutionDocumentV3::new` | `CompiledSourceDocumentV3` |
| `compile_resolved_document_v2` (650) | sealed-doc ingress (tests/gates) → same `compile_resolved_parts`, no execution doc | `CompiledDocumentV2` |
| `analyze_source_document_v2` (688) | parse → checked → open authority; partial authority → `incomplete_open_analysis` (never fabricated) | `AnalysisDocumentV2` |
| `analyze_resolved_document_v2` (725) | `run_document` → `into_analysis` | `AnalysisDocumentV2` |
| `format_source_document_v2` (751) | budget → parse-budget projection → format; no semantic traversal | `String` |
| `classify_single_source_root_v3` (592) | parse only; exactly-one-item enforced (599–607) | family |

## 4. Findings

### E1-01 — `ExecutionFamilyV3::RuleProgram` variant is never constructed in production
- Severity P3, classification DOCUMENTATION_DRIFT.
- Invariant: public DTO vocabulary should match reachable states (repo invariant: "do not derive public contracts from unreachable states"; rust-frontend.md L10 "unsupported payloads need explicit typed rejection").
- Evidence: `execution.rs:159` defines the variant; the only non-encoder reference is `family_tag` (`execution.rs:1978`) and a test. `execution_identity` (compile.rs:1600–1612) maps `RuleMember` → `None`, so no item ever carries it. Contract doc `docs/contracts/frontend-execution-v3.md:114` lists `RuleProgram` in the item-family vocabulary table while line 19 states rule programs "are not V3 execution items".
- Trigger/expected/actual: a downstream reader of the doc table expects possible `rule_program` items; none can ever occur; the digest tag 6 is dead encoding space.
- Refutation attempt: the doc (L19) does document the exclusion, and reserved vocabularies are a legitimate forward-compat device — but then the vocabulary table (L114) should mark it reserved, and the dead tag consumes schema-5 encoding space without a producer.
- Minimal fix: either annotate the doc table entry as reserved-for-future or remove variant+tag at the next schema bump. Owner: pipeline. Compatibility: removing requires contract digest schema bump; annotating is free. Regression oracle: doc lint / enum-variant reachability rail.

### E1-02 — document-wide rule program plan is outside `contract_digest` coverage
- Severity P3, classification PROOF_GAP (documented limitation, coverage semantics unpinned).
- Invariant: digest coverage claims "any DTO change moves it" (execution.rs:136–138) — but the rule program is not in the DTO at all.
- Evidence: `digest_contract` (execution.rs:1927–1953) encodes only `items`; the rule program's `SemanticPlanV2`/stratification live on `CompiledDocumentV2.rule_program` (compile.rs:403–436) and never enter `ExecutionDocumentV3`.
- Trigger/expected/actual: two compilations whose rule-program plans differ (e.g. stratification proof content) share an identical `ExecutionDocumentV3` including qualification digest. Whether that is intended ("the execution contract covers only executable items") is documented at doc L19 but the digest-coverage sentence overstates scope.
- Refutation attempt: source digest + contract digest intentionally separate concerns, and rule programs are explicitly non-executable in V3; a downstream that only executes items is unaffected. Survives as a proof gap only because the digest's documented coverage claim is broader than its actual input set.
- Minimal fix: one clarifying sentence in the contract doc ("contract_digest covers execution items only; rule-program facts live on the compiled document"). Owner: docs/pipeline. No code change. Regression oracle: none needed (documentation).

### E1-03 — `format_source_document_v2` re-parses the source it already budget-checked (double parse)
- Severity P3, classification OPTIONAL_REFACTOR (cost, not semantics).
- Evidence: `format_source_document_v2` (compile.rs:751–774) calls `enforce_source_budget`, then `project_parse_budget`, then `format_source_document_with_parse_budget_v2` which parses again internally (the surface formatter owns its parse). One extra bounded parse per format call.
- Refutation attempt: this reuses the surface owner's single canonical parse-and-format path instead of duplicating a formatter over HIR in the pipeline — the duplication-of-authority alternative would be worse; the cost is one bounded parse. Structurally repeated work is real but small and bounded; not a defect.
- Fix (optional): surface-level `parse_then_format` entry accepting a projected budget. Owner: surface. Compatibility: additive.

### E1-04 — `cap_transaction_error` diagnostic-limit cap replaces richer document failures (intended, verified)
- Severity P3, classification DOCUMENTATION_DRIFT (borderline refuted; recorded because the replacement is lossy by design and undocumented).
- Evidence: compile.rs:877–890 — a document failure carrying more diagnostics than `max_diagnostics` is replaced wholesale by a single `ANALYSIS_DIAGNOSTIC_LIMIT_CODE` error anchored at `zero_location` (point 0). Owner codes/spans of the original diagnostics are dropped.
- Refutation attempt: this is the documented budget contract (fail-closed on diagnostic budget; prevents unbounded error payloads) and `enforce_diagnostic_limit` in `into_analysis` (1088–1092) counts *before* cloning blocked diagnostics per member, showing deliberate ordering. The point-0 anchor is the only location an arbitrary failure set shares (`zero_location` doc, compile.rs:1703). Survives only as: the replacement (not truncation) strategy is not stated in the contract doc. Fix: doc sentence. Owner: docs.

### Refuted candidates (suspected, then cleared)

1. **Exact-key assembly**: `run_document`/`validate_produced_membership` (compile.rs:1142–1325) check missing/extra/duplicate/foreign via `BTreeSet` comparison per plan/member/no-plan class plus `record_produced_identity` duplicate guard; `DocumentMembershipV2::of` owns authored membership; `closure_error` rejects dangling/foreign expression IDs (compile.rs:1654–1672); `close_expression_ids` (execution.rs:1363–1383) fails closed on cross-source IDs. No silent-drop or partial-document path found. Legitimate boundary re-validation over an already-sealed document (caller-built `SemanticDocumentV2` via `compile_resolved_document_v2` is an explicitly supported forged-document ingress — the doc says so, compile.rs:641–643), not duplicated semantic decision.
2. **Projection re-deciding facts**: `project_plan` (execution.rs:577–616), `project_scalar` (328), `project_expression_shape` (1724+) are pure enum/struct mappings from sealed owner types; literal facts preserved exactly (`DeferredI128MinMagnitude → i128::MIN`, float `to_bits()` preserving −0/NaN payloads, strings already decoded). No re-parse, no re-typing, no re-binding found in the sampled projections.
3. **`PlanOutcomeMismatch` as duplicated planning logic**: compile.rs:1627–1642 only *classifies* the owner's `PlanOutcomeV2` against the assembly's plan-required membership — adapter/guard, not a second planner.
4. **Digest determinism**: domain strings + schema versions (`source-bytes.v3`/3, `contract.v5`/5, `qualification.v5`/5), length framing, explicit ordering (`BTreeSet` closure, source-order items). No hash-map iteration observed on digest inputs.
5. **Rule program double compilation**: `compile_rule_program` (compile.rs:1488–1519) retains the stratification proof from `RuleProgramContractV2::from_document` rather than re-running it (doc comment explicit; lane D owns the ingress).
6. **`incomplete_open_analysis`**: partial open authority → typed `Incomplete` with unresolved facts; never promotes to closed compile (compile.rs:698–704, 677–687 doc). Matches repo invariant.

## 5. Checks and limitations

- Static reading only; no build/test executed (subagent infra failed twice; inline audit). All line references are to the snapshot.
- `pipeline/src/diagnostic.rs` (sort keys, code vocabulary) only skimmed — lane F owns transport/ordre audits; handoff: `diagnostic_sort_key` determinism should be cross-checked in F's coverage matrix.
- Handoffs: rule-graph allowance charging lives in logic (`RuleProgramContractV2::from_document`) → lane D; `AnalysisStatusV2::Complete` without execution-document issuance semantics → lane C/E-2 boundary (E-2 confirmed doc coverage).
