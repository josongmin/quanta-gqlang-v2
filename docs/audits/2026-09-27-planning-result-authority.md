# D-05 / D-07: planning result authority and recursion vocabulary

Base: `9bed85a5335c819dc5de7cc7ddb7aa63f1e42fc3`.

D-05 did not demonstrate an incorrect result value. It identified a missing
written authority for the planner's non-relational Set/Unordered decisions.
The existing owner is planning: it derives the result requirements from the
checked family, just as it derives result kind, rendering and evidence. Adding
another Sema cardinality field would duplicate this family policy.

`docs/contracts/planning-results.md` now fixes the current per-family rules and
links them from the language entry point. Relational cardinality is retained
from the sealed root; FIND, CHECK and presentations use the documented planning
rules. Carrier documentation no longer implies every result has a relational
root whose cardinality was copied. Runtime behavior, IR fields, facade exports
and digest schemas are unchanged.

The owner rails assert literal Set, duplicate elimination, no semantic row
order, and the independent Unordered cost field for all seven FIND subkinds,
all six presentation verbs and every admitted CHECK detection-mode / pattern /
nested-graph combination. Presentation witness selection is explicitly absent;
CHECK retains AllWitness for the reference capped graph and None without it.
The existing cap-one rail remains responsible for CanonicalOne. Three separate
production mutants (FIND Set-to-Bag, CHECK Set-to-Sequence, presentation
Set-to-Bag) each compiled and failed one selected assertion. Original source
was restored and the 11 item tests plus one CHECK matrix test passed. Raw logs
and hash metadata are under `target/planning-result-audit/` in the owner
worktree. Clean qualification is a separate receipt.

D-07 was a documentation mismatch: RecursiveClosure already covered recursive
rule fixpoints and cyclic bounded graph traversal. Capability and explain docs
now state both meanings and point to the precise discriminator:
BoundedFixpoint versus BoundDependent in the semantic cost. The FIND matrix
checks exact capabilities, explain recursion and BoundDependent together;
existing rule tests check the opposite nonrecursive case and BoundedFixpoint.
No new capability is fabricated, and no runtime support is inferred from the
shared keyword.

The facade exposes SemanticPlanV2 for inspection. ExecutionItemV3 omits these
result/cost/capability/explain fields; the new document explicitly preserves
that distinction and the downstream admission boundary. This closes the stated
D-05 definition gap and D-07 documentation drift, not the separate optional
D-06/D-08/D-09 refactoring suggestions or the whole audit.
