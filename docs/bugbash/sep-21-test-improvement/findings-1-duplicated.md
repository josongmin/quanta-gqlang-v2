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

---

## §0 Self-review — what this list removes or corrects

Each headline was re-attacked with a fresh command, specifically looking for a test that already
covers the claimed gap. Two claims were **wrong** and are deleted; four were overstated and are
downgraded.

| # | Original claim | Verdict | How verified |
|---|---|---|---|
| **R1** | **M2:** "`catalog.resolve()`'s fail-close path is never reached; deleting the `resolve()` call stays green." | **WRONG — deleted.** Both `resolve()` error branches are asserted with exact code **and** message: `modules.view_inline_cycle` at `modules/tests/v3_catalog_hellgate.rs:218` (+msg `:231`), `modules.certified_required_violation` at `:269` (+msg `:279`). | `grep -rn "view_inline_cycle\|certified" crates/*/tests` |
| **R2** | **A13:** "`v3_composition_type_env.rs` tests at `:84` and `:101` are identical once literals are blanked." | **WRONG — deleted.** Blanked-body hashes differ (`dc3b28495592` vs `f6a88759767e`). The only real point was a redundant trailing `assert!(compiled.plan().is_some())` — cosmetic, and `findings-2-duplicated.md` already targets this file. | length-preserving blank + SHA-1 per test body |
| **R3** | **A1:** "11 of 13 tests in `v3_format_edges.rs` use `assert_roundtrip` as their only oracle." | **CONFIRMED, and stronger than stated.** Measured: 11 tests use `assert_roundtrip` **with no `assert_renders` at all**; only 2 (`value_kinds_render_canonically`, `literal_kinds_render_canonically`) own real goldens. | per-test-fn oracle census |
| **R4** | **B5:** "`compile_item_structural_compiles_value_decl` promises a contract it never checks." | **OVERSTATED — downgraded to a doc fix.** The test asserts `no_serve_reason == ValueDeclaration` **and** `value_type == Some("int")`. Only its doc comment describes an env-open alias projection the body does not build. | read the body |
| **R5** | **M5:** "the frame extractor's escape tracking is unexercised." | **CONFIRMED but defensive-only — downgraded.** Escaped-quote fixtures exist (`v3_diagnostics_parity_hellgate.rs:66-67, 94, 131`) but no fixture places a brace inside the string, and no gqlang source in the suite contains `}`. | fixture census |
| **R6** | **M9:** "`RelationExprKindV1::Project`/`Compose` bodies are never constructed anywhere." | **OVERSTATED — rescoped.** They *are* built/destructured in `hir/tests/v3_hir_expr_kernel.rs:244` and `surface/tests/v3_surface_query.rs:74,132,156,162`. What is unreached is the **modules catalog-view** path: `inline_view()` builds a bare `Source` in both modules tests, so `collect_view_refs`' Project/Compose arms never run. | `grep -rn` + read the `inline_view` helper |
| **R7** | **A15:** "`json.rs` `\u` tests are fake." | **DOWNGRADED to a rename.** The names claim `\u` decoding; the fixtures are raw UTF-8. Real escape coverage exists at `:437-475`, and the file's own comment at `:417-421` already concedes this. | read fixtures |
| **R8** | **M1:** "multi-binding producer accumulation is never exercised." | **CONFIRMED, tightened.** `v3_uncovered_branches.rs:87 binding_outputs_is_some_only_for_composition` *does* compile a composition, but asserts `outputs.len() == 1, "one binding -> one output slot"` — a source with **two distinctly-named `let`s that compiles** exists nowhere in the repo (the only two-`let` source, `audit_edge_cases.rs:127`, is a duplicate-name parse reject). | repo-wide string-literal scan for multi-`let` sources |
| **R9** | **I4:** "`nextest slow-timeout` is unreachable." | **DELETED as non-actionable.** True but harmless; nothing to fix. | n/a |
| **R10** | **A9:** "the via-waypoint bound is unasserted." | **CONFIRMED after challenge.** `max_path_length` *is* asserted elsewhere (`graph_query_hellgate.rs:76` = `Some(8)`, `depth_waypoint_conflict.rs:126` = `Some(5)`) — but those are **depth**-derived. `graph.rs:288-292`'s `waypoints > 0 → Some(waypoints)` value is asserted by no test. | branch-by-branch trace |

**Rejected as duplicates** (fix there, not here): `self_validation.rs:40` (`BTreeSet` uniqueness),
`modules/catalog.rs:185` `active_packs`, `graph.rs:238` unreachability, the four zero-assertion
diagnostic codes, the doc-gate const-emptying vacuity, `examples/` orphan corpus, and the
`audit_edge_cases` / `audit_18_families` removals — all owned by `findings-2-duplicated.md`.

**Mechanical context for the whole list** (nothing else in the repo has this measurement):
810 test fns; **84 contain no assertion at all**; 84 whose only check is `unwrap`/`expect`;
41 using `matches!(.., X { .. })` and discarding the payload; 38 asserting only a weak predicate;
12 tautologies; **40 duplicate-body groups covering 130 tests**.

---

## §1 Assertions that cannot fail

| ID | Location | Problem | Action | Sev |
|---|---|---|---|---|
| **A1** | `surface/tests/v3_format_edges.rs:38-43` | `text1`/`text2` **are** `format_item(&item1)`/`format_item(&item2)`, so lines 39-43 recompute the pair line 38 already asserted. **11 of the 13 tests in this file have no golden at all** (§0 R3). A formatter that silently drops a clause stays green. | Give the 11 tests `assert_renders(src, expected)` goldens, as the two `*_render_canonically` tests already do. | **High** |
| **A2** | `surface/tests/v3_format_hellgate.rs:164` | Comment promises "each same family"; body asserts only `doc2.items.len() == items.len()`. A VIEW rendered as any other one-item family keeps the count and stays idempotent. | Compare `items[i]`/`items2[i]` by discriminant (the item-level helper already does this at `:55`). | Med |
| **A3** | `surface/tests/v3_document_hellgate.rs:80` | Test is named `..._with_exact_span` but asserts `diag.span.start >= prefix.len()` — any span from the middle item's start to EOF passes, including one reported at the *third* item. | Pin the offset: `assert_eq!(diag.span.start, 41)` (the `db` token where `->` was expected). | Med |
| **A4** | `surface/tests/v3_document_hellgate.rs:93, 99, 114` | `empty_statement_rejects`, `leading_separator_rejects`, `missing_separator_between_items_rejects` are **identical modulo the literal** and share one oracle, `UnexpectedToken`. They cannot distinguish `parser.rs:164` (empty statement) from `parser.rs:181` (missing separator) — one generic reject passes all three. | Assert `"empty statement"` + span for the first two, `"expected ';'"` at the following token for the third. | Med |
| **A5** | `surface/tests/v3_family_hellgate.rs:344` | `CatalogPayloadV1::View(ViewBodyV1::Inline(_)) => {}` discards the parsed query. No other **surface parse** rail inspects the inline body, so a `parse_view` mutation storing a default/truncated `QueryHirV1` passes (the round-trip oracle of A1 cannot catch it either). | Bind the inner query and assert `format_item(q) == "from users u \| where u.active"`. | Med |
| **A6** | `surface/tests/v3_family_hellgate.rs:435` (+`:402`) | Duplicates a stronger rail on **identical input**: `v3_family_edges.rs:246` feeds the same `gate g \| binding=all` to `assert_unsupported`, which pins the code **and** the full message. Same for `profile` at `:402` vs `edges:230`. | Delete the hellgate pair. | Low |
| **A7** | `planning/tests/v3_capability_derivation.rs:117` | Test claims the join "walks BOTH arms". Both arms are `Scan` and the result is a `BTreeSet`, so deleting `collect(&join.right, caps)` (`capability.rs:103`) still yields `{Join, RelationScan}` with `len == 2`. **Apply / Union / SemiJoin repeat this.** | Use distinct arms: `join(filter(scan a), limit(scan b))` → 4 capabilities. | Med |
| **A8** | `planning/tests/v3_explain_provenance.rs:222` | "Left then right" pre-order: both arms are `scan(..)` → both `Scan`, so **swapping left and right yields an identical vec**. Same at `apply:244`, `union:266`, `semijoin:288`, `antijoin:310`. | Use `join(filter(scan a), limit(scan b))` → `[Join, Filter, Scan, Limit, Scan]`. | Med |
| **A9** | `logic/tests/v3_graph_query_hellgate.rs:173` | `reachability_with_via_waypoint_admits` asserts only `contract.kind`. The waypoint-derived bound `graph.rs:288-292` (`waypoints > 0 → Some(waypoints)`) is asserted by **no** test — every other `max_path_length` assertion is depth-derived (§0 R10). | `assert_eq!(contract.witness.max_path_length, Some(1))` + a two-waypoint case. | Med |
| **A10** | `pipeline/tests/v3_pipeline_compile.rs:52` | `assert!(!cost.estimated_cardinality_class.keyword().is_empty())` — `keyword()` is `pub const fn -> &'static str` over five non-empty literals (`planning/src/cost.rs:42-46`), so this holds even for a hardcoded class. The doc claims "all 5 fields populated". | Assert concrete classes for this query (`LinearInInput` / `None` / `Unordered`). | Low |
| **A11** | `types/src/op.rs:204-205` | `assert!(lt.iter().all(\|ov\| ov.result == TypeV1::bool()))` — `all()` on an **empty** iterator is true and nothing asserts `lt` is non-empty. | Add `assert_eq!(lt.len(), 4)` first. *(Limited impact: `v3_types_universe.rs::comparison_types_to_bool` pins `int < int -> bool`.)* | Low |
| **A12** | `planning/tests/v3_cost_classification.rs:365` | Claims to exercise `worst_class`'s `>=` tie branch. `rank()` is **injective** over five classes (`plan.rs:311-319`), so equal ranks imply equal classes and `>=` vs `>` return identical values — the branch is unobservable. | Drop the `>=` claim; cover left-wins with distinct classes. | Low |
| **A14** | `cli/tests/command_doc_gate.rs:29` | `usage.contains("fmt")` is satisfied by `v3-fmt`, and no rail ever runs bare `fmt`, so deleting `"fmt" \|` at `cli/src/main.rs:53` stays green. (Bare `query` **is** pinned elsewhere, so exposure is `fmt` only.) | Split the `<query\|fmt\|v3-query\|v3-fmt>` alternation on `\|` and set-compare to `CLI_COMMAND_TOKENS_V1`. | Med |
| **A15** | `diagnostics-stdio/src/json.rs:403`, `:410` | `valid_surrogate_pair_decodes_to_astral_scalar_v1` and `bmp_unicode_escape_decodes_v1` name `\u` decoding but feed **raw UTF-8** (a literal emoji / `é`), exercising the plain-char push arm, not `parse_unicode_escape_v1`. Real coverage is at `:437-475`. | Rename to `..._passthrough_v1`. | Low |

---

## §2 Name / behaviour mismatch — cannot fail for the reason its name gives

| ID | Location | Name promises | Body verifies | Action | Sev |
|---|---|---|---|---|---|
| **B1** | `surface/tests/v3_family_hellgate.rs:940` | the PRODUCER column reaches `parse_producer` | `at_composition` (`family.rs:130-137`) only routes `let`-bodies whose producer token is `from`/`sample`/`find`; `explain` is rejected as a let-value. A mutation making `parse_producer` accept `explain` still yields `UnexpectedToken`. | Drive the real producer slot with a second binding (see M1's two-`let` source). | Med |
| **B2** | `binder/tests/v3_binder_mutants.rs:394` | `resolve_shape_walks_nested_record_and_list` | only `shape_leaf` + `shape_list` are built; `shape_record` appears **zero times** in `crates/quanta-gqlang-v2-binder/`. Emptying the Record arm (`resolve.rs:604-608`) leaves **every** binder test green. | Build a `b.shape_record` containing the leaf and assert the field symbol resolves; or rename. | **High** |
| **B3** | `binder/tests/v3_composition_hellgate.rs:80` | the FIND body **and** `WITH` both resolve to the LET | only `target_of(body_ref_node)` is read. Deleting the `Bound` arm of `resolve_with` (`resolve.rs:399-406`) still passes — the composition's own node id is never queried. | `assert!(matches!(result.target_of(composition.id), ...))`. | Med |
| **B4** | `pipeline/tests/v3_use_pack_authority.rs:147` | a **composition-local** pack-context rebuild | the test builds the context itself and asserts a constructor round-trip; the source is **top-level**, not inside a composition. Production only clones (`compile.rs:791`) and never calls `new`. If `compile_composition` dropped pack authority, this still passes. | Drive `USE PACK` inside a real composition body, or delete as a duplicate of the `:57` rail. | Med |
| **B5** | `pipeline/tests/v3_uncovered_branches.rs:139-142` | the doc comment says an **alias-dependent projection stays env-open**, so it compiles structurally | the body compiles `let x = 1 + 2` — a closed literal decl. (The test's assertions are otherwise sound — see §0 R4.) | Fix the comment or add the alias-projection case. | Low |

---

## §3 Production branches no test reaches

| ID | Branch | Why unreachable | Action | Sev |
|---|---|---|---|---|
| **M1** | `pipeline/src/compile.rs:794` — per-binding accumulation loop | The only test that compiles a composition (`v3_uncovered_branches.rs:87`) uses **one** `let` and asserts `outputs.len() == 1`. No compiling multi-`let` source exists in the repo; the sole two-`let` source (`audit_edge_cases.rs:127`) is a duplicate-name parse reject. The loop body therefore runs exactly once. | Extend that test to a two-binding source (distinct names) and assert `outputs.len() == 2` with the second slot lifted. **The oracle already exists — this is a one-string change.** | **High** |
| **M3** | `planning/src/plan.rs:242` — final-lane early return in `evidence_for` | The only successful `ProofConfirmed` rail (`v3_planning_truth.rs:282`) uses `result_kind: ProofExplain`, whose match arm (`:249`) also returns `ProofTree`. Deleting the early return leaves everything green, and `Table + ProofConfirmed` (permitted by `reject_unsupported_mix:188`) silently downgrades to `None`. | Add a `result_kind: Table` + `truth_lane: ProofConfirmed` rail asserting `ProofTree`. | **High** |
| **M4** | `planning/src/item.rs:110` — `Composition(c) => classify_item(&c.body)` | `grep` over all of `crates/quanta-gqlang-v2-planning/` finds **zero** `CompositionHirV1` construction; the pipeline decomposes compositions via `compile_composition` instead. Making the arm always return `NoPlan(RelationalDeferred)` survives every test. | Add a `v3_item_hellgate` rail wrapping a CHECK body in `CompositionHirV1`. | Med |
| **M7** | `surface/src/format.rs:311` — binderless-quantifier render arm | The only fixture producing one (`v3_surface_expr_edges.rs:199`, `exists ( a > 1 )`) is **parsed but never formatted**; every format test uses a binder. | `assert_eq!(format_item(parse_program("rule r : x if exists ( a > 1 )")), "rule r : x if exists (a > 1)")`. | Med |
| **M8** | `cli/src/main.rs:196-199` — `read_input_file_v1` **success** path; `:190` — `"input cannot be empty"` guard | All three `--file` rails (`v3_cli_front_door_contract.rs:95,140,188`) use nonexistent paths or extra args, so only the `Err` branch runs. A mutation returning `Err` unconditionally passes while `query --file <valid>` is broken. No test passes an empty inline argument. | Add `query --file <tmp>` (exit 0, same plan lines as inline) and `query ""` (exit 2 / usage class). **Two tests close the whole front-door claim.** | **High** |
| **M9** | `modules/src/catalog.rs:345` — `collect_view_refs` `Project`/`Compose` recursion | Both modules tests build their catalog VIEW payload through `inline_view()`, which hardcodes `RelationExprKindV1::Source` (`v3_catalog_hellgate.rs:52-59`, `v3_modules_mutation_rails.rs:36-44`). Replacing the arms with `=> {}` keeps all tests green, so a cycle through a projected inline body is undetected. *(Project/Compose **are** exercised elsewhere — see §0 R6.)* | Register `view::a` → projected(`view::b`) and `view::b` → `view::a`; assert `modules.view_inline_cycle`. | Med |
| **M10** | `modules/src/manifest.rs:242` — dependency on a pack **absent from the graph** (`None` arm) | `dependency_unsatisfied_rejects` only reaches the `Some(version)` arm; `manifest_and_lock_round_trip` declares a present, in-range dependency. | Add a graph with a declared dependency and no such manifest; assert code + Display. | Low |
| **M11** | `modules/src/graph.rs:347` — `canonical_cycle` prefix-slice + rotate-to-smallest | Both cycle rails start the DFS on the cycle's **own smallest** member (`v3_modules_hellgate.rs:141` `m_a→m_b→m_c→m_a`; `v3_modules_graph.rs:82` self-loop `m`), so `start == 0` and `min_pos == 0`. Forcing `start = 0` or deleting `min_by` is a **no-op**. | Add root `aaa` importing `z`, cycle `z↔m`; assert `cycle == ["m","z","m"]`. | Med |
| **M12** | `logic/src/source.rs:115` — `lower()`'s `Some(predicate)` branch | All three lowering tests pass `None` (`v3_logic_source.rs:37,55,70`). The correlation predicate is the only reason a semi/anti join is not a cartesian test, and it is never exercised. | Add cases with `Some(bool_expr)`; assert the lowered `SemiJoinV1`/`AntiJoinV1` predicate is `Some`. | Med |
| **M13** | `logic/src/recursion.rs:363` — `Aggregated` cross-SCC edge arm | The only aggregated literal in the repo (`v3_logic_hellgate.rs:287`, `rank :- rank`) is an intra-SCC self-loop that rejects earlier at `recursion.rs:243`. Treating `Aggregated` as monotone here survives every test. | Add `p :- aggregated(q)`, `q :- positive(base)` in distinct SCCs; assert `stratum_of(p) == stratum_of(q) + 1`. | Med |
| **M5** | `diagnostics-stdio/src/server.rs:108` — escape tracking in the frame extractor | No fixture pairs an **escaped quote with an in-string brace**; no gqlang source in the suite contains `}`. Escape-vs-no-escape yields identical boundaries. Defensive branch (§0 R5). | Add `{"method":"v3_diagnostics","query":"a\"}b"}`; pin the response line. | Low |
| **M6** | `surface/src/family.rs:285-292` — `parse_ref`'s `::` branch | `family.rs:263` documents `repo::auth` as supported, but no surface test parses a gqlang source containing `::`. Every `::` in the tests is a Rust path or a builder argument (`v3_catalog_hellgate.rs:131-135`), not parsed input. The colon arm, single-colon acceptance and missing-segment reject never run. | Add `check repo::auth \| confirm=skip` asserting `rule_ref.name == "repo::auth"`, plus `check a: \| ...` for the reject. | Med |

---

## §4 CI steps that are structurally incapable of failing

| ID | Problem | Evidence | Action | Sev |
|---|---|---|---|---|
| **I1** | **The doc-test step compiles zero doc-tests**, and runs **three times**: `Justfile:20`, `lint.yml:41`, `lint.yml:105`. | `grep -rn '```' crates/*/src/` returns exactly **4 fence markers, all ` ```text ` prose** (`surface/src/query.rs:10,18`; `surface/src/unit.rs:10,13`). No ` ```rust ` fences and no indented Rust blocks exist. All three invocations always report 0 tests. | Add **one** real ` ```rust ` example so the step can fail, **or** delete it from `Justfile:20` and both CI sites. | **High** |
| **I2** | **`[build-dependencies]` bypass all three dependency guards** in the architecture lint. | `tools/architecture/check_architecture.py:148-155`: `dependency_kind()` returns `None` for cargo's `"build"` kind (`:152`), and the loop at `:186-188` does `if kind is None: continue`. A build-dependency on `quanta-runtime`, a legacy `quanta-gqlang-*` crate, or an undeclared `quanta-gqlang-v2-*` crate skips lines 190-205 entirely. No fixture emits a build edge. | Treat `"build"` like `"dev"` (or reject it loudly); add a build-edge fixture asserting the forbidden-prefix error. **[distinct from findings-2 §1 G3/G4, which are the stale member/layering tables]** | **High** |

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
| M9 | — | net-new; §0 R6 rescopes it. |
| M11 | §4 H2 | H2 is `graph.rs:238`; M11 is `graph.rs:347` — different functions. |
| A1 | §5.2 #45 | #45 names the line; A1 supplies the blast radius (11 tests, no golden). |
| M1, M3, M4, M5, M6, M7, M8, M10, M12, M13, A2-A13, B1, B2, B3, B5, I1, I2 | — | net-new. |

---

## §7 Fix order

1. **I1** — make the doc-test step real or delete it. Three invocations, zero coverage.
2. **I2** — close the `build`-kind bypass. One predicate + one fixture.
3. **M8** — `query --file <valid>` and `query ""`. Two tests close the CLI front-door claim.
4. **B2** — build the record shape; the `resolve_shape` Record arm is currently deletable.
5. **M1** — extend `binding_outputs_is_some_only_for_composition` to two bindings. One string.
6. **A1** — give the 11 `assert_roundtrip` tests goldens. Live silent-regression hole.
7. **M3 / B4** — pin the lane rule and the composition-local pack rebuild.
8. **B3 / M4 / M5 / M7 / M9 / M11 / M12 / M13** — one focused fixture each.
9. **A2-A12, A14, B1, B5** — per-test oracle strengthening; mechanical once the contract is named.
