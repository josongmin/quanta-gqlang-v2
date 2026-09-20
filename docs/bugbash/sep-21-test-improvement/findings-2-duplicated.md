# Findings 2 — Duplicated / Removable Tests: Final Action List

**Purpose.** This file is a *decision-ready action list*. The 4-round audit narrative (19 parallel subagents, coverage measurement, per-test duplication mapping) has been reduced to the items that actually require a change, plus an overreach review of the audit's own claims. Every item carries the evidence needed to verify it.

- **Audited revision:** `e896a11` (committed). All `file:LINE` refs are pinned there.
- **Working tree:** was **actively changing during the audit** (ticket 01/08/09 repair in flight). Delta: `crates/quanta-gqlang-v2-lsp/` → `crates/quanta-gqlang-v2-diagnostics-stdio/` (9 test files, 41 tests, 1,385 lines, byte-identical except the crate name); `crates/quanta-gqlang-v2-sema/` added (7-line intentional stub, no tests); workspace now 14 members. LSP-slice paths below are already remapped to `diagnostics-stdio`.
- **Baseline:** 90 test files, 743 `#[test]` fns, 19,715 lines; +79 in-`src` unit tests = 822 total. No `#[ignore]`, no `#[cfg]`-gated test, no `#[should_panic]`, no unused helper, no unused `mod support;` — **zero dead tests found.**
- **Relevance warning:** most non-test findings here are **already ticketed** in `docs/plans/sep-21-gqlang-sota-frontend-reconstruction/FINAL_AUDIT.md` (same base `e896a11`). Read §6 before acting.

---

## §0 Overreach review — claims this audit retracts or narrows

Each headline claim was re-checked against a fresh command. One reasoning gap, four overstatements, two wrong numbers, and several items that are **reasoned rather than executed**:

| # | Original claim | Correction | How verified |
|---|---|---|---|
| R1 | Every flagged `keyword()`/`token()` pin was described as "enums with **no production consumer**" | **Overstated.** The *enums* are used in production (e.g. `QueryKindV1`: 65 `src` references). It is the **accessor** that has no production call site. The conclusion holds; the phrasing was wrong | enumerated all **20** production `.keyword()`/`.token()` call sites in `crates/*/src` (excluding the `#[cfg(test)]` module in `types/src/ty.rs`) |
| R2 | `NoPlanReasonV1::keyword()` has no consumer "because the CLI renders a different enum" | **Conclusion right, but I had not confirmed the other enum existed.** It does: `NoServeReasonV1` lives in `pipeline/src/compile.rs:247` (not planning). `NoPlanReasonV1` (`planning/src/item.rs:53`) is never rendered | type resolution across pipeline/cli/planning |
| R3 | "`audit_edge_cases.rs`: 17 tests are bare `is_err()`" | **Imprecise.** Measured: **21 tests, 18 `is_err()` sites, exactly 1 `stage()` assertion and 1 `code()` assertion, 0 span assertions** | `grep -c` over the file |
| R4 | "`audit_18_families.rs`: delete all but f02–f08 + f09_*", later "20 deletable / 6 keep" | **Two passes disagreed on `f01` and on `f11/f13/f15`.** Honest statement: **6 tests are mandatory** (`f03`, `f04`, `f07`, `f08`, both `f09_check_semantic_confirm_*`); `f01`, `f11`, `f13`, `f15` are **borderline**; the remaining **17–20 are removable**. Range kept instead of a point estimate | per-test mapping |
| R5 | "Round 4 refuted 10 first-pass claims" | **Arbitrary and understated** — at least 17 were refuted or corrected. Number dropped; the corrections in §0/§5.5 are the substance | count of correction rows |
| R6 | "`leakage.rs` has 26 tokens; 18 enforced nowhere" | **Off by one.** **27** tokens; 8 duplicated in `check_architecture.py:108-117`; **19 enforced nowhere** | parsed `RUNTIME_TRUTH_DENYLIST_V1` |
| R7 | "4 red gates" | **Currently 3 red + 1 repaired-in-worktree.** `cargo deny`'s 6 wildcard deps were fixed during the audit; `taplo` moved 6→7 offenders; `typos` 9→6 findings | `git diff` on the manifest; re-ran both gates |
| R8 | The self-comparing span assertion "hides a live mutant" | **Reasoned, not executed.** No mutant was built (read-only mandate). Label it a hypothesis with a named candidate mutant | n/a |
| R9 | Coverage "Result 4" presented as the strongest result | **Kept only because the arithmetic checks out:** baseline 7,867 = full suite 8,091 − transport crate 224, so the baseline was a complete pre-rename suite run. Still an accumulated-profile inference, not a per-target re-run | 8,091 − 224 = 7,867 |
| R10 | "97 `assert_eq!` pairs repeat across files" | **Dropped.** The scan matched shared conventions (`Some(1)` exit codes) as well as real overlap; it was never evidence of duplication | n/a |
| R11 | "The suite was curated, not accumulated" | **Narrowed to the measurement:** no dead tests exist. No claim about authorial intent | greps + helper-usage check |
| R12 | "11 of 18 families never reach a front door" | **Narrowed:** they have **no dedicated CLI or LSP rail**; they are not unreachable | layer-matrix grep |
| R13 | "39 of 119 pipeline fns are restatements" | **Approximate** (two passes: 38 and 39). Treat as "≈1/3 of the pipeline test fns" | per-file mapping |
| R14 | "the ledger is 50/50 stale" | **Both numbers are true at different times:** 47/50 missing at `e896a11`; 50/50 after the rename moved the last 3 files | existence check, run twice |
| R15 | (inherited assumption) "the LSP `stdio_*` files are totality rails" | **Refuted and corrected:** no `proptest`, no `catch_unwind`. Only the `surface` and `pipeline` proptests assert panic-freedom — and the `surface` one has **no `catch_unwind`**, so an abort escapes | grep for `never_panic`/`catch_unwind` |

**Not retracted** (verified by command; listed so they are not re-litigated): `check_tree` has zero production callers; `FIELD_NAMES` has no production reader and appears in exactly 3 test assertions; `graph.rs:238` is unreachable; the 4 unasserted diagnostic codes; `binary_stderr_v1` appears in only 2 places; the allow-header metrics; all coverage numbers in §7.

---

## §1 Gate blockers — fix first, they mask everything else

| ID | Problem | Evidence | Action |
|---|---|---|---|
| G1 | **`taplo` fails on 7 files** and is the **first** gate `just ci` hits, so arch-lint / deny / typos never even run | `taplo fmt --check` → 6 `Cargo.toml` (algebra, conformance, frontend-contract, logic, pipeline, planning) + `.taplo.toml` | `taplo fmt` |
| G2 | **`typos` fails on 6 test-file comments** — all one false positive (a three-letter prefix hyphen-joined to a word) | `surface/tests/v3_mutation_rails.rs:87,168`, `surface/tests/v3_family_hellgate.rs:946`, `modules/tests/v3_modules_version_lock_edges.rs:50,150,155` (+2 `src` hits) | de-hyphenate the compounds, or add the prefix to `.typos.toml` `extend-words`. **This report tripped the same rule and was rewritten** — the fix is known-good |
| G3 | **`check_architecture.py` is unconditionally red and needs TWO repairs** | `EXPECTED_WORKSPACE_MEMBERS` (`:44-51`) lists 3 deleted crates; `ALLOWED_INTERNAL_DEPENDENCIES` (`:52-62`) maps cli/lsp/conformance to the deleted `-ir`. Patching only the member list still yields 3 layering errors (cli: hir/pipeline/planning/surface/types; diagnostics-stdio: pipeline; conformance: hir/pipeline/planning/surface) | re-encode **both** tables against the real 14-member DAG. Today **10 of 14 crates are outside every layering check** (only cli/conformance/sema are in scope) |
| G4 | Dead allowlist entries in the same script | `ALLOWED_DEV_EDGES` (`:71-78`) whitelists `cli→conformance` / `lsp→conformance` edges that exist in **no** manifest; `RUNTIME_TRUTH_SCAN_EXEMPT_FILES` (`:82-84`) exempts the dead `leakage.rs` | delete all three |
| G5 | `.semgrep/qglang-frontend.yml` **rule 7 can never fire** | its include paths are `/crates/quanta-gqlang-v2-{syntax,sema,ir}/src/**`, all deleted | delete or repoint the rule |
| G6 | `cargo deny` — **verify then close** | the 6 wildcard path deps in `frontend-contract/Cargo.toml:15-20` were fixed during this audit | run `cargo deny check`; no code change needed |
| G7 | Add a **member-parity rail** so G3 cannot silently rot again | the arch script is the only place the crate list is duplicated | one test asserting `EXPECTED_WORKSPACE_MEMBERS == cargo metadata` members |

> G1–G6 are the same set already recorded as **F03 (P0)** with owner tickets 01/09 — this table adds the exact repair scope, not a new finding.

---

## §2 Correctness / user-visible — net-new, nothing else owns these

| ID | Problem | Evidence | Action |
|---|---|---|---|
| **C1** | **The root README still advertises six deleted LSP methods**, contradicting `docs/language/README.md:88-93` in the same repo | `README.md:58` → ``supported read-only methods: `hover`, `completion`, `symbols`, `diagnostics`, `zero_query`, `quickfix` ``. Authoritative: `diagnostics-stdio/src/server.rs:23` = `["v3_diagnostics"]`. **Still present in the worktree** | replace with `v3_diagnostics`; extend `method_allowlist_doc_gate.rs` (or add a gate) to read the **root** README — today it reads only `docs/language/README.md`, which is why the drift survives |
| **C2** | **Every file-inventory doc in the repo is unguarded** — this class broke live during the audit | the `lsp`→`diagnostics-stdio` rename silently broke **7 of the 42** rail paths in `docs/language/README.md`; nothing detected it (found and repaired by hand). Same class: `test_hellgate_ledger.md` (**50/50** refs now missing), the example list | add **one** test that extracts every `crates/.../tests/*.rs` path mentioned in `docs/language/README.md` (and the ledger, if kept) and fails on a missing file. **Highest-leverage single item in this report** |
| C3 | `examples/` is an **orphan corpus** with a phantom rail in its docs | 18 `*.qglang` files; zero test or `src` references; the ledger advertises `examples_release_smoke.rs` as the 17-path doc gate — **that test does not exist**; the root README also claims a "CLI shipped-example matrix" that does not exist | add one loop-rail over `examples/**/*.qglang` (parse + compile + `fmt` idempotence + list/count vs both READMEs), **or** delete the corpus and its doc listings. Do not leave it as-is |
| C4 | 18 vs 17 example drift | `docs/language/README.md:95-116` lists 18; root `README.md:43-46` and `docs/language/examples_index.md` list 17 — `examples/composition/let_with_find.qglang` is missing from both | add the path; C2's rail prevents recurrence |
| C5 | Delete the stale ledger | `docs/language/test_hellgate_ledger.md` references the deleted `-syntax`/`-sema`/`-ir` crates and 50 non-existent files; its anchor `#proof-rails-test-hellgates` is broken (real heading: `## Proof Rails`). Linked from `README.md:92` | delete it and repoint `README.md:92` to `docs/language/README.md#proof-rails` (that list is 42/42 valid) |
| C6 | `test_repro_cases.txt` is committed debris the hygiene guard cannot see | tracked since `7087a9d`; the guard `current_head_artifact_hygiene.rs:42-53` checks `.rs/.rlib/.rmeta/.o/.d/.sh/.bash/.bak/.orig/~` — **not `.txt`**; `.gitignore` covers `/test_*.rs` but not `test_*.txt` | `git rm`; add `/test_*.txt` to `.gitignore`; extend the guard predicate |

> C1 / C3 / C4 / C5 are covered in substance by **F14 (P1)** ("docs and proof ledgers overclaim current behavior"). **C2's guard and the ledger's 100% staleness are net-new.**

---

## §3 Dead code and dead data

| ID | Problem | Evidence | Action |
|---|---|---|---|
| **D1** | **`conformance/src/leakage.rs` (130 lines + 4 unit tests) has zero consumers** — and the invariant it was written for has **no passing enforcement at all** | `pub mod leakage;` at `src/lib.rs:24`; no crate depends on `quanta-gqlang-v2-conformance` (normal or dev); no reference outside the file. History: `1164655` added it, `7d3ab6f`+`ad2317d` wired it into two sweeps (440 lines), then `39e2e6d` **deleted both sweeps and left the fixture**. 27 tokens; 8 duplicated in `check_architecture.py`; **19 enforced nowhere** | **preferred:** restore one output-sweep rail per front door (the READMEs promise "runtime truth is never asserted as real"; nothing currently checks it). **or:** delete the module + `pub mod leakage;` + the exempt-file and dev-edge entries, and correct the docs advertising the deleted sweeps |
| D2 | `SupplementalSurfaceUnitSpecV1.unit` is **write-only** | `conformance/src/lib.rs:217`; only used as `{:?}` in panic text (`self_validation.rs:142,150,157,163`) | assert it, or delete the field |
| D3 | `CatalogV1.active_packs` is write-only, so its "rail" asserts nothing | `modules/src/catalog.rs:185` is `let _ = self.active_packs.insert(...)`; no accessor; `v3_catalog_hellgate.rs::use_pack_known_manifest_resolves` (`:308-321`) asserts only `.unwrap()` | expose + assert the activation, or drop the field. Weakest test in the suite |
| D4 | Test-only public accessors (no production caller) | verified: `PackGraphV1::module_count()` (`graph.rs:401`), `CatalogV1::asset()` (`catalog.rs:199`), `CatalogV1::is_empty()` (`:211`), `CatalogV1::certified_required()` (`:193` — the live check reads the field at `:293`), `LexicalScopeV1::depth()` (`binder/scope.rs:142`) | delete accessor + its test, **or** mark as intentional public API |
| D5 | `keyword()`/`token()` pins whose **accessor** has no production call site | `SortDirectionV1::keyword()` (`algebra/node.rs:497`), `NoPlanReasonV1::keyword()` (`planning/item.rs:67`), `FamilyKindV1::keyword()` (`conformance/lib.rs:104`), `DedupIdentityV1`/`WitnessTieBreakV1` (`logic`), `ExistencePolarityV1::keyword()` (`logic/source.rs:33`), `QueryKindV1::token()`/`SeverityClassV1::token()` (`modules/metadata.rs:26,52` — **zero** `.token()` call sites in `src`) | delete the spelling pins (keep the `from_keyword` round-trips). The legitimate ones — rendered by `cli/src/v3_plan.rs` — are listed in §0 R1/R2 and must stay |
| D6 | Test-file naming / doc drift | `_mutants` and `_mutation_rails` are the same purpose under two names; no doc defines any prefix; `v3_logic_punchlist.rs:11` says "Three" but lists two; `v3_algebra_probe.rs:9` calls itself punch-list; both `audit_*.rs` use `//` not `//!`; `v3_cost_classification.rs:12-13` misattributes `keyword()` to `plan.rs`; `logic/tests/v3_logic_source.rs:76` line pointer off by one; **23 test files** carry the `//! syntax->sema->ir path pinned:` header with deleted crate names; `cli/tests/v3_cli_lsp_parity.rs` still says `lsp`; 8 files carry stale "not built yet" / pre-cutover comments | normalize or document the prefixes; fix the doc claims; reword the stale comments |

> D1 is the same dead file referenced by G4. `check_tree` / PROD-02 is **F09** — an open design decision, see §8.

---

## §4 Real coverage holes (net-new; all confirmed by command)

| ID | Hole | Evidence | Action |
|---|---|---|---|
| **H1** | 4 of 68 diagnostic codes are **asserted in zero test files** | `algebra.ill_formed_aggregate.empty_aggregate_projection`, `logic.graph_endpoint_unexpected`, `logic.non_monotone_recursion.negated_recursion`, `modules.unknown_module` — each: 0 files | add one negative fixture per code. `logic.graph_endpoint_unexpected` is already *reached* by `test_endpoint_conflict.rs`, which only `println!`s the error — assert the code there instead |
| **H2** | `modules/src/graph.rs:238` (`origins.len() > 1`) is **unreachable by any test** | module iteration is sorted (`graph.rs:306`), so in the only ambiguity test the re-exporter `facade` sorts before the importer `query` and the error fires at the *other* check `graph.rs:313` first; the test claiming to pin `:238` does not | add a fixture where the importer sorts first, **then** delete the redundant assertion |
| **H3** | Surface FIND keyword → `GraphQueryKindV1` mapping is **unpinned for `Flow` / `PointsTo` / `Summary`** | the format rails are stability-only (`format(parse(x)) == format(parse(format(parse(x))))`), so `"flow" => Taint` survives; only `Slice` is pinned at parse level | add `assert_eq!(gq.kind, GraphQueryKindV1::Flow)` etc. to the surface subkind rails |
| **H4** | The "ordered diagnostics" claim is **vacuous**, and multi-diagnostic order is untested | `diagnostics-stdio/src/handlers/v3_diagnostics.rs:66-67` = `Ok(_) => Vec::new()` / `Err(d) => vec![d]`; `pipeline/src/diagnostic.rs:11-12` says exactly one is carried; `compile.rs:1029` takes `.first()` with no >1 test | add a binder fixture producing two diagnostics and pin the order; stop describing the 4 LSP rails as ordering coverage |
| H5 | No CLI rail for `modules.unknown_pack_manifest`; no LSP fixture for `binder.unresolved_symbol` / `types.bad_overload` / `logic.unbounded_witness.zero_path_length_bound`; **11 of 18 families have no dedicated front-door rail** | grep-confirmed | add these **together with** the §5 deletions, so front-door coverage does not shrink |
| H6 | `check_tree` has **zero production callers** — ~40 test fns guard an orphaned public API | `hir/src/integrity.rs:208` + `hir/src/lib.rs:80`; no non-test call site (only doc comments). Rails in `hir/tests/*` and `surface/tests/*` (`program()` helper) | decide (§8): wire it into the production seam, or document it as downstream-only. Until then those rails cannot fail for a production regression |

---

## §5 Test-suite cleanup — verified duplicates

**Rules.** (a) Every row names the test that already fails if the same production behavior changes. (b) Round-4 mutant falsification **refuted** several first-pass calls; those are in §5.5 and must **not** be deleted. (c) Never delete the coupled candidates of §5.4 as a group.

### 5.1 Verified removable (delete or merge)

| # | Delete | Covered by |
|---|---|---|
| 1 | `pipeline/tests/v3_normalize_item_decl_rule.rs::decl_closed_body_type_error_rejects_at_type` | `v3_diagnostic_surface.rs::type_diagnostic_exposes_inner_code_and_displays` (same source, same code) + `::span_is_inner_source_offsets_not_default_placeholder` |
| 2 | `pipeline/tests/audit_18_families.rs::f23_require_certified_meta` | conformance `lib.rs:378` row via `self_validation.rs:74-113` (adds `result_kind` + `effect`) |
| 3 | `pipeline/tests/audit_edge_cases.rs::require_certified_no_args` | same as #2 |
| 4 | conformance `USE PACK` example row (`lib.rs:370-373`) — repoint or dedupe | `pipeline/tests/v3_use_pack_authority.rs::use_pack_without_pack_authority_fails_closed_unknown_manifest` (same source and code, **plus** stage) |
| 5 | `binder/tests/v3_binder_mutants.rs::shadow_search_reports_outer_shadow_when_visible` | `v3_binder_hellgate.rs::shadowed_binding_rejects` |
| 6 | `binder/tests/v3_binder_mutants.rs::let_visible_until_scope_closed` | `v3_binder_scope.rs::let_binding_resolves` |
| 7 | `hir/tests/v3_item_hellgate.rs::nested_item_ids_are_globally_unique_through_one_builder` | same file `::composition_nests_query_producer_and_find_body` (`assert_sound(..., 4)`) |
| 8 | `hir/tests/v3_hir_expr_kernel.rs:297-304` — `is_binary_plus(...)` type property (a compile-time fact) | trim 8 lines; keep the two `check_tree` unwraps |
| 9 | `logic/tests/v3_logic_punchlist.rs::witness_bind_rejects_zero_path_length_bound` | `v3_logic_hellgate.rs::cyclic_zero_path_length_bound_rejects` (same shape + message + span) |
| 10 | `logic/tests/v3_logic_hellgate.rs::unknown_quantifier_rejects` | `v3_logic_source.rs::unknown_quantifier_rejects_with_typed_code` (the wrapper delegates) |
| 11 | `logic/tests/v3_logic_recursion.rs::negation_chain_converges_regardless_of_declaration_order` | `::deep_negation_chain_layers_strata_monotonically` — **the test's premise is false**: `recursion.rs:286/:301/:349` erase declaration order |
| 12 | `logic/tests/v3_logic_recursion.rs::acyclic_view_composition_admits_no_recursion` | `::self_loop_is_recursive_but_singleton_base_is_not` + `::positive_transitive_closure_admits_with_proof` |
| 13 | `logic/tests/v3_graph_query_depth_waypoint_conflict.rs::taint_depth_only_still_admits` (keep one of the pair) | `v3_graph_query_hellgate.rs::bounded_taint_admits_with_witness_contract` |
| 14 | `modules/tests/v3_modules_mutation_rails.rs::validate_accepts_value_inside_allowed_set` | `v3_modules_packs.rs::agreeing_model_packs_compose`. **Keep** the rejects sibling at `:99` |
| 15 | `modules/tests/v3_modules_mutation_rails.rs::reexport_with_two_distinct_origins_is_ambiguous` | `v3_modules_hellgate.rs::ambiguous_import_two_reexports_rejects` — but do **H2** first |
| 16 | `modules/tests/v3_modules_version_lock_edges.rs::verify_against_accepts_exact_lock` | `v3_modules_packs.rs::manifest_and_lock_round_trip` |
| 17 | `modules/tests/v3_modules_version_lock_edges.rs::verify_against_rejects_version_drift` | `v3_modules_hellgate.rs::lock_mismatch_rejects` |
| 18 | `surface/tests/v3_family_edges.rs::find_through_and_sanitizers_empty_reject` | `v3_family_hellgate.rs::find_empty_list_clauses_reject` (same branch, asserts message **and** span) |
| 19 | 3 round-trip input lines in `v3_format_edges.rs` (`:98`, `:110`, `:207`) | byte-identical to `v3_format_hellgate.rs:85/:91/:74`, whose helper is a strict superset |
| 20 | `cli/tests/v3_cli_fmt_idempotence.rs::v3_fmt_preserves_string_literals_idempotently_v1` | property by `v3_format_edges.rs::string_escapes_round_trip`; CLI plumbing by `v3_fmt_is_a_fixed_point_on_messy_advanced_inputs_v1` |
| 21 | `diagnostics-stdio/tests/stdio_json_escape_negative.rs::session_recovers_after_a_malformed_escape_v1` | `stdio_json_object_negative.rs::malformed_object_does_not_poison_session_v1` — same `Err`-then-continue arm |
| 22 | `diagnostics-stdio/tests/stdio_command_plane_hellgate.rs::rejected_frame_does_not_poison_session_v1` | same arm; envelope already asserted by `non_allowlisted_methods_fail_closed_v1` |
| 23 | `diagnostics-stdio/tests/stdio_byte_robustness_hellgate.rs::session_drains_high_count_glued_frames_in_process_v1` | its strictly-stronger binary sibling `::binary_drains_high_count_glued_frames_v1` (coverage differs by 4 lines) |
| 24 | `planning/tests/v3_planning_truth.rs::recursive_witness_shape_is_path_witness_with_explicit_selection` | every assertion maps elsewhere (result_kind ← `v3_item_hellgate.rs:85`; render ← `:87`; selection ← `v3_cost_classification.rs:494`; cardinality ← `:411`; recursion ← `:447`; witness ← `:516`; capability ← `:419`). **Largest single win** |
| 25 | `planning/tests/v3_planning_cost_schema.rs::there_are_exactly_five_cost_fields` | implied by the verbatim array at `:19` |
| 26 | `planning/tests/v3_cost_classification.rs::union_takes_multiplicative_over_bounded` (`:144`) | `:338` same arm/branch; the extra input is pinned by `:384` (**keep `:384`**) |
| 27 | `planning/tests/v3_cost_classification.rs::acyclic_all_witness_is_bounded_set` (`:502`) | `:485` — strictly stronger (also asserts `witness_selection`) |
| 28 | `planning/tests/v3_cost_classification.rs::result_kind_keywords_are_stable` (`:641`) | CLI rails exercise all four via `keyword()` |
| 29 | `planning/tests/v3_item_hellgate.rs::gate_declaration_is_no_plan_marker` | `::view_declaration_is_no_plan_marker` (payload ignored at `item.rs:105`) |
| 30 | `planning/tests/v3_item_hellgate.rs::sample_classifies_to_table` | `::show_classifies_to_table_read_only` — one arm (`item.rs:207`) |
| 31 | `planning/tests/v3_explain_provenance.rs::from_root_fixpoint_sets_recursive_flag` | `::produced_plan_carries_recursive_explain_note` |
| 32 | `types/tests/v3_types_check_edges.rs::path_projection_unknown_field_on_record_rejects` | `v3_types_hellgate.rs::bad_path_projection_rejects` (same record/field/spans + full Display) |
| 33 | 2 rows inside `v3_types_universe.rs::null_is_assignable_into_optional` | `types/src/ty.rs:619/:626`; keep the unique `Null→optional<int>` row |
| 34 | `algebra/tests/v3_algebra_mutants.rs::sort_direction_keywords_are_exact_stable_strings` | accessor has no production call site (§0 R1) |
| 35 | `planning/tests/v3_item_hellgate.rs::no_plan_reason_keywords_are_stable_verbatim` | idem |
| 36 | `types/tests/v3_types_check_edges.rs::scalar_free_helper_matches_typev1_scalar` | `check.rs:384-386` is literally `TypeV1::scalar(s)` — `f(x) == f(x)` |
| 37 | `pipeline/tests/audit_18_families.rs` — **17–20 of 26 tests** | conformance registry (kind + `result_kind` + `effect`), CLI surfacing rail, `v3_uncovered_branches.rs`, `v3_composition_type_env.rs`. **Keep the 6 mandatory** (§0 R4) |
| 38 | `pipeline/tests/audit_edge_cases.rs` — **20 of 21 tests** (18 `is_err()` sites, 1 `stage()`, 1 `code()`, 0 spans) | `surface/tests/v3_family_edges.rs:45-304` owns every clause-omission reject with code+message+span; the stage taxonomy is covered by `cli/tests/v3_cli_item_surfacing.rs`. **Prefer strengthening over deleting** — see §8.4 |
| 39 | `diagnostics-stdio/tests/stdio_command_plane_hellgate.rs` `REJECTED_CASES_V1`: trim 19 rows → 2–3 | all 19 hit one catch-all arm (`server.rs:186`) — **but the rows are the only byte-exact pin of `/` passthrough** through `json_string_v1` (`json.rs:251`). Keep ≥1 `/`-bearing row plus ≥1 `-`/uppercase row, or keep #22 |
| 40 | `pipeline/tests/v3_alias_type_closure_gap.rs` — 2 of 3 tests | test 1 ← `v3_frontend_context.rs::context_first_compile_closes_alias_typing_explicitly`; test 2 restated by test 3 in-file |
| 41 | `planning/tests/v3_planning_cost_schema.rs` — the file's remaining test `fixed_cost_field_names_are_stable_verbatim` (after #25), i.e. the whole 35-line file | `FIELD_NAMES` is also pinned verbatim at `pipeline/tests/v3_pipeline_compile.rs:42` — see §5.4 coupling: keep exactly one of the two |

### 5.2 Tautological / cannot-fail assertions

| # | Fix | Note |
|---|---|---|
| 42 | `v3_diagnostic_surface.rs:53` `assert_eq!(diag.span(), diag.span())` | **replace, don't delete:** `assert_eq!(diag.span(), SpanV1::point(25))` (or `assert_ne!(…, SpanV1::default())`). Candidate mutant `diagnostic.rs:137 Self::Parse(inner) => inner.span → SpanV1::default()` (reasoned, not executed) |
| 43 | `v3_surface_proptest.rs:429` `prop_assert_eq!(lex(&s), lex(&s))` + the 3 determinism proptests | delete `lex_is_deterministic`; decide consciously on the other three. This is the **second** self-comparing assertion — an `assert_eq!`-only scan misses it |
| 44 | 12 × `prop_assert!(r.is_ok() \|\| r.is_err())` in `v3_surface_proptest.rs` | delete the expression (keep the call — panic-freedom is real); note `r` becomes unused |
| 45 | `v3_format_hellgate.rs:59-62` (`same_structure`), `:64-66` (third pass), `v3_format_edges.rs:39-43` | the repo's own plan forbids formatter-output-as-oracle (`tickets/02-…:235,:267`); delete the restatements |
| 46 | Both doc gates | `command_doc_gate.rs:36-45` and `method_allowlist_doc_gate.rs:32-49` pass **vacuously** if their const is emptied — add a non-vacuity assert (they remain the only const↔usage↔README guards) |
| 47 | `conformance/tests/self_validation.rs:36-44` | the `BTreeSet` uniqueness check can never fail (duplicate variants are a compile error) |

### 5.3 Scaffolding (mechanical, ~425 duplicated lines)

| # | Fix | Payoff |
|---|---|---|
| 48 | Add per-crate `tests/support/mod.rs` in this order: `planning` (`id, span, span_at, col, lit, pred, node, node_at, scan, fixpoint_proof`), `logic` (`sp, id, gref` + import bundle), `algebra` (`sp, seg, alias`), `modules`, `pipeline`, `surface` (`parse, same_structure`) | **planning 142 / logic 48 / algebra 33** redundant lines; 24 helper bodies duplicated ≥2× = **325 lines**, of which **307 are intra-crate** |
| 49 | Replace the 5 hand-rolled `SurfaceItemV1 -> ProgramItemHirV1` matches with the existing `SurfaceItemV1::into_program_item()` (`surface/src/parser.rs:55,66`) | `v3_family_hellgate.rs:31-36`, `v3_document_hellgate.rs:20-27`, `v3_format_hellgate.rs:25-30`, `v3_format_edges.rs:23-28`, `v3_mutation_rails.rs:211-216` |
| 50 | Hoist the byte-identical `CLEAN_QUERY_V1` (×4) / `CLEAN_QUERY_LINE_V1` (×3) into `diagnostics-stdio/tests/support` | 8 lines |
| 51 | Canonicalize the `#![allow(...)]` header: **91 of 92 files**, 136 attributes, **50 distinct reason strings**, 514 lines | one 6-line header per crate. **Do NOT delete them:** CI runs `cargo clippy --workspace --all-targets -D warnings` (test targets included); only `just clippy` excludes tests. Cargo has no target-specific `[lints]` (RFC 3389 defers it) |
| 52 | Merge the cli/diagnostics-stdio support *logic* (keep the per-package `env!("CARGO_BIN_EXE_…")` lookup — pass the path in, or export a `bin_exe!` macro) | ~15 lines; the decoders are byte-identical today |

### 5.4 Coupled candidates — never delete as a group

| Coupling | Why |
|---|---|
| `v3_planning_cost_schema.rs` **and** `v3_pipeline_compile.rs` | together they hold 3 verbatim `SemanticCostV1::FIELD_NAMES` assertions (`:21`, `:34`, `:42`). One may go — **not both**. The const has no production reader (§5.6), so these are its only guard |
| `conformance/tests/registry_non_vacuous.rs` tests 1–2 | the **only** kill for `root_family_specs_v1() → Vec::new()` / `supplemental_surface_unit_specs_v1() → Vec::new()`: `self_validation.rs:34/:129` assert the **const arrays**, not the function outputs. Making the fn `const` is not free (`Iterator::collect` is not const on 1.81) |
| `logic/tests/test_endpoint_conflict.rs` | the only rail for `logic.graph_endpoint_unexpected`, and it asserts nothing but `Err`. **Strengthen it (H1), don't delete it** |
| `pipeline/tests/audit_18_families.rs` | holds the only end-to-end compiles of `find flow`/`slice`/`points_to`/`summary` and both CHECK-with-`using find` rails |
| `diagnostics-stdio/tests/stdio_frame_edges.rs` | holds the LSP suite's **only** clean-run stderr assertions: `binary_stderr_v1` appears at exactly `:128` and `:168` in the whole tree, and `:168` is slated for deletion. **Collapse the 5 binary mirrors into one parameterized test that KEEPS `assert!(binary_stderr_v1(&out).is_empty())`** |
| `diagnostics-stdio/tests/stdio_frame_extractor_edges.rs` | keep only cases 1–2 (non-`{` reject; brace depth ≥2 — in-crate unit tests only reach depth 1). **Case 3 is not a rail:** it pins the whole-buffer-clear behaviour, which is **F36 (P1)** and confirmed-deletion **#5** in the reuse audit |

### 5.5 KEEP — refuted as deletions (do not remove)

| Item | Why it must stay |
|---|---|
| `v3_family_hellgate.rs::find_duplicate_endpoint_clause_rejects` (`:679`) | the repo's **only** non-depth duplicate-key input; a key-restricted guard survives without it |
| `v3_family_hellgate.rs::find_duplicate_list_clause_rejects` (`:686`) | idem (duplicate `via=`) |
| `v3_family_hellgate.rs::find_empty_list_clauses_before_next_clause_reject` (`:706`) | the **only** input where an empty-list key is followed by `\|` instead of EOF |
| `v3_family_edges.rs:230-233` (profile full sentence) | the hellgate only `.contains("capabilities")` |
| `v3_family_edges.rs:246` (gate full sentence) | the hellgate only `.contains("checks")` |
| `v3_format_edges.rs` enum-arm inputs | owns unique literal enum arms (`mode=skip/prompt/require`, `graph-ml`, all 7 show categories, direction/through/window) — 0 removable by deletion, ~13 by merge into the hellgate helper |
| `v3_family_hellgate.rs:550-660` (10 per-family duplicate-clause tests) | each pins a **separate call site** (`family.rs:608,722,852,932,1001,1097,1147,1201,1274,1370`) |
| `v3_surface_hellgate.rs::record_literal_duplicate_field_rejects` vs `v3_surface_query.rs::duplicate_result_shape_field_is_rejected` | two distinct sites (`expr.rs:359` vs `shape.rs:45`) |
| `cli/tests/v3_cli_lsp_parity.rs` vs `diagnostics-stdio/tests/v3_diagnostics_parity_hellgate.rs` | complementary: CLI-binary↔in-process vs pipeline↔spawned-binary; each pins a key the other does not |
| `binder/tests/v3_binder_mutants.rs` (10 of 13) | uniquely pin `scope.rs:192`, `resolve_rule`, `resolve_presentation`, `resolve_shape`, `close_scope`, `module_graph` len/is_empty |
| `modules/tests/v3_modules_mutation_rails.rs` (9 of 11) | unique branches: `validate` `None` vs outside-allowed-set, `model_conflict` first-seen order, `pack_graph` counts |
| `algebra/tests/v3_algebra_mutants.rs` (5 of 6) | fully disjoint from its hellgate |

### 5.6 Production-side duplication

| # | Fix | Evidence |
|---|---|---|
| 53 | Make `render_cost_lines_v1` read `SemanticCostV1::FIELD_NAMES` (or assert equality once at the CLI boundary) | `planning/src/cost.rs:156` declares the const as the drift guard; `cli/src/v3_plan.rs:232-253` hardcodes the same 5 names. The const has **no production reader**, and the CLI test's "a rename downstream breaks this rail" comment is **false** |

---

## §6 Already owned by tickets — do not redo

From `docs/plans/sep-21-gqlang-sota-frontend-reconstruction/FINAL_AUDIT.md` (base `e896a11`) and `IMPLEMENTATION_REUSE_AUDIT.md`:

| This report | Existing ID | Owner |
|---|---|---|
| §1 gates (taplo/arch/deny/typos/semgrep) | **F03** (P0) | tickets 01, 09 |
| H6 + `check_tree` orphan | **F09** (P1) | tickets 02, 04, 07 |
| C1, C3, C4, C5 (stale methods / examples / ledger) | **F14** (P1) | tickets 01, 09 |
| §5.1 #37 and the layer-ownership conclusion | **F29** (P2, "18-family conformance is routing smoke") | tickets 01, 09 |
| D5 + #53 (dead metadata / unread const) | **F31** (P0) | tickets 04, 07 |
| §5.4 stdio case 3 (whole-buffer-clear) | **F36** (P1) + reuse-audit deletion #5 | ticket 08 |
| The **entire `lsp` transport slice** (#21–23, #39, all §5.4 stdio rows) | reuse-audit lane 08: "nine old `lsp` transport tests — tracked-move and rewrite the byte cases for **NDJSON**; delete object-concatenation expectations and old names in the same cut" | ticket 08 — **these findings have a short shelf life; re-derive after the NDJSON cut** |
| #37 / #38 (fold the `audit_*.rs` files into the registry) | reuse-audit lane 09 | ticket 09 |

**Net-new in this report** (absent from F01–F37): the per-test-fn duplicate mapping (§5.1), the coverage measurements (§7), the scaffolding metrics (§5.3), H1–H5, C2's guard, D1–D4, D6, and the falsification record (§5.5).

---

## §7 Measurements kept as evidence (not action items)

- **Coverage** (measured with `cargo-llvm-cov`, `RUSTUP_TOOLCHAIN=stable`). Full suite covers **8,091** `crates/*/src` lines. Independent per-target: `self_validation` 2,894 · `audit_18_families` 2,339 · `audit_edge_cases` 1,484 · `stdio_frame_edges` 1,279 · `stdio_byte_robustness_hellgate` 1,276 · `v3_family_edges` 817.
  - **No target is a coverage subset of another** (unique contribution vs the other five: 315 / 103 / 72 / 34 / 7 / 4) → single-target-subset deletion arguments are invalid; justify by assertion strength (§5).
  - **At suite level the 4 non-transport targets add 0 lines**: an accumulated profile of 7,867 — a complete pre-rename suite run, verified as 8,091 − 224 transport lines — did not grow when each was added. Deleting any of them costs **no** source-line coverage; the only question is which assertions are lost.
  - The **strongest coverage-level duplication is the stdio pair** (7 / 4 lines apart, ≈99.6 % identical), not pipeline↔conformance.
  - Only 6 of 90 targets were measured individually; no full 90-target matrix exists.
- **No executed mutation testing.** Every "safe to delete" is a hand-constructed mutant argument, not a run. Running `cargo-mutants` would convert §5 into measurement — the highest-value follow-up.
- **Scaffolding:** 24 helper bodies duplicated ≥2× = 325 redundant lines (307 intra-crate); 77 duplicate `use` lines; 8 duplicate const lines; 514 allow-header lines across 91/92 files. The assumed verbatim `//! v3 … rail` prologue duplication **does not exist** (only 4 doc lines repeat, twice each).
- **Cross-crate input duplication is low:** only 7 surface-source literals appear in tests of 2+ crates.
- **Clean surfaces:** no `#[ignore]` / `#[cfg]`-gated / `#[should_panic]` tests; no `benches/`, `fuzz/`, `[features]`, or `proptest-regressions/`; no test spawns a process; exactly one test touches the filesystem (read-only); **0 doctests**, so `cargo test --doc` passing is vacuous.

---

## §8 Open decisions (need a human, not a patch)

1. **`check_tree` (F09/H6):** wire it into the production seam that accepts hand-built HIR, or declare it a downstream-only API contract. Until decided, ~40 test fns are mislabelled as behavior rails.
2. **`leakage.rs` (D1):** restore the two output sweeps (the READMEs' "never asserts runtime truth" promise currently has no mechanism, and 19 of 27 tokens are enforced nowhere), or delete the module and correct the docs.
3. **`examples/` (C3):** add a rail, or delete the corpus. Deleting contradicts three documents; keeping it unrailed contradicts "parser-stable".
4. **`audit_edge_cases.rs` (#38):** strengthen its 18 `is_err()` assertions into code+span assertions, or keep it as an explicitly-named smoke file. It is a suite-level coverage subset whose unique lines are weakly asserted — neither fact alone justifies deletion.
5. **Test-only accessors (D4):** delete accessor + test, or declare them intentional public API.
6. **`sema` stub:** intentionally empty (documented). Confirm it needs no rail before the semantic-cutover ticket lands.
