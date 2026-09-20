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
10a. `crates/quanta-gqlang-v2-pipeline/Cargo.toml` — integrator only; remove
     direct HIR, Binder, and types edges after migrating production imports;
     the final pipeline edges are surface+sema+algebra+logic+planning+modules
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
2. Add `SemanticPlanInputV2` over relational, graph, check, presentation,
   document rule-program, REQUIRE CERTIFIED meta, and checked composition
   body contracts. Catalog, value Declaration, and USE PACK are explicit
   nonplan outcomes, not silently skipped items.
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
    second field list.

## Required Planning Input

```rust
pub enum SemanticPlanInputV2 {
    Relational(LoweredRelationItemV2),
    Graph(GraphQueryContractV2),
    Check(CheckContractV2),
    Presentation(PresentationContractV2),
    Rule(RuleProgramContractV2),
    MetaRequireCertified(ResolvedMetaV2),
    Composition(CheckedCompositionPlanInputV2),
}

pub struct SemanticPlanV2 {
    authority_lane: AuthorityLaneV2,
    evaluation_determinism: EvaluationDeterminismV2,
    evidence: EvidenceRequirementV2,
    effect: EffectClassV2,
    result: ResultContractV2,
    capabilities: BTreeSet<CapabilityV2>,
    semantic_cost: SemanticCostV2,
    explain: ExplainNoteV2,
}
```

The planner may derive summaries from sealed logical algebra/logic contracts but
may not inspect authored HIR options.
`ResultContractV2` is not an empty result-kind wrapper: it retains the closed
`ResultKindV2`, per-row `RenderRequirementV2`, optional bounded
`WitnessSelectionPolicyV2`, and `MultiplicityOrderContractV2` (including
cardinality/order semantics). FIND also retains the exact graph witness
depth/cap/dedup/tie-break provenance from `GraphQueryContractV2` rather than
reclassifying HIR. `ExplainNoteV2` retains a checked root location, ordered
logical algebra family list, and recursive flag. Migrate the existing
`PublicPlanV1` fields; do not drop them while replacing its raw input.
Tests must compare these fields across logic/algebra → planner → facade → CLI,
including wrong origin, witness cap, order, and recursive-flag mutations.
`CheckedCompositionPlanInputV2` retains the composition root ID, ordered
producer item IDs and their checked outputs, exact `WITH` binding identity,
and one sealed body input. It dispatches to the body's existing plan policy;
it is not a second classifier. Do not turn nested producer outputs into
extra top-level plans. `ResolvedMetaKindV2::UsePack`, Catalog, and value
Declaration produce checked `NoPlanReasonV2` outcomes. An inline VIEW body
is lowered through Ticket 05 for its typed relation/schema proof but its
Catalog declaration still has no per-declaration serving plan.

## Required Public Facade

`quanta-gqlang-v2-frontend-contract` exports exactly these functions and stable
DTO roots; nested DTO fields use curated explicit exports, never crate globs:

```rust
compile_source_document_v2(SourceIdV2, &str, &ClosedFrontendAuthorityV2)
    -> Result<CompiledDocumentV2, FrontendTransactionErrorV2>
analyze_source_document_v2(SourceIdV2, &str, &OpenAnalysisAuthorityV2)
    -> Result<AnalysisDocumentV2, FrontendTransactionErrorV2>
format_source_document_v2(SourceIdV2, &str, &FrontendBudgetV2)
    -> Result<String, FrontendFailureV2>
```

Stable root types are `ClosedFrontendAuthorityV2`,
`ClosedFrontendAuthorityBuilderV2`, `OpenAnalysisAuthorityV2`,
`FrontendBudgetV2`, `CompiledDocumentV2`,
`AnalysisDocumentV2`, `SemanticPlanV2`, `FrontendDiagnosticV2`,
`FrontendAuthorityErrorV2`, `FrontendTransactionErrorV2`, and the curated
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
`Err(FrontendTransactionErrorV2)` branch maps to wire error. Fully resolved open analysis
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
   V2 planning-owner schema assertion. Record the CLI consumer assertion as a
   Ticket 08 deferred gate; Ticket 07 does not edit CLI across the owner
   boundary. Never delete both sides before `SemanticCostV2` has a production
   reader.
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
4. `[MOVE+REWRITE]` `crates/quanta-gqlang-v2-planning/tests/v3_item_hellgate.rs` to `crates/quanta-gqlang-v2-planning/tests/v3_semantic_item_contracts.rs` — CHECK/EXPLAIN/presentation/Meta/Composition and explicit Catalog/Declaration nonplan matrices
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
10. REQUIRE CERTIFIED plan only from the exact modules-validated directive;
    Model Catalog and USE PACK remain typed nonplan outcomes, independent of
    provider/provenance display text
11. Composition Query/FIND/CHECK/EXPLAIN bodies dispatch via the same sealed
    plan policy; wrong-body, wrong-producer, and swapped WITH target IDs reject
12. inline VIEW relation schema/provenance is lowered and checked once, while
    the Catalog item remains a single nonplan top-level entry
13. result render requirement, witness selection/cap/tie-break,
    multiplicity/order, and explain location/family/recursive flag survive
    planner→facade→CLI with exact values

## Mutation Cases

1. hardcode evaluation determinism
2. drop CHECK emit mode
3. reclassify FIND from raw HIR
4. reintroduce one facade glob export
5. swap composition body or inline VIEW producer under a valid same-source ID
6. replace the modules-certified Meta proof with a same-spelling string or
   force a Model Catalog declaration into an advisory serving plan
7. retain the dead `unsupported_reason_code=none` success field or introduce
   physical cost labels in the V2 schema

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
presentation/meta/composition inputs, plus source order. Construction compares
exact sets for both top-level items and nested producer/body identities:

```text
expected_applicable_item_ids == produced_item_ids
```

Missing, duplicate, extra, cross-document, and wrong-kind outputs are typed
failures. Positional zip is forbidden. CHECK assembly verifies that its nested
`GraphQueryContractV2` carries the identity named by `ResolvedCheckV2`.
Composition assembly verifies each ordered producer's ID/output kind, the
`WITH` reference target, and its one body ID/contract; same-document but
wrong-body or wrong-producer transplants reject. Inline VIEW's algebra output
is matched to its Catalog payload without creating a top-level plan key.
The pipeline/facade `SourceIdV2` is the canonical HIR-defined identity
re-exported through sema, not a duplicate alias or hidden direct HIR Cargo
edge. Audit all pipeline/planning production imports and manifest edges after
the cut; strict final architecture must not depend on a transitional
pipeline→HIR/Binder/types path.

Use `Result<AnalysisDocumentV2, FrontendTransactionErrorV2>` and
`Result<CompiledDocumentV2, FrontendTransactionErrorV2>` at the public facade. A
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

| Sealed input | AuthorityLaneV2 | EvaluationDeterminismV2 |
|---|---|---|
| ordinary typed relation, graph, CHECK, or presentation | frontend_checked | deterministic iff retained effects say deterministic |
| exact `REQUIRE CERTIFIED` item matched to modules admission proof | certified_requirement_validated | derived from effects; this does not certify unrelated outputs |
| Model catalog declaration | no serving plan | not applicable; advisory is not inferred from its name/provider bytes |
| unresolved open-analysis item | no plan | not applicable |

No accepted source producer currently carries a general certified/advisory
truth lane for arbitrary relations. `RelationSourceV2::provenance` is a free
string, not certification. Ticket 04 must retain the exact modules-validated
`REQUIRE CERTIFIED` directive proof; Ticket 07 consumes it. Do not classify
catalog/model/host text into a lane. A future advisory model plan requires a
new typed authority producer and separate packet, not a planner guess.

The V1 `SemanticCostV1::FIELD_NAMES`/CLI renderer pin five fields, but every
successful `PublicPlanV1` producer assigns `unsupported_reason_code: None`;
rejection returns a typed `PlanningDiagnosticV1` instead. V2 deliberately
uses four ordered success fields: `estimated_cardinality_class`,
`recursion_risk_class`, `witness_explosion_risk_class`, and
`ordering_contract`. Move unsupported reason codes solely to typed failure
diagnostics. Use one `SemanticCostV2` ordered accessor/iterator and update the
planning-owner schema assertion here. Ticket 08 owns the CLI consumer renderer
and old fifth-field CLI tests; Ticket 07 records that deferred consumer gate.
A successful CLI plan must emit exactly the four names and
must not render `unsupported_reason_code=none`. Do not introduce `cpu`,
`memory`, `io`, `network`, or `coordination` fields: those suggest physical
estimates the frontend cannot derive. The cardinality *class* remains
qualitative; no numeric cardinality estimate or physical operator is added.

### Public closed-authority path

The facade owns `ClosedFrontendAuthorityBuilderV2` and bounded declarative DTOs
for module, relation, graph-reference selector, source-registry, and budget
inputs. The builder invokes
pipeline, which delegates validation/sealing to modules/types/sema, then returns
`ClosedFrontendAuthorityV2`; the facade adds no direct modules/types
dependency. Raw maps,
empty defaults, and direct internal-crate dependencies by hosts are forbidden.
Only the minimum checked DTO/builders needed by this path are curated exports.

Facade proof is two-part: (1) an AST/source inventory of every `pub use`; and
(2) an external fixture whose sole dependency is `frontend-contract`, proving
both positive construction/compile and compile-fail for internal owner types.
A doctest that merely lacks direct dependencies is not sufficient.

### Inherited external delta audit (`fcf106b..f9ec954`)

Commit `f9ec954` is an **UNQUALIFIED PARTIAL CONSUMER MIGRATION**, not Ticket 07
progress. Its 89 planning tests exercise the old V1 vocabulary/classifier.
Planning still depends directly on HIR and publicly exports `classify_item` and
`PublicPlanV1`; the commit adds another raw-HIR
`classify_composition_body`. That helper has no composition-specific rail,
maps every CHECK to `alert_plan`, and collapses composition EXPLAIN to a generic
proof plan, losing the exact mode/confirm/emit and evidence/truth contracts.

Ticket 07 must delete all raw-HIR classifiers in the same cut, remove the HIR
dependency, and route every option through the exhaustive semantic mapping.
Keep only vocabulary algorithms whose inputs are already sealed. At the
current inherited start, planning tests are focused V1 green, pipeline is red
with 25 compiler errors, and strict planning/binder Clippy is red because the
Ticket 05 algebra dependency contains 6 denied allow-attribute errors. Receipt
07 must report those as distinct evidence classes and cannot count architecture
lint green while 22 transitional normal edges remain as final-DAG proof.

Requires 05 and 06. Blocks 08 and 09.

## 2026-09-21 Live-Tree Final Reconciliation (HEAD `f9ec954`)

The 89 inherited planning tests prove the V1 vocabulary and classifier only.
`PlanRequestV1` still exposes a raw algebra root and caller-selected result,
effect, and truth; `PublicPlanV1` has no physical tree by design. Do not add a
physical optimizer to satisfy the old review. The repair is a sealed semantic
input and exhaustive semantic-policy projection, with the existing cost,
capability, result, explain, and witness vocabulary retained where valid.

Before implementing the facade, build an exact-key integration table from
Ticket 04 semantic items, Ticket 05 relation outputs, and Ticket 06 graph/rule
outputs. For every applicable item, require one correct-kind result; for
declarations require a checked `NoPlanReasonV2`; for rules require one
document-program plan and exact member references rather than a plan copied
per rule. Inject missing, extra, duplicate, foreign, wrong-kind, and
same-ID/wrong-nested-graph outputs. Only after this table passes may the
facade expose `CompiledDocumentV2`/`AnalysisDocumentV2`.

The facade's closed-authority builder must be exercised from an isolated
external crate depending only on `frontend-contract`; positive construction
and negative internal-root imports are both required. Explicitly inventory
all `pub use` lines and normal Cargo edges after removal of the present HIR,
algebra, planning, surface, and types glob dependencies. A V1 owner test count
or empty doctest run is not a V2 external-consumer receipt.

Ticket 05's exact test inventory found an omitted pipeline consumer:
`crates/quanta-gqlang-v2-pipeline/tests/v3_relation_source_authority.rs`
still reads old algebra fields/string source IDs. Migrate its real source-ID,
column-origin, and wrong-authority assertions to the sealed semantic→algebra
contract in this Ticket 07 integration gate. Also migrate planning's direct
algebra field access after Ticket 05 makes node fields private; the earlier
89 V1 tests do not cover that cut. Do not silently delete either rail to
obtain a workspace compile.

## Composition And Group Consumer Audit Delta

Ticket 04's Binder already owns sequential LET visibility and the
non-capturing outer-composition barrier. Its sema producer must provide an
exact `(composition ID, LET ordinal) -> typed local output` table; planning
does not re-resolve names or treat LET output as a host catalog asset. For a
Composition plan, require exactly one checked `WITH` target in that table,
one kind-compatible sealed body result, and exact nested producer/body item
identities. Reject missing, extra, duplicated, wrong-kind, forward/self,
foreign-composition, and same-display-name/wrong-ordinal outputs; no plan is
created for each LET as an independent root item.

The nested producer output is not newly invented by planning. Ticket 04
migrates the exact existing Sample row `Set<{path: RepoPath}>` and bounded
PATH/REACHABILITY FIND row `Set<{function: EntityRef}>` from the V1 pipeline
into sealed sema facts; other FIND kinds have no relation row. Ticket 07
consumes those facts and migrates `v3_composition_type_env.rs` in the pipeline
integration suite. Its V1 empty-projection/schema-open success becomes a
closed-compile negative; no missing local row may fall back to an unrelated
host relation with the same name.

For grouped queries, consume Ticket 05's key-only public result contract and
aggregate-only partition proof. `ResultContractV2` may expose only the final
row columns; it must not leak pre-group input columns or infer an aggregate
from a string. Explain retains the checked Group/Sort stages, including both
sort directions' sealed `NULLS LAST` policy. Add a planner→facade negative
that swaps Group output and partition-input identities while keeping the
same display names.
