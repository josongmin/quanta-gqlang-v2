# Lane E — Pipeline assembly / V3 DTO / facade / front doors (integrated, round 2)

Audit structure — **two parallel rounds, two tracks each**:
- Round 1: E-1 (assembly/projection, inline after subagent failures ×2) + E-2 (facade/front doors, subagent).
- Round 2: E-3 (deep dive: diagnostic.rs + remaining execution.rs + digest field-coverage matrix + panic audit, subagent) + E-4 (adversarial refutation of all round-1 findings + test-inventory sweep, subagent).

Sub-reports: `lane-E1-assembly-projection.md`, `lane-E2-facade-front-doors.md`, `lane-E3-deep-dive.md`, `lane-E4-adversarial.md`, plus `projection-map.md` / `projection-input-E2.md`.

## Snapshot / boundary

- Shared snapshot `docs/audits/lane-b-snapshot-20260927-010019`, HEAD `eaeb68ea2711d411616511713fb016d828cd80be` (branch `main`), dirty tracked+untracked included, sha256-verified fresh for lane-E files. `pipeline/src/compile.rs`, `cli/src/main.rs` dirty — all conclusions snapshot-scoped.
- Static audit only across all four tracks; no builds/tests executed (recorded limitation).

## Consolidated findings after round-2 adversarial refutation

No P0/P1, no CONFIRMED_DEFECT.

### Surviving

| ID | Sev | Class | Summary | Owner |
|---|---|---|---|---|
| **E2-01** (≡E3-8) | **P2** | STRUCTURAL_RISK | No independent whole-stream digest oracle exists anywhere in `tools/` (run_local.py's sha256 is tree-cleanliness only; no second encoder references the `qglang.execution`/`contract.v5` domains). Field-mutation rails miss item_id, family tag, root stage key, children order, top-level item order — coverage hole reinforces: encoder+TSV co-drift passes all gates. | pipeline tooling |
| E1-01 | P3 | DOCUMENTATION_DRIFT | `ExecutionFamilyV3::RuleProgram` variant + digest tag 6 + doc vocab row never constructed; no reserved-vocabulary statement in any doc; rule tests pin absence, not reachability. | pipeline/docs |
| E2-04 | P3 | PROOF_GAP | No seam classifies a parsable-but-unsupported root (Rule/Declaration/Catalog) and takes the documented pre-authority rejection branch. | tests |
| E3-1 | P3 | PROOF_GAP | Algebra/planning bridge-code parity with owner `code()` unasserted (logic asserts parity, algebra/planning assert literals only); diagnostic.rs:26-27 module doc overclaims. | pipeline tests |
| E3-2 | P3 | DOC_DRIFT/RISK | `pipeline.analysis.*` codes minted in compile.rs (consts 82/91/92 + inline literals 940/945/950) with no closed enum; diagnostic.rs:15-16 wrongly claims ownership. | pipeline |
| E3-3 | P3 | DOC_DRIFT | `PipelineStageV2` exported but never constructed on production paths; prefix-derivability claim wrong for the two planning→logic relays. | pipeline |
| E3-4 | P3 | DOC_DRIFT | `canonicalize_diagnostics` adds full-struct dedup Sema's canonicalization lacks; comment attributes it to "canonical Sema order"; equal-key ties keep insertion order (deterministic, undocumented). | pipeline docs |
| E3-5 | P3 | DOC_DRIFT | `empty_blocked_failure` reuses `missing_item_result` code for an unrelated impossible condition. | pipeline |
| E3-6 | P3 | STRUCTURAL_RISK/PROOF_GAP | `"invalid_closed_builtin"`/`"invalid_closed_operator"` sentinel fallbacks; builtin name is the only digest-encoded identity → invalid ids would silently collide instead of failing closed; None arms untested. | pipeline |
| E3-7 | P3 | OPTIONAL_REFACTOR | Dual encoding of `ExecutionFamilyV3` (tag byte vs keyword); vestigial constant leading tags. No collision today. | pipeline (optional) |
| E1-02 | P3→narrowed | DOC_DRIFT (rustdoc only) | Public contract doc (:99) already exact; overclaim lives only in rustdoc execution.rs:136-138. Optional comment clarification. | pipeline |
| E1-03 | P3 | OPTIONAL_REFACTOR | `format_source_document_v2` double-parse; bounded, authority-correct — cost only. | surface (optional) |
| E2-14 | P3 | DOCUMENTATION_DRIFT | Contract doc omits TSV digest derivation provenance (pairs with E2-01). | docs |
| E2-02 | downgraded | intended-limitation note | Internal ingress discipline REFUTED as a risk: `policy.toml` exact internal-edge allowlist + `check_architecture.py` wired into `just ci`. Residual (external sibling repos depending on the published pipeline crate) is out of repo scope; fixture gate models the sanctioned pattern. | — |

### Refuted outright (with refuting evidence)

- **E2-03** — hard-coded `"pipeline.analysis.diagnostic_limit_exceeded"`: string matches compile.rs:92 exactly; `resource_boundaries.rs:200-226` behaviorally pins the branch (a rename flips the response and fails the rail). Residual: use the exported constant (style nit).
- **E2-07** — cross-document same-raw-ID `item()` hit: `frontend_external_consumer/src/main.rs:385-402` already pins it. Round-1 miss.
- **E2-13** — supported-family-no-payload: `conformance/tests/self_validation.rs:252-307` asserts `relation()`/`composition()` presence === family across the full registry; NoPlan families assert empty execution docs.
- **E1-04** — diagnostic-cap replacement: pinned by `v3_analysis_document_assembly.rs:61-83` (single diagnostic, exact code, source anchor) + stdio rail; contract doc has no diagnostic claims. Residual: one optional doc sentence.

## Confirmed clean (cumulative, all rounds)

- **Ingress DAG**: cli/stdio/conformance/external-consumer → frontend-contract → pipeline only; enforced internally by the architecture policy gate (`policy.toml` + `check_architecture.py`, in `just ci`); no globs; parse-only paths demand no semantics.
- **Exact-key assembly**: missing/extra/duplicate/foreign/wrong-kind fail closed with typed `DocumentAssemblyCodeV2`; blocked stages reported before membership comparison; no silent drop/partial success.
- **Projection honesty**: pure mappings from sealed owners; i128::MIN, float bits (±0 canonicalized at types owner, ty.rs:730-735), decoded strings preserved; expression closure source-scoped, fail-closed; composition projections length/kind-checked.
- **Digest field coverage — COMPLETE** (E-3 §3 matrix): every field of all 9 public DTO structs encoded exactly once in `digest_contract`; `Unknown` variant included (tag 7). Zero coverage gaps (rule program exclusion is by-design, documented).
- **Determinism**: no HashMap/HashSet in pipeline production code; BTreeSet/BTreeMap only; stable sorts; domain+schema separation (3/5/5); length framing; one tag byte per variant.
- **No production panics**: all 95 unwrap/expect/panic/indexing matches are `#[cfg(test)]`-gated.
- **Unforgeability**: private fields, `pub(crate)` constructors deriving all digests, 15 gated compile-fail probes.
- **Transport fidelity**: typed errors keep stable codes; Display used for message text only; truncation bounded at the Sema constructor.
- **ID discipline**: raw ID ≠ provenance documented and now *tested* at the facade (E2-07 refutation).

## Handoffs

- Lane D: rule-graph allowance single-charge at `RuleProgramContractV2::from_document`.
- Lane C: `AnalysisStatusV2::Complete`-without-execution-document; open-authority unresolved-fact vocabulary.
- Lane F (coverage-matrix inputs): E2-01 (whole-stream oracle absence + mutation-rail coverage holes: item_id/family tag/stage key/children order/item order), E3-1 (bridge-code parity), E3-6 (sentinel fallback arms untested), E4's rejection-oracle matrix over the four pipeline test files (`lane-E4-adversarial.md`).
- Coordinator: E3-2/E3-3 join E1-01 in a "dead/unowned public vocabulary" cluster — same root pattern (exported or documented vocabulary without a production producer or closed enum), separate regression oracles.
- Cross-lane A: `CompiledDocumentV2` vs `ExecutionDocumentV3` classification = legitimate projection (pure mapping evidence, E-1).

## Verdict

**증거 범위 내 일관됨 (consistent within evidence scope)**: no confirmed defects; 1 P2 structural risk (independent digest-oracle absence, now adversarially reinforced), a P3 cluster of documentation/proof gaps concentrated in diagnostic-code vocabulary and test-rail coverage. Static-only, snapshot-scoped; round-2 refutation removed 4 round-1 findings and downgraded 1 — the surviving set survived explicit refutation attempts with named guards/tests checked.
