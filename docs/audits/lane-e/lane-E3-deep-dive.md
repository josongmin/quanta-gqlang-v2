# Lane E-3 (round 2 deep dive): diagnostic surface + remaining execution.rs regions + digest field coverage + panic audit

- Snapshot: `docs/audits/lane-b-snapshot-20260927-010019` (HEAD `eaeb68ea2711d411616511713fb016d828cd80be`, dirty files included, sha256-verified). Read-only; not modified.
- Scope: `crates/quanta-gqlang-v2-pipeline/src/diagnostic.rs` (full, 921 lines); `execution.rs` regions ~380-577, ~707-1270, ~1430-1724, ~2009-end (put_* helpers + in-module tests); digest field-coverage cross-check for all public DTO structs; panics/unwraps reachable from the four facade entry points.
- Method: static reading only. **No builds or tests were run** (evidence rule: static proof, record limitation). Line numbers refer to the snapshot files.
- Verdict up front: **no new P0/P1/P2**. Eight P3 findings (4 PROOF_GAP/DOCUMENTATION_DRIFT-heavy), plus six suspected-then-refuted candidates with their refutations. Positive invariants verified: full digest field coverage (matrix below), deterministic ordering everywhere in scope, no production panics/unwraps/indexing, typed errors keep their stable codes through transport.

---

## 1. Diagnostic surface (`diagnostic.rs`, full)

### 1.1 Ordering determinism — VERIFIED, no defect

`diagnostic_sort_key` (diagnostic.rs:145-166) derives its key purely from public accessors: `(source_id.raw, span.start, span.end)` → `stage` → `severity` → `code` → `message`. No hash-map, no address/order-dependent input. `canonicalize_diagnostics` (diagnostic.rs:169-175) uses `sort_by` (stable) + `dedup`.

**Parity proof with the Sema owner**: the tuple is field-for-field identical to Sema's private `FrontendDiagnosticV2::sort_key` (sema/src/diagnostic.rs:136-156), so the pipeline's claim (diagnostic.rs:139-143) of reproducing the canonical Sema ordering holds for the key itself. Two residual asymmetries are filed as E3-4 below (dedup and tie order), both deterministic in practice.

**No hash-ordered structures**: `rg "HashMap|HashSet"` over `crates/quanta-gqlang-v2-pipeline/src` returns zero matches in production code; the only ordered containers used are `BTreeSet` (execution.rs:1368) and `BTreeMap` (compile.rs:917, analysis path) — both iteration-deterministic.

### 1.2 Code vocabulary — single-owner property holds for `pipeline.document.*`, not for `pipeline.analysis.*`

`DocumentAssemblyCodeV2` (diagnostic.rs:50-101) is a closed enum with one `as_str` owner; all 13 codes are `pipeline.document.*` literals named exactly once. Bridge functions `algebra_code`/`logic_code`/`planning_code`/`no_plan_code` (diagnostic.rs:183-350) are exhaustive matches, so a new owner variant breaks compilation rather than degrading a code — the module's stated fail-closed mechanism (diagnostic.rs:22-27) is structurally real.

However, the pipeline's *other* self-minted codes are not enum-owned: `OPEN_AUTHORITY_INCOMPLETE_CODE` (compile.rs:82), `PARSE_BUDGET_PROJECTION_FAILED_CODE` (compile.rs:91), `ANALYSIS_DIAGNOSTIC_LIMIT_CODE` (compile.rs:92), and three inline literals `pipeline.analysis.missing_module_authority` / `...missing_relation_authority` / `...missing_graph_authority` (compile.rs:940, 945, 950). See E3-2.

### 1.3 Stage tagging

Stage tags on projected diagnostics use Sema's `FrontendStageV2`: surface parse → `Resource` (only `ResourceExhausted`) or `Integrity` (diagnostic.rs:364-368); algebra/logic/planning/assembly → `Validation` (diagnostic.rs:398, 414, 429, 445); open-analysis missing-authority → `Declaration` (compile.rs:961). Consistent and monotone. `PipelineStageV2` (the pipeline-local six-stage enum) is exported but never constructed on any production path — E3-3.

### 1.4 Transport fidelity — VERIFIED

No typed error is flattened to a bare string with its code lost. The one Display-based transport (compile.rs:905-914, `try_extend_document` failure) deliberately keeps `error.code()` as the diagnostic code (compile.rs:908) and uses `error.to_string()` only as the human message. Message truncation is bounded at the Sema constructor (`truncate_utf8`, sema/src/diagnostic.rs:96). `assembly_transaction_error` takes `&'static str` messages (diagnostic.rs:439-452) — pipeline-owned, statically checked.

---

## 2. `execution.rs` remaining regions

### 2.1 Value types / finite float / union variants (~380-577) — VERIFIED

- `project_value_type` (399-455) is a total ordered if-chain over every `ValueTypeV1` kind with a documented fail-closed `Unknown` remainder (451-454).
- `ExecutionFiniteFloatV3` (859-873) is raw IEEE-754 bits. **Negative-zero determinism verified at the owner**: `FiniteF64V2::try_new` canonicalizes `value == 0.0` to `0.0f64.to_bits()` (types/src/ty.rs:726-735) and rejects non-finite values (`types.float_non_finite`); the owner test `finite_float_normalizes_signed_zero_and_orders_deterministically` (types/tests/v3_types_universe.rs:75-94) pins ±0 equality. `value.get().to_bits()` at execution.rs:1731 round-trips those canonical bits stably. No NaN can enter (construction-rejected). Refuted candidate R2.
- `project_call_target` Builtin fallback `"invalid_closed_builtin"` (536) and `project_operator` fallback `"invalid_closed_operator"` (896-899) are the one soft spot — E3-6.

### 2.2 Schema / order keys / sources / bindings / call targets / operators / correlations / sample modes (~707-1270) — VERIFIED

Struct definitions (620-1258) are pure data with `const fn` getters. Projections:

- `project_plan` (577-616): exhaustive over authority/determinism/evidence/effect/policy enums; the `PlanPolicyV2::*` payload interiors are deliberately dropped (family only) — documented as fixed-field schema (lib.rs D22 note).
- `project_source` (1863-1889): maps the three resolved source kinds; namespace is `Some` for Relation/View, `None` for Composition — encoded with a presence byte downstream (2272-2278), so the asymmetry is lossless.
- `project_binding` (1891-1898): copies source_id/owner_node/ordinal/name verbatim; all four are digest-covered.
- Sample modes (1642-1648) and graph vocabularies (1555-1659): exhaustive 1:1 const matches, each mirrored by a duplicate-free tag test (2852-2904).

### 2.3 Composition projection rest + graph query projection (~1430-1724) — VERIFIED with one P3

- `project_composition` (1415-1530) is a same-variant + same-item-id latch: a Query producer only accepts a `Relation` projection whose `item_id` equals `entry.producer().item_id()` (1429-1436), Sample only accepts a Sample presentation kind with an `else` fail (1441-1449), Graph checks item identity (1461-1468), and the body match latches identity for Query/Graph (1484-1501). Cross-item transplant of same-variant payloads is rejected — tested at 3062-3117. This closes the "silently transplanted payload" invariant.
- `project_graph_query` (1532-1553) copies every contract field; `max_path_length` comes from `contract.witness().max_path_length()` (1550) — single witness sub-contract, no second source of truth.
- `project_node` (1661-1710) / `project_expression` (1712-1722) / `collect_operator_roots` (1308-1324): the closure root set is exactly predicates ∪ expressions ∪ correlation references ∪ schema partition keys ∪ order-key expressions, recursively over children; `close_expression_ids` (1363-1383) returns a **BTreeSet-sorted, deduplicated** identity list and fails closed on Foreign (1371-1373) and Dangling (1377-1379). Order pinned by test (3650). Refuted candidate R1.

### 2.4 put_* encode helpers + in-module tests (~2009-end)

Encoding rules documented at 1900-1907 (domain separation, big-endian, length prefixes, one tag byte per variant, explicitly ordered collections) are followed uniformly. `put_usize` saturation to `u64::MAX` (2641-2645) is documented and unreachable on real targets. `sha256` (2655-2660) delegates to `sha2`. `hex_lower` (51-60) is total.

Test quality (2662-4381): pinned golden vectors (2720-2743), duplicate-free tag sets for all 24 tag functions (2752-2905), an **independent re-implementation** of the graph-query encoder cross-checked byte-for-byte (3120-3277), per-field mutation rails for every graph field, sample scope, plan fields, schema fields, expression payloads, call targets, composition binding order (3296-4058), closure semantics (3632-3670), owner-fidelity tests through authored sources (4213-4381, including an independently hand-derived host-predicate fingerprint at 4362-4367). This is strong for a syntactic audit gate. The one continuity gap is E3-8.

---

## 3. Digest field-coverage cross-check (scope item 3)

Method: for each public DTO struct, list its fields from the definition and verify exactly one `put_*` emission in the `digest_contract` path (1927-1953 → put_composition/put_relation/put_node/put_schema/put_graph_query/put_expression_shape/put_literal/put_value_type/put_binding/put_operator/put_call_target/put_key/put_item_id/put_str).

| DTO struct (definition) | Fields | Encoding site(s) | Covered exactly once |
|---|---|---|---|
| `ExecutionItemV3` (180-186) | item_id, family, relation?, composition?, plan | 1934-1950 (+ presence bytes 1944, 1949) | ✅ 5/5 |
| `ExecutionPlanV3` (231-237) | authority, determinism, evidence, effect, policy_family | 1936-1940 | ✅ 5/5 |
| `ExecutionRelationV3` (620-624) | item_id, root, expressions | put_relation 2248-2256 | ✅ 3/3 |
| `ExecutionRelationalOperatorV3` (643-654) | family, join_mode?, stage, source?, schema, predicates, expressions, correlations, partition_input, children | put_node 2258-2305 | ✅ 10/10 |
| `ExecutionSchemaV3` (702-707) | fields, cardinality, partition_keys, order | put_schema 2454-2472; order keys: expr/dir/null_order/value_type 2466-2471 | ✅ 4/4 |
| `ExecutionCompositionV3` (982-989) | bindings, with_binding, body_item_id, body_family, body_relation?, body | put_composition 2026-2055 | ✅ 6/6 |
| `ExecutionCompositionBindingV3` (1026-1033) | binding, producer_item_id, producer_family, schema?, relation?, producer | 2030-2044 | ✅ 6/6 |
| `ExecutionGraphQueryV3` (1195-1206) | kind, references, depth, direction?, summary_facets, relation_shape, dedup_identity, max_witnesses, max_path_length?, witness_tie_break | put_graph_query 2097-2127 | ✅ 10/10 |
| `ExecutionGraphTargetV3::Host` (1163-1167) | kind, spelling, entity_subtype? | 2146-2155 | ✅ 3/3 |
| `ExecutionGraphTargetV3::Composition` (1168-1171) | binding, capability | 2161-2169 | ✅ 2/2 |
| `ExecutionGraphReferenceV3` (1176-1179) | role, target | 2101-2102 | ✅ 2/2 |
| `ExecutionExpressionV3` (770-774) | id, value_type, shape | 2253-2255 | ✅ 3/3 |
| `ExecutionSourceV3` (913-918) | authority, kind, namespace?, name | 2269-2279 | ✅ 4/4 |
| `ExecutionBindingV3` (939-944) | source_id, owner_node, ordinal, name | put_binding 2619-2624 | ✅ 4/4 |
| `ExecutionOrderKeyV3` (744-749) | expression, direction, null_order, value_type | 2467-2470 | ✅ 4/4 |
| `ExecutionFieldV3` (728-731) | name, value_type | 2457-2458 | ✅ 2/2 |
| `ExecutionOperatorV3` (879-882) | symbol, overload_id | put_operator 2625-2628 | ✅ 2/2 |
| `ExecutionCallTargetV3` (527-531) | name / name / fingerprint | put_call_target 2589-2604 | ✅ all arms |
| `ExecutionLiteralV3` (851-857), `ExecutionFiniteFloatV3` | payload | put_literal 2433-2453 | ✅ all arms |
| `ExecutionValueTypeV3` (349-365) incl. `Union`/`Unknown` and `ExecutionUnionVariantV3` (384-387) | all arms | put_value_type 2474-2518; union tag+type 2511-2514; **Unknown IS encoded** (tag 7, 2516) | ✅ all arms |

**Result: zero coverage gaps.** Every field of every public DTO in scope is encoded exactly once; no `put_*` helper silently skips a field. The only intentional exclusion is *semantic*, not byte-level: the `Unknown` value-type variant deliberately does not discriminate its cause (documented at 360-363) — it still occupies a distinct tag. (Refuted candidate R6.) Mutation rails independently back the matrix: every field listed above has at least one `assert_ne!` digest-movement test (3296-4058, 3812-3986).

Document-level: `digest_contract` binds `source_id.raw()` + item count + items (1931-1932); `digest_qualification` binds domain string, contract schema version, source_id, and both 32-byte legs (1961-1967). Cross-binding is sound (source-byte mutation moves qualification but not contract — test 3615-3629).

---

## 4. Panics / unwraps reachable from the four facade entry points — CLEAN

`rg "unwrap\(\)|expect\(|panic!|unreachable!|\.remove\(0\)"` over all three pipeline source files returns 95 matches; **every one is inside a `#[cfg(test)] mod tests`** (execution.rs tests begin 2662 with `#[expect(clippy::expect_used, clippy::indexing_slicing, clippy::panic, ...)]` at 2663-2668; diagnostic.rs tests 492-496; compile.rs tests 1927+). Production-path totality devices are all `unwrap_or`-style and documented: `hex_lower`'s `unwrap_or('?')` (execution.rs:56), `put_usize`'s `unwrap_or(u64::MAX)` (2644), `usize::try_from(limit).unwrap_or(usize::MAX)` (compile.rs:856, 884), and the two display fallbacks at 536/896-899 (see E3-6). Indexing (`bindings[0]` etc.) appears only in tests. **No production panic path found.**

---

## 5. Findings

All P3. Each: violated invariant / why it applies / trigger-expected-actual / static proof / minimal root fix + owner / regression oracle / refutation attempt.

### E3-1 · PROOF_GAP (P3) — algebra/planning bridge-code parity with owner `code()` is unasserted; module doc overclaims
- **Invariant**: diagnostic.rs:26-27 states "the document-assembly suites additionally assert the bridged string equals the owner's own `code()` output for every reachable variant."
- **Why it applies**: in-crate tests assert owner-parity only for logic codes (`assert_eq!(logic_code(&d), expected); assert_eq!(diagnostic.code(), expected)` — diagnostic.rs:641-644, 676-678, 718-720, 745-747). The algebra test (578-619) and planning test (751-921) assert the bridge literals only — never the owner's `code()`; `rg algebra_code` over `tests/` finds no integration parity assertion either (owner-side tests like algebra/tests/v3_algebra_hellgate.rs:36 pin the owner string independently, which is exactly the drift the bridge is supposed to prevent).
- **Trigger/expected/actual**: someone edits one literal in `algebra_code` (or the owner renames a code) → expected: a test fails naming the divergence; actual: both sides stay green against their own literals and the streams silently disagree.
- **Static proof**: cited lines; no `.code()` comparison in the algebra/planning test bodies.
- **Root fix**: extend the two tests to construct owner diagnostics and assert `bridge(x) == x.code()` for each arm, mirroring the logic pattern; owning crate `quanta-gqlang-v2-pipeline` (test-only change).
- **Oracle**: mutate one literal in `algebra_code` → new assertion must fail.
- **Refutation attempt**: "integration suites cover it end-to-end" — only for the handful of codes reachable from authored failure sources (e.g. v3_compiled_document_assembly.rs:1980, 2487), not "every reachable variant"; the doc's universal claim remains unproven. Not refuted.
- **Limitation**: static only; did not run the suites.

### E3-2 · DOCUMENTATION_DRIFT + STRUCTURAL_RISK (P3) — open-analysis code vocabulary has no single owner
- **Invariant**: "the pipeline's own exact-key assembly codes … and open-analysis codes" are owned by this module (diagnostic.rs:15-16); closed-vocabulary principle (AGENTS: stable codes, closed so callers match without parsing text, diagnostic.rs:45-49).
- **Why it applies**: the `pipeline.analysis.*` and `pipeline.parse_budget_projection_failed` codes are minted in compile.rs — two exported consts (82, 91), one private const (92), three inline literals (940, 945, 950). No enum, no exhaustiveness rail.
- **Trigger/expected/actual**: a typo in the inline literal at compile.rs:945 → expected: compile error or test failure; actual: a silently different stable code reaching consumers.
- **Static proof**: grep table in §1.2.
- **Root fix**: fold the six codes into a closed `PipelineAnalysisCodeV2` enum (or at least a const block) in diagnostic.rs next to `DocumentAssemblyCodeV2`; owner `quanta-gqlang-v2-pipeline`.
- **Oracle**: literal-independence test mirroring `every_pipeline_owned_code_is_an_independent_literal` (508-575).
- **Refutation attempt**: "consts are already single-definition" — true for the three consts, false for the three inline literals; and the module-boundary claim in the doc header is wrong either way. Not refuted.

### E3-3 · DOCUMENTATION_DRIFT (P3) — `PipelineStageV2` is exported dead surface with an unimplemented rationale
- **Invariant**: public API claims must be backed (doc at diagnostic.rs:103-107: "this enum exists so the pipeline can report which stage short-circuited").
- **Why it applies**: `PipelineStageV2` is re-exported (lib.rs:38) but constructed nowhere outside its own as_str test (grep §1.2/§1.3: only diagnostic.rs:566-571). Actual stage tagging uses Sema's `FrontendStageV2`. Additionally the claim "the stage is also derivable from the stable code prefix" (105-106) breaks for `IllegalGraphQuery`/`IllegalRuleProgram`, which relay `logic.*` codes through the planning stage (257-258).
- **Trigger/expected/actual**: a downstream consumer reads the enum expecting stage reporting → expected: a producer; actual: none.
- **Static proof**: exhaustive grep for `PipelineStageV2` across all crates (9 hits, all definition/test/export).
- **Root fix**: either remove the enum from the public export or wire it; amend the prefix-derivability comment to note the two logic relays; owner pipeline.
- **Oracle**: `rg PipelineStageV2` on production paths after the change.
- **Refutation attempt**: "it's for future facade use" — nothing in the facade/conformance snapshot consumes it (grep over `crates/` shows no consumer). Drift stands.

### E3-4 · DOCUMENTATION_DRIFT (P3) — `canonicalize_diagnostics` attributes dedup to "the canonical Sema order", which Sema does not define
- **Invariant**: comment accuracy for the ordering contract (diagnostic.rs:139-143, 168).
- **Why it applies**: Sema's `FrontendFailureV2::try_new` sorts by the same key but does **not** dedup (sema/src/diagnostic.rs:167-173). The pipeline additionally dedups on full-struct equality after a stable sort, so (a) exact duplicates the Sema path would keep are dropped here, and (b) two diagnostics equal on the five sort-key fields but differing in `secondary` spans survive in stable-insertion order — deterministic only because every producer assembles diagnostics in a fixed stage order.
- **Trigger/expected/actual**: reader assumes Sema-defined dedup semantics → expected: Sema parity; actual: pipeline-local dedup + tie rule not documented anywhere.
- **Static proof**: cited lines on both sides.
- **Root fix**: one-line comment fix in pipeline ("Sema key order, plus pipeline-local full-struct dedup; equal-key ties keep insertion order"), or push dedup into Sema's canonicalization if parity is the intent. No behavior change required.
- **Oracle**: none needed for comment fix; for a behavior change, a fixture with an equal-key distinct-secondary pair asserting documented order.
- **Refutation attempt**: "dedup is harmless because owners never emit exact duplicates" — plausible but unproven statically; regardless, the comment-vs-owner mismatch is the finding. Behavior itself: not classified as a defect.

### E3-5 · DOCUMENTATION_DRIFT (P3) — `empty_blocked_failure` reuses `pipeline.document.missing_item_result` for an unrelated impossible condition
- **Invariant**: "Each code is minted only by this crate's document assembler" for its named condition (diagnostic.rs:45-49; `MissingItemResult` = "an authored item that requires one produced result has none", 52-53).
- **Why it applies**: the impossible-empty fallback (diagnostic.rs:478-490) mints `MissingItemResult` with message "a blocked stage reported no diagnostic" at a synthesized zero location. A consumer matching the code would misclassify the (unreachable) condition.
- **Trigger/expected/actual**: `document_transaction_failure(vec![])` → expected: a code naming the actual condition; actual: `missing_item_result`.
- **Static proof**: both call sites (compile.rs:1041, 1063) follow diagnostic-producing loops, so emptiness is unreachable in the current graph; the function signature still admits it.
- **Root fix**: dedicated fallback code in `DocumentAssemblyCodeV2` (or fold E3-2's enum work and add `BlockedStageEmpty`); owner pipeline.
- **Oracle**: unit test `document_transaction_failure(Vec::new())` asserting the new code.
- **Refutation attempt**: "unreachable, therefore no risk" — accepted for severity (P3), but the closed-vocabulary invariant is still bent and the fix is one variant. Not refuted as drift.

### E3-6 · STRUCTURAL_RISK / PROOF_GAP (P3) — closed-vocabulary fallbacks degrade to a shared sentinel string instead of failing closed
- **Invariant**: malformed/unsupported input must not become silently-omitted or colliding identity (AGENTS failure-behavior rule); `ExecutionCallTargetV3::Builtin.name` is the *identity* payload encoded into the contract digest (put_call_target 2591-2594 encodes only the name).
- **Why it applies**: `project_call_target` maps a `None` builtin name to `"invalid_closed_builtin"` (execution.rs:536); `project_operator` maps a `None` symbol to `"invalid_closed_operator"` (896-899). If the owner ever admits an invalid id, N distinct invalid builtins collapse to one digest identity (Builtin) and operators lose their display witness while keeping `overload_id` — the Builtin case is the identity-affecting one.
- **Trigger/expected/actual**: owner grows/permits an id whose `name()` is `None` → expected: projection rejects; actual: a sentinel string flows into the digest.
- **Static proof**: cited lines; no test exercises the `None` arm of either accessor (grep over tests shows no `invalid_closed_builtin` fixture).
- **Root fix**: since the vocabularies are closed, replace the fallback with a fail-closed projection error (or at minimum a distinct-per-id encoding); owner pipeline, coordinated with sema's `OperatorIdV2`/builtin id totality proof.
- **Oracle**: unit test constructing the invalid id (if the owner API permits) expecting rejection rather than a sentinel.
- **Refutation attempt**: "name() is total by construction, so the arm is dead" — likely true today (that is why this is P3, not P2), but the pipeline's own style is to fail closed on unreachable remainders (cf. 451-454, which returns `Unknown` only for a documented total-projection reason, and the exhaustive-match policy in diagnostic.rs). The asymmetry is unproven-safe. Not refuted.

### E3-7 · OPTIONAL_REFACTOR (P3) — dual encoding of `ExecutionFamilyV3` and vestigial constant leading tags in the digest stream
- `item.family` is encoded as a tag byte (1935, `family_tag`) while `producer_family`/`body_family` — the same enum — are encoded as keyword strings (2032, 2048, `keyword()`). Both maps are injective and duplicate-free (tests 2758-2766, 297-307), so there is no collision today; the cost is three places to update per new variant (tag fn, keyword fn, tag test) and reader confusion.
- `put_composition` and `put_relation` each emit a constant leading `1` (2027, 2248) with no `0` arm inside the helper — optionality is encoded by the caller (1941-1950). The constant is a harmless domain disambiguator, but it reads like a presence byte that never has the other value.
- **Fix (optional)**: unify on one encoding per enum; drop or comment the leading tags. Owner pipeline, digest schema bump not required if byte stream is preserved (pure comment for the tags; unifying family encoding *would* bump `CONTRACT_DIGEST_SCHEMA_VERSION` — do it only at the next incompatible change).
- **Refutation attempt**: "changing the encoding moves golden vectors" — correct, which is exactly why this is optional/no-op-today and bundled with any future schema bump rather than done standalone.

### E3-8 · PROOF_GAP (P3) — golden vectors are producer-derived from in-module fixtures (continuity with prior-round P2)
- The pinned digests (2731-2742) are computed from `relation_item()` and friends defined in the same file as the encoder (2994-3059); nothing external and independently authored fixes the byte layout for the relation/composition/item substreams. The graph substream is the exception — test 3120-3277 cross-checks against a locally independent encoder — but that encoder is still in-module.
- **Root fix**: publish fixed hex vectors for one minimal document in `docs/contracts/frontend-execution-v3.md` (or a conformance fixture crate outside the pipeline crate) and assert against the document text, so a coordinated edit of encoder+fixtures cannot keep the test green. Owner: pipeline + docs contract.
- **Oracle**: flip a tag pairing (e.g. swap two `scalar_tag` values) → external vector test must fail even though in-module fixtures would be regenerated consistently.
- **Refutation attempt**: "the mutation rails already pin field coverage" — they pin *movement*, not *values*; two simultaneous changes (encoder swap + fixture regeneration) pass all rails. Not refuted. (Matches the prior round's P2 classification; kept P3 here only because this lane adds no new evidence of actual drift.)

---

## 6. Suspected-then-refuted candidates

- **R1 — "expression-table order may follow hash iteration"**: refuted; `close_expression_ids` uses `BTreeSet` (execution.rs:1368) and the sorted result is pinned by test (3650). No HashMap/HashSet exists anywhere in the pipeline crate's production code.
- **R2 — "`ExecutionFiniteFloatV3` may distinguish −0.0 or admit NaN"**: refuted; owner canonicalizes signed zero at construction (types/src/ty.rs:730-731) and rejects non-finite values (727-729), with owner tests (types/tests/v3_types_universe.rs:75-94).
- **R3 — "`put_composition`'s leading tag could alias the caller's presence byte"**: refuted; the caller writes the optionality byte (1946-1950) and the helper's constant `1` is a separate stream position; per-field length prefixes keep the stream prefix-free (2026-2055).
- **R4 — "`canonicalize_diagnostics` dedup could drop semantically distinct diagnostics"**: refuted as a determinism/correctness defect — dedup is full-struct equality, so diagnostics differing in any field (incl. secondary spans) survive; the residual is only the documentation asymmetry filed as E3-4.
- **R5 — "typed error flattened to string at compile.rs:912"**: refuted; the stable `code()` is preserved on the same diagnostic (compile.rs:908); `to_string()` is message transport only.
- **R6 — "`Unknown` value-type is excluded from the contract digest (coverage gap)"**: refuted; it is explicitly encoded with tag 7 (execution.rs:2516) and covered by the distinct-encodings test (3760-3798); the documented exclusion is cause-discrimination, not byte omission.

---

## 7. Summary table

| ID | Class | Sev | One-liner | Owner |
|---|---|---|---|---|
| E3-1 | PROOF_GAP | P3 | algebra/planning bridge↔owner `code()` parity unasserted; module doc overclaims "every reachable variant" | pipeline (tests) |
| E3-2 | DOCUMENTATION_DRIFT/STRUCTURAL_RISK | P3 | `pipeline.analysis.*` codes minted in compile.rs without a closed enum; module-ownership claim wrong | pipeline |
| E3-3 | DOCUMENTATION_DRIFT | P3 | `PipelineStageV2` exported but never produced; rationale comment unbacked; prefix-derivability wrong for logic relays | pipeline |
| E3-4 | DOCUMENTATION_DRIFT | P3 | `canonicalize_diagnostics` credits Sema for a dedup Sema does not do; tie rule undocumented | pipeline (+sema if unified) |
| E3-5 | DOCUMENTATION_DRIFT | P3 | impossible-empty fallback reuses `missing_item_result` code for an unrelated condition | pipeline |
| E3-6 | STRUCTURAL_RISK/PROOF_GAP | P3 | `"invalid_closed_builtin"`/`"invalid_closed_operator"` sentinel fallbacks; Builtin name is digest identity → potential silent collision | pipeline (+sema proof) |
| E3-7 | OPTIONAL_REFACTOR | P3 | dual encoding of `ExecutionFamilyV3` (tag vs keyword) + vestigial constant leading tags in put_relation/put_composition | pipeline |
| E3-8 | PROOF_GAP | P3 | golden digest vectors producer-derived from in-module fixtures (continuity of prior-round P2) | pipeline + docs contract |

No P0/P1/P2. Digest field coverage: complete (§3). Production panic paths: none (§4). Ordering determinism: verified end-to-end for the audited regions (§1.1, §2.3).

*Limitation: static audit only — no compilation or test execution against the snapshot, per evidence rules. Every claim above is file:line-cited from the snapshot tree.*
