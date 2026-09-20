# 07 — Semantic Planning And Public Facade

Status: `PLANNED`

Decision authority: [00 — Packet Scope And Pinned Decisions](./00-packet-scope-and-pinned-decisions.md)

## Findings Owned / To Close

F13, F18, F20's consumer half, F21, F23's public exposure, and F31.

## Goal

Make planning a pure consumer of verified semantic contracts and expose only a
small stable frontend API.

## Current-Head Reuse/Delete/Build Audit

| Class | Assets | Concrete action |
|---|---|---|
| `KEEP` | capability, semantic-cost, effect, result, evidence and witness vocabularies; valid capability/cost traversals; `produce_plan` legality checks | preserve terminology and algorithms whose input-independent semantics remain valid |
| `MIGRATE` | `PlanRequestV1`/`produce_plan`, explain provenance, graph/relational traversals | accept sealed semantic contracts and retained effect metadata |
| `DELETE` | `classify_item`/family raw-HIR classifiers, option-dropping matches, hardcoded truth, facade glob exports | remove in the same cut that migrates their callers |
| `NEW` | `SemanticPlanInputV2`, separated authority/evaluation fields, curated facade DTO surface | do not create a parallel planner beside `produce_plan` |

The seven planning suites are migrated by responsibility: vocabulary/cost/
capability assertions stay; raw-HIR classification fixtures are rewritten or
deleted. `frontend-contract/tests/downstream_contract.rs` is replaced by the
curated positive integration test and facade `compile_fail` doctests, so old
glob-export assertions cannot remain green.

## Write Set

1. `crates/quanta-gqlang-v2-planning/src/plan.rs`
2. `crates/quanta-gqlang-v2-planning/src/item.rs`
3. `crates/quanta-gqlang-v2-planning/src/effect.rs`
4. `crates/quanta-gqlang-v2-planning/src/cost.rs`
5. `crates/quanta-gqlang-v2-planning/src/capability.rs`
6. `crates/quanta-gqlang-v2-planning/src/result_kind.rs`
7. `crates/quanta-gqlang-v2-planning/src/diagnostic.rs`
8. `crates/quanta-gqlang-v2-planning/src/explain.rs`
9. `crates/quanta-gqlang-v2-planning/src/lib.rs`
10. `crates/quanta-gqlang-v2-planning/Cargo.toml` — integrator only; remove
    direct HIR edge and add sema
11. `/Cargo.lock` — integrator only if the edge cut changes it
12. `crates/quanta-gqlang-v2-pipeline/src/compile.rs` — integrator only
13. `crates/quanta-gqlang-v2-pipeline/src/lib.rs` — integrator only
14. `crates/quanta-gqlang-v2-pipeline/src/diagnostic.rs` — integrator only
15. `crates/quanta-gqlang-v2-frontend-contract/src/lib.rs` — integrator only
16. `crates/quanta-gqlang-v2-frontend-contract/Cargo.toml` — integrator only
17. the exact planning/pipeline/facade tests listed below
18. `docs/plans/sep-21-gqlang-sota-frontend-reconstruction/receipts/07.md`
19. `crates/quanta-gqlang-v2-pipeline/tests/v3_pipeline_compile.rs`
20. `crates/quanta-gqlang-v2-planning/tests/support/mod.rs` when repeated
    planning helpers from touched tests are consolidated
21. `tools/test_frontend_external_consumer.py` plus isolated positive and
    negative fixtures under
    `crates/quanta-gqlang-v2-frontend-contract/tests/external_consumer/`

## Implementation

1. Remove raw-HIR `classify_item`, `classify_find`, `classify_check`, and
   `classify_presentation` entry points.
2. Add `SemanticPlanInputV2` over relational, graph, check, presentation, and
   rule contracts.
3. Separate `AuthorityLaneV2` from `EvaluationDeterminismV2`.
4. Derive query determinism, monotonicity, capabilities, and semantic cost from
   retained expression effects.
5. Preserve exact FIND witness contract in plan output.
6. Validate and retain CHECK mode/confirm/emit semantics.
7. Validate and retain EXPLAIN evidence/truth semantics.
8. Retain SAMPLE mode/limit, COMPARE metric, and EXPORT format/profile.
9. Keep physical execution and provider fields absent.
10. Replace facade glob exports with selected document APIs and stable DTOs.
11. Hide raw HIR builders, unchecked constructors, and normalizer helpers.
12. Assemble every semantic document through ticket 05 algebra, ticket 06
    logic, and this ticket's planning before constructing `CompiledDocumentV2`.
    Open analysis receives explicit stage statuses and never borrows the V1 path.
13. Give `SemanticCostV2` one ordered field iterator/accessor as the canonical
    schema order; ticket 08's CLI renderer consumes it rather than hardcoding a
    second five-field list.

## Required Planning Input

```rust
pub enum SemanticPlanInputV2 {
    Relational(LoweredRelationItemV2),
    Graph(GraphQueryContractV2),
    Check(CheckContractV2),
    Presentation(PresentationContractV2),
    Rule(RuleProgramContractV2),
}

pub struct SemanticPlanV2 {
    authority_lane: AuthorityLaneV2,
    evaluation_determinism: EvaluationDeterminismV2,
    evidence: EvidenceRequirementV2,
    effect: EffectClassV2,
    result: ResultContractV2,
    capabilities: BTreeSet<CapabilityV2>,
    semantic_cost: SemanticCostV2,
}
```

The planner may derive summaries from sealed logical algebra/logic contracts but
may not inspect authored HIR options.

## Required Public Facade

`quanta-gqlang-v2-frontend-contract` exports exactly these functions and stable
DTO roots; nested DTO fields use curated explicit exports, never crate globs:

```rust
compile_source_document_v2(SourceIdV2, &str, &ClosedFrontendAuthorityV2)
    -> Result<CompiledDocumentV2, FrontendFailureV2>
analyze_source_document_v2(SourceIdV2, &str, &OpenAnalysisAuthorityV2)
    -> Result<AnalysisDocumentV2, FrontendFailureV2>
format_source_document_v2(SourceIdV2, &str, &FrontendBudgetV2)
    -> Result<String, FrontendFailureV2>
```

Stable root types are `ClosedFrontendAuthorityV2`,
`ClosedFrontendAuthorityBuilderV2`, `OpenAnalysisAuthorityV2`,
`FrontendBudgetV2`, `CompiledDocumentV2`,
`AnalysisDocumentV2`, `SemanticPlanV2`, `FrontendDiagnosticV2`, and the curated
host-assignable `SourceIdV2`. The facade
also exports `FrontendFailureV2`. It does not expose owner constructors for HIR,
types, algebra, logic, or planning.

Ticket 07 is the first ticket allowed to define these final functions/types.
`compile_source_document_v2` returns success only after all applicable
relational, rule, graph, item, and plan contracts are present. A declaration
that legitimately has no executable plan carries an explicit checked
`NoPlanReasonV2`; absence is never represented by a skipped stage.

`AnalysisDocumentV2` stores each downstream stage as a non-optional status:

```text
Complete(payload)
BlockedByDiagnostics(ordered_diagnostics)
NotApplicable(NoPlanReasonV2)
```

Only `SemanticAnalysisV2::Complete` may invoke checked algebra, logic, and
planning. `Incomplete` maps later stages to `BlockedByDiagnostics`; the public
`Err(FrontendFailureV2)` branch maps to wire error. Fully resolved open analysis
may carry complete stage
payloads but remains `AnalysisDocumentV2`. There is no implicit or fallible
conversion to `CompiledDocumentV2`.

## Audited Existing-Test Disposition

1. Delete the proven redundant recursive-witness aggregate test, duplicate
   union/witness classification rows, duplicate gate/sample markers, and
   duplicate explain recursive flag after their stronger owner assertions are
   migrated.
2. Delete `NoPlanReasonV1::keyword` and its spelling pin with the V1 raw-HIR
   classifier. It has no production consumer.
3. Replace `v3_planning_cost_schema.rs` plus the V1 pipeline duplicate with one
   V2 owner-schema assertion and one CLI consumer assertion. Never delete both
   sides before `SemanticCostV2` has a production reader.
4. Preserve every cost/capability/result keyword that the final CLI actually
   renders. Remove only verified no-consumer pins.
5. Normalize touched lint headers and consolidate `id/span/col/lit/pred/node`
   helpers inside planning test support after semantic migration.
6. Add compile-fail facade tests proving incomplete analysis and fully resolved
   open analysis cannot convert into `CompiledDocumentV2`.
7. Absorb the four borderline routing assertions from
   `audit_18_families::{f01,f11,f13,f15}` into the relevant planning/item owner
   matrices before ticket 09 removes those routing rows.

## Item Contract Matrix

1. CHECK:
   - pattern mode forbids semantic confirm
   - semantic mode requires semantic confirm
   - hybrid requires the explicitly documented pattern and semantic inputs
   - `confirm=skip|prompt|require` maps to an explicit authority/evidence policy
   - `emit=sarif` changes render/export contract
2. EXPLAIN:
   - evidence and truth values map independently
   - unsupported pairs reject with a planning code
3. FIND:
   - copy exact witness contract, never only a risk class
4. SAMPLE:
   - retain mode and positive limit
5. COMPARE:
   - retain metric and both resolved asset IDs
6. EXPORT:
   - retain format/profile and export-only effect
7. Relational:
   - fold expression effects; `now()` yields nondeterministic evaluation

## Step-By-Step Procedure

1. Introduce V2 lane/effect/result/cost structures.
2. Implement one planner function per semantic input variant.
3. Migrate relational planning from `PlanRequestV1` to the sealed contract.
4. Migrate graph planning and delete graph reclassification.
5. Migrate CHECK and presentation families with exhaustive option matches.
6. Add effect aggregation across all expressions in logical algebra.
7. Delete raw-HIR planning entry points.
8. Add pipeline final assembly from `SemanticDocumentV2` plus ticket 05/06
   outputs and reject any missing applicable contract.
9. Add closed/open source and checked-document pipeline entry points; export
   only the source entry points through the facade.
10. Build a curated facade exporting source/document functions and stable DTOs.
11. Add facade `compile_fail` doctests for each forbidden owner root and a
    positive downstream integration test for every curated root; no snapshot
    test dependency is required.
12. Remove algebra/HIR/planning/surface/types glob exports in the same cutover.

## Test Files

1. `[UPDATE]` `crates/quanta-gqlang-v2-planning/tests/v3_capability_derivation.rs` — sealed-input capabilities/effects
2. `[UPDATE]` `crates/quanta-gqlang-v2-planning/tests/v3_cost_classification.rs` and `crates/quanta-gqlang-v2-planning/tests/v3_planning_cost_schema.rs` — retain vocabulary/schema
3. `[UPDATE]` `crates/quanta-gqlang-v2-planning/tests/v3_explain_provenance.rs` — sealed provenance only
4. `[MOVE+REWRITE]` `crates/quanta-gqlang-v2-planning/tests/v3_item_hellgate.rs` to `crates/quanta-gqlang-v2-planning/tests/v3_semantic_item_contracts.rs` — CHECK/EXPLAIN/presentation matrices
5. `[UPDATE]` `crates/quanta-gqlang-v2-planning/tests/v3_planning_hellgate.rs` and `crates/quanta-gqlang-v2-planning/tests/v3_planning_truth.rs` — legality, effect, witness retention
6. `[MOVE+REWRITE]` `crates/quanta-gqlang-v2-frontend-contract/tests/downstream_contract.rs` to `crates/quanta-gqlang-v2-frontend-contract/tests/public_api_surface.rs`
7. `[UPDATE]` `crates/quanta-gqlang-v2-frontend-contract/src/lib.rs` — forbidden-import `compile_fail` doctests
8. `[ADD]` `crates/quanta-gqlang-v2-pipeline/tests/v3_compiled_document_assembly.rs`
9. `[ADD]` `crates/quanta-gqlang-v2-pipeline/tests/v3_analysis_document_assembly.rs`
10. `[ADD]` `crates/quanta-gqlang-v2-frontend-contract/tests/external_consumer/` — fixture with only the facade dependency
11. `[MIGRATE+RUN DEFERRED]` pipeline document, relational, group/order,
    rule, graph, and legacy-ratchet tests enumerated by receipts 04-06

Keep existing cost/capability/provenance cases. Delete raw-HIR classification
fixtures after their semantic-contract replacement; do not retain both public
contracts.

## Exact Verification

```bash
cargo test -p quanta-gqlang-v2-planning
cargo test -p quanta-gqlang-v2-pipeline --test v3_compiled_document_assembly
cargo test -p quanta-gqlang-v2-pipeline --test v3_analysis_document_assembly
cargo test -p quanta-gqlang-v2-frontend-contract
cargo test -p quanta-gqlang-v2-frontend-contract --doc
python3 tools/test_frontend_external_consumer.py
cargo test -p quanta-gqlang-v2-pipeline --test v3_document_semantics_v2
cargo test -p quanta-gqlang-v2-pipeline --test v3_relational_semantics_v2
cargo test -p quanta-gqlang-v2-pipeline --test v3_source_rule_document
cargo test -p quanta-gqlang-v2-pipeline --test v3_graph_contract_retention
cargo clippy -p quanta-gqlang-v2-planning -p quanta-gqlang-v2-pipeline -p quanta-gqlang-v2-frontend-contract --all-targets -- -D warnings
python3 tools/architecture/check_architecture.py
rg -n '^pub use .*::\*' crates/quanta-gqlang-v2-frontend-contract/src
git diff --check
```

The `rg` command must return no matches.

## Forbidden Shortcuts

1. adding a nondeterministic truth-lane alias while leaving hardcoded behavior
2. retaining HIR classification as a deprecated fallback
3. flattening verified witness fields into qualitative risk only
4. exposing internal V2 nodes through the facade for migration convenience
5. adding physical/runtime fields to semantic plans
6. manufacturing a final compiled/analysis DTO from ticket 04 semantics without
   running applicable algebra, logic, and planning stages

## RED

1. `now()` plans as deterministic
2. CHECK option permutations produce identical plans
3. EXPLAIN evidence/truth permutations produce identical plans
4. FIND boundedness details disappear after planning
5. facade exposes every internal type and constructor
6. no V2 path can assemble a complete `CompiledDocumentV2` from ticket 04-06
   outputs without falling back to V1

## Tests

1. deterministic vs nondeterministic builtin plan tests
2. pure/impure and monotone/nonmonotone effect aggregation
3. CHECK legal/illegal mode matrix
4. EXPLAIN evidence/truth matrix
5. SAMPLE/COMPARE/EXPORT payload retention
6. FIND exact depth/cap/dedup/tie-break retention
7. compile-fail facade doctests for representative HIR builder, type checker,
   algebra node, logic implementation, and planning implementation imports
8. no-runtime-token architecture scan
9. final document assembly rejects a deliberately omitted algebra, logic, or
   plan contract and preserves explicit `NoPlanReasonV2` for non-executable
   declarations

## Mutation Cases

1. hardcode evaluation determinism
2. drop CHECK emit mode
3. reclassify FIND from raw HIR
4. reintroduce one facade glob export

## Definition Of Done

1. planning has no raw HIR dependency
2. every plan field is derived from a sealed input contract
3. plan naming/documentation is explicit that it is semantic, not physical
4. frontend-contract depends on pipeline and exposes only curated APIs/DTOs
5. ticket 04 semantic output cannot bypass complete pipeline assembly

## Dependencies

## 2026-09-21 Final Adversarial Audit Corrections

This section is normative and supersedes conflicting earlier wording.

### Integration gate

Ticket 07 is the first cross-stage integration gate. Before planning, compile
sema, binder, algebra, logic, planning, and pipeline together; migrate every
deferred consumer from receipts 04-06; run the deferred group/order/rule/graph
tests; and require zero old private-HIR references. A 04-06 owner pass is not a
substitute.

### Exact assembly and error model

`LoweredDocumentV2` is pipeline-owned and contains private exact-key maps for
algebra outputs, graph outputs, the optional document rule program, and
presentation inputs, plus source order. Construction compares exact sets:

```text
expected_applicable_item_ids == produced_item_ids
```

Missing, duplicate, extra, cross-document, and wrong-kind outputs are typed
failures. Positional zip is forbidden. CHECK assembly verifies that its nested
`GraphQueryContractV2` carries the identity named by `ResolvedCheckV2`.

Use `Result<AnalysisDocumentV2, FrontendFailureV2>` and
`Result<CompiledDocumentV2, FrontendFailureV2>` at the public facade. A
successful `AnalysisDocumentV2` has only `Complete`, `BlockedByDiagnostics`, or
`NotApplicable` item states; transport-level `error` is the `Err` branch and is
not a fourth internal status.

`CompiledDocumentV2` has private fields and accessors for source identity,
canonical diagnostics, source-ordered `CompiledItemV2` entries, and authority
fingerprint. Each item is a closed enum whose variants carry the relevant
checked contract and `Option<SemanticPlanV2>`; `None` is legal only for the
explicit `NotApplicable` variants. No parallel arrays are public.

The document-wide `RuleProgramContractV2` receives exactly one plan stored in
`CompiledRuleProgramV2`. Each source rule item stores a checked
`RuleProgramMemberV2 { program_id, member_item_id }` reference and does not
duplicate the plan. Rule member IDs are excluded from per-item plan-key
equality and are instead checked for exact equality with the program member
set. Missing, extra, duplicate, or foreign member IDs reject.

### Planning vocabulary mapping

Pin an exhaustive conversion table in code and tests:

| Existing input | AuthorityLaneV2 | EvaluationDeterminismV2 |
|---|---|---|
| local/exact | local | deterministic iff effects say deterministic |
| catalog/certified | certified | derived from effects |
| model/advisory | advisory | derived from effects |
| unresolved open-analysis item | no plan | not applicable |

All `SemanticCostV2` fields remain qualitative and ordered exactly as
`cpu`, `memory`, `io`, `network`, `coordination`; the public iterator returns
`SemanticCostDimensionV2` in that order. No cardinality estimate or physical
operator is introduced.

### Public closed-authority path

The facade owns `ClosedFrontendAuthorityBuilderV2` and bounded declarative DTOs
for module, relation, source-registry, and budget inputs. The builder invokes
pipeline, which delegates validation/sealing to modules/types/sema, then returns
`ClosedFrontendAuthorityV2`; the facade adds no direct modules/types
dependency. Raw maps,
empty defaults, and direct internal-crate dependencies by hosts are forbidden.
Only the minimum checked DTO/builders needed by this path are curated exports.

Facade proof is two-part: (1) an AST/source inventory of every `pub use`; and
(2) an external fixture whose sole dependency is `frontend-contract`, proving
both positive construction/compile and compile-fail for internal owner types.
A doctest that merely lacks direct dependencies is not sufficient.

Requires 05 and 06. Blocks 08 and 09.
