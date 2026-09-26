# Lane E-2 — Facade / Front Doors / DTO Contract / External Consumers

Audit type: static, read-only. No implementation changes.
Source boundary (fixed, sha256-verified, dirty files included):
`docs/audits/lane-b-snapshot-20260927-010019` @ HEAD `eaeb68ea2711d411616511713fb016d828cd80be`.
Below, `SN/` abbreviates that snapshot root. All `file:line` references resolve inside `SN/`.
No commands were executed against the snapshot; every claim is static proof and marked as such.
The only executed operation was `mkdir` for this report directory (outside the snapshot).

Classification vocabulary: CONFIRMED_DEFECT / STRUCTURAL_RISK / PROOF_GAP / DOCUMENTATION_DRIFT /
OPTIONAL_REFACTOR, with P0–P3 severity.

---

## 1. Ingress discipline (Task 1)

### 1.1 Cargo dependency DAG — CONFIRMED clean (no defect)

Static evidence from manifests:

| Crate | Dependencies | Verdict |
| --- | --- | --- |
| `quanta-gqlang-v2-frontend-contract` | `quanta-gqlang-v2-pipeline` only (`SN/crates/quanta-gqlang-v2-frontend-contract/Cargo.toml`, "Sole owner edge" comment) | clean |
| `quanta-gqlang-v2-cli` | `quanta-gqlang-v2-frontend-contract` + `libc` (`SN/crates/quanta-gqlang-v2-cli/Cargo.toml`) | clean |
| `quanta-gqlang-v2-diagnostics-stdio` | `quanta-gqlang-v2-frontend-contract` + `serde`/`serde_json` (`SN/crates/quanta-gqlang-v2-diagnostics-stdio/Cargo.toml`) | clean |
| `quanta-gqlang-v2-conformance` | `quanta-gqlang-v2-frontend-contract` (`SN/crates/quanta-gqlang-v2-conformance/Cargo.toml`) | clean |
| `tools/frontend_external_consumer` | standalone `[workspace]`, single path dep on the facade (`SN/tools/frontend_external_consumer/Cargo.toml`) | clean |

Every `use` in the four consumers resolves through `quanta_gqlang_v2_frontend_contract::`
(grep over `SN/crates/quanta-gqlang-v2-cli/src/**`, `SN/crates/quanta-gqlang-v2-diagnostics-stdio/src/**`,
`SN/crates/quanta-gqlang-v2-conformance/src/lib.rs` and its test, `SN/tools/frontend_external_consumer/src/main.rs`).
No consumer names `quanta_gqlang_v2_pipeline`, `_sema`, `_planning`, `_surface`, or any owner crate directly.
The facade itself re-exports from the pipeline only (no glob: `SN/crates/quanta-gqlang-v2-frontend-contract/src/lib.rs:16-66`;
glob absence is additionally gated by `GLOB_RE` in `SN/tools/test_frontend_external_consumer.py:230`).

### 1.2 Parse-only paths — CONFIRMED no hidden semantic demand

`classify_single_source_root_v3` (`SN/crates/quanta-gqlang-v2-pipeline/src/compile.rs:592-610`) is
parse-only, requires exactly one item, and returns only the root-family enum; the contract doc explicitly
frames it as a routing aid that "does not resolve semantic catalogs, issue an execution document, or grant
host/runtime authority" (`SN/docs/contracts/frontend-execution-v3.md:9-15`). `format_source_document_v2`
"performs no semantic traversal" (`SN/crates/quanta-gqlang-v2-pipeline/src/compile.rs:751-758`). The CLI
`fmt` and `query` commands call only facade entries
(`SN/crates/quanta-gqlang-v2-cli/src/commands/v3_format.rs:11-20`, `v3_query.rs:8-36`). No front door
performs a parse and then demands semantic validation outside the facade pipeline.

### 1.3 Findings

**E2-02 — STRUCTURAL_RISK P3: facade-only ingress is convention plus a one-fixture gate, not a structural guarantee.**
Violated invariant (stage direction): "only frontend-contract is the public downstream seam."
Why it applies: the pipeline crate itself is a normal public library crate with a broad `pub use` surface
(`SN/crates/quanta-gqlang-v2-pipeline/src/lib.rs:28-70`); nothing in Cargo semantics stops a sibling repo
from depending on it directly, or a future workspace member from adding a `quanta-gqlang-v2-pipeline`
dependency. The only mechanical enforcement is `tools/test_frontend_external_consumer.py`
(`exact_path_dependency`, `SN/tools/test_frontend_external_consumer.py:374-392`, and the manifest line pin
at `:622`), which checks exactly one fixture manifest.
Trigger: a downstream adds `quanta-gqlang-v2-pipeline` to its Cargo.toml. Expected: impossible or gated.
Actual: possible; only the sample fixture is checked (static proof; unexecuted).
Minimal root fix (owner: repo policy/tooling, not a crate): add a workspace-level rail (e.g. a `just ci`
step or cargo-deny rule) asserting that no manifest outside `crates/quanta-gqlang-v2-pipeline`'s own
tests references owner crates except `frontend-contract -> pipeline`. Compatibility: none. Regression
oracle: a deliberately offending manifest must fail the rail. Refutation attempt: publication is
`publish = false` only on conformance and the tool; frontend-contract itself is publishable — publishing
pipeline would also publish the bypass surface, so the risk is real but low-frequency (P3).

---

## 2. frontend-contract public surface and unforgeability (Task 2)

### 2.1 Surface shape — CONFIRMED accessor-only, curated, closed

The facade is 67 lines of named re-exports, no globs, no `pub mod`, no traits
(`SN/crates/quanta-gqlang-v2-frontend-contract/src/lib.rs:1-67`). The surface is dual-pinned:
(a) `PINNED_ALLOWLIST` (≈130 names) diffed against `cargo public-api --simplified` output in both
directions (`SN/tools/test_frontend_external_consumer.py:118-225, 471-525`), and (b) the compiled
external consumer touches every curated root (`pin_curated_roots`, `SN/tools/frontend_external_consumer/src/main.rs:937`).
Over-claiming in either direction fails the gate (comment at `SN/tools/test_frontend_external_consumer.py:115-118`).

### 2.2 ExecutionDocumentV3 unforgeability — CONFIRMED real (static proof)

- `ExecutionDocumentV3` fields are private (`source_id`, `items`, three digests:
  `SN/crates/quanta-gqlang-v2-pipeline/src/execution.rs:69-76`); the only constructor is
  `pub(crate) fn new` (`execution.rs:78-91`) which *derives* all three digests internally from
  `(source_id, source bytes, items)` — a caller cannot supply a digest.
- `ExecutionDigestV3([u8; 32])` (`execution.rs:32-33`): tuple field private; only `as_bytes`/`to_hex`
  accessors (`execution.rs:38-47`). No `From<[u8;32]>`, no `Default`, no serde on any Execution DTO
  (grep for `Default|serde|Serialize` over `execution.rs`: no matches on DTO types).
- `ExecutionFiniteFloatV3(u64)` (`execution.rs:861`): private field, only `bits`/`get`
  (`execution.rs:864-872`); constructed exactly once, from the checked `TypedLiteralFactV2`
  (`execution.rs:1727-1731`). A downstream cannot construct it from arbitrary non-finite bits —
  matching the contract claim (`SN/docs/contracts/frontend-execution-v3.md:23`).
- No trait with default methods is exported by the facade (no `pub use ... Trait` in lib.rs), so
  trait-default forgery is structurally absent.
- All Execution structs expose `const fn`/`&` accessors only (grep inventory in
  `projection-input-E2.md`).

### 2.3 Compile-fail privacy probes — CONFIRMED present and gated

15 negative probes `d01`–`d15` (`SN/tools/frontend_external_consumer/negatives/`), each pinned to an
expected rustc diagnostic code *and* message needle (`SN/tools/test_frontend_external_consumer.py:231-274`),
including `d15_private_execution_document_new` → `E0624` on `ExecutionDocumentV3::new`,
`d03`/`d09` struct-literal-over-private-fields, `d13` private `bindings`,
`d14` retired `CompiledItemV1` import. The harness attributes each cargo JSON diagnostic to the exact
probe source path and rejects unanchored evidence (`negative_observations`,
`SN/tools/test_frontend_external_consumer.py:655-722`).

### 2.4 Findings

**E2-06 — OPTIONAL_REFACTOR P3: diagnostic/failure vocabulary is publicly constructible.**
`FrontendDiagnosticV2::new`, `FrontendFailureV2::one`/`try_new` are public (exercised by the consumer at
`SN/tools/frontend_external_consumer/src/main.rs:1450-1500`), so a downstream can fabricate diagnostics
and failures. Violated invariant: none — these are inert vocabulary types carrying no authority; missing
authority is represented by `Incomplete`, not by diagnostics. Trigger/expected/actual: a consumer could
render fabricated diagnostics as if frontend-issued; harmless to the frontend's own guarantees.
Minimal root fix (owner: sema/facade, only if a consumer need emerges): none now; if tightened, keep
read-only accessors and demote constructors to `pub(crate)`. Compatibility: breaking; needs a contract
version. Regression oracle: a new negative probe `d16_construct_frontend_diagnostic` must fail to compile.
Refutation attempt: the stdio transport (the only real diagnostic consumer) never constructs them, so
there is no demonstrated need either way — classification stays OPTIONAL_REFACTOR, not a defect.

---

## 3. Plan-required items, NoPlanReason, rule program, standalone payloads (Task 3)

Producer vs consumer responsibility is documented sharply: the contract states non-plan declarations are
absent from the execution document with `NoPlanReasonV2` retained on the compiled result, that Graph /
CHECK / presentation / meta-require-certified "currently carry their checked family/policy but no
standalone executable payload", that these are "not implicitly executable", and that the admission owner
must add a payload or "reject them with an explicit typed reason before effects"
(`SN/docs/contracts/frontend-execution-v3.md:19`). Rule members and the rule program are explicitly not
V3 execution items (`:19`). The producer "never silently drops a plan-required item"
(`:19`). This is a documented limitation, not a defect; the DTO mirrors it:
`ExecutionItemV3::relation()/composition()` return `Option` while `plan()` is unconditional
(`SN/crates/quanta-gqlang-v2-pipeline/src/execution.rs:214-224`, with the comment "Producer-derived
policy and authority; callers cannot replace it").

Findings:

**E2-13 — PROOF_GAP P3: the facade boundary does not itself pin "supported family, no standalone payload".**
The external consumer pins composition payloads exhaustively
(`SN/tools/frontend_external_consumer/src/main.rs:242-314, 1241-1440`) but never compiles a top-level
Graph/Check/Presentation/MetaRequireCertified item and asserts
`relation().is_none() && composition().is_none() && plan().family() == ...`. That producer-side property
lives only in pipeline owner tests (per `SN/docs/language/test_hellgate_ledger.md:82`). Trigger: a future
producer change adds a partial graph payload; expected: facade consumers can rely on
"no standalone payload" staying exactly as documented; actual: nothing at the facade boundary would
notice a change in this specific property. Minimal root fix (owner: tools/frontend_external_consumer):
add one closed-authority compile per unsupported family asserting payload absence. Compatibility: none.
Regression oracle: the new assertions themselves. Refutation attempt: the public-api diff would catch
new accessors but not behavioral payload presence — the gap is real, though low severity because the
producer tests cover it.

**E2-04 — PROOF_GAP P3: the documented classify-rejection use case is unexercised.**
The contract invites consumers to "reject an unsupported family before constructing closed authority"
(`SN/docs/contracts/frontend-execution-v3.md:12-15`). The consumer exercises only `NoItem` and
`MultipleItems` (`SN/tools/frontend_external_consumer/src/main.rs:1041-1065`); it never classifies a
parsable-but-unsupported family and takes the documented rejection branch. Fix (owner: tools):
one classify call on, e.g., a `gate` root demonstrating pre-authority routing rejection. Refutation
attempt: classify correctness for families is pinned in pipeline tests; only the *consumer pattern*
is unproven — hence PROOF_GAP, not defect.

NoPlanReason coverage at the facade: `NoPlanReasonV2` is re-exported
(`SN/crates/quanta-gqlang-v2-frontend-contract/src/lib.rs:46`), rendered by the CLI
(`SN/crates/quanta-gqlang-v2-cli/src/v3_plan.rs:41-45`), and probed by the consumer
(`main.rs:1518-1531`). Conformance pins dispositions per root family including seven
`catalog_declaration` / one `meta_activation` no-plan families
(`SN/crates/quanta-gqlang-v2-conformance/src/lib.rs:46-160`), and those expectations are *executed
against the facade* in `SN/crates/quanta-gqlang-v2-conformance/tests/self_validation.rs` — see §7.

---

## 4. Deterministic encoding and digests (Task 4)

### 4.1 Encoder vs contract doc — CONFIRMED matching (static line-by-line proof)

Domain separation and schema versioning: three distinct domain strings
(`qglang.execution.source-bytes.v3`, `.contract.v5`, `.qualification.v5`) each followed by `U32BE`
schema (3 for source, 5 for contract/qualification) —
`SN/crates/quanta-gqlang-v2-pipeline/src/execution.rs:1917-1923, 1927-1952, 1956-1967`, exactly the
table at `SN/docs/contracts/frontend-execution-v3.md:98-100`. Schema-version accessors exist on the
document (`execution.rs:119, 132, 145`) so receipts never infer the algorithm from a hash value.

Field order — verified element-by-element against the doc (`:102-108`):
- item: ItemId, family, authority, determinism, evidence, effect, policy-family, OPT relation, OPT
  composition (`execution.rs:1933-1948`);
- presence framing: absent `U8(0)`, present payload begins `U8(1)` with "no second presence byte" —
  `put_relation` opens with `put_u8(out, 1)` (`execution.rs:2247-2249`), `put_composition` likewise
  (`:2024`), matching doc `:106`;
- node order: family, OPT join-mode, stage key, OPT source, schema, predicates, expressions,
  correlations, partition inputs, children (`execution.rs:2258-2315` = doc `:104`);
- graph query order: kind, role-tagged targets, depth, OPT direction, summary facets, relation shape,
  dedup, max witnesses, OPT max path length, tie-break (`execution.rs:2097-2128` = doc `:106`);
- qualification binds source id + both raw 32-byte legs (`execution.rs:1956-1967` = doc `:100`).

Framing primitives: `put_str`/`put_usize` emit `U64BE` lengths (`execution.rs:2637-2645`), `put_u32`/`put_u8`
fixed width (`:2646-2650`); SHA-256 via the pinned `sha2 = "=0.10.9"`
(`SN/crates/quanta-gqlang-v2-pipeline/Cargo.toml:22`); all collections are encoded in structural
(source/identity/owner-stable) order, never hash order (`execution.rs:1900-1906`).

Digest ownership: all three digests are computed inside the single `pub(crate)` constructor
(`execution.rs:78-91`); inputs are exactly `(source_id, raw source bytes, execution items)` for
contract/source respectively, and (source_id, source_digest, contract_digest) for qualification —
no runtime ID, provider, generation, lease, or budget enters (doc `:85`; encoder has no such inputs
statically).

### 4.2 Findings

**E2-01 — STRUCTURAL_RISK P2: golden-vector provenance is derived, not independent, at the whole-stream level.**
Violated invariant: "Digests: ... golden-vector provenance (independent oracle vs derived)" — the audit's
own evidence bar, and the repo invariant that deterministic encoding is *proven*, not merely repeated.
Why it applies: the TSV consumer test compiles each row through the same encoder under test and compares
to the TSV (`SN/crates/quanta-gqlang-v2-pipeline/tests/v3_compiled_document_assembly.rs:2001-2052`), and
the external consumer replays only two composition rows from the *same TSV*
(`SN/tools/frontend_external_consumer/src/main.rs:313-392`, whose comment admits "this fixture never
implements a digest encoder"). The TSV's expected values were necessarily produced by this encoder
("regenerated shared vectors", `SN/docs/language/test_hellgate_ledger.md:82`). Independent evidence
exists but is partial: the published SHA-256 empty vector (`execution.rs:2677-2681`), an independent
test-only graph-substream encoder calling no production writer (`execution.rs:3119+`, ledger `:82`),
and per-field mutation rails (`execution.rs:2667+`). Trigger: a co-drift where an encoder field-order
change is accompanied by TSV regeneration (an ordinary refactor). Expected: an out-of-band oracle
notices the wire format changed. Actual: every gate passes; downstream receipts silently break.
Reproduction: static (unexecuted) — the only whole-stream expected values in the repo trace back to
the encoder itself. Minimal root fix (owner: pipeline tooling): add a small out-of-tree oracle script
(e.g. Python, mirroring doc `:96-108`) that recomputes ≥1 full TSV row from raw bytes, checked into
`tools/` and run by `just ci`; record in the TSV header which rows are oracle-verified. Compatibility:
none (test-only). Regression oracle: mutate one field order in `put_node` without touching the oracle —
the oracle row must fail. Refutation attempt: the field-mutation rails already catch single-field
omission, and the graph substream has a true second encoder; but no whole-document second
implementation exists for the relation/canonical path, so the refutation fails for the top-level
stream — finding stands at P2.

**E2-14 — DOCUMENTATION_DRIFT P3 (minor): contract doc does not state TSV derivation provenance.**
`SN/docs/contracts/frontend-execution-v3.md:174` describes what the TSV pins but not that expected
digests are producer-derived; `docs/misc/checklist-scenariolist-09-25.md:1376` calls the TSV "shared
producer/consumer whole-document vectors" without an independence claim. A reader can mistake the TSV
for an independent oracle. Fix (owner: docs/contracts): one sentence stating provenance and the
independence boundary (empty-vector, graph substream, mutation rails). Pairs with E2-01.

Refuted encoding candidates (no finding):
- `put_usize` saturates at `u64::MAX` (`execution.rs:2641-2645`) → theoretical length collision.
  Refuted: inputs are bounded by the checked transaction (source bytes ≤ u32 budget, collections
  bounded); `u64::MAX` is unreachable, and the comment documents the totality intent.
- `hex_lower`'s `unwrap_or('?')` (`execution.rs:57-62`) could emit `?`. Refuted: nibbles are 0–15,
  `char::from_digit(_, 16)` is total there.
- Contract digest ignores non-plan items/diagnostics → under-coverage. Refuted: by contract, the
  document *is* only plan-required items (`:19`); source bytes are covered by the source digest and
  bound by the qualification digest, so nothing executable is uncovered.

---

## 5. Raw ID vs transaction-local scope (Task 5)

Facade APIs do **not** conflate ID match with provenance; the conflation risk is documented and
typed at every touchpoint:

- `ExecutionDocumentV3::item` doc: "Document-local exact-key lookup; the ID alone is not provenance
  proof" (`SN/crates/quanta-gqlang-v2-pipeline/src/execution.rs:106-108`); the contract doc devotes a
  full paragraph to transaction-local identity, digest≠provenance, and the admission owner's binding
  duty (`SN/docs/contracts/frontend-execution-v3.md:17,19`).
- The consumer demonstrates the intended discipline: same source under a different `SourceId` keeps
  the source digest but changes contract/qualification digests, and a foreign item ID misses
  (`SN/tools/frontend_external_consumer/src/main.rs:1219-1241`).

**E2-07 — PROOF_GAP P3: the same-raw-ID cross-document hit is documented but not pinned at the facade.**
The contract states "a separately compiled document may reuse the same transaction-local raw ID" and
lookup "is not rejected" (`SN/docs/contracts/frontend-execution-v3.md:19`). The facade consumer only
pins the *miss* case (different source id, `main.rs:1239-1241`). The positive same-(source_id,item_id)
cross-document `Some` is pinned only in pipeline owner tests (ledger `:82`: "a reused raw ID is not
provenance proof"). Trigger: producer changes `item()` to also validate document identity; expected:
facade consumers would see behavior change; actual: no facade-level test notices. Fix (owner:
tools/frontend_external_consumer): compile the same source twice under the *same* `SourceId` and pin
that lookup of document A's item ID in document B returns `Some` — documenting, at the seam, that this
is exactly why admission must bind provenance itself. Refutation attempt: is this a defect? No — the
behavior is the documented contract; only the proof at the seam is missing.

No facade API returns an item by *name*, and digests are never offered as lookup keys — no
ID/digest-as-provenance conflation found.

---

## 6. CLI exit codes / streams; diagnostics transport fidelity (Task 6)

### 6.1 CLI — CONFIRMED disciplined (no defect)

Static map from `SN/crates/quanta-gqlang-v2-cli/src/main.rs`:
- exit 0: `help`/`--help` as sole argument (usage to **stdout**, `:78-82`), successful `fmt` (`:121-131`),
  `query` with `AnalysisStatusV2::Complete` (`SN/crates/quanta-gqlang-v2-cli/src/commands/v3_query.rs:30-33`);
- exit 1: budget/configuration failure (`:85-92`), io errors (`:47-53`), command errors (`:113-117, 125-129`),
  `query` Incomplete/Error (`v3_query.rs:30-33`) — with the analysis body on **stdout** and empty stderr;
- exit 2: all usage errors (`:40-46`), including over-budget inline/file input (`:232-236, 268-273`).
All error text is line-value encoded (`SN/crates/quanta-gqlang-v2-cli/src/line_value.rs:7-27`), so
untrusted messages cannot forge the line protocol. Binary tests pin stdout/stderr emptiness and exact
codes (`SN/crates/quanta-gqlang-v2-cli/tests/ingress_matrix.rs:63-170`,
`open_analysis_contract.rs:49-116`). No defect found.

### 6.2 Diagnostics transport — CONFIRMED faithful, one soft coupling

`WireDiagnosticV2` carries `stage`, `code`, `severity`, `message`, `source_id`, `span_start`,
`span_end` (`SN/crates/quanta-gqlang-v2-diagnostics-stdio/src/server.rs:33-41`; projection at
`handlers/v3_diagnostics.rs:74-89`), asserted by `ndjson_contract.rs:256-269`. Typed transaction
errors are not flattened to bare strings: document failures keep the full diagnostic list; authority
errors keep `code` + message as a structured `authority_error` protocol error
(`handlers/v3_diagnostics.rs:60-66`). Session/status discipline: per-line errors set status 1 without
poisoning the stream, bounded frames/responses, explicit flush per exchange
(`SN/crates/quanta-gqlang-v2-diagnostics-stdio/src/lib.rs:56-135`).

**E2-03 — STRUCTURAL_RISK P3: transport string-couples to a pipeline diagnostic code.**
`handlers/v3_diagnostics.rs:45-53` special-cases the literal string
`"pipeline.analysis.diagnostic_limit_exceeded"` to surface the producer's own limit failure.
Why it applies: the repo treats diagnostic codes as stable contract strings, but the facade already
demonstrates the better pattern — `OPEN_AUTHORITY_INCOMPLETE_CODE` is exported as a constant
(`SN/crates/quanta-gqlang-v2-pipeline/src/compile.rs:82`, re-exported at facade `lib.rs:42`).
Trigger: pipeline renames the code. Expected: transport fails loudly at compile time. Actual: silent
degradation to the generic `enforce_diagnostic_cap` path — which is behavior-preserving (the cap is
re-derived from the budget, `handlers/v3_diagnostics.rs:69-85`), so the failure mode is soft, hence P3
not P2. Minimal root fix (owner: pipeline/facade): export the constant and consume it in the handler.
Compatibility: additive. Regression oracle: rename the constant's value; the handler must fail to
compile. Refutation attempt: the string also appears in tests? Grep shows only this one runtime use
plus pipeline internals — the coupling is single-site and cheap to fix; finding stands as risk, not defect.

---

## 7. Conformance and the external-consumer gate (Task 7)

### 7.1 Independence — CONFIRMED executable expectation, not copied rules

The conformance crate stores *expectations* (family → keyword/source/disposition) as literals
(`SN/crates/quanta-gqlang-v2-conformance/src/lib.rs:46-160`), and
`SN/crates/quanta-gqlang-v2-conformance/tests/self_validation.rs` executes every spec through the
facade (`analyze_source_document_v2`, `compile_source_document_v3`) and checks real dispositions,
including `NoPlan` reasons, plan kinds, rule-program stratification, and unsupported-source rejection
(`UNSUPPORTED_SOURCE_V2`). It is not a copy of producer internals; drift fails the test rather than
propagating. The python gate is likewise an independent structural oracle: manifest topology, facade
allowlist diff against `cargo public-api`, negative-compile evidence attribution, and git-cleanliness
fingerprinting (`SN/tools/test_frontend_external_consumer.py` throughout).

### 7.2 Rejection paths — mostly exercised

Exercised at the facade/seam: classify `NoItem`/`MultipleItems`; closed-authority `NoPlan
MetaActivation`; foreign-item lookup miss; `FrontendFailureV2::try_new(vec![])` rejection; empty-id /
wrong-method / unknown-field / oversized-frame / oversized-source transport rejections
(`server_rejection_matrix.rs:19-103`, `ndjson_contract.rs:310-326`,
`resource_boundaries.rs`); CLI usage/io/utf8/budget rejections (`ingress_matrix.rs:116-170`).
Gaps: E2-04 (unsupported-family classify rejection) and E2-13/E2-07 above.

### 7.3 Fixture drift risks — CONFIRMED fail-closed

- Missing target: the consumer panics `missing shared vector: {case}` if a pinned row disappears
  (`main.rs:356-358`); row shape is validated (`:359-364`).
- Same-count drift: the pipeline TSV test asserts exactly 15 rows
  (`v3_compiled_document_assembly.rs:2049`, "the shared V3 corpus must not be vacuous"), and the
  consumer asserts exactly two selected composition vectors (`main.rs:390-392`) plus exact
  case/profile/source-id/source text per row (`:365-371`). Any TSV edit trips a hard failure — drift
  cannot pass silently; the cost is coupling (a deliberate TSV extension requires touching the consumer
  count), which is the intended trade.
- "Empty scan": no scan-emptiness fixture lives at this seam (scan payloads are producer-lane
  concerns); the nearest analogues — `sampled` producer schema assertions (`main.rs:242-314`) — pin
  exact non-empty shapes. No drift vector found here.

---

## 8. Consolidated findings

| ID | Class | Sev | One-liner | Owner |
| --- | --- | --- | --- | --- |
| E2-01 | STRUCTURAL_RISK | P2 | Whole-stream golden vectors are producer-derived; no independent whole-document digest oracle | pipeline tooling |
| E2-02 | STRUCTURAL_RISK | P3 | Facade-only ingress enforced by convention + one-fixture gate, not structurally | repo policy rail |
| E2-03 | STRUCTURAL_RISK | P3 | stdio handler hard-codes `pipeline.analysis.diagnostic_limit_exceeded` string | pipeline/facade + stdio |
| E2-04 | PROOF_GAP | P3 | Documented classify-reject-unsupported-family consumer pattern never exercised | tools/frontend_external_consumer |
| E2-06 | OPTIONAL_REFACTOR | P3 | `FrontendDiagnosticV2::new`/`FrontendFailureV2` constructors publicly fabricable | sema/facade |
| E2-07 | PROOF_GAP | P3 | Same-raw-ID cross-document lookup `Some` not pinned at the facade seam | tools/frontend_external_consumer |
| E2-13 | PROOF_GAP | P3 | "Supported family, no standalone payload" not asserted by the facade consumer | tools/frontend_external_consumer |
| E2-14 | DOCUMENTATION_DRIFT | P3 | Contract doc omits TSV digest derivation provenance | docs/contracts |

Confirmed-clean (no finding): ingress DAG (§1.1), parse-only discipline (§1.2), curated closed surface +
both-direction public-api pinning (§2.1), `ExecutionDocumentV3`/digest/finite-float unforgeability with
compile-fail probes (§2.2-2.3), encoder-vs-doc byte-order parity incl. domain/schema separation and
presence framing (§4.1), digest ownership/stability inputs (§4.1), documented-not-defect plan-required
gaps (§3), ID≠provenance documentation (§5), CLI exit/stream discipline (§6.1), wire diagnostic fidelity
(§6.2), conformance independence and fail-closed fixture coupling (§7).

## 9. Refuted candidates

1. `put_usize` `u64::MAX` saturation length-collision — unreachable under bounded transactions (§4.2).
2. `hex_lower` `?` fallback — total for base-16 nibbles (§4.2).
3. Contract digest "misses" non-plan items — by-contract coverage via source+qualification legs (§4.2).
4. Transport loses span/status — disproven, `ndjson_contract.rs:256-269` pins all six fields.
5. CLI `query` exit 1 with stdout body is a stream-discipline defect — intended open-analysis contract,
   pinned by `open_analysis_contract.rs:49-58` (stdout body, empty stderr).
6. `help` size-hint probe fragility in `main.rs:78-82` — `ArgsOs` size hint is exact over remaining
   argc and the behavior is binary-tested (`ingress_matrix.rs:105-114`).
7. Facade "hides" `compile_resolved_document_v2`/`CompiledDocumentV2` drift — intentional and documented
   (`frontend-execution-v3.md:7`; pipeline lib.rs comment "the facade exports only the source entry
   points"); the public-api allowlist pins the exclusion.
8. Consumer replaying only 2 of 15 TSV rows is under-coverage of the facade compile path — refuted as a
   *defect*: the full corpus is pinned at the producer, and the consumer's replay exists to prove
   facade reachability of shared vectors, which two rows do establish (residual risk folded into E2-01).

## 10. Limitations

Static audit only; no builds, tests, or the python gate were executed (read-only mandate). Rust privacy
and surface claims rest on source inspection plus the repo's own gated evidence (negative-compile
attribution, public-api diff), not on a fresh compile. `SemanticItemIdV2`/`SourceIdV2` internals live in
the sema crate (outside this lane's file list) and were consulted only via the pipeline's encoding and
doc statements. Dirty-file state (`SN/git-status.txt`: modified `cli/src/main.rs`,
`pipeline/src/compile.rs`, surface format files) is included in the audited snapshot; findings cite the
dirty-state lines as-is.
