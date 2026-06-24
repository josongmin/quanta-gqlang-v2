# 07 Planning Truth Evidence And Effects v3

## Goal

Complete plan-shape-aware semantics for capability, truth, evidence, and effect
classification.

## Why

Current planning producer maps root kind to a single capability/truth lane. It
cannot express that a query now contains joins, aggregates, recursive closure,
candidate-set export, or proof-only evidence requirements.

Current head already ships `quanta-gqlang-v2-planning` as the owner lane. This
ticket extends and hardens that lane; it does not authorize a second planning
classifier or result-kind stack.

## Deliverables

1. planning IR over normalized algebra/logic shape
2. capability requirement derivation from actual query features
3. truth lane model over query/program semantics
4. evidence requirements
5. effect model:
   - read-only
   - proof/evidence only
   - export-only
   - mutation-forbidden
6. result-kind model:
   - table
   - alert
   - path/witness
   - proof/explain
7. source-location and witness requirements for result rendering
8. logical explain/provenance contract over normalized semantics
9. semantic cost/cardinality/recursion-risk signals for tooling
10. multiplicity/order contract for rendered results
11. witness-selection policy:
   - canonical-one
   - top-k
   - all-witness
12. fixed semantic-cost schema:
    - `estimated_cardinality_class`
    - `recursion_risk_class`
    - `witness_explosion_risk_class`
    - `ordering_contract`
    - `unsupported_reason_code`
13. planning diagnostics for unsupported mixes

## Required Semantics

1. query shape, not root label, drives planning
2. AI/candidate/proof lanes stay explicit and typed
3. final policy truth remains impossible without the right proof semantics
4. result kind and witness shape derive from normalized semantics, not ad-hoc
   CLI formatting
5. logical explain/provenance stays distinct from downstream runtime physical
   planning or substrate profiling
6. unordered results stay semantically unordered; tooling may canonicalize
   rendering, but the language must not pretend an order exists
7. witness selection policy must be explicit rather than backend-accidental
8. semantic-cost fields are host-agnostic and stable across CLI/LSP/tests

## Non-Goals

1. this ticket does not introduce new source-language query semantics
2. this ticket does not redefine algebra or recursion legality
3. feature classification must consume normalized algebra/logic nodes rather
   than re-inspecting syntax/root kinds ad hoc
4. this ticket does not choose semantica runtime-native execution structures;
   it only emits host-agnostic planning/truth/evidence requirements
5. this ticket does not invent non-relational compiled artifacts; explicit
   artifact/no-serve contracts for standalone decl/rule units are ticket `15`

## Scenarios Unlocked

1. aggregate query requiring catalog + metric + grouping capabilities
2. recursive reachability query requiring graph/path semantics
3. candidate-set generation plus reconfirmation/proof planning
4. alert query with file/span plus witness payload expectations
5. explain-only request surfacing rule/path provenance without execution DTOs
6. path query that asks for canonical shortest witness versus all witnesses
7. cost inspection request returning stable risk/cardinality fields without
   runtime-native jargon

## Definition Of Done

1. planning no longer depends only on `PublicPlanRootKindV1`
2. advanced query features surface in planning output
3. truth/evidence/effect rails are green for both valid and invalid shapes
4. result-kind and witness/location expectations are explicit in planning output
5. logical explain/provenance and semantic cost signals are explicit in the
   host-agnostic planning output
6. multiplicity/order semantics and witness-selection mode are explicit in
   planning output
7. semantic-cost output uses the fixed field set rather than ad-hoc strings

## Dependencies

1. requires `00`
2. depends on `04` and `05`
3. may consume `01` and `03` outputs indirectly, but must not redefine them
4. explicit non-relational artifact closure lands in `15`, not here

## Likely Touch Points

1. `crates/quanta-gqlang-v2-planning/src/plan.rs`
2. `crates/quanta-gqlang-v2-planning/src/result_kind.rs`
3. `crates/quanta-gqlang-v2-planning/src/effect.rs`
4. `crates/quanta-gqlang-v2-planning/src/cost.rs`
5. `crates/quanta-gqlang-v2-planning/src/capability.rs`
6. `crates/quanta-gqlang-v2-planning/src/explain.rs`
7. `crates/quanta-gqlang-v2-planning/src/diagnostic.rs`

## Mandatory Worker Outputs

1. host-agnostic planning output keyed off normalized algebra/logic shape
2. fixed semantic-cost schema with stable field names
3. explicit witness-selection and multiplicity/order contracts

## Proof Rails

1. extend `crates/quanta-gqlang-v2-planning/tests/v3_planning_truth.rs`
2. extend `crates/quanta-gqlang-v2-planning/tests/v3_planning_hellgate.rs`
3. extend `crates/quanta-gqlang-v2-planning/tests/v3_planning_cost_schema.rs`
4. add one rail for unsupported advanced mixes and one for fixed cost-schema
   stability

## Reject Or Escalate

1. syntax or binder reinspection inside planner:
   forbidden
2. runtime-native DTO or physical-plan leakage:
   belongs outside this repo
3. ad-hoc string cost fields that bypass the fixed schema
4. silently routing non-relational units through planning as if they were
   relational queries:
   belongs to `15`
