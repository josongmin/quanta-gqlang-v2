# Findings 1 (duplicated) — Non-Falsifiable / Unasserted Tests: Final Action List

**Purpose.** The decision-ready residue of `findings-1.md` after an adversarial self-review. Only
items that **require a change** survive. Items refuted, duplicated elsewhere, or merely cosmetic
were removed and are recorded in §0 so they are not re-litigated.

- **Companion:** `findings-1.md` is the full narrative (all claims, all evidence, all reasoning).
  This file is the action list. `findings-2-duplicated.md` owns the **duplicate/removable-test**
  axis; §6 below maps every overlap so nothing is fixed twice.
- **Audited revision:** snapshot `f0fc3dce0f672d7d`; **every `file:LINE` re-verified against the
  live working tree at write time.** Two citations drifted during the audit and are corrected
  here (`types/src/op.rs` 202 → 204-205; `json.rs` offsets).
- **Working tree moved twice during the audit** (`lsp` → `diagnostics-stdio` rename at 02:51-02:58,
  then `hir/src/*`, `cli/src/main.rs`, `tools/`, `Justfile`, `lint.yml` while writing). Re-verify if
  it moves again.
- **Scope:** 810 `#[test]` fns across 101 files, including **11 in-`src` `#[cfg(test)]` modules that
  no prior audit examined**. Per-test inventory: `test-suite-inventory.tsv` (this directory).

> ### ⚠️ RE-BASELINE — read before executing anything below
>
> The tree moved again after this list was written. Status re-verified at **HEAD `ecc38f5` + dirty
> working tree** (the concurrent modules-v2 session; it also committed this file):
>
> | | at audit snapshot | at `ecc38f5` |
> |---|---|---|
> | test fns / files | 810 / 101 | **978 / 110** |
> | no assertion at all | 84 | **28** |
> | weak-predicate-only | 38 | 25 |
> | no exact oracle | 50 | 40 |
> | `matches!(.., { .. })` payload-discarding | 41 | 42 |
> | `is_ok() \|\| is_err()` tautologies | 12 | **0** |
> | unwrap-as-only-check | 84 | 99 |
> | duplicate-body tests | 130 | 129 |
>
> `test-suite-inventory.tsv` has been **regenerated** (979 rows, 110 files, content stamp
> `0d9db05deafa34c5`) and now carries a `bodyhash` column — regenerate any time with
> `python3 inventory_scan.py <repo-root> out.tsv`. **The tree was actively changing during
> regeneration** (HEAD `ecc38f5` + a dirty, still-moving modules/types rewrite), so treat the TSV
> as point-in-time; the command is the source of truth, and the scanner is deterministic
> (verified identical on frozen input).
>
> **Already actioned by the concurrent session — do not redo:** the 12 tautology lines and
> `lex(&s)==lex(&s)` (**0 remaining**), the proptest rail trim (22 → 9 props), and **I2**
> (`dependency_kind` now returns `"build"` explicitly, *raises* on unknown kinds, and ships a
> `build-edge` fixture + a `build_edges` table).
>
> **Invalidated — target code deleted by the modules-v2 rewrite:** **M7** (the binderless render
> arm no longer exists; `LogicalQuantifier` now always renders a binder list), **M9**, **M10**,
> **M11** (`collect_view_refs`, the manifest `None` arm and `canonical_cycle` are gone; the whole
> `modules` test set was restructured). Do not write fixtures against them.
>
> **Line corrections applied below:** A1 → 10 of 14 golden-less (was 11 of 13), B2 → `:369`,
> M6 → `:312-319`, M4 → arm renamed `classify_composition_body`.
>
> Every other `file:LINE` below is **snapshot-anchored, not re-verified**. Re-resolve each one at
> execution time, and only after `remediation-plan.md` §1 **Gate 0** (suite compiles green) passes.

---

## §0 Self-review — what this list removes or corrects

Each headline was re-attacked with a fresh command, specifically looking for a test that already
covers the claimed gap. Two claims were **wrong** and are deleted; four were overstated and are
downgraded.

| # | Original claim | Verdict | How verified |
|---|---|---|---|
| **R1** | **M2:** "`catalog.resolve()`'s fail-close path is never reached; deleting the `resolve()` call stays green." | **WRONG — deleted.** Both `resolve()` error branches were asserted with exact code **and** message: `modules.view_inline_cycle` at `modules/tests/v3_catalog_hellgate.rs:218` (+msg `:231`), `modules.certified_required_violation` at `:269` (+msg `:279`). *(Evidence file has since been removed by the modules-v2 rewrite at `ecc38f5`; the refutation was valid at review time and is why M2 never entered the action list.)* | `grep -rn "view_inline_cycle\|certified" crates/*/tests` |
| **R2** | **A13:** "`v3_composition_type_env.rs` tests at `:84` and `:101` are identical once literals are blanked." | **WRONG — deleted.** Blanked-body hashes differ (`dc3b28495592` vs `f6a88759767e`). The only real point was a redundant trailing `assert!(compiled.plan().is_some())` — cosmetic, and `findings-2-duplicated.md` already targets this file. | length-preserving blank + SHA-1 per test body |
| **R3** | **A1:** "11 of 13 tests in `v3_format_edges.rs` use `assert_roundtrip` as their only oracle." | **CONFIRMED, and stronger than stated.** Measured: 11 tests use `assert_roundtrip` **with no `assert_renders` at all**; only 2 (`value_kinds_render_canonically`, `literal_kinds_render_canonically`) own real goldens. | per-test-fn oracle census |
| **R4** | **B5:** "`compile_item_structural_compiles_value_decl` promises a contract it never checks." | **OVERSTATED — downgraded to a doc fix.** The test asserts `no_serve_reason == ValueDeclaration` **and** `value_type == Some("int")`. Only its doc comment describes an env-open alias projection the body does not build. | read the body |
| **R5** | **M5:** "the frame extractor's escape tracking is unexercised." | **CONFIRMED but defensive-only — downgraded.** Escaped-quote fixtures exist (`v3_diagnostics_parity_hellgate.rs:66-67, 94, 131`) but no fixture places a brace inside the string, and no gqlang source in the suite contains `}`. | fixture census |
| **R6** | **M9:** "`RelationExprKindV1::Project`/`Compose` bodies are never constructed anywhere." | **OVERSTATED — rescoped.** They *are* built/destructured in `hir/tests/v3_hir_expr_kernel.rs:244` and `surface/tests/v3_surface_query.rs:74,132,156,162`. What is unreached is the **modules catalog-view** path: `inline_view()` builds a bare `Source` in both modules tests, so `collect_view_refs`' Project/Compose arms never run. | `grep -rn` + read the `inline_view` helper |
| **R7** | **A15:** "`json.rs` `\u` tests are fake." | **DOWNGRADED to a rename.** The names claim `\u` decoding; the fixtures are raw UTF-8. Real escape coverage exists at `:437-475`, and the file's own comment at `:417-421` already concedes this. | read fixtures |
| **R8** | **M1:** "multi-binding producer accumulation is never exercised." | **CONFIRMED, tightened.** `v3_uncovered_branches.rs:87 binding_outputs_is_some_only_for_composition` *does* compile a composition, but asserts `outputs.len() == 1, "one binding -> one output slot"` — a source with **two distinctly-named `let`s that compiles** exists nowhere in the repo (the only two-`let` source, `audit_edge_cases.rs:127`, is a duplicate-name parse reject). | repo-wide string-literal scan for multi-`let` sources |
| **R9** | **I4:** "`nextest slow-timeout` is unreachable." | **DELETED as non-actionable.** True but harmless; nothing to fix. | n/a |
| **R10** | **A9:** "the via-waypoint bound is unasserted." | **CONFIRMED after challenge.** `max_path_length` *is* asserted elsewhere (`logic/tests/v3_graph_query_hellgate.rs:76` = `Some(8)`, `logic/tests/v3_graph_query_depth_waypoint_conflict.rs:126` = `Some(5)`) — but those are **depth**-derived. `logic/src/graph.rs:288-292`'s `waypoints > 0 → Some(waypoints)` value is asserted by no test. | branch-by-branch trace |

**Rejected as duplicates** (fix there, not here): `self_validation.rs:40` (`BTreeSet` uniqueness),
the `modules` `active_packs` write-only field, `modules/src/graph.rs:238` unreachability, the four zero-assertion
diagnostic codes, the doc-gate const-emptying vacuity, `examples/` orphan corpus, and the
`audit_edge_cases` / `audit_18_families` removals — all owned by `findings-2-duplicated.md`.

**Mechanical context** — snapshot figures for the audit revision; see the RE-BASELINE banner above
for the current counts and the regeneration command:
810 test fns; **84 contain no assertion at all**; 84 whose only check is `unwrap`/`expect`;
41 using `matches!(.., X { .. })` and discarding the payload; 38 asserting only a weak predicate;
12 tautologies; **40 duplicate-body groups covering 130 tests**.

---

## §1 Assertions that cannot fail

| ID | Location | Problem | Action | Sev |
|---|---|---|---|---|
| **A1** | `surface/tests/v3_format_edges.rs:38-43` | `text1`/`text2` **are** `format_item(&item1)`/`format_item(&item2)`, so lines 39-43 recompute the pair line 38 already asserted. **10 of the 14 tests in this file have no golden at all** (re-census at `ecc38f5`; snapshot figure was 11 of 13). A formatter that silently drops a clause stays green. | Give the golden-less tests `assert_renders(src, expected)` goldens, as the `*_render_canonically` tests already do. | **High** |
| **A2** | `surface/tests/v3_format_hellgate.rs:164` | Comment promises "each same family"; body asserts only `doc2.items.len() == items.len()`. A VIEW rendered as any other one-item family keeps the count and stays idempotent. | Compare `items[i]`/`items2[i]` by discriminant (the item-level helper already does this at `:55`). | Med |
| **A3** | `surface/tests/v3_document_hellgate.rs:80` | Test is named `..._with_exact_span` but asserts `diag.span.start >= prefix.len()` — any span from the middle item's start to EOF passes, including one reported at the *third* item. | Pin the offset: `assert_eq!(diag.span.start, 41)` (the `db` token where `->` was expected). | Med |
| **A4** | `surface/tests/v3_document_hellgate.rs:93, 99, 114` | `empty_statement_rejects`, `leading_separator_rejects`, `missing_separator_between_items_rejects` are **identical modulo the literal** and share one oracle, `UnexpectedToken`. They cannot distinguish `surface/src/parser.rs:164` (empty statement) from `surface/src/parser.rs:181` (missing separator) — one generic reject passes all three. | Assert `"empty statement"` + span for the first two, `"expected ';'"` at the following token for the third. | Med |
| **A5** | `surface/tests/v3_family_hellgate.rs:344` | `CatalogPayloadV1::View(ViewBodyV1::Inline(_)) => {}` discards the parsed query. No other **surface parse** rail inspects the inline body, so a `parse_view` mutation storing a default/truncated `QueryHirV1` passes (the round-trip oracle of A1 cannot catch it either). | Bind the inner query and assert `format_item(q) == "from users u \| where u.active"`. | Med |
| **A6** | `surface/tests/v3_family_hellgate.rs:435` (+`:402`) | Duplicates a stronger rail on **identical input**: `surface/tests/v3_family_edges.rs:246` feeds the same `gate g \| binding=all` to `assert_unsupported`, which pins the code **and** the full message. Same for `profile` at `:402` vs `family_edges:230`. | Delete the hellgate pair. | Low |
| **A7** | `planning/tests/v3_capability_derivation.rs:117` | Test claims the join "walks BOTH arms". Both arms are `Scan` and the result is a `BTreeSet`, so deleting `collect(&join.right, caps)` (`planning/src/capability.rs:103`) still yields `{Join, RelationScan}` with `len == 2`. **Apply / Union / SemiJoin repeat this.** | Use distinct arms: `join(filter(scan a), limit(scan b))` → 4 capabilities. | Med |
| **A8** | `planning/tests/v3_explain_provenance.rs:222` | "Left then right" pre-order: both arms are `scan(..)` → both `Scan`, so **swapping left and right yields an identical vec**. Same at `apply:244`, `union:266`, `semijoin:288`, `antijoin:310`. | Use `join(filter(scan a), limit(scan b))` → `[Join, Filter, Scan, Limit, Scan]`. | Med |
| **A9** | `logic/tests/v3_graph_query_hellgate.rs:173` | `reachability_with_via_waypoint_admits` asserts only `contract.kind`. The waypoint-derived bound `logic/src/graph.rs:288-292` (`waypoints > 0 → Some(waypoints)`) is asserted by **no** test — every other `max_path_length` assertion is depth-derived (§0 R10). | `assert_eq!(contract.witness.max_path_length, Some(1))` + a two-waypoint case. | Med |
| **A10** | `pipeline/tests/v3_pipeline_compile.rs:52` | `assert!(!cost.estimated_cardinality_class.keyword().is_empty())` — `keyword()` is `pub const fn -> &'static str` over five non-empty literals (`planning/src/cost.rs:42-46`), so this holds even for a hardcoded class. The doc claims "all 5 fields populated". | Assert concrete classes for this query (`LinearInInput` / `None` / `Unordered`). | Low |
| **A11** | `types/src/op.rs:204-205` | `assert!(lt.iter().all(\|ov\| ov.result == TypeV1::bool()))` — `all()` on an **empty** iterator is true and nothing asserts `lt` is non-empty. | Add `assert_eq!(lt.len(), 4)` first. *(Limited impact: `v3_types_universe.rs::comparison_types_to_bool` pins `int < int -> bool`.)* | Low |
| **A12** | `planning/tests/v3_cost_classification.rs:365` | Claims to exercise `worst_class`'s `>=` tie branch. `rank()` is **injective** over five classes (`planning/src/plan.rs:311-319`), so equal ranks imply equal classes and `>=` vs `>` return identical values — the branch is unobservable. | Drop the `>=` claim; cover left-wins with distinct classes. | Low |
| **A14** | `cli/tests/command_doc_gate.rs:29` | `usage.contains("fmt")` is satisfied by `v3-fmt`, and no rail ever runs bare `fmt`, so deleting `"fmt" \|` at `cli/src/main.rs:53` stays green. (Bare `query` **is** pinned elsewhere, so exposure is `fmt` only.) | Split the `<query\|fmt\|v3-query\|v3-fmt>` alternation on `\|` and set-compare to `CLI_COMMAND_TOKENS_V1`. | Med |
| **A15** | `diagnostics-stdio/src/json.rs:403`, `:410` | `valid_surrogate_pair_decodes_to_astral_scalar_v1` and `bmp_unicode_escape_decodes_v1` name `\u` decoding but feed **raw UTF-8** (a literal emoji / `é`), exercising the plain-char push arm, not `parse_unicode_escape_v1`. Real coverage is at `:437-475`. | Rename to `..._passthrough_v1`. | Low |

---

## §2 Name / behaviour mismatch — cannot fail for the reason its name gives

| ID | Location | Name promises | Body verifies | Action | Sev |
|---|---|---|---|---|---|
| **B1** | `surface/tests/v3_family_hellgate.rs:940` | the PRODUCER column reaches `parse_producer` | `at_composition` (`family.rs:130-137`) only routes `let`-bodies whose producer token is `from`/`sample`/`find`; `explain` is rejected as a let-value. A mutation making `parse_producer` accept `explain` still yields `UnexpectedToken`. | Drive the real producer slot with a second binding (see M1's two-`let` source). | Med |
| **B2** | `binder/tests/v3_binder_mutants.rs:379` | ~~only `shape_leaf` + `shape_list` are built~~ **DONE (this session).** The test now wraps the list in a `shape_record` field, so `resolve_shape` must recurse Record → List → Leaf; `target_of(inner_ref.id)` transitively proves the Record arm ran. **Proof:** binder scope 53/53 green; with the mutant `Record(_) => {}` injected the test fails exactly as predicted (`Option::unwrap()` on `None`); restored and re-verified green. | ~~High~~ **closed** |
| **B3** | `binder/tests/v3_composition_hellgate.rs:80` | the FIND body **and** `WITH` both resolve to the LET | only `target_of(body_ref_node)` is read. Deleting the `Bound` arm of `resolve_with` (`resolve.rs:399-406`) still passes — the composition's own node id is never queried. | `assert!(matches!(result.target_of(composition.id), ...))`. | Med |
| **B4** | `pipeline/tests/v3_use_pack_authority.rs:147` | a **composition-local** pack-context rebuild | the test builds the context itself and asserts a constructor round-trip; the source is **top-level**, not inside a composition. Production only clones (`compile.rs:791`) and never calls `new`. If `compile_composition` dropped pack authority, this still passes. | Drive `USE PACK` inside a real composition body, or delete as a duplicate of the `:57` rail. | Med |
| **B5** | `pipeline/tests/v3_uncovered_branches.rs:139-142` | the doc comment says an **alias-dependent projection stays env-open**, so it compiles structurally | the body compiles `let x = 1 + 2` — a closed literal decl. (The test's assertions are otherwise sound — see §0 R4.) | Fix the comment or add the alias-projection case. | Low |

---

## §3 Production branches no test reaches

| ID | Branch | Why unreachable | Action | Sev |
|---|---|---|---|---|
| **M1** | `pipeline/src/compile.rs:794` — per-binding accumulation loop | The only test that compiles a composition (`v3_uncovered_branches.rs:87`) uses **one** `let` and asserts `outputs.len() == 1`. No compiling multi-`let` source exists in the repo; the sole two-`let` source (`audit_edge_cases.rs:127`) is a duplicate-name parse reject. The loop body therefore runs exactly once. | Extend that test to a two-binding source (distinct names) and assert `outputs.len() == 2` with the second slot lifted. **The oracle already exists — this is a one-string change.** | **High** |
| **M3** | `planning/src/plan.rs:242` — final-lane early return in `evidence_for` | The only successful `ProofConfirmed` rail (`v3_planning_truth.rs:282`) uses `result_kind: ProofExplain`, whose match arm (`:249`) also returns `ProofTree`. Deleting the early return leaves everything green, and `Table + ProofConfirmed` (permitted by `reject_unsupported_mix:188`) silently downgrades to `None`. | Add a `result_kind: Table` + `truth_lane: ProofConfirmed` rail asserting `ProofTree`. | **High** |
| **M4** | `planning/src/item.rs:110` — `Composition(c) => classify_composition_body(&c.body)` *(arm re-anchored at `ecc38f5`; was `classify_item(&c.body)`)* | `grep` over all of `crates/quanta-gqlang-v2-planning/` finds **zero** `CompositionHirV1` construction; the pipeline decomposes compositions via `compile_composition` instead. Making the dispatch always return `NoPlan(RelationalDeferred)` survives every test. | Add a `v3_item_hellgate` rail wrapping a CHECK body in `CompositionHirV1`. | Med |
| **M7** | ~~`surface/src/format.rs:311` — binderless-quantifier render arm~~ | **INVALIDATED at `ecc38f5`** — the `if q.binders.is_empty()` branch no longer exists; `BoolExprKindV1::LogicalQuantifier` now unconditionally renders `exists <binders> in (<body>)` (`format.rs:338-349`). No action. | ~~Med~~ |
| **M8** | `cli/src/main.rs:196-199` — `read_input_file_v1` **success** path; `:190` — `"input cannot be empty"` guard | All three `--file` rails (`v3_cli_front_door_contract.rs:95,140,188`) use nonexistent paths or extra args, so only the `Err` branch runs. A mutation returning `Err` unconditionally passes while `query --file <valid>` is broken. No test passes an empty inline argument. | Add `query --file <tmp>` (exit 0, same plan lines as inline) and `query ""` (exit 2 / usage class). **Two tests close the whole front-door claim.** | **High** |
| **M9** | ~~`modules/src/catalog.rs:345` — `collect_view_refs` `Project`/`Compose` recursion~~ | **INVALIDATED at `ecc38f5`** — `collect_view_refs` no longer exists in `modules/src/catalog.rs`; the modules crate was restructured into the v2-authority shape and its test set replaced. Re-scope only if equivalent inline-view recursion reappears. | ~~Med~~ |
| **M10** | ~~`modules/src/manifest.rs:242` — dependency on a pack **absent from the graph** (`None` arm)~~ | **INVALIDATED at `ecc38f5`** — the `UnknownModule` arm is gone from `manifest.rs`. No action. | ~~Low~~ |
| **M11** | ~~`modules/src/graph.rs:347` — `canonical_cycle` prefix-slice + rotate-to-smallest~~ | **INVALIDATED at `ecc38f5`** — `canonical_cycle` no longer exists in `modules/src/graph.rs` (file is now 294 lines). No action. | ~~Med~~ |
| **M12** | `logic/src/source.rs:115` — `lower()`'s `Some(predicate)` branch | All three lowering tests pass `None` (`v3_logic_source.rs:37,55,70`). The correlation predicate is the only reason a semi/anti join is not a cartesian test, and it is never exercised. | Add cases with `Some(bool_expr)`; assert the lowered `SemiJoinV1`/`AntiJoinV1` predicate is `Some`. | Med |
| **M13** | `logic/src/recursion.rs:363` — `Aggregated` cross-SCC edge arm | The only aggregated literal in the repo (`v3_logic_hellgate.rs:287`, `rank :- rank`) is an intra-SCC self-loop that rejects earlier at `recursion.rs:243`. Treating `Aggregated` as monotone here survives every test. | Add `p :- aggregated(q)`, `q :- positive(base)` in distinct SCCs; assert `stratum_of(p) == stratum_of(q) + 1`. | Med |
| **M5** | `diagnostics-stdio/src/server.rs:108` — escape tracking in the frame extractor | No fixture pairs an **escaped quote with an in-string brace**; no gqlang source in the suite contains `}`. Escape-vs-no-escape yields identical boundaries. Defensive branch (§0 R5). | Add `{"method":"v3_diagnostics","query":"a\"}b"}`; pin the response line. | Low |
| **M6** | `surface/src/family.rs:312-319` — `parse_ref`'s `::` branch *(was `:285-292`; re-anchored at `ecc38f5`)* | `family.rs` documents `repo::auth` as a supported reference form, but no surface test parses a gqlang source containing `::`. Every `::` in the tests is a Rust path or a builder argument, not parsed input. The colon arm, single-colon acceptance and missing-segment reject never run (branch confirmed still present at `ecc38f5`). | Add `check repo::auth \| confirm=skip` asserting `rule_ref.name == "repo::auth"`, plus `check a: \| ...` for the reject. | Med |

---

## §4 CI steps that are structurally incapable of failing

| ID | Problem | Evidence | Action | Sev |
|---|---|---|---|---|
| **I1** | ~~**The doc-test step compiles zero doc-tests**~~ | **DONE (this session).** Added a real ` ```rust ` example to `logic/src/source.rs` (`ExistencePolarityV1::from_keyword`). **Proof:** `cargo test -p quanta-gqlang-v2-logic --doc` went from `0 passed` to `1 passed`; with a deliberately wrong assertion the step reports `FAILED` (1 failed), restored and re-verified green. The three invocations (`Justfile:20`, `.github/workflows/lint.yml:41`,`:105`) now run a test that can fail. | ~~High~~ **closed** |
| **I2** | ~~**`[build-dependencies]` bypass all three dependency guards**~~ | **DONE by the concurrent session** — `tools/architecture/check_architecture.py` now maps `kind == "build"` explicitly, **raises** on unknown kinds (`dependency_kind`, `:159-161`), and the lint carries a `build_edges` table (`:222`) plus a `build-edge` fixture consumed by `test_architecture.py`. Do not redo. | ~~High~~ |

---

## §5 Not in this list

- **`examples/` orphan corpus** (18 `.qglang` files, zero references outside `docs/`, a phantom
  README rail) — `findings-2-duplicated.md` §2 C3/C4. Re-confirmed here: `grep` over `*.rs`,
  `*.sh`, `*.py`, `*.yml`, `Justfile`, `*.toml` returns nothing outside `docs/`.
- **Removals / duplicates** (130 duplicate-body tests, `audit_edge_cases`, `audit_18_families`,
  `v3_planning_cost_schema`, `test_endpoint_conflict`, `active_packs`) —
  `findings-2-duplicated.md` §5.1, §3 D3, §4 H1.
- **Rejected after review:** `catalog.resolve()` coverage (§0 R1), the
  `composition_type_env` "duplicate" (§0 R2), the `nextest slow-timeout` note (§0 R9).

---

## §6 Overlap with `findings-2-duplicated.md`

| This list | Owned by findings-2 as | Note |
|---|---|---|
| A14 | §5.2 #46 | #46 is const-emptying vacuity; A14 is the `fmt` substring — fix together. |
| A15 | §5.1 #21 | #21 targets the same escape fixtures from the duplicate angle. |
| B4 | §5.1 #4 | #4 concerns a different test in the same file; both edits land there. |
| ~~M9 / M11~~ | — | **invalidated at `ecc38f5`** (targets deleted) — see banner and §3. |
| A1 | §5.2 #45 | #45 names the line; A1 supplies the blast radius (10 of 14 tests golden-less at `ecc38f5`). |
| Tautologies / `lex(&s)==lex(&s)` / proptest trim | §5.2 #43, #44 | **actioned by the concurrent session** — 0 remaining. |
| **I2** | — | **actioned by the concurrent session** (`build` kind + fixture + raise-on-unknown). |
| M1, M3, M4, M5, M6, M8, M12, M13, A2-A13, B1, B2, B3, B5, I1 | — | net-new. |

---

## §7 Fix order
*(re-cut again: **I1 and B2 are closed** — this session, with mutant-kill proof for B2 and
fail-proof for I1. I2 + tautologies were already done by the concurrent session.
M7/M9/M10/M11 dead. Remaining High items M8/M1/A1 are **blocked**: the `pipeline` lib does not
compile (25 errors, E0223-class) and `cli` depends on it, so they cannot be verified until
`remediation-plan.md` Gate 0 passes.)*

> **Re-verified after the concurrent writer merged into the same files** (they added ~69 lines to
> `v3_binder_mutants.rs` and new catalog code to `binder/src/resolve.rs`): my record-wrap hunk and
> the `resolve_shape` injection site are byte-identical, so the B2 mutant-kill proof **transfers**;
> binder scope **54/54** green (their +1 test included), `logic` doc-test **1 passed**,
> `cargo clippy -p binder --all-targets -D warnings` green. `clippy -p logic` could not finish —
> upstream `algebra` lib is mid-repair by the other writer (pre-existing, not ours).
>
> **Later probe:** the concurrent writer is now editing test files across the workspace mid-flight
> (`expected item, found [` syntax-stage errors in `logic`/`binder`/`surface` test targets —
> `binder` was 54/54 green minutes earlier). **No crate is scoped-verifiable in that window.**
> Per §5's own rule, no oracle is authored while it cannot be seen red; the remaining items wait
> for the writer's next commit, then proceed M8 → M1 → A1.

1. ~~**I1**~~ — **closed** (doc-test added in `logic`, 0 → 1 executed, fail-proof shown).
2. **M8** — `query --file <valid>` and `query ""`. *Blocked on Gate 0 (pipeline lib).*
3. ~~**B2**~~ — **closed** (record-wraps-list; mutant-kill shown).
4. **M1** — extend `binding_outputs_is_some_only_for_composition` to two bindings. *Blocked on Gate 0.*
5. **A1** — give the 10 golden-less `assert_roundtrip` tests goldens. *Blocked on Gate 0 + `surface` drift.*
6. **M3 / B4** — pin the lane rule and the composition-local pack rebuild.
7. **B3 / M4 / M5 / M6 / M12 / M13** — one focused fixture each.
8. **A2-A12, A14, B1, B5** — per-test oracle strengthening; mechanical once the contract is named.
