# AI Exploration Profile Status

The current v3 frontend does **not** ship a separate AI exploration profile,
provider integration, or AI execution lane. The old `AiExplorationPlanV1`,
`AiObjectiveV1`, `ExplorationBudgetV1`, and `WorkingSetV1` carriers are not in
the current workspace or the public frontend facade. The former
`quanta-gqlang-v2-ir --test ai_profile_contract` command is not a valid proof
rail and must not be used as a support claim.

`FIND`, `EXPLAIN`, `SAMPLE`, `COMPARE`, and `EXPORT` are language/presentation
families handled by the shared checked frontend pipeline. Their presence in
syntax, examples, or planning IR does not imply an AI provider, data-generation
binding, executed result, or runtime availability. For current behavior and
proof boundaries, use the [language surface](README.md),
[V3 frontend contract](../contracts/frontend-execution-v3.md), and
[test contract map](test_hellgate_ledger.md).
