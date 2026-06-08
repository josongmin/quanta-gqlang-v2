# QGLang v2 Language Surface

`quanta-gqlang-v2` ships a parser-stable frontend contract for a read-only shell.

Public guarantees:

1. parser-stable root decoding across syntax, sema, and normalized IR
2. read-only CLI/LSP affordances only
3. no runtime truth in public language docs, examples, or smoke tests
4. experimental catalog and AI-adjacent surfaces remain typed frontend metadata

Shipped root families:

1. core query roots: `SELECT`, `FIND`, `CHECK`, `EXPLAIN`
2. interactive roots: `SHOW`, `DESCRIBE`, `SAMPLE`, `COMPARE`, `EXPORT`
3. declaration roots: `VIEW`, `RULE`, `MODEL`, `PACK`, `PROFILE`, `GATE`
4. meta roots: `USE PACK`, `REQUIRE CERTIFIED`

Shipped example families:

1. core: `examples/select/basic.qglang`, `examples/find/taint.qglang`, `examples/check/hybrid_rule.qglang`, `examples/explain/why_no_result.qglang`
2. interactive: `examples/show/capabilities.qglang`, `examples/describe/rule_detail.qglang`, `examples/sample/retrieval.qglang`, `examples/export/sarif.qglang`, `examples/compare/semantic_delta.qglang`
3. declarations: `examples/view/catalog_users.qglang`, `examples/rule/shell_exec.qglang`, `examples/model/embeddings_code.qglang`, `examples/pack/security_default.qglang`, `examples/profile/strict_review.qglang`, `examples/gate/release_gate.qglang`
4. meta: `examples/use_pack/secure_pack.qglang`, `examples/require_certified/strict_only.qglang`

Read-only binary fronts:

1. CLI: `parse`, `sema`, `query`, `fmt`, `doctor`, `headless`
2. LSP stdio JSON methods: `hover`, `completion`, `symbols`, `diagnostics`, `zero_query`, `quickfix`
3. malformed argv, malformed JSON, unknown methods, and mutating requests fail closed

Typed frontend surfaces (recovered local tickets):

These are typed frontend metadata only — read-only, presentation-level, and never
runtime truth claims. Degraded reasons are presentation-only and share a single
authority enum; they are not invented runtime truth.

1. AI exploration profile (`E05E`): `AiExplorationPlanV1`, `AiObjectiveV1`,
   `ExplorationBudgetV1`, and `WorkingSetV1` are an explicit profile over the same
   DSL. Reconfirmation is same-intent replay through `ci_gate` or `lint`, and the
   AI lane cannot emit final policy truth directly. See [ai-profile.md](./ai-profile.md).
2. Public planning IR (`E06C`): planning carries explicit `required_capabilities`,
   `truth_expectation`, `bridge_mode`, and export mode; roots that require planning
   metadata fail closed when it is absent, and lowering-ready normalization
   preserves planning metadata rather than dropping it.
3. Metric inventory and external findings (`E11C`): `metric.<name>` is a typed,
   host-ingested vocabulary and external findings are public frontend data nouns —
   not analyzer-computed truth.
4. Explain and path-witness UX (`E12C`): explain presentation, path-witness
   rendering, degraded badges (via the shared degraded-reason authority), and
   why-no-result guidance are presentation-only.

Out of scope:

1. runtime execution substrate
2. provider registry routing
3. product closure claims about backend materialization

Unsupported claims:

1. unsupported result counts, fix generation, or execution success promises
2. unsupported host DTO publication through frontend IR
3. unsupported assertions that examples prove end-to-end runtime behavior
