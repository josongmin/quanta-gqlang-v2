# Findings 1 — Non-Falsifiable / Unasserted Tests: Final Action List

**Purpose.** This is a *decision-ready action list* for the half of the test-quality problem that
`findings-2-duplicated.md` does not own: assertions that **cannot fail**, tests whose **name
promises a contract the body never checks**, **unreached production branches**, and **CI steps
that are structurally incapable of failing**. It complements findings-2 rather than restating it —
see the cross-reference table in §7 before acting on anything here.

- **Audited revision:** frozen snapshot `f0fc3dce0f672d7d` (working tree at ~03:00, post `lsp` →
  `diagnostics-stdio` rename). Every `file:LINE` below was **re-verified against the live working
  tree at write time**; two citations had drifted and are corrected here (`types/src/op.rs`
  202 → 204-205; json.rs offsets).
- **Working tree moved twice during the audit.** First the `lsp` → `diagnostics-stdio` rename
  (02:51–02:58). Then again while this report was being written: `hir/src/expr.rs`,
  `hir/src/relation.rs`, `cli/src/main.rs`, `diagnostics-stdio/tests/*`, `tools/lint/**`,
  `Justfile`, `.github/workflows/lint.yml`, `Cargo.toml`, and two crate manifests all changed.
  Line numbers here are valid as of the current tree; re-verify if the tree moves again.
- **Baseline:** 810 `#[test]` fns across **101 files** — 92 integration files **plus 11 in-`src`
  `#[cfg(test)]` modules that no prior audit examined**. The machine-readable per-test inventory
  ships beside this file at `test-suite-inventory.tsv` (one row per test: file, line, name,
  assertion count, body size, mechanical flags).
- **Relevance warning:** the overlap with findings-2 is deliberate and bounded — §7 lists every
  item that findings-2 already owns so it is fixed once, in one place.

> ### ⚠️ SUPERSEDED AS AN ACTION LIST — historical narrative only
>
> The tree moved again after this was written (HEAD is now `ecc38f5` + dirty; the concurrent
> modules-v2 session also committed this file). Do **not** execute from this document.
>
> - **Execute from `findings-1-duplicated.md`** — it carries the re-baseline banner (978 test fns
>   / 110 files at `ecc38f5`, versus 810 / 101 below), marks the items the concurrent session
>   already actioned (the 12 tautologies, `lex(&s)==lex(&s)`, the proptest trim, **I2**) and the
>   items whose target code the modules-v2 rewrite deleted (**M7, M9, M10, M11**), and re-anchors
>   the drifted lines (A1, B2, M4, M6).
> - `test-suite-inventory.tsv` has been **regenerated at `ecc38f5`** and now includes a `bodyhash`
>   column; regenerate with `python3 inventory_scan.py <repo-root> out.tsv`. The counts in §0 below
>   are the **snapshot** figures, retained deliberately as the audit's evidence.
> - Citations in this file that name a bare filename (`parser.rs`, `graph.rs`, `plan.rs`,
>   `capability.rs`, `main.rs`, `lint.yml`) are prose abbreviations; the crate context is given in
>   the same row. The action list resolves them to full paths.

---

## §0 Exhaustive accounting — the measurement no other report has

Every one of the 810 test functions was classified mechanically (length-preserving
comment/string blanking, brace-matched bodies; see `test-suite-inventory.tsv`). This is the
backbone that makes the rest of the report checkable: nothing is sampled.

| Class | Count | Note |
|---|---|---|
| Test fns total | **810** | 101 files |
| **Contain no assertion at all** | **84** | body has no `assert*`/`panic!`/`unwrap_err`/`expect_err` |
| Only "check" is `.unwrap()`/`.expect()` | **84** | `UNWRAP_AS_ONLY_CHECK` |
| **No exact oracle** (`assert_eq!`/`assert_ne!`/`panic!`/`unwrap_err`) | **50** | every assertion is a predicate |
| **`matches!(.., X { .. })`** — payload discarded | **41** | the variant tag is asserted, the payload named in the test is not |
| Every assertion is a weak predicate | **38** | `is_err()`/`is_ok()`/`is_some()`/`is_none()` only |
| Tautology `assert!(r.is_ok() \|\| r.is_err())` | **12** | all in one file |
| `let _ =` in a test body | 3 | |
| **Duplicate-body groups** (identical once literals are blanked) | **40 groups / 130 tests** | 16% of the suite is one fixture literal apart |

**Hard negatives confirmed:** 0 `#[ignore]`, 0 `#[should_panic]`, 0 `assert!(true)`, 0 empty test
bodies, 0 sleeps/timing dependence, 0 duplicate test-fn names, 0 unused `mod support;`.

> **Reconciliation with findings-2.** findings-2 §"Baseline" states *"zero dead tests found"*. That
> is true under its definition (no test is unreachable, unused, or ignored). It is **not** in
> tension with the 84 no-assertion tests above — those tests *run and can fail*, they just cannot
> fail **for the reason their name gives**. Both statements are correct; do not let either one
> close the other.

### 0.1 The three largest duplicate-body groups (largest single consolidation win)

| Group | Size | Members (all identical once the fixture literal is blanked) |
|---|---|---|
| `pipeline/tests/audit_edge_cases.rs:52` | **10** | `find_unknown_subkind_rejects`, `sample_zero_limit_rejects_parse`, `export_missing_kind_rejects_parse`, `composition_undefined_with_parse_guard`, +6 |
| `surface/tests/v3_family_hellgate.rs:550` | **10** | `check_duplicate_confirm_clause_rejects`, `explain_duplicate_evidence_clause_rejects`, `sample_duplicate_mode_clause_rejects`, `compare_duplicate_metric_clause_rejects`, +6 |
| `pipeline/tests/audit_edge_cases.rs:78` | **7** | `check_missing_confirm_rejects_parse`, `compare_missing_candidate_rejects_parse`, `profile_missing_admission_rejects_parse`, `use_pack_missing_lockfile_rejects_parse`, +3 |

These are **not** "similar tests" — they are the *same test* with a different fixture and one shared
weak oracle. findings-2 §5.1 #37/#38 reaches the same conclusion for `audit_18_families.rs` and
`audit_edge_cases.rs` by per-test mapping; this table gives the mechanical proof and extends it to
`v3_family_hellgate.rs`, which findings-2 does not flag.

---

## §1 Assertions that cannot fail

Rows marked **[net-new]** are not in findings-2. Rows marked **[see findings-2 §x]** must be fixed
there, not here — they are listed so the class is complete.

| ID | Location | Problem | Action |
|---|---|---|---|
| **A1** | `surface/tests/v3_format_edges.rs:38-43` | `text1`/`text2` **are** `format_item(&item1)`/`format_item(&item2)`; lines 39-43 recompute and compare exactly the pair line 38 already asserted. The oracle is the formatter compared to itself. **11 of the 13 tests in this file use this as their only oracle.** | **[see findings-2 §5.2 #45]** — but note the blast radius is 11 tests, not 1 line. Replace with `assert_renders(src, expected)` goldens so a lossy-but-stable formatter fails. |
| **A2** | `surface/tests/v3_format_hellgate.rs:164` | **[net-new]** The comment promises "each same family"; the body asserts only `doc2.items.len() == items.len()`. A formatter rendering a VIEW as any other one-item family keeps the count and stays idempotent, so the promised per-item family preservation is unverified. | Compare `items[i]`/`items2[i]` by discriminant (and per-item text), not `Vec::len`. |
| **A3** | `surface/tests/v3_document_hellgate.rs:80` | **[net-new]** Test named `exact_span` asserts `diag.span.start >= prefix.len()`. Any span from the middle item's start to EOF satisfies it — a reject reported at the *third* item passes. | Pin the exact offset: `assert_eq!(diag.span.start, 41)` (the `db` token where `->` was expected). |
| **A4** | `surface/tests/v3_document_hellgate.rs:93` | **[net-new]** Three separator tests have **identical bodies** modulo the literal and share one oracle, `diag.code == UnexpectedToken`. None checks span or message, so they cannot distinguish `parser.rs:164` (empty statement) from `parser.rs:181` (missing separator) — one generic reject passes all three. | Assert the distinct diagnostics: `"empty statement"` + span for `;;`/leading `;`, `"expected ';'"` at the following token for the missing separator. |
| **A5** | `surface/tests/v3_family_hellgate.rs:344` | **[net-new]** Inline VIEW body assertion discards the parsed query: `CatalogPayloadV1::View(ViewBodyV1::Inline(_)) => {}`. The fixture's `\| where u.active` pipeline is never checked, so a `parse_view` mutation storing a default/truncated `QueryHirV1` passes. No other surface rail inspects the inline body. | Bind the inner query and assert `format_item(inline_query) == "from users u \| where u.active"`. |
| **A6** | `surface/tests/v3_family_hellgate.rs:435` (+`:402`) | **[net-new]** Duplicates stronger twins on **identical input**: `v3_family_edges.rs:246` feeds the same `gate g \| binding=all` to `assert_unsupported`, which also pins the code and the full message. Same for `profile` at `:402` vs `edges:230`. | Delete the hellgate pair; the edges rails have strictly stronger oracles on the same fixtures. |
| **A7** | `planning/tests/v3_capability_derivation.rs:117` | **[net-new]** Test says the join "walks BOTH arms". Both arms are `Scan` and `capabilities` is a `BTreeSet`, so deleting `collect(&join.right, caps)` (`capability.rs:103`) still yields `{Join, RelationScan}` with `len == 2`. **Apply / Union / SemiJoin repeat the same unverified claim.** | Give each arm a distinct capability: `join(filter(scan a), limit(scan b))` → `{Join, Filtering, Limiting, RelationScan}`. |
| **A8** | `planning/tests/v3_explain_provenance.rs:222` | **[net-new]** "Left then right" pre-order: both arms are `scan(..)` → both `Scan`, so **swapping left and right yields an identical vec**. Same at `apply:244`, `union:266`, `semijoin:288`, `antijoin:310`. | Use distinguishable arms: `join(filter(scan a), limit(scan b))` → `[Join, Filter, Scan, Limit, Scan]`. |
| **A9** | `logic/tests/v3_graph_query_hellgate.rs:173` | **[net-new]** `reachability_with_via_waypoint_admits` never asserts the derived bound it exists to prove: mutating `graph.rs:290` to any other non-zero value survives `WitnessContractV1::bind`. | `assert_eq!(contract.witness.max_path_length, Some(1))` plus a two-waypoint case. |
| **A10** | `pipeline/tests/v3_pipeline_compile.rs:52` | **[net-new]** `assert!(!cost.estimated_cardinality_class.keyword().is_empty())` — every variant's `keyword()` is a **non-empty const literal** (`planning/src/cost.rs:42-128`), so this holds even for a hardcoded class. The doc claims "all 5 fields populated"; the oracle proves only that some string is returned. | Assert concrete classes for this query (`LinearInInput` / `None` / `Unordered`). |
| **A11** | `types/src/op.rs:204-205` | **[net-new]** `assert!(lt.iter().all(\|ov\| ov.result == TypeV1::bool()))` — `all()` on an **empty** iterator is true, and nothing asserts `lt` is non-empty. | Add `assert_eq!(lt.len(), 4)` before the `all`. *(Impact limited: `v3_types_universe.rs::comparison_types_to_bool` pins `int < int -> bool`.)* |
| **A12** | `planning/tests/v3_cost_classification.rs:365` | **[net-new]** `union_of_equal_arms_stays_linear` claims to exercise `worst_class`'s `>=` tie branch. `rank()` is **injective** over the five classes (`plan.rs:311-319`), so equal ranks imply the same class and `>=` vs `>` return identical values — the branch is unobservable. | Drop the `>=` claim; cover the left-wins path with distinct classes (`union(join(b,c), scan(a))` → Multiplicative). |
| **A13** | `pipeline/tests/v3_composition_type_env.rs:108` | **[net-new]** Identical to lines 84-97 once literals are blanked, and `assert!(compiled.plan().is_some())` delegates to the body plan — removing producer-output lifting still passes (the line-45 test is the real D34 rail). | Drop the duplicate, or assert the lifted row is actually consumed (a bad expression over `x.id` rejects). |
| **A14** | `cli/tests/command_doc_gate.rs:29` | **[partially net-new]** `usage.contains("fmt")` is satisfied by `v3-fmt`, and no rail ever runs bare `fmt`, so deleting `"fmt" \|` at `main.rs:53` stays green. *(Bare `query` **is** pinned elsewhere, so the exposure is `fmt` only.)* | Split the `<query\|fmt\|v3-query\|v3-fmt>` alternation on `\|` and set-compare to `CLI_COMMAND_TOKENS_V1`; run the binary with each token. **[related: findings-2 §5.2 #46, which covers const-emptying vacuity]** |
| **A15** | `diagnostics-stdio/src/json.rs:403`, `:410` | **[net-new, low]** `valid_surrogate_pair_decodes_to_astral_scalar_v1` and `bmp_unicode_escape_decodes_v1` both name **`\u` escape decoding** but feed **raw UTF-8** (`{"k":"😀"}`, `{"k":"é"}`), so they exercise the plain-char push arm, not `parse_unicode_escape_v1`. Real escape coverage was added later at `:437`, `:446`, `:454`, `:466` (via `u_escape_v1`), and the file's own comment at `:417-421` concedes it. | Rename to `raw_astral_char_passthrough_v1` / `raw_bmp_char_passthrough_v1`. |
| A16 | `surface/tests/v3_surface_proptest.rs:317…395` (12 sites), `:429`; `pipeline/tests/v3_diagnostic_surface.rs:53`; `surface/tests/v3_format_hellgate.rs:59`; `conformance/tests/self_validation.rs:40`; `modules/tests/v3_catalog_hellgate.rs:309`; `logic/tests/test_endpoint_conflict.rs:32`; `planning/tests/v3_planning_cost_schema.rs:19`; `pipeline/tests/audit_edge_cases.rs:82` | The remaining cannot-fail assertions found here are **already owned** by findings-2 and are not repeated. | **[see findings-2 §5.2 #42-#47, §3 D3, §4 H1, §5.4, §5.1 #38]** |

---

## §2 Name / behaviour mismatch — the test cannot fail for the reason its name gives

| ID | Location | Name promises | Body actually verifies |
|---|---|---|---|
| **B1** | `surface/tests/v3_family_hellgate.rs:940` | the PRODUCER column reaches `parse_producer` | `at_composition` (`family.rs:136`) only routes `from`/`sample`/`find` bodies, so `explain` is rejected as a let-value. A mutation making `parse_producer` accept `explain` still yields `UnexpectedToken`. |
| **B2** | `binder/tests/v3_binder_mutants.rs:394` | `resolve_shape_walks_nested_record_and_list` | only `shape_leaf` + `shape_list` are built. `shape_record` appears **zero times** in `crates/quanta-gqlang-v2-binder/`. Emptying the Record arm (`resolve.rs:604-608`) → `{}` leaves **every** binder test green. |
| **B3** | `binder/tests/v3_composition_hellgate.rs:80` | the FIND body **and** `WITH` both resolve to the LET | only `target_of(body_ref_node)` is read. Deleting the `Bound` arm of `resolve_with` (`resolve.rs:399-406`) still passes — the composition's own node id is never queried. |
| **B4** | `pipeline/tests/v3_use_pack_authority.rs:147` | a **composition-local** pack-context rebuild | the test builds the context itself and asserts a constructor round-trip; the `src` is **top-level**, not inside a composition. Production only clones (`compile.rs:791`), never calls `new`. If `compile_composition` dropped pack authority, this test still passes. |
| **B5** | `pipeline/tests/v3_uncovered_branches.rs:143` | the env-open alias-dependent projection path is covered | the body compiles `let x = 1 + 2` — a **closed literal** decl. A mutation rejecting env-open alias projections still passes. |
| B6 | `logic/tests/v3_logic_recursion.rs:280` | declaration order does not matter | **[see findings-2 §5.1 #11]** — premise is false; `BTreeSet` iteration erases order. |
| B7 | `logic/tests/v3_logic_hellgate.rs:278` | the file doc claims every test asserts the typed code+message | **[see findings-2 §5.1 #10]** — asserts only `.is_none()`. |

---

## §3 Unreached production branches (net-new; each verified by reading `src/`)

Each row: a real branch in `src/` that **no test in the suite reaches**, so it can be deleted or
inverted without a single failure. findings-2 §4 H1-H6 covers a different set (diagnostic codes,
`graph.rs:238`, FIND-kind mapping, diagnostic ordering, front-door rails, `check_tree`); these are
additive.

| ID | Branch | Why it is unreachable |
|---|---|---|
| **M1** | `pipeline/src/compile.rs:794` — multi-binding producer-output accumulation | **Every compiling composition fixture in the repo has exactly one `let`.** The only two-let source (`audit_edge_cases.rs:127`) is a duplicate-name *parse reject*. So `install_composition_binding_v1` (`:803`) runs exactly once and the D34 accumulation path is unexercised. **Highest-value gap in this report.** |
| **M2** | `pipeline/src/compile.rs:749` — `catalog.resolve()` fail-close | Only `register()`'s duplicate-name reject is driven by pipeline tests; deleting the `resolve()` call stays green. Reach it with `[require certified, profile p \| capabilities=c1 \| admission=allowlist]`. |
| **M3** | `planning/src/plan.rs:242` — final-lane early return in `evidence_for` | The only successful `ProofConfirmed` rail (`v3_planning_truth.rs:282`) uses `result_kind: ProofExplain`, whose match arm (`:249`) also returns `ProofTree`. Deleting the early return leaves everything green, and `Table + ProofConfirmed` (permitted by `reject_unsupported_mix:188`) silently downgrades to `None`, undetected. |
| **M4** | `planning/src/item.rs:110` — `ProgramItemHirV1::Composition(c) => classify_item(&c.body)` | No planning test constructs `CompositionHirV1`, and the pipeline decomposes compositions via `compile_composition` rather than `classify_item`. Making the arm always return `NoPlan(RelationalDeferred)` survives every test. |
| **M5** | `diagnostics-stdio/src/server.rs:108` — frame-extractor escape tracking | No fixture pairs an **escaped quote with an in-string brace**. Every backslash fixture uses `\uXXXX`/`\n`, and the only escaped-quote frame has no brace inside its string, so escape-vs-no-escape yields identical boundaries. |
| **M6** | `surface/src/family.rs:286` — `parse_ref`'s `::` branch | `family.rs:263` documents `repo::auth` as supported, but the only test using `::` (`describe rule::owasp.sqli`) has both colons consumed by `parse_describe:807-808` before `parse_ref` runs. The colon arm, single-colon acceptance and missing-segment reject never run. |
| **M7** | `surface/src/format.rs:311` — binderless-quantifier render arm | The only fixture producing an empty-binder quantifier (`v3_surface_expr_edges.rs:199`) is **never formatted**; every format test uses a binder. |
| **M8** | `cli/src/main.rs:196-199` — `read_input_file_v1` **success** path; `cli/src/main.rs:190` — `"input cannot be empty"` guard | All three `--file` rails (`v3_cli_front_door_contract.rs:95,140,188`) use nonexistent paths or extra args, so only the `Err` branch runs. A mutation returning `Err` unconditionally passes while `query --file <valid>` is broken. No CLI test passes an empty inline argument, so the exit-2/usage guard at `:190` is untested too. |
| **M9** | `modules/src/catalog.rs:345` — `Project`/`Compose` recursion in `collect_view_refs` | Only `Source` bodies are constructed in modules tests, and the one parsed non-Source body is a `Filter`. Replacing these arms with `=> {}` keeps all tests green, so a cycle through a projected inline body is undetected. |
| **M10** | `modules/src/manifest.rs:242` — dependency on a pack **absent from the graph** (the `None` arm) | `dependency_unsatisfied_rejects` only reaches the `Some(version)` arm; `manifest_and_lock_round_trip` declares a present, in-range dependency. |
| **M11** | `modules/src/graph.rs:347` — `canonical_cycle` prefix-slicing + rotate-to-smallest | Both cycle rails start the DFS on the cycle's own smallest member, so `start == 0` and `min_pos == 0`. Forcing `start = 0` or deleting `min_by` is a **no-op** for the asserted cycles. |
| **M12** | `logic/src/source.rs:115` — `lower()`'s `Some(predicate)` branch | All three lowering tests pass `None` (`v3_logic_source.rs:37,55,70`). The correlation predicate is the only reason a semi/anti join is not a cartesian test, and it is never exercised. |
| **M13** | `logic/src/recursion.rs:363` — `Aggregated` cross-SCC edge arm | The only aggregated literal in the repo (`v3_logic_hellgate.rs:287`, `rank :- rank`) is an intra-SCC self-loop that rejects earlier at `recursion.rs:243`. Treating `Aggregated` as monotone here survives every test. |
| M14 | `modules/src/graph.rs:218` — `modules.unknown_module` | **[see findings-2 §4 H1]** — one of the four zero-assertion diagnostic codes. |

---

## §4 CI / infrastructure steps that are structurally incapable of failing

This is the highest-leverage section: these are not test defects, they are **guards that report
green forever**.

| ID | Problem | Evidence | Action |
|---|---|---|---|
| **I1** | **The doc-test step compiles zero doc-tests.** `cargo test --workspace --doc` runs in `Justfile:20` **and twice in CI** (`lint.yml:41` and `:105`) — three invocations of a step that cannot fail. | `grep -rn '```' crates/*/src/` returns exactly **4 fence markers, all ` ```text ` prose** (`surface/src/query.rs:10,18`; `surface/src/unit.rs:10,13`). No ` ```rust ` fences and no indented Rust blocks exist anywhere under `crates/*/src`. | Add **one** real ` ```rust ` example (e.g. parse → HIR in `surface`) so the step can fail, **or** delete the step from `Justfile:20` and both CI sites. Do not leave a three-times-redundant no-op. |
| **I2** | **`[build-dependencies]` bypass all three dependency guards** in the architecture lint. | `tools/architecture/check_architecture.py:148-155`: `dependency_kind()` returns `None` for cargo's `"build"` kind (`:152`), and the loop at `:186-188` does `if kind is None: continue`. A build-dependency on `quanta-runtime`, a legacy `quanta-gqlang-*` crate, or an undeclared `quanta-gqlang-v2-*` crate therefore **skips lines 190-205 entirely**. No fixture emits a build edge, so all 5 tests stay green. | Treat `"build"` like `"dev"` (or reject it loudly), and add a build-edge fixture plus a test asserting the forbidden-prefix error string. **[related but distinct: findings-2 §1 G3/G4 — those are the stale member/layering tables; this is the `kind` filter]** |
| I3 | `examples/` orphan corpus + phantom rail | **[see findings-2 §2 C3/C4]** — 18 `.qglang` files, zero test/`src`/script references. Re-confirmed here: `grep -rn "examples/\|\.qglang"` across `*.rs`, `*.sh`, `*.py`, `*.yml`, `Justfile`, `*.toml` returns **nothing outside `docs/`**. | Fix in findings-2. |
| **I4** | `.config/nextest.toml` `slow-timeout = { period = "30s", terminate-after = 2 }` is unreachable — the whole suite runs in ~7 s, so the guard has never fired and neither direction is tested. | config vs measured runtime | Low priority; note only. Do not tune it to make it fire. |

---

## §5 Claims this audit retracts or narrows

Every headline was re-checked against a fresh command. Recorded so this report is not read as
"everything the sub-auditors said".

| ID | Original claim | Correction | How verified |
|---|---|---|---|
| **X1** | "`tools/**/tests/*.py` are never run by CI or the Justfile" | **FALSE — retracted.** Both run: `Justfile:30,34` and `lint.yml:33,39` invoke `test_architecture.py` and `test_semgrep_non_vacuity.py`. | read every job and recipe |
| **X2** | "`audit_edge_cases.rs` has 17 bare `is_err()` tests" | **Imprecise, twice.** Measured: **21 tests, 18 `is_err()` occurrences, 11 bare `assert!(result.is_err()`, exactly 1 `stage()` and 1 `code()` assertion, 0 span assertions.** findings-2 §0 R3 measured the same file and got 18/1/1/0 — consistent. | `grep -c` over the file |
| **X3** | "`conformance/tests/registry_non_vacuous.rs:24` length check is a separate finding" | **Dropped as a same-root-cause duplicate** of `self_validation.rs:40` (the `BTreeSet` uniqueness rail), which is already findings-2 §5.2 #47. | code read |
| **X4** | "`v3_format_hellgate.rs:59` `same_structure` is a separate defect from `v3_format_edges.rs:39-43`" | **Same defect, same root cause** — identical `format_item(a) == format_item(b)` self-comparison. Merged into A1 and cross-referenced to findings-2 §5.2 #45. | code read |
| **X5** | "`same_structure`'s third-pass block is entailed by line 51" | **Imprecise.** `text3 = format(parse(text2))` is a *further iteration*, not the same comparison. Only the `same_structure` half holds. | code read |
| **X6** | "`shape_record` is used by at least one binder test" | **FALSE.** Zero occurrences in `crates/quanta-gqlang-v2-binder/`. B2 stands. | `grep -rn shape_record` |
| **X7** | "`v3_types_check_edges.rs:243` adds a new branch" | **FALSE** — identical fixture, code and Display to `v3_types_hellgate.rs:210`. Already findings-2 §5.1 #32. | code read |
| **X8** | "the CLI `--file` success path is covered by `error_class_drives_distinct_exit_codes_v1`" | **FALSE** — that rail also uses a nonexistent path. M8 stands. | code read |
| **X9** | "`json.rs` `\u` tests exercise the escape decoder" | **Half false, and the file already knows.** `:403`/`:410` feed raw UTF-8 (names are wrong); the real escape coverage is at `:437-475`. Downgraded to **low** severity because the author's own comment at `:417-421` documents the correction. | code read |
| **X10** | "the 4 pipeline proptests are 23% of suite CPU because of 512 cases" | **Narrowed.** The wall clock is fine (7.3 s total). Only the two `*_never_panics` variants are wasted, and because of the tautology in A16, not because of the case count. **Do not reduce `PROPTEST_CASES`.** | timing log |

**Not retracted** (verified by command; listed so they are not re-litigated): the 12 tautology
sites; `diag.span() == diag.span()`; the `BTreeSet` uniqueness rail; the `CatalogV1.active_packs`
write-only field; `modules/src/graph.rs:238` unreachability; the `examples/` orphan corpus; the
zero-fence doc-test finding; the `build`-kind bypass.

---

## §6 Fix order

Ordered by regressions-prevented per line changed.

1. **I1 — make the doc-test step real or delete it.** Three invocations, zero coverage; the cheapest
   possible fix and it is currently pure false assurance.
2. **I2 — close the `build`-kind bypass.** One predicate change plus one fixture; it is the only
   hole in the architecture guard.
3. **A1 — replace the round-trip oracle with goldens.** 11 tests in one file currently pass for a
   formatter that silently drops clauses. This is a live regression hole, not a style issue.
4. **M8 — add `query --file <valid>` and `query ""`.** Two small tests close the entire CLI
   front-door boundary claim.
5. **M1 — add a two-`let` composition.** The D34 accumulation path has never executed.
6. **M3 / B4 — pin the lane rule and the real composition pack rebuild.** Both currently assert a
   value that a stronger path produces anyway.
7. **B2 / B3 — build the record shape; assert the `WITH` target.** Two tests named for branches they
   never reach.
8. **§0.1 — fold the 40 duplicate-body groups (130 tests) into table-driven tests.** Start with the
   two 10-test groups. This is also findings-2 §5.1 #37/#38; do it once.
9. **M2, M4-M7, M9-M13** — one negative fixture each. M4 (`Composition` dispatch), M5 (frame escape
   tracking) and M13 (aggregated cross-SCC) are the three most likely to hide a real bug.
10. **A2-A15, B1, B5** — per-test oracle strengthening; mechanical once the contract is named.

---

## §7 Overlap with `findings-2-duplicated.md` — fix once, there

| This report | Owned by findings-2 as |
|---|---|
| A1, A16 (12 tautologies, `diag.span()`, `lex(&s)==lex(&s)`, `same_structure`) | §5.2 #42, #43, #44, #45 |
| A16 (`self_validation.rs:40`) | §5.2 #47 |
| A16 (`v3_catalog_hellgate.rs:309`, `active_packs`) | §3 D3 |
| A16 (`test_endpoint_conflict.rs:32`, `logic.graph_endpoint_unexpected`) | §4 H1 + §5.4 |
| A16 (`audit_edge_cases.rs:82`, `audit_18_families.rs:40`) | §5.1 #37, #38; §0 R3, R4 |
| A16 (`planning_cost_schema.rs:19`, `FIELD_NAMES` no production reader) | §5.4, §5.6 |
| A14 (doc gates) | §5.2 #46 |
| A15 | §5.1 #21 note (escape fixtures) |
| M14 (`modules.unknown_module`) | §4 H1 |
| I3 (`examples/`) | §2 C3, C4 |
| §0.1 (duplicate-body groups) | §5.1 #37, #38 (partially) |
| §0 "84 tests with no assertion" | *not covered* — findings-2 says "zero dead tests found", which uses a different definition (§0 note) |

**Everything else in this report is net-new.** No item appears in both action lists as an
independent action.
