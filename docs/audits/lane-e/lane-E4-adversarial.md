# Lane E-4 — Adversarial verification round (refutation pass)

Audit type: static, read-only. No implementation changes; no builds or tests executed.
Source boundary (fixed, untouched): `docs/audits/lane-b-snapshot-20260927-010019` @ HEAD
`eaeb68ea2711d411616511713fb016d828cd80be`. `SN/` below abbreviates the snapshot root.
Mission: attempt to REFUTE each surviving E1/E2 finding by locating guards, tests, rails, or
intended-limitation documentation the prior rounds missed. A finding survives only if unrefuted.

---

## 1. Per-target verdicts

### E2-01 (P2) — golden vectors producer-derived → **SURVIVES (P2 unchanged)**

Refutation attempts, all executed:

1. **Independent digest oracle in tooling?** None. `rg "qglang.execution|contract.v5|source-bytes"`
   over `SN/tools/**` and `SN/docs/**` (non-audit) returns zero hits — no Python or other
   out-of-tree reimplementation of the encoder exists. `SN/tools/ci/run_local.py:50-98` computes a
   SHA-256, but only as a workspace-tree cleanliness fingerprint (names, mtimes, file bytes), not a
   digest oracle. `SN/tools/lint/` (semantic_honesty.py, semgrep_gate.py) contains no TSV/digest
   logic. `SN/tools/ci/run_local.py:117` compares evidence hashes, again tree fingerprints.
2. **Property tests?** The only proptest deps are surface/pipeline fuzz-adjacent
   (`SN/tools/architecture/policy.toml:101-105`); no proptest touches the digest stream.
3. **Cross-checks that would catch encoder+TSV co-drift?** The TSV test itself compiles each row
   through the encoder under test (`SN/crates/quanta-gqlang-v2-pipeline/tests/v3_compiled_document_assembly.rs:1999-2047`);
   the external consumer replays rows from the same TSV
   (`SN/tools/frontend_external_consumer/src/main.rs:360-392`). The ledger confirms provenance:
   "byte-incompatible contract and qualification streams use schema/domain 5 with **regenerated
   shared vectors**" (`SN/docs/language/test_hellgate_ledger.md:82`). The three independent
   evidences remain partial exactly as E2-2 §4.2 said: published empty-vector SHA-256
   (`SN/crates/quanta-gqlang-v2-pipeline/src/execution.rs:2672-2676`), the test-only graph-substream
   encoder (`execution.rs:3108-3277`, calls no production writer), and per-field mutation rails.
4. **Do the field-mutation rails mutate every DTO field?** No — a subset, though a broad one.
   Covered (all `execution.rs` unit tests): predicates (`:3297`), partition_input (`:3309`),
   correlations (`:3326`), schema fields/cardinality/partition-keys/order (`:3346`),
   expression values/shapes (`:3371`, `:3892` — call target/args, quantifier kind/binders/body,
   exists polarity/root/correlations), composition binding schema (`:3384`), sample scope + every
   graph-query field (`:3394`), binding identity and order (`:3547`), body_relation (`:3581`),
   plan authority/effect/evidence (`:3597`), determinism/policy family (`:3801`), source bytes
   (`:3615`), join modes / source authorities+kinds / body arms / sample modes via
   distinct-digest sets (`:3989-4057`). **Not covered by any mutation rail:** item-level
   `item_id`, the item-level `family` tag (only tag *uniqueness* is pinned, `:2758-2766`),
   the root `stage` NodeKey, children count/order, top-level item-order permutation, and
   `with_binding`/`body_item_id`/`body_family` (only `producer_item_id` at `:3568`).
   So even the "any DTO change moves it" rail coverage has holes — reinforces, not refutes.

Net: no whole-document second implementation exists for the relation/canonical path; co-drift of
encoder + TSV regeneration passes every gate. SURVIVES at P2. (E2-14, the doc-provenance sentence,
also survives unchanged — contract doc `:174` still omits derivation provenance.)

### E1-01 (P3) — `ExecutionFamilyV3::RuleProgram` dead variant → **SURVIVES (P3 unchanged)**

Refutation attempts:

1. **Tests asserting RuleProgram items?** The only non-encoder uses of the variant are the
   keyword/tag arms (`SN/crates/quanta-gqlang-v2-pipeline/src/execution.rs:173,1978`), the
   tag-distinctness rail (`:2765`), and a *panic* arm classifying it as "unexpected authored
   producer family" (`SN/crates/quanta-gqlang-v2-pipeline/tests/v3_compiled_document_assembly.rs:2396`).
   The rule-program tests (`v3_compiled_document_assembly.rs:590-695`) assert rule members compile
   to `PlannedItemV2::RuleMember` with `item.plan().is_none()` — i.e. they pin the *absence* of
   rule-program execution items, never the variant's presence. No reachability rail exists.
2. **Reserved-vocabulary documentation?** `rg "reserved|Reserved"` over `SN/docs/contracts/` and
   `SN/docs/language/{README,release_discipline}.md`: zero hits. The contract doc lists RuleProgram
   in the item-family tag table (`SN/docs/contracts/frontend-execution-v3.md:114`) with no
   reserved/unreachable marking, while `:19` states rule members and the rule program "are not V3
   execution items". The table's preamble ("Within each tag family below, the first listed variant
   has tag 0…") frames it as an encoding map, which is a partial mitigation — but a reader of the
   table alone still cannot tell tag 6 is dead encoding space.

SURVIVES as DOCUMENTATION_DRIFT P3 (annotate-as-reserved or remove at next schema bump).

### E1-02 (P3) — rule program outside `contract_digest` → **SURVIVES, scope narrowed (doc-comment only)**

Refutation found in the *public* contract: the normative encoding table defines
`contract_digest` as exactly `… || SEQ(source-ordered execution items)`
(`SN/docs/contracts/frontend-execution-v3.md:99`), and `:19` explicitly excludes the rule program
from the execution document. The public doc therefore never overclaims coverage. The overstatement
lives only in the producer-internal rustdoc on `qualification_digest`: "Any DTO or source-byte
change moves it" (`SN/crates/quanta-gqlang-v2-pipeline/src/execution.rs:136-138`) — and even that
is technically true of the `ExecutionDocumentV3` DTO it annotates, since the rule program is not a
field of that DTO (`compile.rs:403-436`). The rule-program plan *is* digest-bound elsewhere only
via source bytes, and its proof is pinned byte-for-byte by
`v3_compiled_document_assembly.rs:637-695` (single-ingress stratification, recompute oracle).
Verdict: SURVIVES only as an optional one-sentence clarification of the code doc-comment; the
public contract is already precise. Effectively at the refuted boundary — recommend the
coordinator record it as DOCUMENTATION_DRIFT P3 (code comment) or drop it.

### E2-02 (P3) — facade-only ingress not structural → **DOWNGRADE: internal case REFUTED; external-sibling residual only**

The prior round missed an entire enforcement layer:

- `SN/tools/architecture/policy.toml:41-68` enumerates the **exact** allowed internal normal edges;
  the only edge into the pipeline is `quanta-gqlang-v2-frontend-contract -> quanta-gqlang-v2-pipeline`
  (`:48`). `SN/tools/architecture/check_architecture.py:477-485` fails on any internal edge not in
  the allowlist (`illegal normal/dev/build internal edges`), and membership itself is pinned
  (`:457-462`), so a new workspace member or any sibling crate adding a pipeline dependency fails
  the gate. Kind confusion is also closed (an edge allowed as dev is illegal as normal,
  `policy.toml:30-40`; build edges default-deny, `:72`). The gate is wired into `just ci`
  (`SN/Justfile:48`) with its own test suite (`Justfile:47`, `tools/architecture/tests`).
- Additionally `check_architecture.py:78-83,185-194` forbids `quanta-runtime/sdk/index/provider`
  prefixes and legacy `quanta-gqlang-` crates outright.

So the finding's trigger "a future workspace member adds a `quanta-gqlang-v2-pipeline`
dependency" is **structurally impossible** — refuted. What remains is only the out-of-repo case:
an external sibling repo depending on the pipeline crate directly, where no repo-local rail can
apply by construction; the python fixture gate (`SN/tools/test_frontend_external_consumer.py:374-392,622`)
models the sanctioned external pattern. Residual is also mitigated by path-only internal deps
(`check_architecture.py:226-237` requires the exact member path, no registry source), so consuming
the pipeline externally requires deliberately copying/forking it. Recommend reclassify:
STRUCTURAL_RISK P3 → intended-limitation note (external-repo scope), no repo-internal action.

### E2-03 (P3) — hard-coded `"pipeline.analysis.diagnostic_limit_exceeded"` → **REFUTED (behavior pinned; residual is style only)**

The exact string matches the pipeline constant: `SN/crates/quanta-gqlang-v2-pipeline/src/compile.rs:92`
(`const ANALYSIS_DIAGNOSTIC_LIMIT_CODE: &str = "pipeline.analysis.diagnostic_limit_exceeded";`) vs
`SN/crates/quanta-gqlang-v2-diagnostics-stdio/src/handlers/v3_diagnostics.rs:35`. More
importantly, the prior round's failure-mode claim ("silent degradation") is wrong — there is a
behavioral oracle on the special-case branch:

- `SN/crates/quanta-gqlang-v2-diagnostics-stdio/tests/resource_boundaries.rs:200-226`
  (`diagnostic_and_response_caps_fail_closed`) dispatches a real `v3_diagnostics` request with
  `max_diagnostics = 2` on `"show capabilities"`, which makes the *pipeline* emit its own
  `pipeline.analysis.diagnostic_limit_exceeded` failure (the same source produces 3 diagnostics
  under the same budget shape — pinned at
  `SN/crates/quanta-gqlang-v2-pipeline/tests/v3_analysis_document_assembly.rs:61-83`), and asserts
  the transport answers with `error.code == "diagnostic_limit_exceeded"`
  (`resource_boundaries.rs:224`). If the pipeline code were renamed, the handler's special case
  would stop firing, `enforce_diagnostic_cap` (1 ≤ 2) would admit the failure, and the response
  would become an `ok:true` analysis payload instead of the error object — the test fails loudly.
- The pipeline side of the literal is itself pinned at
  `v3_analysis_document_assembly.rs:77-80` (exact code string + source anchor).

So co-drift is caught by two existing rails; the only surviving content is "use the exported
constant instead of a literal" (the facade already demonstrates the pattern with
`OPEN_AUTHORITY_INCOMPLETE_CODE`, `compile.rs:82`). That is a style/maintainability nit, not a
risk. REFUTED as STRUCTURAL_RISK; residual OPTIONAL_REFACTOR at most.

### E2-04 (P3) — classify-reject pattern unexercised → **SURVIVES (P3)**

Searched `classify_single_source_root_v3` across all seams: pipeline unit tests cover positive
families (Graph/Presentation/Composition/Relation/…) and the `NoItem`/`MultipleItems`/`Frontend(_)`
errors (`SN/crates/quanta-gqlang-v2-pipeline/src/compile.rs:1955-2006`); the external consumer
covers Graph/Composition positives plus `NoItem`/`MultipleItems`
(`SN/tools/frontend_external_consumer/src/main.rs:1041-1065`); the conformance crate never calls
classify (its `RootFamilyV2` is a separate V2 registry, `SN/crates/quanta-gqlang-v2-conformance/src/lib.rs:7`).
No test at any seam classifies a parsable-but-unsupported root (e.g. `Rule`, `Declaration`,
`Catalog` — variants of `SourceRootFamilyV3`, `SN/crates/quanta-gqlang-v2-surface/src/parser.rs:155-180`)
and takes the documented pre-authority rejection branch (`frontend-execution-v3.md:12-15`).
Classify correctness is pinned; the *consumer rejection pattern* is not. SURVIVES as PROOF_GAP P3.

### E2-07 (P3) — cross-document same-raw-ID `item()` hit not pinned at facade → **REFUTED**

The prior round's evidence was incomplete. `SN/tools/frontend_external_consumer/src/main.rs:385-402`:
the consumer compiles **two distinct shared-vector documents** (two TSV rows, both source id 9,
different sources), takes `reused_item` from `second`'s execution document, asserts `first`'s item
carries the identical raw item ID (`:392`), that the qualification digests differ (`:393-396`),
and then pins the positive cross-document hit:

```rust
assert!(
    first.execution().item(reused_item).is_some(),
    "raw item ID lookup is not cross-document provenance admission"
);
```

(`main.rs:400-402`). That is exactly the assertion E2-2 §5 asked for: same-(source_id, item_id)
lookup across two independently compiled documents returns `Some`, with a message documenting why
admission must bind provenance itself. (The `main.rs:1219-1241` foreign-miss case is additional,
not the only pin.) REFUTED.

### E2-13 (P3) — supported-family-no-payload not asserted at facade → **REFUTED**

`SN/crates/quanta-gqlang-v2-conformance/tests/self_validation.rs:252-307` compiles **every family
registry spec** (Graph, Check, Presentation, MetaRequireCertified, Composition, Relation) through
the facade (conformance depends only on `quanta-gqlang-v2-frontend-contract`,
`SN/crates/quanta-gqlang-v2-conformance/Cargo.toml`) and asserts, per family:

- `execution_item.family() == expected_family` (`:290-294`),
- `execution_item.relation().is_some() == (expected_family == Relation)` (`:294-299`),
- `execution_item.composition().is_some() == (expected_family == Composition)` (`:299-304`).

For every non-Relation/non-Composition family that is precisely the "supported family, no
standalone payload" property E2-13 claimed was unpinned at the seam — asserted in both directions
(presence for the payload families, absence for the rest), across the full family registry, plus
`NoPlan` families assert an empty execution document (`:268-270`). The external consumer's
`assert_exact_composition_handoff` additionally pins exhaustive composition payloads. REFUTED.

### E1-04 (P3) — `cap_transaction_error` wholesale replacement → **REFUTED as defect (doc-sentence residual only)**

Behavior is pinned by two independent rails the finding did not account for:

- `SN/crates/quanta-gqlang-v2-pipeline/tests/v3_analysis_document_assembly.rs:61-83`
  (`public_open_analysis_enforces_its_diagnostic_cap`): exact-limit admits all diagnostics; one
  over the limit yields a `Document` failure with **exactly one** diagnostic, the exact code
  literal, and the correct `SourceIdV2` anchor (not a fabricated location).
- `SN/crates/quanta-gqlang-v2-diagnostics-stdio/tests/resource_boundaries.rs:200-226`: the
  transport surfaces the capped failure as the typed protocol error (see E2-03 above).

The replacement strategy (`compile.rs:863-884`) is deterministic, fail-closed, bounded, and
source-anchored. The contract doc contains zero occurrences of "diagnostic"
(`rg -c diagnostic SN/docs/contracts/frontend-execution-v3.md` → none), so the surviving residual
is only "document the replace-not-truncate strategy in the contract doc" — a doc sentence, not a
defect or proof gap. Recommend: REFUTED as DOCUMENTATION_DRIFT-risk; keep optional doc note.

---

## 2. NEW SWEEP — `crates/quanta-gqlang-v2-pipeline/tests/` inventory

| File (lines) | Purpose (one line) | Rejection oracles? |
| --- | --- | --- |
| `v3_compiled_document_assembly.rs` (2786) | End-to-end V3 compile assembly: item identity/order, rule program ingress, exact-key membership kills, projections, literal/DTO parity, digest vectors + mutation rails | **Yes, extensive:** forged rule membership (696), duplicate authored identity (724), missing produced item (742), extra produced item (760), foreign authored identity (777), same-name-wrong-id substitution (795), check-graph identity substitution (817), sealed receipt limit+1 (842, 866), duplicate projection alias pre-plan (1293), composition body from another relation (2428), source depth exact-limit+1 (1904), format structural-depth budget (1974, 1986), over-budget source (2466). Digest-side: closure dangling/cross-source fail-closed (`execution.rs:3654,3664`); digest *mutation* rails are move-oracles (assert digests change), not rejection oracles — no digest "rejection" exists by design. |
| `v3_analysis_document_assembly.rs` (441) | Open-analysis V2 assembly: diagnostic cap, partial-authority blocking, status/mask exactness, blocked-rule contagion, open/closed agreement, budget receipt | **Yes:** diagnostic-limit cap code+shape (61-83); overdeep relation before declaration (84-99, `surface.resource_exhausted`); blocked analysis not compilable (364). |
| `v3_composition_source_query.rs` (230) | Surface→HIR→sema adapter sweep: every composition body family and mixed-document semantic item variants | No rejection oracles (positive parity only). |
| `v3_format_budget.rs` (87, untracked-dirty) | Format output/input budget edges | **Yes:** output tokens reject at cap+1 with original location (21), escaped bytes cap+1 (47), input rejection span/alias exactness (74). |

Coverage-matrix feed — assembly/digest codes **with** rejection oracles: exact-key membership
(duplicate/missing/extra/foreign/substitution), rule membership forgery, receipt limits
(plan/expression/depth/diagnostic/format), composition body transplant, closure dangling and
cross-source IDs, over-budget source/frame. **Without** rejection oracles: digest mutation rails
(positive-only by design), composition_source_query (parity-only), and the E2-01 holes
(item_id/family-tag/stage-key/children/item-order mutations untested).

---

## 3. Consolidated verdict table

| Target | Prior | E-4 verdict | Key evidence |
| --- | --- | --- | --- |
| E2-01 | STRUCTURAL_RISK P2 | **SURVIVES P2** | no out-of-tree oracle anywhere in tools/; rails cover a field subset only |
| E1-01 | DOC_DRIFT P3 | **SURVIVES P3** | no reserved-vocab doc, no reachability rail; only panic-arm/test-absence pins |
| E1-02 | PROOF_GAP P3 | **SURVIVES, narrowed** (code doc-comment only) | public doc `:99` already exact; overclaim only at `execution.rs:136-138` |
| E2-02 | STRUCTURAL_RISK P3 | **DOWNGRADE — internal REFUTED** | `policy.toml:41-68` + `check_architecture.py:477-485`, wired at `Justfile:47-48` |
| E2-03 | STRUCTURAL_RISK P3 | **REFUTED** (residual style nit) | `resource_boundaries.rs:200-226` + `v3_analysis_document_assembly.rs:61-83` pin behavior |
| E2-04 | PROOF_GAP P3 | **SURVIVES P3** | no seam classifies an unsupported family and rejects |
| E2-07 | PROOF_GAP P3 | **REFUTED** | `frontend_external_consumer/src/main.rs:385-402` pins cross-document `Some` |
| E2-13 | PROOF_GAP P3 | **REFUTED** | `conformance/tests/self_validation.rs:252-307` asserts payload absence per family via facade |
| E1-04 | DOC_DRIFT P3 | **REFUTED as defect** (optional doc sentence) | behavior pinned by pipeline + stdio rails; contract doc silent |

## 4. Limitations

Static only; no builds/tests run (read-only mandate). The `resource_boundaries.rs` refutation of
E2-03 assumes the stdio server's open authority shares the request budget's `max_diagnostics`
(the helper at `:15-41` builds `FrontendBudgetV2` from exactly those four knobs), consistent with
the pipeline-side pin producing 3 diagnostics for the same source. All line references resolve
inside `SN/`.
