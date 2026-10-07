# ADR-0006: Code-query evaluation and dependency adoption boundary

- Status: Accepted
- Date: 2026-10-07
- Scope: sequencing and evidence policy; the V4 language/result design remains Proposed

## Context

Ten pinned reference audits and repeated campaign prompts supported the
[typed code-query proposal](0007-typed-code-query-contract-proposal.md).
Detailed audit prose duplicated the proposed design and active action list.
Reference similarities do not prove a local feature or justify a new dependency.

## Decision

1. The first proposed milestone is an installed public Java query joining two
   real source-sealed facts into typed rows. Index, general optimization,
   recursion and flow are not prerequisites for that milestone.
2. QGLang owns host-agnostic logical semantics. Semantica owns source/provider
   admission, physical execution, completion and results. Index is an optional
   candidate access path; exact corpus/source/window identity must be admitted.
3. V4 remains a coordinated breaking proposal. Producer bytes/digest vectors,
   consumer validator/pin and result/completion semantics change together when
   implemented. Existing V3 maintenance does not implement V4.
4. Independent reference results precede rewrites and optimization. Optimizer
   effort cutoff may retain a valid plan; request cancellation, missing coverage
   and resource failure cannot become complete answers.
5. Invalidation plus fresh recomputation precedes maintained incremental
   execution. Replay/delete, publication, lease and retention gates must pass
   together before maintained results are exposed.
6. Retain existing owners. Adoption of another optimizer, recursion runtime or
   library requires comparable workload, semantic, resource and integration-cost
   evidence. No new production dependency is selected here.

Full language/model/flow/path laws remain in Proposed ADR-0007, not this accepted
workflow decision. [A00–A15 and optional Index work](../work/remaining.md#proposed-code-query-work) retain
their original IDs, dependencies and independent acceptance oracles.

## Pinned reference provenance

The following are the 2026-10-04 audit pins, not current upstream-version claims.
Links retain primary source provenance; detailed file/line investigations remain
in Git history under `docs/audits/reference-source/` at the consolidation baseline.

| Reference pin | Recorded use |
| --- | --- |
| [CodeQL f7617abe](https://github.com/github/codeql/tree/f7617abe602d4763cd08c6a57b6a6bc347e7e325) | Typed modules/models and flow/path; public evaluator internals were outside the audit |
| [DataFusion 9547b091](https://github.com/apache/datafusion/tree/9547b091bc99fbcd52c37d67f9ccee645cffc2ae) | Provider/pushdown, physical streams, memory/cancel |
| [Soufflé f53dab8b](https://github.com/souffle-lang/souffle/tree/f53dab8bf4bbf2395a26d2771f6b75395825cc46) | Clauses, SCC/delta, indexes and bounded provenance |
| [Joern bf4f3434](https://github.com/joernio/joern/tree/bf4f3434c28dae885f8bdf2e5540e311f77836e4) | Context/model, endpoint/path and solver limits |
| [Substrait ab7eff6b](https://github.com/substrait-io/substrait/tree/ab7eff6bf7589d4bd94b26eccd3cb040d45efea2) | Versioned schema and explicit consumer subset |
| [Datafrog edd7cfc0](https://github.com/rust-lang/datafrog/tree/edd7cfc0f5ad3b93ce5bbe6a17848db9376b13c0) | Finite tuple/delta prototype; memory/cancel required separate work |
| [Calcite fe1c46d2](https://github.com/apache/calcite/tree/fe1c46d2beff6e1edcc050ffbb03df1724adbaf3) | Guarded rewrites, physical traits, cost and optimizer cutoff |
| [egg 73975c98](https://github.com/egraphs-good/egg/tree/73975c9819efb4af2f3c6c1b44fa53f8c5742574) | Equality saturation; user-supplied equality is not a semantic proof |
| [Differential aa8745f9](https://github.com/TimelyDataflow/differential-dataflow/tree/aa8745f93ea8abe131104fc7885ba4fd47e63902) | Signed updates, frontiers, arrangements and compaction |
| [Kuzu 89f0263c](https://github.com/kuzudb/kuzu/tree/89f0263cc7a1fd9c396d2c4953747a013556a7f9) | Historical adjacency/path reference; no dependency adoption decision |

The earlier Datafrog probe's three tests and 512 small graph cases are prototype
evidence only. No CodeQL evaluator audit, local performance advantage or new
installed typed-join result is established by this consolidation.

## Consequences

Delete reference-audit replicas and worker dispatch prompts after preserving
pins, decisions and outstanding acceptance. Future work updates the one action
owner or Proposed ADR. Promotion to a public contract requires source and paired
producer/consumer evidence.
