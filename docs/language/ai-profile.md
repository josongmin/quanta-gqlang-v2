# AI Exploration Profile

The AI surface in `quanta-gqlang-v2` is a typed **profile over the same DSL**, not a
separate runtime contract. It is frontend metadata only: read-only, presentation
and planning vocabulary, never a backend availability or truth claim.

## Public carriers

1. `AiExplorationPlanV1` — the explicit plan shape
2. `AiObjectiveV1` — the declared objective (diagnose, find, narrow, prove, export)
3. `ExplorationBudgetV1` — explicit candidate / round-trip / reconfirmation budgets
4. `WorkingSetV1` — typed working-set carriers (kind, basis, reasons)

## Rules

1. AI remains a profile on the same language surface, not a new execution lane
2. reconfirmation is explicit as **same-intent replay** through `ci_gate` or `lint`
3. final policy truth cannot be emitted directly from the AI lane
4. provider-local or runtime-local IDs do not leak into the public carriers
5. degraded reasons are presentation-only and use the shared degraded-reason
   authority enum, not duplicated strings

## Proof rail

`cargo test -p quanta-gqlang-v2-ir --test ai_profile_contract`
