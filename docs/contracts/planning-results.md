# Planning result semantics

The planning owner defines `SemanticPlanV2::result()` from checked upstream
facts and the family rules below. Sema does not issue a synthetic relational
cardinality for non-relational items. Relational plans retain their checked
output cardinality; non-relational family rules are applied once by planning.
This document records the current behavior; it does not change it.

| Planned family | Result kind | Multiplicity | Semantic row order | Witness selection |
|---|---|---|---|---|
| FIND (all seven subkinds) | PathWitness | Set | Unordered | CanonicalOne when the sealed cap is 1; otherwise AllWitness up to that cap |
| CHECK | Alert | Set | Unordered | Same rule for its nested graph; absent without a nested graph |
| EXPLAIN | ProofExplain | Set | Unordered | Absent |
| SHOW | Table | Set | Unordered | Absent |
| DESCRIBE | Alert | Set | Unordered | Absent |
| SAMPLE | Table | Set | Unordered | Absent |
| COMPARE | PathWitness | Set | Unordered | Absent |
| EXPORT | Table | Set | Unordered | Absent |

Set means duplicate logical result rows are eliminated. Unordered means the
language promises no row order; a stable serialization order is not a semantic
ordering guarantee. The SAMPLE seed/cap and EXPORT format/effect remain their
separate sealed policies. A PathWitness result kind alone does not imply a
FIND witness-selection policy: COMPARE has none. `TopK` is vocabulary available
in the carrier, but the current planner does not issue it for these families.
FIND's witness dedup identity, path bound and tie-break come from the logic
contract; AllWitness still respects its finite cap.

For relational results, Set eliminates duplicates, Bag preserves multiplicity
without semantic order, and Sequence preserves multiplicity with semantic
order. Rule programs use Set for their least-fixpoint results. REQUIRE CERTIFIED
produces an unordered Set of proof/explain results. Composition derives its
result requirements from its checked body. A typed non-plan outcome has no
result contract.

## Cyclic traversal and recursion

`CapabilityV2::RecursiveClosure` and `ExplainNoteV2::recursive()` cover both
recursive rule closure and bounded cyclic graph witness traversal, including
CHECK's nested graph. They are not a claim that every graph query is a recursive
rule program. Read `semantic_cost().recursion_risk_class` to distinguish them:

| Checked construct | RecursiveClosure capability | Explain recursive | Recursion risk |
|---|---|---|---|
| Recursive rule program | Present | true | BoundedFixpoint |
| Nonrecursive rule program | Absent | false | None |
| Cyclic FIND / CHECK nested graph | Present | true | BoundDependent |

These are qualitative frontend requirements, not runtime cost measurements or
provider capabilities. A host must not use the shared capability keyword alone
to distinguish rule fixpoints from witness traversal.

## Consumer and proof boundary

The facade re-exports `SemanticPlanV2`, through which these plan facts can be
inspected. `ExecutionItemV3` does not separately serialize the result contract,
capability set, semantic cost or explain note. That DTO limitation does not grant
standalone graph/CHECK/presentation execution support. Downstream admission and
runtime implementation remain separately owned; see
[the V3 execution boundary](frontend-execution-v3.md#public-boundary).

`v3_item_hellgate.rs` pins all FIND subkinds and all six presentation verbs against
literal Set/Unordered expectations, including duplicate and witness-selection
semantics. `v3_planning_hellgate.rs` checks the CHECK detection-mode,
pattern-presence and nested-graph-presence matrix, including exact rejection of
invalid combinations. Existing graph/rule tests
pin RecursiveClosure, explain recursion and the corresponding recursion-risk
class together. These are owner-contract tests, not runtime execution tests.
