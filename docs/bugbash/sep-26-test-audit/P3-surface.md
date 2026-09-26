# P3 surface fail-open audit (SOTA++, final)

Scope: `crates/quanta-gqlang-v2-surface/tests/**` (18 `.rs` targets + `tests/support/mod.rs`)
and the `#[cfg(test)]` blocks in `src/` (`equivalence.rs`, `family.rs`, `lexer.rs`).
`tests/spec_cases/`, `examples/`, fixtures and `docs/` were read-only. No production code
changed outside `#[cfg(test)]`.

Method: three layers.
1. Full static read of every owned file against classes A/B/C/D/D'/E/F/G/H.
2. "Does this test actually do its job?" pass over comment/assert mismatches and code-only
   oracles.
3. Six parallel independent audits (family; format/lexer; proptest/budget; query/expr/unit;
   corpus/inventory/support; adversarial mutation-survival sweep). Findings were
   cross-verified against production source; only concrete surviving mutations were fixed.

`support::structural_witness` was independently checked against every HIR `walk_*` hook:
no semantic payload is omitted (ids/spans and paren carriers are deliberately excluded,
matching `src/equivalence.rs`'s documented policy).

Baseline: `cargo test -p quanta-gqlang-v2-surface` green (242 + 1 doctest).
Final: green (248 + 1 doctest). `cargo clippy -p quanta-gqlang-v2-surface --tests`: clean.
`cargo fmt -p quanta-gqlang-v2-surface -- --check`: clean.

## 1. fail-open conversions (A/B/C/D/F/G)

| file | class | fix |
|---|---|---|
| `tests/v3_surface_proptest.rs` `diagnostic_invariants` | D | `if let Some(resource)` payload check → total `match`; `None` arm asserts `code != ResourceExhausted`. |
| `tests/v3_surface_proptest.rs` `..._diagnostics_are_source_bounded` | D | Three `if let Err` (no else) → per-entry total `match`: Ok asserts `check_tree`, Err asserts bounded diagnostic, `lex` Ok runs new `lexed_token_invariants` (span bounds, char boundaries, ordering, terminal `Eof`). Fixed positive controls added (`parse_expr`/`parse_predicate`/`lex` must accept) so an all-Err regression cannot pass. |
| `tests/v3_surface_proptest.rs` round-trip proptests | D | `let Ok(..) else return` → Err branch also runs `diagnostic_invariants`, so no generated case is a silent no-op. |
| `tests/v3_surface_proptest.rs` `arbitrary_document_...` | H | `parsed.source_len` is now asserted against `s.len()` and used to build the document (was re-derived). |
| `tests/v3_surface_proptest.rs` determinism proptests | H | Added full `prop_assert_eq!(&a, &b)` alongside the id/span-free witness, so a reused-builder id leak fails. |
| `tests/v3_format_hellgate.rs` `document_roundtrips` | H | Added per-item `discriminant` + `same_structure` (comment promised "each same family" but only counted). |
| `tests/v3_examples_corpus.rs` `walk` | B | Unreadable/missing `examples/` dir now panics instead of shrinking the corpus. |
| `tests/v3_examples_corpus.rs` `single_root_...` | B | `parse_document` failure now panics instead of `continue`. |
| `src/equivalence.rs` unit | E/G | Added positive equivalence assertion (an always-`false` comparator previously passed). |

## 2. weak-oracle / payload hardening (E/H/D')

| file | fix |
|---|---|
| `tests/v3_family_hellgate.rs` profile/gate | Full ordered `Vec<&str>` of capabilities / checks, not `len` + first element. |
| `tests/v3_family_hellgate.rs` `composition_let_with_parses` | SAMPLE `query_text`/`mode`/`limit`; body GraphQuery `kind`/source/sink. |
| `tests/v3_family_hellgate.rs` `composition_find_producer_...` | Producer `kind`/`depth`; body `rule_ref`/`confirm`. |
| `tests/v3_family_hellgate.rs` `composition_second_producer_...` | Both bindings' name + producer. |
| `tests/v3_family_hellgate.rs` `composition_unconsumed_let_rejects` | OR-ed `contains` message → exact message + with-name span. |
| `tests/v3_family_hellgate.rs` find rails | `find_missing_arrow`, `find_unknown_subkind` (message + full span), SAMPLE zero-limit/empty-query (message + span). |
| `tests/v3_family_hellgate.rs` duplicate-clause rails (13) | `contains("duplicate") && contains(key)` → exact `duplicate <FAMILY> clause '<key>'` + exact span. Kills a wrong-family message. |
| `tests/v3_family_hellgate.rs` catalog decls | `decl.name` asserted for model/pack/profile/gate/ruledef. |
| `tests/v3_family_hellgate.rs` find seed/clause endpoints | `anchor`/`reference`/`pointee` names, not only `is_some()`. |
| `tests/v3_family_hellgate.rs` `find_taint_parses_to_graph_query` | Two sanitizers compared as a full list (was `sanitizers[0]`). |
| `tests/v3_surface_query.rs` `aliased_projection_...` | Projection value paths, record leaf paths, list shape span. |
| `tests/v3_surface_query.rs` `where_filter_and_join_compose` | Filter predicate compare, right source/alias, ON compare op. |
| `tests/v3_surface_query.rs` `group_by_...` | `count` callee + star arg, alias `n`, group key symbol `repo`. |
| `tests/v3_surface_query.rs` `order` | Explicit `desc` retained; omitted direction defaults to `Asc`. |
| `tests/v3_surface_expr_edges.rs` | Call arg payloads, list elements, record field values, compare operands, quantifier body, `Or` operands, paren internals, unary operand, callee; `!=` parse; `_z` identifier; exact close-paren EOF messages/spans. |
| `tests/v3_surface_expr_kernel.rs` | Inner add operands, `Not` payload + missing `check_tree`, path base symbol. |
| `tests/v3_surface_unit.rs` | Declaration binary operands/op, rule head segments + `and.rhs` path, quantifier body paths, rule root span. |
| `tests/v3_surface_expr_edges.rs` star-arg / quantifier tests | Doc comment claimed lowering while asserting only `is_err`; replaced with exact code/message/span negative rails and renamed (`call_star_arg_must_be_the_sole_argument`, `quantifier_without_binders_rejects`). |
| `tests/v3_format_edges.rs` `invalid_escape_forms_reject` | Per-case exact code/message/span (was `is_err()` only) + 7-hex-digit `\u{...}` case. |
| `tests/v3_surface_hellgate.rs`, `v3_family_edges.rs`, `v3_surface_unit.rs` | Code-only reject rails given exact message + span. |

## 3. mutation-survivor kills (new coverage added to existing files)

| production mutant | killing rail |
|---|---|
| `format.rs` drop `;\n` document separator | `v3_format_hellgate.rs::document_roundtrips` pins `"show capabilities;\nshow models"`. |
| `format.rs` swap `via`/`through`, `window`/`frontier`, `reference`/`pointee`, `scope`/`pattern`; shape delimiters; unary precedence | All `v3_format_edges.rs` round-trip calls converted to `assert_renders` exact canonical bytes; `v3_format_hellgate.rs` adds `check r \| scope=s \| pattern=p ...`; `let x = -a * b`, shape record/list pins added. |
| fields silently dropped by any family parser (`scope`, `callable`, `through`, `window`, `frontier`, ...) | Same exact-render conversions. |
| `query.rs` default sort direction `Asc`→`Desc` | `v3_surface_query.rs` asserts omitted direction == `Asc`. |
| `parser.rs` `classify_parsed_root_v3` arm swap | New `parsed_root_classification_matches_every_family` matrix (8 families). |
| `parser.rs` drop optional trailing `;` in `parse_program` | `v3_surface_unit.rs::program_tolerates_trailing_separator`. |
| `expr.rs` drop `!=` from `is_compare_op` | `v3_surface_expr_edges.rs::bare_not_equal_comparison_parses`. |
| `lexer.rs` 7-digit `\u{...}` accepted | 7-hex case in `invalid_escape_forms_reject`. |
| `lexer.rs` drop `_` ident-start | `leading_underscore_identifier_lowers_to_symbol`. |
| `lexer.rs` drop `\r` whitespace | `v3_mutation_rails.rs::space_separates_two_idents` CR case. |
| `query.rs` drop `"exists"` from reserved words | `v3_surface_query.rs::reserved_word_after_source_is_not_an_alias`. |
| `cursor.rs` drop `leave_expression_depth()` | `v3_surface_budget.rs::flat_sibling_width_restores_expression_depth_per_element` (65 shallow siblings). |
| `budget.rs` drop a zero check | `every_budget_dimension_rejects_zero` (7 dimensions). |
| `diag.rs` swap `limit`/`observed` or `code_str` | `exhausted` now pins `code_str` and the exact `"<dim> budget exhausted: limit L, observed O"` message. |
| `shape.rs` list span truncation | list shape span asserted against bracket offsets. |
| `unit.rs` rule root span truncation | `rule.span == 0..src.len()`. |
| `src/lexer.rs` dead payload-allocation counter | Unit test gains a generous-budget positive control asserting `PAYLOAD_ALLOCATIONS >= 2` before the `== 0` cases. |
| `src/family.rs` dropped keyword arm | Unit test enumerates all 16 owned keywords true and sibling roots false. |
| `src/parser.rs` cross-entry drift / formatter structure loss | `document_hellgate::single_item_document_matches_program` compares `parse_program` vs `parse_document`; `examples_corpus` compares `structural_witness` before/after canonical formatting. |
| `src/equivalence.rs` paren-layer stripper | New nested-paren `try_format_item` unit case. |

## 4. Residual disposition (structural closeout in section 8)

- **Examples-corpus identity pin (F)** — closed by an independent count and
  path SHA-256. A scalar fingerprint does not duplicate the maintained path list.
- **`proptest` `try_format_item` self-comparison (E)** — retained as a positive
  property. Existing raw-HIR rejection rails independently reject always-Ok.
- **Expression-depth diagnostic span** — closed by input-specific exact spans,
  independently expected resource limits/counts and formatter ingress equality.
- **env-gated child targets** — one selected/passed test alone did not prove
  executed work. Closed by post-assertion completion markers, bounded subprocess
  custody and an explicit selected-but-unexecuted negative control.
- **`support::structural_witness` paren transparency** — retained policy with
  redundant-paren equality and subtraction regrouping inequality controls.
- **Remaining `contains` message checks** — all eight fragment rails now use
  complete expected messages and complete spans.
- **Shape-limit branch** — no parser-reachable survivor was established; the
  recursive shape parser rejects first at the same limit. This is not counted as
  a mutation kill or a newly covered branch.

## 5. Modified files (17)

`tests/`: `v3_document_hellgate.rs`, `v3_examples_corpus.rs`, `v3_family_edges.rs`,
`v3_family_hellgate.rs`, `v3_format_edges.rs`, `v3_format_hellgate.rs`,
`v3_lexer_symbol_rails.rs`, `v3_mutation_rails.rs`, `v3_surface_budget.rs`,
`v3_surface_expr_edges.rs`, `v3_surface_hellgate.rs`, `v3_surface_proptest.rs`,
`v3_surface_query.rs`, `v3_surface_unit.rs`.
`src/` `#[cfg(test)]`: `equivalence.rs`, `family.rs`, `lexer.rs`.
Report: `docs/bugbash/sep-26-test-audit/P3-surface.md`.

## 6. Verification

| stage | command | result |
|---|---|---|
| baseline | `cargo test -p quanta-gqlang-v2-surface` | green, 242 + 1 doctest |
| final | `cargo test -p quanta-gqlang-v2-surface` | green, 248 + 1 doctest |
| lint | `cargo clippy -p quanta-gqlang-v2-surface --tests --all-features` | clean |
| format | `cargo fmt -p quanta-gqlang-v2-surface -- --check` | clean |
| probes | temporary `tests/zz_probe*.rs` used to sample exact canonical/diagnostic values; both deleted (18 `.rs` targets restored) | — |

`just ci` / `just qualify-local` were not run (integrator-only).

## 7. Post-re-audit verification (HEAD `78f4770`)

The re-audit (`reaudit-2026-09-26.md`) closed its one P3 residual — the five surface
substring rails (`check_using_requires_a_find_query`, `check_missing_confirm_rejects`,
`compare_missing_candidate_rejects`, `composition_duplicate_binding_rejects`,
`find_summary_empty_facets_rejects`) now also pin exact diagnostic spans, and section 4's
scope claim was narrowed accordingly. RA-04/RA-05 belong to the conformance owner, not this
scope.

Re-verified on the current committed HEAD (all P3 markers present; worktree clean for
`crates/quanta-gqlang-v2-surface/`):

| check | command | result |
|---|---|---|
| tests | `cargo test -p quanta-gqlang-v2-surface` | green — 20 targets, **250 passed + 1 doctest** |
| lint | `cargo clippy -p quanta-gqlang-v2-surface --tests --all-features` | clean |
| format | `cargo fmt -p quanta-gqlang-v2-surface -- --check` | clean |
| probes | no `zz*` files present; 18 integration targets + `support/mod.rs` | — |

Historical residual at that verification point (superseded by section 8): examples-corpus identity pin (ledger
designates `examples_index.md` as the single list), proptest `try_format_item`
negative direction owned by `v3_format_hellgate.rs`, expression-depth diagnostic span
(relation-depth span is pinned; other depths pin code+message+resource), env-gated child
targets, and documented `Paren` transparency in `structural_witness`.


## 8. Structural residual closeout

Base is `78f477056cfbc726441d298fe243d3d34efb566f` plus the existing shared
working-tree changes. This closeout owns surface tests, surface dev-dependency
metadata, this record and the corpus ledger description. It changes no production
semantics or public frontend contract. The earlier 250-test receipt is historical.

Acceptance and final behavior:

- The corpus has 18 reviewed paths, pinned independently by count and SHA-256 of
  sorted, u64-little-endian-length-framed UTF-8 paths with domain
  `qglang.examples.paths.v1\0`. The index remains the only maintained path list.
  Intentional path additions/removals/renames require reviewing and updating the
  pins. File content is checked by parse/format/witness rails, not this path hash.
  Discovery no longer flattens entry errors or uses lossy path conversion; type
  errors and symlinks fail closed instead of following an unbounded directory cycle.

- All four deep formatter parents share the existing bounded process helper:
  null stdin, concurrent stdout/stderr drains and a 10-second watchdog. Each child
  emits its exact named completion line only after its assertions and worker join.
  Parents require success, one selected/passed test and exactly one completion
  line. A negative control executes each child with its gate removed, proves that
  libtest still reports a pass, and rejects it as unexecuted work. Direct-child
  cleanup is covered by the existing process-support owner; no descendant cleanup
  claim is made here.
- Budget oracles now pin independently expected limit/observed and exact span,
  including the recursive NOT case and flat value/bool HIR admission. Formatter
  ingress retains the entire expected source-bound diagnostic. Four custom-depth
  acceptance/rejection pairs cover unary, parentheses, result shape and NOT;
  direct value/predicate entrypoints pin their own depth spans and resource payloads.
- All eight remaining family message-fragment rails pin complete messages.
  Missing USE PACK lockfile now also pins 4..8; zero FIND depth pins both span ends.
- The proptest's parser-produced checked/infallible equality remains a positive
  property backed by the existing raw/lossy-HIR rejection rails. No duplicate
  negative suite is added. The checked-formatter always-Ok mutation is exercised
  against those rejection rails.
- A separate witness regression accepts redundant paren carriers and rejects
  left/right subtraction regrouping. Raw unary/path and NOT/NOT EXISTS goldens
  remain separate. The parser's Shape-depth rejection still precedes the HIR
  Shape-limit branch; no unreachable mutation is claimed as a kill.

Evidence is kept in `target/sep26-p3-closure/`: the isolated mutation checkout
pins surface production to the inspected committed HEAD and copies the owner
changes without editing the shared checkout. `baseline.json` and
`mutation-results.json` record commands, source/log hashes, terminal assertion
failures and byte-for-byte restoration. Compiler failures are not mutation kills.
The first shared-checkout CI attempt rejected source drift during doctor; its
log and receipt are preserved as `shared-ci-first.log` and
`shared-ci-first-result.json`. Final `just ci` runs in a detached worktree populated
from one byte-verified shared dirty-source snapshot; `ci.log` and `ci-result.json`
record that isolated source separately. It cannot qualify later edits in the shared
checkout. Corpus path components are joined with `/` before sorting/hashing so the
path identity does not depend on native separator spelling. Local CI is not a clean committed-HEAD, hosted CI or downstream
runtime qualification. Clean-HEAD qualification remains a separate integration step.

The frozen isolated positive owner run passes 113 tests (3 corpus, 66 family,
28 format and 16 budget, including three concurrent owner additions). Sixteen
compiling mutations are rejected by assertions: coordinated shrink, same-count
replacement, omitted child env, removed completion check, erased HIR/parser
depth spans, eight fragment-preserving wrong messages, always-Ok checked formatter
and erased witness binary grouping. An initial always-Ok probe hit unused-import
compiler errors and was discarded; the accepted probe retains the original typed
body after an unconditional success branch and reaches four failing rejection
assertions. The first isolated baseline caught an incorrect 18..23 expectation in
a concurrent budget test; the owner corrected it to the authored 19..24 before
the accepted frozen baseline. Neither invalid run is counted as a mutation kill.

The first frozen CI found a concurrent sema test's wildcard enum arm rejected by
Clippy. Its Rule-only `find_map` is expressed as an `if let` without changing the
fixture or signature assertions. This narrow CI fix is in
`sema/tests/v3_authority_builder.rs`; the original frozen failure log/receipt remain
`frozen-ci-first.log` and `frozen-ci-first-result.json`. The final source is copied
and verified again before rerunning the complete gate.

## 9. Canonical example-content identity (2026-09-27)

The path pin above did not protect a surviving valid-content replacement: changing
`examples/gate/release_gate.qglang` from GATE to `show capabilities` kept both the
surface corpus round-trip and CLI shipped-example rails green. The index still
listed the path under Declaration roots. Conformance compiles only four selected
example files and does not select this GATE example.

The corpus rail now also pins a domain-separated SHA-256 of the ordered pairs
`(path, canonical_source)`, each component framed by a u64 little-endian UTF-8
byte length. The scalar constant is independent of discovery and the maintained
index; it does not duplicate their path list. It intentionally ignores source
comments/whitespace and binds every reviewed example's parsed canonical
content. An isolated valid wrong-family replacement failed by the final digest
assertion; a comment-only edit still passed. The full digest was independently
recomputed from the shipped CLI output for all 18 examples.

This is a syntax/corpus-identity guarantee. Only the four selected conformance
examples carry the separate closed-authority compilation claim; no runtime or
provider behavior is inferred from the new pin.
