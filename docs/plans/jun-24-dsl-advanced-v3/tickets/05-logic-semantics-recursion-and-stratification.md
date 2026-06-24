# 05 Logic Semantics Recursion And Stratification

## Goal

Complete the logic layer needed for CodeQL-class existence, negation, and
recursive reasoning while keeping frontend semantics deterministic.

## Why

Relational algebra alone is insufficient for:

1. existential checks
2. anti-existence checks
3. recursive reachability
4. stratified negation
5. reusable predicate-like rule composition

Current head already ships `quanta-gqlang-v2-logic` as the owner lane. This
ticket deepens and closes that lane; it does not authorize a second recursive
semantics implementation.

## Deliverables

1. logic expression/predicate layer over algebra
2. `exists` / `not exists` semantics
3. recursive closure representation
4. least-fixpoint semantics for positive recursion
5. stratification rules
6. recursive aggregate legality rules
7. diagnostics for illegal recursion or negation cycles
8. rule/view composition semantics over the shared HIR
9. path-pattern and witness-materialization semantics over recursive relations
10. bounded witness materialization contract:
    - dedup identity
    - max witness count
    - optional path-length bound
    - canonical tie-break for equivalent witnesses

## Pinned Semantics

1. recursion legality is explicit, not accidental
2. negation over recursive strata is rejected unless stratified
3. predicate truth remains frontend-owned and typed
4. source-level `exists` / `not exists` semantics are defined here, but their
   normalized algebra target forms remain owned by `04`
5. reachability truth and witness/path materialization are related but distinct
   semantics; the frontend must type and classify both explicitly
6. recursive aggregates are illegal unless their legality and fixpoint behavior
   are explicitly specified by this ticket
7. witness/path enumeration over recursive relations rejects unless boundedness
   and cycle-dedup semantics are explicit

## Non-Goals

1. this ticket does not redefine relational algebra nodes
2. this ticket does not classify planning capabilities, truth lanes, or effects
3. this ticket does not, by itself, interpret source horn-rule syntax; source
   `rule` -> recursive predicate lowering is ticket `14`

## Scenarios Unlocked

1. path existence checks
   - “there exists a taint path from source to sink”
2. anti-existence gates
   - “no public API exists without auth coverage”
3. transitive closure
   - recursive reachability over call/dependency graphs
4. recursive path witness query
   - “show one or more typed path witnesses, not only a boolean”
5. fail-closed recursive aggregate
   - recursive ranking/counting shape is rejected until legality is defined
6. infinite witness family rejection
   - cyclic path family without admitted bound rejects rather than streaming
     unbounded results

## Definition Of Done

1. logic constructs exist as first-class frontend nodes
2. recursion/negation legality rails are green
3. rule composition no longer depends on root-local carrier hacks
4. recursive path truth and witness semantics are both explicit and typed
5. positive recursion semantics and recursive-aggregate rejection/admission are
   explicit and test-covered
6. recursive witness materialization has explicit bound, dedup, and tie-break
   semantics or rejects

## Dependencies

1. requires `00`
2. depends on `01`, `02`, `03`, and `04`
3. planning classification remains downstream in `07`
4. source horn-rule support closes only with `14`

## Likely Touch Points

1. `crates/quanta-gqlang-v2-logic/src/recursion.rs`
2. `crates/quanta-gqlang-v2-logic/src/source.rs`
3. `crates/quanta-gqlang-v2-logic/src/witness.rs`
4. `crates/quanta-gqlang-v2-logic/src/aggregate.rs`
5. `crates/quanta-gqlang-v2-logic/src/graph.rs`
6. `crates/quanta-gqlang-v2-pipeline/src/compile.rs` only if the logic handoff
   seam changes

## Mandatory Worker Outputs

1. least-fixpoint and stratification rules written as explicit frontend
   semantics
2. reject/admit matrix for recursive aggregates
3. bounded witness contract with dedup identity, caps, and tie-break behavior

## Proof Rails

1. extend `crates/quanta-gqlang-v2-logic/tests/v3_logic_recursion.rs`
2. extend `crates/quanta-gqlang-v2-logic/tests/v3_logic_witness.rs`
3. extend `crates/quanta-gqlang-v2-logic/tests/v3_logic_hellgate.rs`
4. extend `crates/quanta-gqlang-v2-logic/tests/v3_logic_source.rs`
5. add one positive recursion rail and one fail-closed infinite-witness rail

## Reject Or Escalate

1. any attempt to hide witness boundedness in CLI/LSP rendering
2. any planner truth/effect policy addition:
   belongs to `07`
3. any unbounded recursive shape that lacks explicit rejection semantics
4. any claim that parser/HIR horn-rule presence equals recursive-rule support:
   belongs to `14`
