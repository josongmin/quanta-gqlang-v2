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

## 4. Residual (reported, not fixed) and reasons

- **Examples-corpus identity pin (F)** — a strongly reduced corpus (delete 19/20 examples and
  their index rows) would still pass. The ledger designates `docs/language/examples_index.md`
  as the single maintained list, so adding a second hard-coded list would violate the
  maintenance rule; non-empty + disk↔index cross-check retained.
- **`proptest` `try_format_item` self-comparison (E)** — the round-trip proptest compares the
  checked formatter to the infallible one; the negative direction is owned by
  `v3_format_hellgate.rs::checked_formatter_rejects_*` (5 exact rails), so not duplicated.
- **`depth.rs` diagnostic span** — `exhausted` pins code/dimension/observed/message but not the
  exact subexpression span (varies per input); code+message+resource are pinned.
- **env-gated child targets** — parents assert `running 1 test` + `1 passed`, so zero-selection
  is closed.
- **`support::structural_witness` paren transparency** — deliberate and documented; a
  grouping-relevant paren changes traversal order and is still caught.
- **Remaining `contains` message checks** — only in rails whose primary oracle (code+span or
  witness) is already exact.

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
