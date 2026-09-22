# Remediation Plan — sep-21 Test Improvement

**Inputs.** `findings-1-duplicated.md` (33 items), `findings-2-duplicated.md` (gates + duplicates),
`test-suite-inventory.tsv` (810 fns at snapshot `f0fc3dce0f672d7d`).

**Status at planning time (2026-09-21 ~03:30): the plan cannot start yet.** See §1.

---

## §1 Blocker — the target is moving and currently red

Measured just now, not assumed:

| Fact | Consequence |
|---|---|
| `cargo nextest` cannot run: `quanta-gqlang-v2-algebra` test binaries fail with **~74 errors** — `HirBuilderV1` no longer has `.path()` / `.compose()` / `.project()` / `.quantifier()` | No test change can be verified until the HIR-builder API change lands or is repaired |
| A **concurrent session is rewriting `quanta-gqlang-v2-modules`** into a v2-authority shape (new `authority.rs`, `signature.rs`, `visibility.rs`, `extension.rs`, `budget.rs`, `version.rs`; 2 new test files) | Any edit in `modules/` now will be rebased away |
| Test count moved **810 → 966 (+156)**; the two new modules test files are not in the inventory | The audit baseline is stale; 156 fns are unaudited |
| **3 findings are already dead**: `collect_view_refs`, `canonical_cycle`, and the manifest `UnknownModule` arm no longer exist in `modules/src/` | M9, M10, M11 are invalidated by the rewrite — re-scope or drop |
| 8 of the 14 files cited in `findings-1-duplicated.md` drifted (`surface/src/family.rs`, `format.rs`, `format_edges.rs`, `binder_mutants.rs`, `modules/src/*`, `planning/item.rs`) | Every `file:LINE` must be re-verified before editing |

**Gate 0 (must pass before Phase 2 onward):**
1. `cargo nextest run --workspace --no-tests pass` is green on a **fixed commit**.
2. The modules-v2 rewrite has landed (no `??`/`M` entries under `crates/quanta-gqlang-v2-modules/`).
3. The concurrent session is not mid-flight on the same files.

If the rewrite runs long, execute only **Phase 1** (touches `Justfile`, `lint.yml`,
`check_architecture.py`, one doc comment — zero overlap with the rewrite).

---

## §2 Execution principles (the do-not list)

1. **Never delete a weak test before its stronger replacement exists and passes.** This is the
   single ordering constraint that shapes every phase below. `findings-2` §5.1 deletions depend on
   additions; `findings-1` A-series strengthening must land first.
2. **Do not reduce `PROPTEST_CASES`** (512). Wall clock was never the problem — 7.3 s for 822
   tests. The waste was the tautology inside two props, not the case count (`findings-1` X10).
3. **Do not remove the 12 `prop_assert!(r.is_ok() || r.is_err())` *calls*.** The call itself is the
   panic-freedom rail; only the assertion line goes. `r` becomes unused — prefix it.
4. **Do not touch the CLI subprocess tests or the `std::mem::discriminant` checks.** nextest's
   1.64 s/report is a scheduling artifact (0.04 s direct, 0.023 s for 16 parallel spawns); the
   discriminant check is genuinely independent of the text assertion.
5. **One PR per batch, each gated on** `just nextest` **and** `cargo clippy --workspace --all-targets
   -- -D warnings` (CI runs clippy on test targets; `just clippy` alone does not).
6. **Re-run the inventory scanner after every phase** and commit the TSV. An inventory that lies is
   worse than none.
7. **Respect repo boundaries** (AGENTS.md): this repo owns syntax/sema/IR/CLI/diagnostics-stdio
   only. No fix may introduce a runtime or host concern — that is what the architecture lint
   (Phase 1) exists to enforce.

---

## §3 Phases

### Phase 1 — Make the guards able to fail (I1, I2)
*Runnable now. Zero overlap with the concurrent rewrite.*

| Step | Item | Change | Verification |
|---|---|---|---|
| 1.1 | **I1** | Add one real ` ```rust ` doc example in `surface` (parse → HIR, ~6 lines) **or** delete the doc-test step from `Justfile:20`, `lint.yml:41`, `lint.yml:105`. Prefer adding: deleting removes a future guard. | `cargo test --workspace --doc` reports ≥1 test |
| 1.2 | **I2** | `tools/architecture/check_architecture.py:148-155` — `dependency_kind()` returns `None` for `"build"`, and `:186-188` skips `None`. Map `"build"` → `"dev"` (or raise an error naming the kind). Add a build-edge fixture to `tools/architecture/tests/fixtures/` + a test asserting the forbidden-prefix message. | `python3 tools/architecture/tests/test_architecture.py` green, new test red before fix |
| 1.3 | *(from findings-2 §1 G7)* | Member-parity rail so the stale-table rot cannot recur. Coordinate with findings-2 owner. | |

**Exit:** a deliberately broken doc-test and a deliberately smuggled build-dep both turn CI red.

### Phase 2 — Close the five High items
*Requires Gate 0. Ordered by independence from drifting crates.*

| Step | Item | Change | Note |
|---|---|---|---|
| 2.1 | **M8** | Two tests in `cli/tests/v3_cli_front_door_contract.rs`: `query --file <tmp>` (exit 0, plan lines identical to the inline run) and `query ""` (exit 2, `cli.error.class=usage`, message `input cannot be empty`). Needs a temp-file helper — `std::env::temp_dir()` + unique suffix; **no new dependency** (repo has no `tempfile`). | `cli/src/main.rs` is stable — safest first |
| 2.2 | **M1** | Extend `v3_uncovered_branches.rs:87` to a two-binding source (distinct names) and assert `outputs.len() == 2` with the second slot lifted. One string change; the oracle already exists. | `compile.rs` stable |
| 2.3 | **B2** | In `binder/tests/v3_binder_mutants.rs:394`, build `b.shape_record(...)` containing the leaf and assert the field symbol resolves `Local`. **Re-verify the line first** — this file drifted. | |
| 2.4 | **M3** | Add `result_kind: Table` + `truth_lane: ProofConfirmed` rail in `planning/tests/v3_planning_truth.rs` asserting `evidence == ProofTree`. | `plan.rs` stable |
| 2.5 | **A1** | Give the 11 golden-less tests in `surface/tests/v3_format_edges.rs` `assert_renders(src, expected)` goldens. **Re-verify first**: `assert_renders` count moved 12 → 13; the file drifted. | Live silent-regression hole |

### Phase 3 — Oracle strengthening (A-series bulk)
*After Phase 2. Mechanical once the contract is named; batch by file so each PR touches one crate.*

| Batch | Items | Files |
|---|---|---|
| 3a surface-document | A3, A4 | `v3_document_hellgate.rs` (stable) — pin exact spans, distinguish empty-statement vs missing-separator |
| 3b surface-format | A2, A5, A6 | `v3_format_hellgate.rs` (per-item discriminant), `v3_family_hellgate.rs` (bind inline VIEW body; delete the two subsumed rails) |
| 3c planning | A7, A8, A10, A12 | `v3_capability_derivation.rs`, `v3_explain_provenance.rs` — distinct arms per side (`filter(scan a)` vs `limit(scan b)`) |
| 3d types/cli/stdio | A11, A14, A15 | `types/src/op.rs` (`len()==4` first), `command_doc_gate.rs` (set-compare the alternation), `json.rs` (rename two tests) |
| 3e pipeline | A13-residue | none — refuted in review; skip |

### Phase 4 — Contract fixtures (B and M remainder)
*One focused negative/positive fixture each. Re-scope the modules items first.*

| Step | Item | Status |
|---|---|---|
| 4.1 | **B1** producer column reaches `parse_producer` | re-verify `family.rs` (drifted) |
| 4.2 | **B3** `WITH` target asserted (`target_of(composition.id)`) | re-verify `resolve.rs` |
| 4.3 | **B4** real composition-local pack rebuild | re-verify `compile.rs:791` (stable) |
| 4.4 | **B5** doc-comment fix | trivial |
| 4.5 | **M4** `Composition` dispatch in `planning/item.rs` | re-verify (drifted) |
| 4.6 | **M5** escaped-quote-with-brace fixture | `server.rs` stable |
| 4.7 | **M6** `parse_ref` `::` branch | re-verify `family.rs` (drifted) |
| 4.8 | **M7** binderless-quantifier render | re-verify `format.rs` (drifted) |
| 4.9 | **M12** `lower()` `Some(predicate)` | re-verify `source.rs` |
| 4.10 | **M13** aggregated cross-SCC | `recursion.rs` stable |
| 4.11 | **M9 / M10 / M11** | **INVALIDATED by the modules-v2 rewrite** — the target branches are gone. Re-scope against the new `authority.rs`/`graph.rs` only if the equivalent recursion/canonicalization/absent-pack logic reappears; otherwise drop. |

### Phase 5 — Consolidation (LAST among test changes)
*Strictly after Phases 3-4: every deletion below is only safe once the surviving oracle is strong.*

| Step | What | Source |
|---|---|---|
| 5.1 | Delete the 12 tautology lines + the 2 subsumed `lex_never_panics_*` rails + `lex_is_deterministic` | findings-1 A16 / findings-2 #43-#44 |
| 5.2 | Fold the **40 duplicate-body groups (130 tests)** into table-driven tests, starting with the two 10-test groups (`audit_edge_cases.rs:52`, `v3_family_hellgate.rs:550`) | findings-1 §0.1 |
| 5.3 | Execute the findings-2 §5.1 deletion list (respecting §5.4 couplings: never both `FIELD_NAMES` pins; strengthen `test_endpoint_conflict` **before** H1; do H2 before deleting #15) | findings-2 |
| 5.4 | Extract `tests/support/mod.rs` into the crates that lack it (`sp` ×15, `Binder::new` ×25, `gref` ×9, `col` ×5, 7 open-coded `SurfaceItemV1` lifts) — **with** 5.2 so each file is edited once | findings-1 / findings-2 #48-#49 |
| 5.5 | Canonicalize the `#![allow(...)]` header (91 files, 514 lines) — do **not** delete; CI clippy covers test targets | findings-2 #51 |

### Phase 6 — Re-audit the delta
*Not optional: the suite grew 810 → 966 while this was being written.*

- Re-run the inventory scanner over the current tree; diff against `test-suite-inventory.tsv`.
- Audit the **+156 new fns** (chiefly `v3_document_predeclaration.rs`, `v3_modules_v2_authority.rs`,
  `v3_rule_type_solver.rs`) with the same lenses.
- Re-verify every remaining `file:LINE` in both findings files; mark survivors.

---

## §4 Sequencing rationale

```
Gate 0 (compile green, rewrite landed)
   │
   ├─ Phase 1 (I1,I2)  ──────────────► independent, run anytime
   │
   ▼
Phase 2 (High closures)  ──► Phase 3 (oracles)  ──► Phase 4 (fixtures)  ──► Phase 5 (deletions)
                                                                      ▲
                              deletions are only safe once strong oracles exist ─┘
Phase 6 (re-audit delta) ──► after Gate 0, in parallel with 2-4; must precede 5.2 re-counting
```

- **Oracles before deletions**: deleting `audit_edge_cases`' 20 tests (findings-2 #38) before A3/A4
  and the surface rails are strengthened would leave those inputs with *no* oracle at all.
- **Scaffolding with consolidation** (5.4 with 5.2): extracting helpers into `tests/support/` and
  then re-collapsing the same files doubles the churn.
- **Phase 1 first** because it is the only work that protects all later work: a guard that cannot
  fail cannot catch a bad edit made in Phases 2-5.

---

## §5 Verification protocol (every PR)

1. `cargo nextest run --workspace --no-tests pass` green.
2. `cargo clippy --workspace --all-targets -- -D warnings` green.
3. `just typos` — the repo's typos gate has previously tripped on hyphenated prefixes in test
   comments; new fixtures will trip it again.
4. For each strengthened oracle: prove it can fail. Introduce the named mutant (e.g.
   `diagnostic.rs:137 Self::Parse(inner) => SpanV1::default()`), watch the test go red, revert.
   **An oracle that has never been seen red is unverified** — this is the difference between this
   plan and the findings it is fixing.
5. Commit the refreshed `test-suite-inventory.tsv`.

---

## §6 Effort

| Phase | Batches | Est. | Risk |
|---|---|---|---|
| 1 | 2 | S | low — tooling only |
| 2 | 5 | S–M | low; M8 needs a temp-file helper |
| 3 | 4 | M | medium — goldens must be written by hand and are the contract |
| 4 | 8 (+3 re-scope) | M | medium — 8 files drifted, every line re-verified |
| 5 | 5 | L | highest — deletions; couplings in findings-2 §5.4 must be respected |
| 6 | 1 | M | low effort, must not be skipped |

S ≤ half a day, M ≤ a day, L multi-day.

---

## §7 What this plan deliberately does not do

- No runtime/backend work (AGENTS.md boundary).
- No reduction of proptest budgets or removal of subprocess front-door tests.
- No re-litigation of items already refuted in `findings-1-duplicated.md` §0 (`catalog.resolve()`
  coverage, the `composition_type_env` duplicate, the slow-timeout note).
- No speculative modules fixes until the v2 rewrite lands and M9/M10/M11 are re-scoped.
