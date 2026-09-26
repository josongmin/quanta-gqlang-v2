# Sep 26 adversarial test challenge

Base: `78f477056cfbc726441d298fe243d3d34efb566f` plus the final-review
uncommitted patch. This follow-up rechecked held notes and suspicious
conditional/skip assertions, then injected reachable boundary regressions.
The completed previous audits and this challenge are different evidence sets;
their mutation counts are not combined into a coverage percentage.

## Confirmed additional gap and repair

The public source-registry error Display was unpinned. A combined mutation
swapped duplicate registration lengths, incremented the unknown source ID,
and reversed invalid span endpoints. It compiled and passed the entire HIR
owner suite (84 unit/integration tests and 3 doctests). This demonstrates a
test gap, not an observed production failure.

`hir/tests/v3_source_registry_diagnostics.rs` now drives the real registry
failure paths and compares full enum payloads, literal codes and literal
messages for duplicate, unknown, inverted and out-of-bounds errors. Authored
values differ (source 17/99, lengths 13/29, span 9..4/3..14), so swapped values
cannot coincide. Exact-end and zero-width success controls prevent a blanket
rejection implementation from satisfying the rail.

Each of the three Display regressions now independently compiles and fails
the new test. All temporary production mutations were restored byte-for-byte.

## Existing rails challenged without changing their oracles

| Injected regression | Test that rejects it |
|---|---|
| Unicode escape digit limit 6 → 5 | Surface full owner run fails `invalid_escape_forms_reject` at the independently pinned byte span (18 instead of 19) |
| Composition frame permits outer capture | Binder `graph_reference_across_composition_barrier_rejects_as_illegal_capture` |
| Catalog ambiguity check disabled across namespace kinds | Modules `same_spelling_cross_kind_is_ambiguous_before_expected_kind_selection` |
| Execution item lookup ignores the requested identity | Pipeline `source_id_is_transaction_local_and_foreign_item_lookup_is_empty` |
| Witness cap no longer enforces the sealed budget | Logic `one_over_sealed_witness_cap_rejects_over_budget` |
| CI accepts a failed summary contradicting successful test events | Python `test_nextest_rejects_summary_that_contradicts_ok_events` |

Eight Rust mutations compile and fail assertions (three new diagnostic oracles
and five existing Rust boundary controls); one interpreted Python gate mutation
fails its assertion. The Unicode candidate was already caught, so no redundant
oracle was added. Compiler errors and zero-selection runs are not credited.

Static skip review found conditional branches preceded by variant/presence
assertions, recursive enumeration base cases, graph traversal visited-node
skips, fixture construction and parent-controlled child probes. These are not
silent acceptance paths. Corpus/index changes remain a maintained-authority
decision; selected family/conformance or integrity tests do not claim full
semantic payload coverage. No demonstrated surviving regression remains from
this challenge.

## Evidence and final gate

Commands, source/log hashes, terminal exits and restoration flags are recorded
under `target/sep26-adversarial-final/`:

- `registry-display-before.json`: original surviving combined mutation.
- `unicode-six-digit-before.json`: rejected candidate, not a new finding.
- `after-results.json`: eight post-review mutation runs, including the Python
  gate probe; the Unicode probe supplies the ninth rejected regression.
- `ci-result.json` and `ci.log`: final repository-wide `just ci` result after
  restoration, formatting, documentation updates and the editing freeze.

Accept the final local gate only if exit is 0, before/after source snapshots
match and the current-run nextest selected/executed/pass counts reconcile.
The dirty-source local gate does not represent clean committed-HEAD or hosted
CI qualification. This patch adds one integration test and coverage records;
production semantics and public API behavior are unchanged.
