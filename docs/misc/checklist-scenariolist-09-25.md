# QGLang V3 final audit: SOTA++ checklist and scenario coverage

- Audit date: 2026-09-25
- Audited source: `14e2d1264a488d9838f54ad84e70a4bda51144bd`
- Audited tree: `3b940b5db007913b710fd291adb690cf378c58ca`
- Current-head note (2026-09-26): repository HEAD before this documentation
  refresh was `a01eecc5ff2d255aabc9eb09011cecd3895218a7`. The `14e2d126…`
  results below are historical source-bound audit evidence, not an exact-HEAD
  qualification receipt for `a01eecc5…`.
- Branch state at audit start: `main == origin/main`; the report and the
  hardening edits described below are working-tree changes
- Scope: syntax, HIR, binding, types, modules, semantic authority, algebra,
  logic, planning, pipeline, frontend contract, CLI, diagnostics stdio, local
  qualification
- Excluded ownership: runtime execution, provider routing, backend DTOs,
  materialization, deployment, activation

This document is a source-bound audit and a forward conformance checklist. It
does not turn a checklist entry into an implemented feature, and it does not
turn repository-local frontend proof into downstream runtime proof.

## 1. Verdict

The final source audit confirmed and fixed seven current-working-tree defects:

- process-group cleanup could silently return while the group remained alive;
- planned FIND policy dropped direction, facets and resolved selectors;
- Composition graph producers bypassed the Logic boundedness/endpoint gate;
- the V3 Composition execution projection dropped scoped SAMPLE targets and
  all graph semantics except the kind;
- the revised execution contract encoding reused the schema-3 digest domain,
  allowing byte-incompatible contract encodings to claim the same version;
- the graph execution handoff still dropped the sealed relation shape, dedup
  identity, path-length cap and deterministic witness tie-break.
- Composition execution projection accepted a same-variant relation or graph
  payload from a different sealed item identity.

The strongest current evidence is:

- the audited base HEAD has a successful clean qualification receipt;
- the post-fix dirty-tree `just ci` discovered, selected, executed and passed
  `1248/1248` Rust tests;
- skipped/ignored tests: `0`;
- leaked tests: `0`;
- the repository has owner tests for every compiler stage and black-box tests
  for both shipped process front doors;
- the facade is checked by a positive external consumer, 15 compile-fail
  privacy probes, an architecture allowlist, and a public-API gate;
- doctests passed `5/5`, qualification tests `48/48`, external-consumer
  positive/negative/privacy checks passed, and static/policy gates passed.

The dirty-worktree gate is current local integration evidence, not a clean-HEAD
qualification receipt. Run `just qualify-local` only after the intended changes
are committed and the caller worktree is clean.

No reproduced product defect remains in this audit. The remaining work is
non-blocking proof-system breadth: a representative cross-stage TCK registry,
bounded investigative fuzzing beyond surface/HIR, bounded independent
algebra/planning models, targeted release mutation campaigns, and executable
traceability for the high-risk checklist rows. These are `P2` improvements,
not prerequisites for every local change or release.

The current execution contract has field-mutation rails, complete graph tag
sets, shared producer/consumer vectors, and a test-only graph canonical encoder
that does not call production encoding helpers. The facade-only consumer also
proves mixed multi-binding CHECK and EXPLAIN bodies. Qualification receipts are
explicitly non-replayable: raw Nextest streams are validated in-run but are not
archived; their hashes are custody identifiers, not embedded evidence.

These are not grounds for adding runtime, provider, or backend behavior to this
repository.

## 2. Status and severity vocabulary

Use these states when converting this document into an executable ledger.

| State | Meaning |
|---|---|
| `PROVEN` | A current owner or process test directly asserts the behavior with a discriminating oracle. |
| `PARTIAL` | Related tests exist, but the exact matrix, boundary, independent oracle, or process boundary is incomplete. |
| `GAP` | No durable executable proof was located for the exact scenario. This is not automatically a production defect. |
| `DEFECT` | Current source can be reproduced returning the wrong result, accepting invalid input, rejecting valid input, hanging, leaking, or claiming false authority. |
| `BOUNDARY` | Intentionally downstream-owned or unsupported. The frontend must reject or retain typed non-executable policy. |
| `CONDITIONAL` | Required only after the named feature or compatibility promise is added. |
| `UNASSESSED` | Requirement-catalog entry not yet mapped to one exact owner test. It is neither a gap nor proof. |

Checkboxes in sections 8-32 are requirement-catalog syntax, not completion
claims. Unless a row has an explicit `Current status:` suffix or appears in the
gap register below, its status is `UNASSESSED`. A checked box may be introduced
only by an executable ledger tool that validates its owner mapping; manually
checking Markdown is not closure. This keeps the scenario catalog useful
without falsely treating unchecked boxes as product defects.

Severity is based on impact, not implementation effort.

| Priority | Meaning |
|---|---|
| `P0` | Wrong authority, wrong plan, wrong execution DTO, silent semantic drift, unbounded resource use, protocol corruption, or fail-open qualification. |
| `P1` | Missing edge/corner proof that can conceal a P0 class regression. |
| `P2` | Maintainability, corpus breadth, diagnostic quality, or non-blocking resilience improvement. |
| `P3` | Optional research or tooling with no current correctness claim. |

### Applicability and blocker policy

The original 513 unchecked bullets were a scenario catalog, not 513
unconditional release blockers. After removing the unconditional per-feature
bundle, the normalized catalog contains 486 checkbox prompts: 450 named
scenario IDs and 36 schema/closure-template bullets. Apply them as follows:

- `P0`: required whenever the owned behavior is reachable.
- `P1`: required only when the change touches the named public contract,
  authority boundary, resource boundary, or failure mode.
- `P2`/`P3`: backlog or release-investigation work; absence is not a product
  defect and does not invalidate an otherwise complete local gate.
- `CONDITIONAL`: inactive until the named feature or compatibility promise is
  actually advertised.
- `BOUNDARY`: an architecture assertion or typed downstream admission rule,
  not a request to implement runtime behavior here.
- release rows are change-triggered; they are not all rerun for unrelated
  source edits.

Only applicable high-risk rows need executable owner mapping. Requiring a
second full compiler, exhaustive Cartesian products, perpetual fuzzing,
routine workspace-wide mutation, or raw-event archival for every change would
add cost without strengthening the current correctness claim.

## 3. Evidence hierarchy

Closure requires the strongest applicable evidence. Stronger rows do not make
weaker stages disappear.

1. exact source and tree frozen;
2. focused owner test with exact positive and negative neighbors;
3. stage-composition test through the shared pipeline;
4. facade-only external-consumer test;
5. shipped CLI or diagnostics-stdio process test;
6. repository-wide `just ci` on unchanged source;
7. clean committed-HEAD `just qualify-local` receipt;
8. downstream consumer admission and runtime execution in the downstream repo.

The following are insufficient by themselves:

- a plan, checklist, ADR, code review, or static source inspection;
- a test count without machine-reconciled inventory and terminal events;
- a focused green test used as a repository-wide qualification claim;
- a stale receipt from a different SHA or tree;
- an interrupted fuzz, mutation, coverage, or benchmark run;
- an IDE result or cached artifact without checkout reproduction;
- a frontend plan used as evidence that a runtime executed it correctly.

## 4. Current owner stack and verified coverage

| Layer | Owned behavior | Primary current evidence | Audit status |
|---|---|---|---|
| Surface | lexer, parser, document roots, formatter, budgets | `v3_family_hellgate`, `v3_document_hellgate`, `v3_format_hellgate`, `v3_surface_budget`, model/proptest/spec corpus | `PROVEN`, corpus breadth `PARTIAL` |
| HIR | identity, spans, integrity, traversal, depth | `v3_item_identity`, `v3_hir_hellgate`, `v3_hir_depth_boundaries`, `v3_visit_traversal` | `PROVEN` |
| Binder | scope, capture, joins, composition, source use | `v3_binder_scope`, `v3_binder_hellgate`, `v3_relation_source_use`, `v3_composition_hellgate` | `PROVEN` |
| Types | closed universe, overloads, effects, rule constraints | `v3_types_universe`, `v3_types_check_edges`, `v3_types_hellgate`, `v3_rule_type_solver` | `PROVEN` |
| Modules | manifests, locks, imports, re-exports, catalog, cycles | `v3_modules_v2_authority`, `v3_document_predeclaration` | `PROVEN` |
| Sema | closed/open authority, transactions, references, budgets | `v3_document_transaction`, `v3_authority_builder`, `v3_reference_resolution`, `v3_semantic_budget` | `PROVEN` |
| Algebra | typed lowering, normalization, correlation, ownership | `v3_algebra_normalization`, `v3_algebra_hellgate`, `v3_algebra_mutants`, `v3_algebra_probe` | `PROVEN`, independent oracle `PARTIAL` |
| Logic | SCC, recursion, polarity, graph bounds, witnesses | `v3_logic_recursion`, `v3_rule_scc_hellgate`, `v3_graph_query_hellgate`, `v3_logic_witness` | `PROVEN` |
| Planning | truth, capabilities, evidence, effects, result, cost | `v3_planning_truth`, `v3_planning_hellgate`, `v3_capability_derivation`, `v3_cost_classification` | `PROVEN`, independent oracle `PARTIAL` |
| Pipeline | one transaction, dispositions, execution producer | `v3_compiled_document_assembly`, `v3_composition_source_query` | `PROVEN` |
| Contract | facade allowlist, privacy, public API | external consumer positive, 15 negative probes, architecture/public-API gates | `PROVEN` |
| CLI | `query`/`v3-query`, `fmt`/`v3-fmt`, ingress and exit | `ingress_matrix`, `open_analysis_contract`, `command_doc_gate`, formatter process tests | `PROVEN` |
| Diagnostics stdio | bounded NDJSON `v3_diagnostics` | `ndjson_contract`, `binary_contract`, `server_rejection_matrix`, `resource_boundaries` | `PROVEN` |
| Qualification | inventory/event reconciliation, process cleanup, receipt custody | CI/qualification Python tests and clean receipt | `PROVEN` |

The exact current mapping remains in
`docs/language/test_hellgate_ledger.md`. This document must not replace that
executable-owner ledger.

## 5. OSS reference matrix

No single OSS is a drop-in reference. The correct comparison is a composite.

| Reference | Relevant QGLang concern | Adopt | Do not import |
|---|---|---|---|
| [Apache Calcite algebra](https://calcite.apache.org/docs/algebra.html) | relation algebra and semantics-preserving rewrites | explicit logical operators, rewrite invariants, correlation and equivalence tests | physical adapters, execution engines, cost-based runtime claims |
| [Apache DataFusion optimizer](https://datafusion.apache.org/library-user-guide/query-optimizer.html) | Rust logical-plan architecture | immutable logical plan, analyzer/optimizer separation, plan observer-style stage evidence | physical plan and Arrow execution ownership |
| [CodeQL queries](https://codeql.github.com/docs/writing-codeql-queries/about-codeql-queries/) | alert/path/diagnostic result families | typed query kind, metadata, source location, result interpretation | CodeQL database and extractor ownership |
| [CodeQL path queries](https://codeql.github.com/docs/writing-codeql-queries/creating-path-queries/) | reachability, flow, taint and witness paths | explicit source/sink, path graph, endpoint/result separation | language-specific extractor facts |
| [Souffle](https://github.com/souffle-lang/souffle) | Horn rules, SCC, recursion, aggregation | stratification, component/module matrices, provenance and recursive stress cases | native evaluator and generated execution runtime |
| [Cedar validation](https://docs.cedarpolicy.com/policies/validation.html) | closed authority and schema validation | validate-before-plan, schema-change revalidation, soundness and differential testing | authorization decisions or default-deny runtime semantics |
| [OPA bundles](https://www.openpolicyagent.org/docs/management-bundles) | packs, manifests, revisions and roots | revision/digest identity, entrypoints, conflict detection, signature-policy test design | remote serving, hot reload and enforcement runtime |
| [Semgrep taint mode](https://semgrep.dev/blog/2022/demystifying-taint-mode/) | source/sink/sanitizer/propagator scenarios | compact positive/negative taint matrices and false-positive resistance | autofix, source mutation and Semgrep-specific execution semantics |
| [openCypher TCK](https://opencypher.org/resources/) | portable language conformance | feature/scenario/prerequisite/input/expected-output corpus | the full Cypher language or mutating graph runtime |
| [SQLLogicTest](https://www.sqlite.org/sqllogictest/doc/trunk/about.wiki) | high-volume equivalence and differential testing | labeled equivalent inputs and canonical expected artifacts | database result oracle when no QGLang runtime is owned |
| [Substrait](https://substrait.io/) | portable relational execution contract | versioning, extension/unknown handling and serialization test design | an unimplemented Substrait compatibility claim |

Reference-derived requirements must be translated to QGLang ownership. For
example, SQLLogicTest-style equivalence applies to parsing, normalization,
planning and digest outputs here, not to rows returned by an executor.

## 6. Confirmed proof gaps

### GAP-001: no executable cross-stage TCK registry (`P2`)

Current state:

- `quanta-gqlang-v2-conformance` has one canonical positive for each of 18
  public root families and two supplemental source units;
- the implementation-neutral surface spec corpus has five paired
  `.qglang`/`.expect` cases;
- the owner tests are broad but distributed across crates.

Missing proof:

- one machine-readable registry containing positive, negative, boundary,
  authority, expected stage/code/span, normalized form, plan disposition,
  execution admission, and digest expectations;
- reverse validation that every registry scenario has an executable owner;
- forward validation that every public enum family/variant is represented.

Bounded acceptance (improvement, not release blocker):

- add a versioned scenario schema and registry;
- generate or drive tests from the registry without weakening owner-specific
  assertions;
- reject duplicate IDs, missing expectations, unknown stages, stale owner
  paths, and scenarios that select zero tests.

### GAP-002: fuzzing stops at surface/HIR (`P2`)

Current state:

- `fuzz/fuzz_targets/surface_document.rs` covers parser totality, accepted HIR
  integrity, parse/format structural equivalence, fixed-point formatting, and
  bounded diagnostics for inputs up to 4096 bytes;
- fuzzing is intentionally outside `just ci` and qualification.

Missing proof candidates:

- bounded NDJSON framing and request decoding fuzz target;
- closed-authority builder and manifest/lock/catalog structured fuzz target;
- parser-to-open-analysis full pipeline fuzz target;
- typed execution encoder/digest structured fuzz target;
- forged HIR/semantic-carrier rejection fuzz target.

Acceptance:

- each target has an explicit input cap, invariant oracle, corpus seed and
  failure-minimization procedure;
- crashes become permanent focused regressions;
- fuzz completion is never represented as exhaustive qualification.

### GAP-003: limited independent algebra/planning oracle (`P2`)

Current state:

- rule SCC/strata has an independent small-graph oracle over 343 graphs;
- algebra and planning have strong exact fixtures and forged-carrier mutants;
- surface has model-based and metamorphic tests.

Missing proof:

- a small, independent relational reference model for operator ordering,
  output schema, provenance, multiplicity and correlation;
- independent derivation of capability/effect/result contracts;
- exhaustive small-tree comparison against production algebra/planning.

Acceptance:

- test-only model shares no production lowering helpers;
- generated trees are bounded and shrinkable;
- mismatches print source, model artifact and production artifact.

### GAP-004: current execution encoding coverage (`CLOSED`)

Current state:

- 15 checked-in V3 vectors cover source identity, whitespace, host schema,
  workspace-files authority, composition mutation, `i128::MIN`, float,
  negative zero, bool, UTF-8 and null;
- producer and external consumer both read the shared vectors.

Optional breadth candidates (`P2`):

- every executable relational operator shape;
- all scalar and nested type shapes;
- every operator/builtin identity and effect combination;
- join modes, sort directions/null order, cardinality and source-authority
  variants;
- composition producer/body variants that actually possess V3 payloads;
- unknown-version and unknown-variant consumer behavior once wire
  serialization exists.

Current closure criteria:

- use pairwise or model-generated vectors for future high-risk interactions,
  never an unbounded Cartesian product;
- every semantically relevant field has a mutation that changes the contract
  or qualification digest;
- source-only whitespace must change source and qualification digests; contract
  digest equality may be asserted only for a named fixture after the projected
  execution DTOs are proven field-identical. The public contract intentionally
  promises no general formatting-equivalence cache key;
- vectors remain producer-owned and independently consumed;
- the graph canonical substream is checked by a test-only encoder that does
  not call production tag or writer helpers.

### GAP-005: generated mutation proof is not current qualification (`P2`)

Current state:

- multiple crates contain explicit mutation-kill rails;
- current `just ci` does not run a workspace-wide mutation tool;
- historical mutation results are not current-source evidence.

Acceptance options:

- keep targeted mutation rails and require a bounded local campaign before a
  major semantic release; or
- define a curated package/file mutation manifest with timeout, survivor
  triage and exact source binding.

Do not add a multi-hour mutation run to the routine local gate. Partial output
must remain non-qualification evidence.

### GAP-006: checklist-to-owner traceability is not executable (`P2`)

This document enumerates expected scenarios, but nothing currently rejects an
unchecked row. Closure requires scenario IDs in a registry or explicit owner
test annotations plus a validation tool. Documentation-only completion is not
coverage completion.

### GAP-007: mixed open-authority masks (`CLOSED`)

`OpenAnalysisAuthorityInputV2` has independently optional module, relation and
graph proofs. Pipeline coverage now iterates all eight three-bit masks and pins:

- status (`Complete`, `Incomplete`, or `Error`);
- exact missing-authority facts and diagnostic order;
- source item order;
- no plan/execution promotion;
- no plan/execution promotion for every incomplete mask.

Empty, whitespace-only and comment-only documents are pinned as `Complete`
under all eight masks. Wrong-registry and wrong-budget proof substitution remain
separately rejected by the Sema authority-owner tests.

### GAP-008: graph selector and endpoint cross-products (`CLOSED`)

Owner tests iterate all seven admitted kinds, admit each kind's exact required
endpoint set, reject removal of every required role, and reject every
kind/unexpected-endpoint pair. They also pin missing-before-unexpected and
stable first-unexpected diagnostics. Sema rejects duplicate singular roles.
Source-to-final-plan tests retain:

- direction, summary facets, sanitizer, scope, window, frontier, via and
  through retention from source through Sema, graph contract and plan policy;
- exact selector identity in Sema's deterministic role order.

### GAP-009: multi-binding composition through the facade (`CLOSED`)

All body families reach the shared pipeline, and Sema owner tests already pin
local-before-host lookup and same-name host fallback rejection. The facade-only
consumer now proves a deliberate same-name host/local collision, sequential
SAMPLE-to-query multi-binding dependency order, local scoped SAMPLE, a complete
graph producer/body handoff, and mixed multi-binding CHECK and EXPLAIN bodies.
Exhaustive producer/body Cartesian products are not required; owner tests cover
the semantic matrix and the external fixture covers each public body family.

### GAP-010: raw Nextest streams are not retained (`CLOSED`)

The Nextest gate parses and reconciles the full generated inventory and
terminal event stream during the run, then writes counts plus SHA-256 hashes to
`target/ci/nextest-evidence.json`. The clean qualification receipt embeds that
summary, but does not retain the raw inventory/event bytes. Current-run
correctness is fail-closed; later independent replay from the receipt alone is
not possible.

The repository chooses the explicit non-replayable policy. Raw streams are
ephemeral inputs to a fail-closed current run. Persisted hashes identify those
inputs but cannot reconstruct them. Raw retention remains an optional custody
feature, not a correctness prerequisite.

### Actionable gap register

| Gap | Priority | Status | Owner | Exact remaining work |
|---|---:|---|---|---|
| GAP-001 | P2 | `GAP` | conformance + pipeline | representative registry for public roots and high-risk cross-stage contracts |
| GAP-002 | P2 | `GAP` | process/pipeline owners | bounded local fuzz targets beyond surface/HIR |
| GAP-003 | P2 | `GAP` | algebra + planning | bounded independent models; never a second full compiler |
| GAP-004 | P1 | `PROVEN` | pipeline execution contract | current fields/tags mutate digests; shared vectors and independent graph encoder agree |
| GAP-005 | P2 | `GAP` | qualification | bounded release mutation campaign, not routine CI |
| GAP-006 | P2 | `GAP` | conformance tooling | executable traceability for applicable P0/P1/public/release rows only |
| GAP-007 | P1 | `PROVEN` | Sema + pipeline | all eight masks, exact diagnostics and empty/comment-only behavior pinned |
| GAP-008 | P1 | `PROVEN` | logic + planning + pipeline | full unexpected-role matrix and source-to-final-plan selector retention pinned |
| GAP-009 | P1 | `PROVEN` | external consumer | every body family crosses the facade, including mixed CHECK/EXPLAIN multi-binding |
| GAP-010 | P2 | `PROVEN` | qualification | explicit non-replayable receipt policy; hashes are custody identifiers |

## 7. TCK record schema

Every executable scenario should carry these fields.

```text
id
version
feature
layer
priority
status
reference_system
prerequisites
input_kind
input
authority_fixture
budget_fixture
expected_acceptance
expected_stage
expected_diagnostic_code
expected_message
expected_primary_span
expected_related_spans
expected_normalized_surface
expected_hir_witness
expected_type
expected_effect
expected_capabilities
expected_result_contract
expected_plan_disposition
expected_execution_admission
expected_source_digest
expected_contract_digest
expected_qualification_digest
mutation_of
positive_neighbor
owner_test
process_boundary
downstream_boundary
```

Schema rules:

- [ ] IDs are globally unique and stable.
- [ ] A rejected scenario names exactly one expected first failing stage.
- [ ] A negative scenario names a valid positive neighbor.
- [ ] Diagnostic scenarios assert code and exact primary span; message is
  asserted when contractual.
- [ ] Plan scenarios assert the complete relevant payload, not only an enum
  discriminant.
- [ ] Ordering scenarios use asymmetric fixtures.
- [ ] Budget scenarios contain exact-limit and limit-plus-one pairs.
- [ ] Authority scenarios state open or closed authority explicitly.
- [ ] Process scenarios state stdout, stderr, exit code and framing.
- [ ] `BOUNDARY` scenarios name the downstream owner and expected frontend
  rejection or non-executable disposition.
- [ ] Every scenario resolves to at least one actual selected test.
- [ ] Registry validation rejects orphan tests only when the repository adopts
  an explicit requirement that all conformance tests be registered.

## 8. Surface, lexer and parser scenario checklist

Owner references: `crates/quanta-gqlang-v2-surface/src/lexer.rs`,
`crates/quanta-gqlang-v2-surface/src/parser.rs`,
`crates/quanta-gqlang-v2-surface/src/family.rs`,
`crates/quanta-gqlang-v2-surface/src/expr.rs`, and Surface owner tests.

### 8.1 Bytes, Unicode and tokens

- [ ] `SURF-BYTE-001` empty source.
- [ ] `SURF-BYTE-002` whitespace-only source for every supported whitespace
  category.
- [ ] `SURF-BYTE-003` ASCII, two-byte, three-byte and four-byte UTF-8 before
  and inside a token; spans remain byte offsets at character boundaries.
- [ ] `SURF-BYTE-004` invalid UTF-8 is rejected at CLI/stdio ingress and never
  passed to the Rust `str` parser.
- [ ] `SURF-BYTE-005` NUL, CR, CRLF, tab and other control characters have an
  explicit accept/reject policy.
- [ ] `SURF-TOK-001` every single-character punctuation token.
- [ ] `SURF-TOK-002` every multi-character operator is atomic; each partial
  prefix and whitespace/comment split rejects.
- [ ] `SURF-TOK-003` `::` is atomic in every reference family.
- [ ] `SURF-TOK-004` `->` is atomic and missing either side rejects at the
  exact byte.
- [ ] `SURF-TOK-005` `-` followed by a non-digit is not silently accepted as a
  numeric literal.
- [ ] `SURF-TOK-006` dot without a following digit is not consumed as a float.
- [ ] `SURF-TOK-007` comments at start, between tokens, after a root and at EOF.
- [ ] `SURF-TOK-008` comment insertion cannot split an atomic token.
- [ ] `SURF-TOK-009` contextual keywords remain usable in explicitly permitted
  value/path/name positions.
- [ ] `SURF-TOK-010` keyword prefixes and suffixes do not match whole keywords.

### 8.2 Literals and containers

- [ ] `SURF-LIT-001` zero, positive/negative one, `i128::MAX`, `i128::MIN`, and
  both one-overflow neighbors.
- [ ] `SURF-LIT-002` float integer/fraction boundaries, leading/trailing dot
  policy, exponent policy, overflow, underflow and non-finite rejection.
- [ ] `SURF-LIT-003` `-0.0` spelling is retained at surface and normalized only
  by the checked typed-value owner.
- [ ] `SURF-LIT-004` boolean and null keywords versus same text in contextual
  identifier positions.
- [ ] `SURF-LIT-005` empty string, quotes, slash, newline/tab escapes, Unicode,
  malformed escape and unterminated string.
- [ ] `SURF-LIT-006` empty/non-empty/trailing-comma list policy.
- [ ] `SURF-LIT-007` record field order, duplicate fields, empty record policy
  and nested containers.
- [ ] `SURF-LIT-008` container width exact cap and cap-plus-one.

### 8.3 Expressions

- [ ] `SURF-EXPR-001` complete precedence ladder with structurally different
  operands.
- [ ] `SURF-EXPR-002` left/right associativity for every binary operator.
- [ ] `SURF-EXPR-003` repeated unary operators and unary/path precedence.
- [ ] `SURF-EXPR-004` parentheses preserve intended value versus boolean
  grouping.
- [ ] `SURF-EXPR-005` path access with empty, missing and contextual segments.
- [ ] `SURF-EXPR-006` call with zero, one, many and maximum arguments.
- [ ] `SURF-EXPR-007` star argument allowed position and mixed-star rejection.
- [ ] `SURF-EXPR-008` quantifier with zero, one, multiple and duplicate binders.
- [ ] `SURF-EXPR-009` nested EXISTS/NOT EXISTS polarity and grouping.
- [ ] `SURF-EXPR-010` expression depth and node limits are independent and
  checked at exact and one-over boundaries.
- [ ] `SURF-EXPR-011` long flat chains cannot bypass structural depth limits.
- [ ] `SURF-EXPR-012` malformed expression at every expected-token point
  returns a bounded diagnostic rather than panic or hang.

### 8.4 Relation pipelines

- [ ] `SURF-REL-001` scan only.
- [ ] `SURF-REL-002` each legal stage individually after a scan.
- [ ] `SURF-REL-003` every legal adjacent-stage pair.
- [ ] `SURF-REL-004` illegal orderings fail at the first illegal stage.
- [ ] `SURF-REL-005` repeated filter/project/order policy.
- [ ] `SURF-REL-006` inner join requires `on`; cross join forbids `on`.
- [ ] `SURF-REL-007` self join and asymmetric nested joins.
- [ ] `SURF-REL-008` group keys zero/one/many/duplicate/default-name conflict.
- [ ] `SURF-REL-009` scalar aggregate versus grouped aggregate.
- [ ] `SURF-REL-010` multi-key sort retains authored order, direction and null
  ordering.
- [ ] `SURF-REL-011` nested result record/list shape, empty shape, duplicate
  output name and width limit.
- [ ] `SURF-REL-012` nested relation/EXISTS depth exact cap and cap-plus-one.
- [ ] `SURF-REL-013` unknown pipeline stage and valid keyword-prefix neighbor.

### 8.5 Graph, check and presentation families

- [ ] `SURF-GRAPH-001` all seven subkinds parse and classify exactly.
- [ ] `SURF-GRAPH-002` source/sink/seed endpoint cardinality per subkind.
- [ ] `SURF-GRAPH-003` depth absent, zero, one, exact cap, cap-plus-one and
  numeric overflow.
- [ ] `SURF-GRAPH-004` every direction keyword and unknown direction.
- [ ] `SURF-GRAPH-005` `via`, `through`, `sanitizers`, `scope`, `window`,
  `frontier`, `facets` empty/one/many/duplicate/order cases.
- [ ] `SURF-GRAPH-006` duplicate scalar and duplicate list clauses.
- [ ] `SURF-GRAPH-007` clause names in unsupported subkinds reject rather than
  being ignored.
- [ ] `SURF-CHECK-001` every check mode, confirm mode, emit mode and nested-find
  combination.
- [ ] `SURF-CHECK-002` missing required clause and duplicate clause matrix.
- [ ] `SURF-PRES-001` explain/show/describe/sample/compare/export positive
  matrix.
- [ ] `SURF-PRES-002` every verb's missing, duplicate, unknown and wrong-type
  option matrix.
- [ ] `SURF-PRES-003` sample limit zero, one, cap and overflow.

### 8.6 Catalog, meta and composition

- [ ] `SURF-CAT-001` view/model/pack/profile/gate/ruledef positive matrix.
- [ ] `SURF-CAT-002` inline versus referenced view payload.
- [ ] `SURF-CAT-003` missing, duplicate, unknown and wrong-type options for
  every catalog family.
- [ ] `SURF-META-001` use-pack activation/lockfile combinations.
- [ ] `SURF-META-002` require-certified exact syntax and trailing-token reject.
- [ ] `SURF-COMP-001` every admitted producer family with every admitted body
  family.
- [ ] `SURF-COMP-002` each forbidden producer/body family fails closed.
- [ ] `SURF-COMP-003` zero, one, many bindings; duplicate names; keyword-shaped
  binding names.
- [ ] `SURF-COMP-004` nested/local source use and explicit `with` identity.
- [ ] `SURF-DOC-001` one root, all root families, mixed roots and maximum item
  count.
- [ ] `SURF-DOC-002` leading, repeated, trailing and missing separator policy.
- [ ] `SURF-DOC-003` malformed middle item reports the exact item span and
  issues no partial document.

## 9. Formatter and structural-equivalence checklist

- [ ] `FMT-001` every accepted scenario formats and reparses.
- [ ] `FMT-002` second format is byte-identical.
- [ ] `FMT-003` parse/format/parse preserves structural witness ignoring only
  IDs, spans and semantically neutral parentheses.
- [ ] `FMT-004` all root families and mixed documents retain authored order.
- [ ] `FMT-005` precedence-changing parentheses are retained.
- [ ] `FMT-006` unnecessary parentheses follow one canonical policy.
- [ ] `FMT-007` string literal values survive escaping exactly.
- [ ] `FMT-008` UTF-8 identifiers/strings remain valid and spans of reparsed
  output are source-local.
- [ ] `FMT-009` raw HIR with empty shape, partial graph endpoints, staged join
  RHS or unrepresentable state rejects.
- [ ] `FMT-010` parser-invalid formatted output rejects before being returned.
- [ ] `FMT-011` a semantic field dropped, reordered, deduplicated or defaulted
  is detected by the structural comparator.
- [ ] `FMT-012` deep raw shapes and alternating relation/boolean trees remain
  stack safe.
- [ ] `FMT-013` formatter failure retains original source identity.
- [ ] `FMT-014` formatting never implies closed semantic or runtime validity.

## 10. HIR integrity checklist

- [ ] `HIR-ID-001` IDs are unique across all roots and all nested nodes.
- [ ] `HIR-ID-002` same raw ID in another source/document is not provenance.
- [ ] `HIR-ID-003` duplicate root/nested/cross-root identity rejects.
- [ ] `HIR-SPAN-001` inverted, out-of-range and wrong-source spans reject.
- [ ] `HIR-SPAN-002` zero-width spans are accepted only where explicitly legal.
- [ ] `HIR-SPAN-003` every auxiliary reference/name/field span is checked.
- [ ] `HIR-SHAPE-001` all closed enum variants are visited exhaustively.
- [ ] `HIR-SHAPE-002` each public constructor rejects impossible empty or mixed
  states.
- [ ] `HIR-SHAPE-003` programmatic raw HIR cannot bypass depth/node/width limits.
- [ ] `HIR-WALK-001` traversal visits each node exactly once in authored order.
- [ ] `HIR-WALK-002` asymmetric child fixtures prove traversal direction.
- [ ] `HIR-WALK-003` very deep owned and rejected raw trees drop without stack
  overflow.
- [ ] `HIR-OWN-001` checked ownership is established before recursive use.
- [ ] `HIR-OWN-002` a node transplanted between documents/sources rejects.

## 11. Binder and scope checklist

- [ ] `BIND-SCOPE-001` local declaration, parameter, row alias, projection
  alias, group key and quantifier binder scopes.
- [ ] `BIND-SCOPE-002` duplicate binding rejects at the second declaration.
- [ ] `BIND-SCOPE-003` permitted shadowing and forbidden shadowing matrix.
- [ ] `BIND-SCOPE-004` capture barriers prevent accidental outer binding use.
- [ ] `BIND-SCOPE-005` nested EXISTS records exact non-empty and explicit-empty
  capture sets.
- [ ] `BIND-SCOPE-006` captures attach to the correct crossed boundary.
- [ ] `BIND-JOIN-001` RHS alias is invisible to LHS and visible to ON/later
  stages.
- [ ] `BIND-PROJ-001` projection replaces the row frame and drops prior aliases.
- [ ] `BIND-GROUP-001` post-group frame exposes only keys and valid aggregates.
- [ ] `BIND-MOD-001` local declaration, direct import and explicit re-export
  precedence.
- [ ] `BIND-MOD-002` same-origin diamond collapses; different-origin candidates
  remain ambiguous.
- [ ] `BIND-MOD-003` ambiguity ordering and primary span are input-order
  invariant.
- [ ] `BIND-SRC-001` prior composition binding beats host fallback.
- [ ] `BIND-SRC-002` self and forward references do not fall back to host.
- [ ] `BIND-SRC-003` duplicate graph-use key rejects before arbitrary selection.
- [ ] `BIND-DIAG-001` multiple diagnostics sort by canonical location/code,
  independent of emission order.

## 12. Type-system checklist

- [ ] `TYPE-UNIV-001` every scalar, record, union, tuple, optional, collection
  and relation type constructor.
- [ ] `TYPE-UNIV-002` type depth, node and name-byte boundaries.
- [ ] `TYPE-UNIV-003` duplicate record field and union variant rejection.
- [ ] `TYPE-LIT-001` every literal maps to one checked typed literal.
- [ ] `TYPE-LIT-002` non-finite floats are non-constructible; negative zero
  normalization is canonical.
- [ ] `TYPE-OP-001` every unary/binary operator positive domain.
- [ ] `TYPE-OP-002` no candidate, one candidate and ambiguous candidate.
- [ ] `TYPE-OP-003` operand-order-sensitive invalid pairs.
- [ ] `TYPE-CALL-001` builtin and host predicate namespace separation.
- [ ] `TYPE-CALL-002` positional/star argument arity and type mismatches.
- [ ] `TYPE-CALL-003` purity, determinism, monotonicity, aggregation and cost
  effects propagate to parents.
- [ ] `TYPE-UNKNOWN-001` unknown authority remains typed unknown and cannot
  fabricate a known type or plan.
- [ ] `TYPE-ERR-001` one primary type error does not create unbounded cascading
  diagnostics.
- [ ] `TYPE-RULE-001` anchored recursive components infer consistently under
  declaration/input permutations.
- [ ] `TYPE-RULE-002` unanchored recursion rejects as recursive ambiguity, not
  by an arbitrary default.
- [ ] `TYPE-RULE-003` conflicting anchors, wrong arity, wrong callee binding and
  call transplant reject.
- [ ] `TYPE-REL-001` relation row fields preserve source identity, optionality,
  cardinality and provenance.
- [ ] `TYPE-CTX-001` aggregate/contextual expected types are applied only in
  legal positions.

## 13. Module, manifest, lock and catalog checklist

- [ ] `MOD-ID-001` module/pack/version/digest canonical spelling and alias
  rejection.
- [ ] `MOD-GRAPH-001` empty, singleton, chain, diamond, disconnected graph,
  self-cycle and multi-node cycle.
- [ ] `MOD-GRAPH-002` competing cycles choose an actual canonical witness under
  every input permutation.
- [ ] `MOD-GRAPH-003` acyclic tails are excluded from cycle witnesses.
- [ ] `MOD-GRAPH-004` graph node/edge/step/authority-byte exact boundaries.
- [ ] `MOD-GRAPH-005` deep graphs remain stack safe and do not enumerate paths
  exponentially.
- [ ] `MOD-RES-001` declaration before direct import before explicit re-export.
- [ ] `MOD-RES-002` direct import is not an implicit downstream export.
- [ ] `MOD-RES-003` declaration visibility is enforced before re-export lookup.
- [ ] `MOD-RES-004` direct-import shadowing cannot suppress downstream
  re-export ambiguity.
- [ ] `MANIFEST-001` dependency, export, suite and member duplicate detection.
- [ ] `MANIFEST-002` version-range lower/upper edge and empty-range rejection.
- [ ] `MANIFEST-003` dependency graph cycle and missing-coordinate rejection.
- [ ] `LOCK-001` exact pack coordinate and content digest match.
- [ ] `LOCK-002` missing, duplicate, unused and conflicting pins.
- [ ] `LOCK-003` transitive closure is fully pinned and input-order invariant.
- [ ] `LOCK-004` lock location cannot reference an unregistered/foreign source.
- [ ] `PACK-001` language compatibility at lower/upper/excluded boundaries.
- [ ] `PACK-002` pack activation/lockfile mapping is one-to-one and deterministic.
- [ ] `PACK-003` changed content under same name/version cannot reuse authority.
- [ ] `CAT-001` every catalog payload kind has positive and wrong-kind fixtures.
- [ ] `CAT-002` missing, duplicate, unresolved and cyclic dependencies.
- [ ] `CAT-003` document predeclaration permits intended forward references only.
- [ ] `CAT-004` host/local collision and family-kind mismatch.
- [ ] `CAT-005` profile/gate/certification coherence independent of authored
  item order.
- [ ] `CAT-006` schema/catalog/pack/lock mutation after sealing is rejected.
- [ ] `PACK-SIGN-001` signature identity/verification is `CONDITIONAL`; do not
  imply cryptographic verification from a content digest alone.
- [ ] `PACK-CUSTODY-001` manifest digest and lock-pin equality are described as
  host-supplied identity proof, not verification of package bytes.
- [ ] `PACK-FACADE-001` the public closed-authority builder's dependency-free
  pack subset is tested separately from the full module-owner pack DAG.
- [ ] `PACK-FACADE-002` absence of public transitive-pack ingestion remains an
  explicit facade limit or future product choice, not a hidden implementation
  defect.

## 14. Semantic authority and transaction checklist

- [ ] `SEMA-TXN-001` one source/document produces one immutable transaction.
- [ ] `SEMA-TXN-002` zero, one, mixed and maximum item counts.
- [ ] `SEMA-TXN-003` one item failure prevents partial compiled authority.
- [ ] `SEMA-TXN-004` authored order is retained independently of internal
  predeclaration order.
- [ ] `SEMA-MEM-001` every produced semantic item belongs to the authored
  document exactly once.
- [ ] `SEMA-MEM-002` missing, extra, duplicate, foreign and same-name/wrong-ID
  substitutions reject.
- [ ] `SEMA-AUTH-001` closed authority seals schema, relation, graph, catalog,
  pack and lock inputs.
- [ ] `SEMA-AUTH-002` source/location identity is checked for every nested
  authority input.
- [ ] `SEMA-AUTH-003` open analysis never invents host relation, graph,
  catalog, pack or lock facts.
- [ ] `SEMA-AUTH-004` complete open analysis remains non-execution-authoritative.
- [ ] `SEMA-AUTH-005` incomplete versus error status precedence is exact.
- [ ] `SEMA-AUTH-006` all eight module/relation/graph optional-proof masks on a
  relation-using, graph-using and catalog-using document.
- [ ] `SEMA-AUTH-007` each supplied proof rejects foreign registry, wrong
  budget, and same-spelling/different-identity substitution.
- [ ] `SEMA-AUTH-008` empty and whitespace-only document behavior is pinned for
  all eight proof masks, including intentional vacuous completion.
- [ ] `SEMA-REF-001` resolved references retain source-qualified identity and
  provenance.
- [ ] `SEMA-REF-002` wrong family/kind produces a typed semantic diagnostic.
- [ ] `SEMA-REL-001` scan/filter/project/join/group/aggregate/sort contracts.
- [ ] `SEMA-REL-002` inner/cross join sealed invariants cannot be forged.
- [ ] `SEMA-REL-003` group partition identity and output origin are retained.
- [ ] `SEMA-EXISTS-001` correlated EXISTS/NOT EXISTS polarity and capture set.
- [ ] `SEMA-GRAPH-001` every endpoint and auxiliary reference role resolves to
  the exact authority kind.
- [ ] `SEMA-PRES-001` all authored presentation options survive semantic
  resolution.
- [ ] `SEMA-BUDGET-001` source, tokens/items, expressions, semantic steps,
  modules, rules, witnesses, diagnostics and response caps remain distinct.
- [ ] `SEMA-BUDGET-002` budget is checked before expensive recursive/graph
  allocation wherever possible.

## 15. Algebra and normalization checklist

- [ ] `ALG-SCAN-001` scan retains exact source authority, row schema,
  cardinality and provenance.
- [ ] `ALG-FILTER-001` filter retains the types-proven predicate and child.
- [ ] `ALG-PROJ-001` projection retains expression identity, alias, output type
  and row-aligned origin.
- [ ] `ALG-JOIN-001` inner join retains ON and both distinct scan occurrences.
- [ ] `ALG-JOIN-002` cross join never gains a synthetic predicate.
- [ ] `ALG-JOIN-003` self join preserves two occurrence identities.
- [ ] `ALG-GROUP-001` grouped aggregate retains authored partition order.
- [ ] `ALG-GROUP-002` scalar aggregate has one canonical implicit empty group.
- [ ] `ALG-SORT-001` multi-key direction/null policy and authored order.
- [ ] `ALG-EXISTS-001` pure EXISTS conjunct lowers to semijoin.
- [ ] `ALG-EXISTS-002` pure NOT EXISTS conjunct lowers to antijoin.
- [ ] `ALG-EXISTS-003` ordinary conjunct survives as a residual filter.
- [ ] `ALG-EXISTS-004` conjunctive EXISTS attaches left-to-right.
- [ ] `ALG-EXISTS-005` EXISTS under disjunction and unsupported negation reject
  rather than being rewritten unsoundly.
- [ ] `ALG-CORR-001` direct and nested correlation boundaries keep separate
  exact capture sets.
- [ ] `ALG-OWN-001` absent, foreign, wrong-owner and transplanted nested root
  reject.
- [ ] `ALG-FORGE-001` inner-without-ON, cross-with-ON, aggregate-without-group,
  mixed scalar/aggregate and wrong scan occurrence reject.
- [ ] `ALG-NORM-001` normalization is deterministic and idempotent.
- [ ] `ALG-NORM-002` equivalent surface forms produce equivalent normalized
  semantics under a test-only structural witness.
- [ ] `ALG-NORM-003` non-equivalent forms do not collide through sorting,
  deduplication or default insertion.
- [ ] `ALG-BUDGET-001` relation, boolean and expression walks charge exact
  steps and reject one-over.
- [ ] `ALG-MODEL-001` bounded generated algebra trees match an independent
  reference model. Current status: `GAP`.

## 16. Logic, rule graph and witness checklist

- [ ] `LOGIC-PROG-001` empty, singleton, acyclic, self-recursive, mutually
  recursive and disconnected programs.
- [ ] `LOGIC-PROG-002` program member order and exact document membership.
- [ ] `LOGIC-EDGE-001` positive, negated and aggregated polarity on every
  expression wrapper path.
- [ ] `LOGIC-EDGE-002` builtin calls never become local rule vertices.
- [ ] `LOGIC-EDGE-003` missing, extra, duplicate, nonlocal, transplanted,
  flipped-polarity and wrong-call-ID edges reject.
- [ ] `LOGIC-SCC-001` SCCs and strata are invariant under rule/literal order.
- [ ] `LOGIC-SCC-002` independent small-graph oracle agrees with production.
- [ ] `LOGIC-SCC-003` negative cycle witness is a real canonical edge path.
- [ ] `LOGIC-SCC-004` recursive aggregate rejection has fixed precedence over
  lesser errors.
- [ ] `LOGIC-SCC-005` nonrecursive aggregate remains outside recursion
  rejection.
- [ ] `LOGIC-DEPTH-001` deep chains and large rule counts are stack safe.
- [ ] `LOGIC-BUDGET-001` raw rule/literal caps reject before graph allocation.
- [ ] `LOGIC-BUDGET-002` node, edge and step exact/cap-plus-one boundaries.
- [ ] `GRAPH-KIND-001` reachability positive, self, cycle, disconnected and
  bounded cases.
- [ ] `GRAPH-KIND-002` flow source/sink requirements and wrong role.
- [ ] `GRAPH-KIND-003` slice seed/direction/extra-endpoint matrix.
- [ ] `GRAPH-KIND-004` taint source/sink/sanitizer matrix.
- [ ] `GRAPH-KIND-005` path endpoints, waypoint order and dedup identity.
- [ ] `GRAPH-KIND-006` points-to callable/entity role and mandatory depth.
- [ ] `GRAPH-KIND-007` summary facets and bounds.
- [ ] `GRAPH-BOUND-001` waypoint count never substitutes for path depth.
- [ ] `GRAPH-BOUND-002` every admitted kind has both depth and witness caps.
- [ ] `GRAPH-WIT-001` endpoint, selector, bound, dedup-identity and tie-break
  contracts are statically validated. Concrete zero/one/multiple path results,
  cycle traversal and witness materialization are `BOUNDARY` downstream work.
- [ ] `GRAPH-WIT-002` tie-break, witness cap, node order and dedup are sealed.
- [ ] `GRAPH-WIT-003` source/sink endpoints are distinct from path explanation
  nodes, following the CodeQL-style result separation.
- [ ] `GRAPH-TAINT-001` source, sink and sanitizer identities, authored order,
  endpoint legality and bounded selector retention are frontend-owned. Concrete
  sanitizer timing, branch relevance and taint propagation results are
  `BOUNDARY` downstream work.
- [ ] `GRAPH-TAINT-002` propagator semantics are `CONDITIONAL` until the
  language owns an explicit propagator construct.

## 17. Planning checklist

- [ ] `PLAN-ID-001` absent, foreign, wrong-kind, transplanted and same-ID
  wrong-payload carriers reject.
- [ ] `PLAN-REL-001` every relation arm produces table/read-only policy.
- [ ] `PLAN-GRAPH-001` every graph subkind produces path-witness/evidence-only
  policy with exact provenance.
- [ ] `PLAN-CHECK-001` check retains complete options and exact nested graph
  identity.
- [ ] `PLAN-PRES-001` all six presentation verbs retain sealed payload and
  policy.
- [ ] `PLAN-COMP-001` every admitted composition body family and exact binding
  identity.
- [ ] `PLAN-RULE-001` rule program retains members, strata and recursion policy.
- [ ] `PLAN-META-001` use-pack is typed non-plan; require-certified has the
  reserved certified lane.
- [ ] `PLAN-NOPLAN-001` every no-plan reason and catalog kind has stable typed
  vocabulary.
- [ ] `PLAN-CAP-001` capability derivation covers every algebra operator and
  nested arm exactly once.
- [ ] `PLAN-EFFECT-001` authority, determinism, evidence and effect are folded
  without escalation or loss.
- [ ] `PLAN-RESULT-001` result kind, multiplicity, ordering and render
  requirement are coherent.
- [ ] `PLAN-COST-001` qualitative cardinality, recursion and witness-risk
  classes for every family.
- [ ] `PLAN-COST-002` qualitative labels are not numeric performance claims.
- [ ] `PLAN-EXPLAIN-001` provenance walks root-first and left-before-right with
  asymmetric fixtures.
- [ ] `PLAN-BUDGET-001` exact step limit and refused-node diagnostic.
- [ ] `PLAN-DETER-001` identical sealed input produces field-identical plan.
- [ ] `PLAN-MODEL-001` independent capability/effect/result derivation for
  bounded small plans. Current status: `GAP`.

## 18. Pipeline and disposition checklist

- [ ] `PIPE-CLASS-001` every supported root maps to exactly one source family.
- [ ] `PIPE-CLASS-002` zero, multiple and malformed single-root classification
  fail explicitly.
- [ ] `PIPE-TXN-001` parse, bind, type, sema, algebra, logic and plan diagnostics
  retain their exact stage and canonical order.
- [ ] `PIPE-TXN-002` a failed earlier stage prevents later authority issuance.
- [ ] `PIPE-TXN-003` open analyze and closed compile share stage semantics but
  differ in authority issuance.
- [ ] `PIPE-DISP-001` each of 18 root families has exact planned/no-plan
  disposition.
- [ ] `PIPE-DISP-002` standalone let and rule have explicit artifact/no-serve
  disposition.
- [ ] `PIPE-DISP-003` rule program has one document-wide plan and exact members.
- [ ] `PIPE-DISP-004` unsupported mixes reject with typed logic/planning code.
- [ ] `PIPE-MEM-001` authored and produced membership are equal: no missing,
  extra, duplicate or foreign item.
- [ ] `PIPE-SRC-001` source bytes and source identity bind the compiled result.
- [ ] `PIPE-FMT-001` format entry enforces the same source budget and canonical
  formatter.
- [ ] `PIPE-TCK-001` every TCK row compiles through the facade, not an internal
  shortcut. Current broad registry is `PARTIAL`.
- [ ] `PIPE-OPEN-001` mixed open-authority masks never promote an item or issue
  an execution document and retain exact unresolved-fact ordering. Current
  status: `PROVEN` for all eight masks and empty/comment-only inputs.

## 19. V3 execution contract checklist

- [ ] `EXEC-DOC-001` document item order equals authored plan-required order.
- [ ] `EXEC-DOC-002` non-plan declarations are absent but remain typed in the
  compiled document.
- [ ] `EXEC-DOC-003` item lookup is exact document-local identity and does not
  claim cross-document provenance.
- [ ] `EXEC-FAM-001` relation payload is complete.
- [ ] `EXEC-FAM-002` composition binding/body payload is complete.
- [ ] `EXEC-FAM-003` graph/check/presentation/meta policy has no standalone
  executable payload and cannot be treated as runtime-admitted.
- [ ] `EXEC-FAM-004` rules/document-wide rule program are not V3 execution
  items.
- [ ] `EXEC-REL-001` scan/filter/project/join/group/aggregate/sort tree mapping.
- [ ] `EXEC-REL-002` schema field order/type/optionality/cardinality/provenance.
- [ ] `EXEC-EXPR-001` literal, reference, unary, binary, call, path, record,
  list, quantifier and relation-exists mapping.
- [ ] `EXEC-EXPR-002` operator and builtin identity are carried without a
  downstream registry lookup.
- [ ] `EXEC-LIT-001` i128 min/max, finite float, normalized negative zero,
  bool, UTF-8 string and null.
- [ ] `EXEC-LIT-002` execution values never reparse literal source text.
- [ ] `EXEC-CORR-001` correlation owner, occurrence and capture identity.
- [ ] `EXEC-COMP-001` ordered bindings, producer payload and body source.
  Current status: `PROVEN`; one tagged producer projection is required per
  sealed binding, body variants carry only their matching payload, and a count
  mismatch fails instead of truncating through parallel-vector `zip`.
- [ ] `EXEC-COMP-002` mixed multi-binding composition and same-name host/local
  source collision through the facade-only external consumer. Current status:
  same-name collision, SAMPLE-to-query dependency order, scoped SAMPLE, and
  graph producer/body are `PROVEN`; CHECK/EXPLAIN mixed breadth remains.
- [ ] `EXEC-PRIV-001` fields/constructors required to forge invalid DTOs remain
  private.
- [ ] `EXEC-PRIV-002` every intended public accessor is reachable through the
  facade-only consumer.
- [ ] `EXEC-DIG-001` source digest changes with source bytes.
- [ ] `EXEC-DIG-002` contract digest changes with every semantic contract field.
- [ ] `EXEC-DIG-003` qualification digest binds source and contract.
- [ ] `EXEC-DIG-004` source-only whitespace changes source and qualification
  digests; contract-digest equality is asserted only for fixtures whose
  projected execution DTOs are first proven equal. No general formatting-
  equivalence digest promise is inferred.
- [ ] `EXEC-DIG-005` digest encoding is length-delimited, domain-separated and
  independent of map/hash iteration.
- [ ] `EXEC-DIG-006` producer and independent consumer match shared golden
  vectors. Current status for `EXEC-DIG-001` through `EXEC-DIG-006`: `PROVEN`
  for the currently represented execution contract. Source bytes retain the
  unchanged schema/domain 3; contract and qualification encodings use the
  incompatible schema/domain 5 with regenerated shared vectors. The graph
  substream additionally has a test-only independent encoder.
- [ ] `EXEC-VEC-001` targeted pairwise vector coverage for future high-risk
  interactions. Current status: `P2`; exhaustive Cartesian coverage is not
  required for the current contract.
- [ ] `EXEC-WIRE-001` forward/backward wire compatibility is `CONDITIONAL`;
  the current Rust facade contract must not imply a serialized wire protocol.

## 20. Facade and external-consumer checklist

- [ ] `API-ALLOW-001` only `frontend-contract` is a public downstream seam.
- [ ] `API-ALLOW-002` CLI, diagnostics stdio and conformance import only the
  facade.
- [ ] `API-ALLOW-003` owner crates cannot leak through dependencies, features,
  re-exports or build/dev edges.
- [ ] `API-PRIV-001` all 15 current compile-fail privacy probes match exact
  source, error code and diagnostic text.
- [ ] `API-PRIV-002` a new private field/constructor requires a new negative
  probe only when externally attackable.
- [ ] `API-POS-001` facade-only positive consumer builds locked/offline and runs
  the exact Cargo-emitted artifact.
- [ ] `API-POS-002` the consumer inspects real relation/composition payloads,
  scoped SAMPLE, complete graph composition payloads, mixed multi-binding
  CHECK/EXPLAIN bodies, and shared digest vectors. Current status: `PROVEN`.
- [ ] `API-VER-001` public API diff uses the pinned dated nightly and cannot
  silently fall back to another toolchain.
- [ ] `API-VER-002` semver policy is defined before the first stable release.
- [ ] `API-FEAT-001` every advertised public feature combination compiles
  alone; no workspace feature unification hides missing dependencies. Current
  status: `CONDITIONAL`; no public facade feature matrix is advertised.
- [ ] `API-CACHE-001` fixture cache is owner-marked, path-confined and refuses
  symlink/redirection attacks.
- [ ] `API-CACHE-002` unchanged fixture bytes preserve mtimes; changed bytes
  invalidate the build. Correct invalidation is required; unchanged-mtime reuse
  is a `P2` performance property, not semantic correctness.
- [ ] `API-ART-001` missing, relative, duplicate, redirected or cache-external
  executable artifacts reject.

## 21. CLI checklist

- [ ] `CLI-CMD-001` command registry is the single authority for token, usage
  and dispatch.
- [ ] `CLI-CMD-002` `query == v3-query` and `fmt == v3-fmt` at process output.
- [ ] `CLI-CMD-003` unknown/untrusted command renders as one escaped line.
- [ ] `CLI-ARGV-001` missing/excess command/file arguments.
- [ ] `CLI-ARGV-002` non-UTF-8 command, inline source and file path.
- [ ] `CLI-IN-001` inline empty, whitespace-only, valid and invalid source.
- [ ] `CLI-IN-002` file missing, directory, FIFO/device/socket, regular symlink,
  broken symlink and symlink-to-nonregular.
- [ ] `CLI-IN-003` zero-byte, whitespace-only, exact source cap and cap-plus-one
  file.
- [ ] `CLI-OUT-001` success/failure stdout and stderr ownership.
- [ ] `CLI-OUT-002` diagnostic escaping cannot collapse the multiline error
  block or inject terminal control bytes.
- [ ] `CLI-EXIT-001` help/success/incomplete/error/usage/I/O exit codes.
- [ ] `CLI-QUERY-001` query uses open analysis and cannot issue an execution
  document.
- [ ] `CLI-FMT-001` format success is not semantic or runtime validation.
- [ ] `CLI-PROC-001` signal and broken-pipe behavior at the shipped CLI process
  boundary. Timeout and descendant cleanup belong to the test/qualification
  harness because the CLI does not spawn children.
- [ ] `CLI-DOC-001` help and docs contain the exact command set, not substring
  matches.

## 22. Diagnostics NDJSON checklist

- [ ] `STDIO-FRAME-001` one JSON object per line; brace-balanced multiline JSON
  is not accepted as framing.
- [ ] `STDIO-FRAME-002` LF, CRLF, EOF without newline and arbitrary short reads.
- [ ] `STDIO-FRAME-003` whitespace-only frames are ignored according to the
  documented policy.
- [ ] `STDIO-FRAME-004` exact frame limit and limit-plus-one across many reads.
- [ ] `STDIO-FRAME-005` overflow discards only the offending frame and recovers.
- [ ] `STDIO-JSON-001` malformed JSON, invalid UTF-8, duplicate keys, wrong
  field types and unknown fields.
- [ ] `STDIO-ID-001` string and number IDs admitted; null/bool/object/array IDs
  rejected.
- [ ] `STDIO-METHOD-001` only `v3_diagnostics` is accepted.
- [ ] `STDIO-METHOD-002` legacy, LSP, JSON-RPC and mutating methods reject.
- [ ] `STDIO-RESP-001` status and ordered typed diagnostics are deterministic.
- [ ] `STDIO-RESP-002` every response flushes before processing the next
  request.
- [ ] `STDIO-RESP-003` exact response cap, cap-plus-one and bounded fallback
  response.
- [ ] `STDIO-RESP-004` fallback itself fits at its exact minimum boundary.
- [ ] `STDIO-ERR-001` semantic errors are protocol responses; framing/protocol
  failures use the documented process failure status.
- [ ] `STDIO-IO-001` read failure, writer failure, broken stdout and interrupted
  read retain I/O identity.
- [ ] `STDIO-IO-002` a bad request does not poison the following valid request.
- [ ] `STDIO-FUZZ-001` bounded decoder/framer fuzz target. Current status:
  `P2 GAP`; process boundary and cap tests remain the release authority.

## 23. Diagnostic quality checklist

- [ ] `DIAG-CODE-001` every public diagnostic enum variant has one stable code.
- [ ] `DIAG-STAGE-001` first failing stage cannot be relabeled downstream.
- [ ] `DIAG-SPAN-001` primary span identifies the offending token/node, not a
  default root span.
- [ ] `DIAG-SPAN-002` related spans carry distinct supporting locations.
- [ ] `DIAG-ORDER-001` diagnostics sort deterministically by the documented key.
- [ ] `DIAG-DEDUP-001` duplicate facts do not create duplicate diagnostics.
- [ ] `DIAG-CAP-001` exact diagnostic cap and bounded terminal exhaustion
  diagnostic.
- [ ] `DIAG-MSG-001` messages cannot contain unescaped source/control data in
  machine output.
- [ ] `DIAG-MUT-001` targeted wrong-code, wrong-span, wrong-stage and
  arbitrary-first mutants fail owner tests. Current status: `P2`; require it
  when changing diagnostic selection, not as a perpetual full mutation run.

## 24. Metamorphic and differential suite

- [ ] `META-001` parse -> format -> parse structural equivalence.
- [ ] `META-002` format -> format byte fixed point.
- [ ] `META-003` parser-built versus equivalent programmatic HIR validation.
- [ ] `META-004` permutation of declarations whose authored order is not
  semantic does not change resolution.
- [ ] `META-005` rule/literal permutation preserves SCCs and strata.
- [ ] `META-006` parenthesis insertion/removal preserves meaning where neutral.
- [ ] `META-007` alpha-renaming local bindings preserves meaning and changes
  only contractually named identities/digests; public declarations are not
  assumed alpha-equivalent.
- [ ] `META-008` an irrelevant sibling does not alter an existing item's
  semantic payload. Document/item identity and aggregate digests may change.
- [ ] `META-009` raising a budget above observed usage does not change semantic
  output; the budget receipt may change.
- [ ] `META-010` exact-budget and larger-budget successful semantic outputs are
  equal after excluding the budget receipt.
- [ ] `META-011` repeated compile in one process and fresh processes is
  identical.
- [ ] `META-012` permutation is invariant only for collections declared
  unordered; authored-order collections must retain and document the order.
- [ ] `DIFF-001` production SCC/strata versus independent graph oracle.
- [ ] `DIFF-002` production algebra versus independent bounded model. `GAP`.
- [ ] `DIFF-003` production plan-policy derivation versus independent bounded
  model. `GAP`.
- [ ] `DIFF-004` production digest versus independent test-only canonical
  encoder. Current status: `PROVEN` for the graph canonical substream plus
  shared whole-document golden vectors; broader independent models are `P2`.
- [ ] `DIFF-005` facade producer versus external consumer golden vectors.
- [ ] `DIFF-006` two parser entry points classify/parse the same single root.

## 25. Adversarial resource and complexity suite

- [ ] `RES-001` source bytes, frame bytes, tokens and document items each have
  exact/cap-plus-one tests.
- [ ] `RES-002` expression depth/nodes, relation depth, collection width and
  projection width remain independent.
- [ ] `RES-003` semantic, module, rule and planning steps remain independent.
- [ ] `RES-004` witness depth/count, diagnostics and response bytes remain
  independent.
- [ ] `RES-005` invalid input is budgeted before expensive recovery or graph
  construction.
- [ ] `RES-006` long flat sequences cannot evade recursive depth limits.
- [ ] `RES-007` deep nested structures reject or complete without process-stack
  failure.
- [ ] `RES-008` wide declarations/imports avoid accidental quadratic or
  exponential behavior.
- [ ] `RES-009` branching manifest/catalog graphs do not enumerate all paths.
- [ ] `RES-010` frontend SCC/module cycle witness search retains only bounded
  state. Runtime graph traversal is `BOUNDARY`.
- [ ] `RES-011` huge duplicate/ambiguity candidate sets remain capped and
  deterministic.
- [ ] `RES-012` failure paths do not emit unbounded source excerpts.
- [ ] `RES-013` every harness-managed timeout sends group `SIGKILL`, reaps the
  direct child, and fails closed if the group remains observable after the
  bounded cleanup window. This does not claim ownership of escaped sessions.
- [ ] `RES-014` timeout means only “did not finish within bound”; no unsupported
  root-cause claim is emitted.
- [ ] `RES-015` repeated failures do not leak temp files, locks, children or
  owned cache state.

## 26. Determinism and reproducibility suite

- [ ] `DET-001` no semantic output depends on HashMap/HashSet iteration.
- [ ] `DET-002` canonical sorting has a total key and a deterministic tie-break.
- [ ] `DET-003` frontend cycle diagnostics, witness-contract ordering and
  canonical tie-break policy are permutation invariant where authored order is
  not semantic. Concrete runtime path/witness selection is `BOUNDARY`.
- [ ] `DET-004` authored ordering remains intact where it is semantic.
- [ ] `DET-005` locale, timezone, terminal width and color do not alter machine
  output.
- [ ] `DET-006` known outcome-affecting caller `RUSTFLAGS`,
  Cargo/Nextest/property-test settings and QGLang child selectors are sanitized
  by full gates. Other environment values are recorded or ignored by contract;
  blanket environment erasure is not required.
- [ ] `DET-007` exact toolchain and locked dependency graphs are used.
- [ ] `DET-008` build cache reuse cannot substitute stale evidence for the
  current source.
- [ ] `DET-009` qualification records HEAD, tree, inputs, tools, timings,
  reconciled inventory/event counts and hashes, and log hash. Current status:
  `PROVEN`. The receipt is intentionally non-replayable because raw Nextest
  streams are not archived; hashes are custody identifiers only.
- [ ] `DET-010` source snapshots match before the first group and after each
  group. This detects between-group drift; it is not continuous in-group file
  monitoring.

## 27. Local CI and qualification checklist

- [ ] `CI-INV-001` generated test inventory is non-empty.
- [ ] `CI-INV-002` ignored/skipped inventory is rejected.
- [ ] `CI-INV-003` every selected test has exactly one terminal event.
- [ ] `CI-INV-004` missing/extra/duplicate/unnamed terminal events reject.
- [ ] `CI-EVID-001` raw inventory/events are retained with verified hashes, or
  the receipt explicitly declares that it is non-replayable from hashes alone.
- [ ] `CI-SUM-001` summary run/passed/failed reconcile with selected/completed
  events.
- [ ] `CI-LEAK-001` event and textual leak reports reconcile and any leak fails.
- [ ] `CI-PROC-001` a non-zero child exit cannot be hidden by parsed output.
- [ ] `CI-PROC-002` timeout and interrupt send group `SIGKILL`, reap the direct
  child, and fail closed when the same process group is still observable after
  the cleanup window.
- [ ] `CI-COMP-001` each leaf appears exactly once in one serial composition.
- [ ] `CI-COMP-002` no nested recipe silently repeats an expensive gate.
- [ ] `CI-PREF-001` offline preflight checks both locked dependency graphs and
  exact toolchains before builds.
- [ ] `CI-EXT-001` external-consumer stamp is single-use and bound to run ID,
  locks, toolchain and gate source.
- [ ] `CI-DENY-001` RustSec DB origin/revision/freshness and same-run evidence.
  Routine offline gates validate a pinned, recently refreshed revision;
  network refresh is an explicit preparation step.
- [ ] `CI-SEM-001` Semgrep has positive-fixture non-vacuity and zero production
  findings.
- [ ] `CI-HONEST-001` semantic-honesty rules have real source mutants and a
  production scan.
- [ ] `QUAL-CLEAN-001` caller and managed worktree are clean and known.
- [ ] `QUAL-SHA-001` receipt SHA/tree equal checkout before and after.
- [ ] `QUAL-PATH-001` managed checkout/cache paths and ownership markers reject
  redirects/symlinks.
- [ ] `QUAL-RECEIPT-001` failed and timed-out runs cannot produce a successful
  receipt.
- [ ] `QUAL-RECEIPT-002` receipt/log writes are atomic and path-confined.
- [ ] `QUAL-REUSE-001` changed source at the stable manifest path causes a real
  rebuild.
- [ ] `QUAL-NET-001` routine qualification stays offline after explicit cache
  preparation/advisory refresh.
- [ ] `QUAL-CI-001` hosted GitHub Actions are not required; local qualification
  remains the repository authority.

## 28. Security and supply-chain checklist

- [ ] `SEC-UNSAFE-001` workspace forbids unsafe code.
- [ ] `SEC-PANIC-001` production paths forbid panic/unwrap/expect/todo and
  unchecked indexing according to workspace lints.
- [ ] `SEC-PATH-001` untrusted paths cannot escape owned roots through symlinks
  or redirection.
- [ ] `SEC-PROC-001` exact Cargo-emitted artifacts are executed; PATH lookup or
  second `cargo run` is not accepted as proof.
- [ ] `SEC-SRC-001` untrusted source and file names are escaped in output.
- [ ] `SEC-DEPS-001` dependency versions and checksums are locked.
- [ ] `SEC-DENY-001` advisories, licenses, bans and sources are checked with the
  pinned cargo-deny policy.
- [ ] `SEC-DB-001` advisory DB is official, clean, recent and revision-bound.
  Freshness means the explicit refresh timestamp meets policy, not that every
  offline gate contacts the network.
- [ ] `SEC-TOOL-001` all qualification tools are version-pinned or their exact
  observed version is recorded.
- [ ] `SEC-BUNDLE-001` content digest is not mislabeled as a cryptographic pack
  signature.
- [ ] `SEC-DOS-001` parser, graph and protocol resource-exhaustion paths are
  fail-closed and bounded.

## 29. Compatibility and release checklist

- [ ] `REL-SCOPE-001` release notes state frontend-only ownership.
- [ ] `REL-SCOPE-002` examples make no runtime/provider/materialization claim.
- [ ] `REL-API-001` facade API diff is reviewed against the intended baseline.
- [ ] `REL-API-002` breaking changes follow the declared pre-1.0/stable semver
  policy. Current status: change-triggered; it is not an unrelated-edit gate.
- [ ] `REL-LANG-001` grammar change updates parser, formatter, HIR, docs,
  examples, conformance and negative neighbors.
- [ ] `REL-DIAG-001` diagnostic code changes are treated as contract changes.
- [ ] `REL-DIGEST-001` canonical encoding changes require versioning and new
  shared vectors. Current status: `PROVEN`; source-byte encoding remains v3,
  while the changed contract and qualification encodings are v5 and the
  facade-only consumer replays the regenerated vectors.
- [ ] `REL-BUDGET-001` default/max budget changes require exact boundary and
  downstream compatibility review.
- [ ] `REL-PACK-001` module/manifest/lock changes revalidate existing authority
  fixtures.
- [ ] `REL-CONS-001` downstream consumer qualification is source-pinned and
  separate from QGLang qualification.
- [ ] `REL-PUB-001` merge/push/publication/deployment/activation are reported as
  distinct states.

## 30. Explicit downstream boundary scenarios

These rows collapse to four architecture assertions, not twelve independent
runtime E2E release blockers:

1. the frontend owns no runtime, provider, storage, deployment, or activation;
2. public DTOs contain no backend SDK or materialized-result type;
3. a validated policy/plan is not evidence that an effect or result occurred;
4. downstream code must explicitly admit or reject each execution family.

The detailed rows below are review prompts for those assertions. They are not
features to implement in this repository.

- [ ] `BOUND-RUNTIME-001` frontend never executes a query.
- [ ] `BOUND-PROVIDER-001` frontend never discovers or selects a provider.
- [ ] `BOUND-BACKEND-001` frontend DTO contains no backend SDK type.
- [ ] `BOUND-STORAGE-001` frontend owns no persistent catalog or result store.
- [ ] `BOUND-GRAPH-001` frontend seals graph requirements but does not traverse
  a runtime graph.
- [ ] `BOUND-TAINT-001` frontend validates taint query structure but does not
  claim concrete program-flow findings.
- [ ] `BOUND-CHECK-001` alert/SARIF policy does not mean an alert was emitted.
- [ ] `BOUND-PRES-001` explain/show/describe/sample/compare/export policy does
  not mean rendering or export occurred.
- [ ] `BOUND-PACK-001` pack validation does not mean remote download, serving,
  hot reload or activation occurred.
- [ ] `BOUND-MAT-001` no result materialization claim.
- [ ] `BOUND-DEPLOY-001` no deployment or activation claim.
- [ ] `BOUND-ADMIT-001` downstream must explicitly admit or reject every V3
  family before effects.

## 31. Applicability matrix for new features

Every feature needs the always-applicable core; trigger-specific rows apply only
when the change exposes that risk. “Every box for every feature” is rejected.

| Class | Required scenarios |
|---|---|
| Always | minimal positive; primary invalid neighbor; exact diagnostic; deterministic repeat; owner test; final local gate; ownership/release-boundary documentation |
| New enum/mode | each reachable variant; unknown or invalid variant |
| Syntax/format | parse-format-reparse and byte fixed point |
| Public facade | facade-only external consumer and privacy neighbor |
| Execution contract | admission/disposition plus mutation-sensitive digest/vector |
| Budgeted collection | exact lower/upper/cap-plus-one and independent charge |
| CLI/stdio | process exit/channel/framing and hostile ingress |
| Authority-sensitive | open/closed behavior, wrong authority, and foreign/transplanted carrier |
| Composable/nestable | nested, mixed-document, dependency order, and wrong-scope neighbor |
| Forgeable checked carrier | programmatic forged-carrier rejection |
| Release contract change | compatibility review and release note |

A realistic second positive, full pairwise combinations, fuzzing, or generated
mutation is added only when the changed risk justifies it.

## 32. Closure gates for the identified gaps

### TCK improvement scope (`P2`)

- [ ] registry schema exists and self-validates;
- [ ] representative public roots and high-risk cross-stage contracts have rows;
- [ ] all rows resolve to executable tests;
- [ ] expected stage/code/span/disposition are machine-checked;
- [ ] owner tests remain the semantic authority.

### Fuzz improvement scope (`P2`)

- [ ] surface target retained;
- [ ] NDJSON target added;
- [ ] authority/module structured target added;
- [ ] full open-analysis target added;
- [ ] execution encoder/digest target added;
- [ ] each target has bounded corpus and reproducer workflow;
- [ ] no fuzz claim is folded into routine qualification.

### Independent-oracle improvement scope (`P2`)

- [ ] bounded relational AST/model generator;
- [ ] independent algebra interpreter/witness;
- [ ] independent planning policy derivation;
- [ ] exhaustive small-tree comparison;
- [ ] shrinkable counterexamples;
- [ ] no production helper reuse in oracle logic.

### Execution-vector closure for the current contract

- [x] every canonical tag set is duplicate-free and all graph variants are
  enumerated;
- [x] every current graph semantic field has a mutation-sensitive contract
  test;
- [x] producer and external consumer both validate the shared whole-document
  vectors;
- [x] incompatible canonical encoding changes require a schema/domain bump;
- [x] an independent test-only graph encoder matches production bytes without
  calling production encoding helpers;
- [ ] future high-risk interactions may add targeted pairwise vectors (`P2`).

## 33. Recommended implementation order

1. `P0`: preserve current owner boundaries and keep all existing qualification
   rails fail-closed.
2. Run focused owner tests during implementation, then `just ci` once.
3. Commit the intended source and run `just qualify-local` for a clean exact-HEAD
   receipt.
4. Qualify downstream admission separately if any runtime claim is required.
5. Add the TCK, fuzz, independent-model, pairwise-vector, or mutation `P2`
   improvements only when their risk/cost tradeoff is justified.

## 34. Final acceptance definition

The current change can be accepted when:

- no `DEFECT` remains;
- every applicable `P0`, and every change-triggered `P1`, is `PROVEN` or
  explicitly `BOUNDARY` with a typed fail-closed contract;
- high-risk proofs map to a current executable owner;
- no ignored/skipped/zero-selected test is accepted;
- all shared vectors reconcile producer and consumer;
- current `just ci` passes on unchanged source;
- the final committed source passes `just qualify-local` with an exact SHA/tree
  receipt;
- runtime/provider/backend/deployment claims, if any, have separate downstream
  evidence.

Unchecked `P2`/`P3`, inactive `CONDITIONAL`, template bullets, and explicitly
downstream `BOUNDARY` rows do not block acceptance. Any failed applicable gate,
unclosed defect, or missing clean receipt must still be reported exactly.

## 35. Source-backed audit pointers

These pointers explain why the remaining items are classified as proof gaps
rather than confirmed defects.

| Finding | Current source/test evidence |
|---|---|
| Mixed open-authority masks | `crates/quanta-gqlang-v2-sema/src/authority.rs:1451-1519`; `crates/quanta-gqlang-v2-sema/tests/v3_reference_resolution.rs:68-87`; `crates/quanta-gqlang-v2-pipeline/tests/v3_analysis_document_assembly.rs:178-252` |
| Vacuous empty-document status | `crates/quanta-gqlang-v2-pipeline/src/compile.rs:507-535` and `875-977` |
| Host-supplied pack digest custody | `crates/quanta-gqlang-v2-sema/src/authority.rs:1125-1136`; `crates/quanta-gqlang-v2-modules/src/lockfile.rs:289-339` |
| Public dependency-free pack subset | `crates/quanta-gqlang-v2-sema/src/authority.rs:1250-1295`; full graph authority remains in `crates/quanta-gqlang-v2-modules/src/authority.rs:84-227` |
| Graph endpoint/selector matrix | full required/missing/unexpected-role loops and deterministic error precedence in `crates/quanta-gqlang-v2-logic/src/graph.rs`; duplicate singular-role rejection in `crates/quanta-gqlang-v2-sema/src/item.rs`; source-to-final-plan selector retention in `crates/quanta-gqlang-v2-pipeline/tests/v3_compiled_document_assembly.rs` |
| Multi-binding external composition | body reachability in `crates/quanta-gqlang-v2-pipeline/tests/v3_composition_source_query.rs`; same-name owner cases in `crates/quanta-gqlang-v2-sema/tests/v3_composition_query.rs`; facade-only collision, sequential multi-binding, scoped SAMPLE, graph producer/body, CHECK and EXPLAIN fixtures in `tools/frontend_external_consumer/src/main.rs` |
| Digest independent oracle | field-mutation and independent graph-substream encoder tests in `crates/quanta-gqlang-v2-pipeline/src/execution.rs`; shared producer/consumer whole-document vectors in `crates/quanta-gqlang-v2-pipeline/tests/fixtures/execution-v3-vectors.tsv` |
| Formatting-equivalent digest boundary | `docs/contracts/frontend-execution-v3.md` promises no general contract-digest equality; `crates/quanta-gqlang-v2-pipeline/tests/v3_compiled_document_assembly.rs:1450-1473` proves DTO and contract-digest equality for one named whitespace fixture |
| Process-group cleanup | `tools/process_guard.py` sends group `SIGKILL`, reaps the direct child, and now raises if the group remains observable after the bounded cleanup window; `tools/qualification/tests/test_process_guard.py` pins both disappearance and fail-closed behavior |
| Nextest evidence portability | `tools/ci/nextest_gate.py` reconciles raw inventory/events in-run and persists counts plus hashes; `tools/qualification/qualify_local.py` embeds only the summary; `tools/qualification/README.md` defines the receipt as intentionally non-replayable |
| Surface-only fuzz target | `fuzz/fuzz_targets/surface_document.rs`; the boundary is explicitly documented in `fuzz/README.md` |
| Current root conformance breadth | one canonical case per 18 roots in `crates/quanta-gqlang-v2-conformance/src/lib.rs`; executable validation in `crates/quanta-gqlang-v2-conformance/tests/self_validation.rs` |
| Surface fixture-corpus breadth | five independent `.qglang`/`.expect` pairs under `crates/quanta-gqlang-v2-surface/tests/spec_cases`; broad Rust owner coverage exists outside this portable corpus |
| Last clean qualification baseline | `target/qualification/14e2d1264a488d9838f54ad84e70a4bda51144bd/receipt.json` records successful `1239/1239`, zero skipped/ignored and zero leaks for the pre-edit source snapshot; it does not qualify the current dirty source |

Line numbers bind this audit snapshot only. The executable tests and current
source, not these line numbers, remain authoritative after later edits.
