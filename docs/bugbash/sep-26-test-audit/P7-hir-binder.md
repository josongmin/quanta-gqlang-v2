# P7: HIR and binder proof gaps resolved in the working tree

## TA-09: auxiliary-span visitor coverage

Before: exact auxiliary-span order was asserted for catalog payloads only; other tests counted visited nodes without proving the auxiliary callbacks.

Now: `v3_visit_traversal.rs` traces exact auxiliary spans across value paths/calls/records and call-star, logical binders, relation source/projection aliases, result shape, declarations/rules, graph references, legality, all presentation variants, standalone SAMPLE/EXPLAIN, catalog, and composition. An out-of-bounds presentation reference must fail HIR integrity. The traversal target passed 14 tests. Temporarily removing the relation source-name callback made its focused test fail; the mutation was restored. This is branch-focused coverage, not a claim that every possible HIR shape is enumerated.

## TA-10: immediate noncapturing barrier

Before: the existing barrier test placed `NonCapturing` on a middle frame, leaving the top-frame early guard untested.

Now: `v3_binder_mutants.rs` pairs a directly noncapturing child (no outer shadow) with a capturing child (exact `OuterShadow` report and outer span). The focused binder test passed. Temporarily deleting the early guard made it fail; the mutation was restored.

## Round 2: remaining owner-scope gaps closed

A second, parallel adversarial pass (six independent read-only subagents plus
author verification of every candidate against production) closed the remaining
remediable gaps. No production behavior failure was found; all newly pinned
expected values matched production on the first run.

### TA-11: close-scope positive control and over-pop control

Before: `close_scope_pops_the_frame` was negative-only, and its promised "WITHOUT
the close, `x` resolves" companion did not exist; a `close_scope` that cleared
more than one frame also survived.

Now: `v3_binder_mutants.rs` adds `without_close_scope_the_let_still_resolves`
(the missing positive control) and `close_scope_leaves_outer_frames_visible`
(one close leaves the outer frame's binding resolvable), killing clear-all /
double-pop.

### TA-12: graph/relation illegal-capture arms

Before: only value-symbol capture across a barrier was tested; the
`IllegalCapture` arms of `resolve_local_ref` (graph references) and
`resolve_relation` (relation sources) had no test.

Now: `v3_binder_hellgate.rs` adds
`graph_reference_across_composition_barrier_rejects_as_illegal_capture` (a
composition body FIND referencing an outer binding) and
`relation_source_across_noncapturing_barrier_rejects_as_illegal_capture`
(a relation source referencing a `Let` across an explicit `NonCapturing`
frame). Both assert the exact diagnostic count, code, name, primary span, and
`binding_span`.

### TA-13: composition resolution arms and exact-frame oracles

Before: the Sample/GraphQuery producer arms, the legality scope/pattern loop,
and the graph-constraint `via`/`through`/`sanitizers`/`scope`/`window`/`frontier`
chains were never exercised; `capture_sets()` and `partition_candidates()` were
checked only by length; the `WITH` consumer's own resolution was never queried.

Now: `v3_composition_hellgate.rs` adds
`legality_reference_slots_resolve_to_the_composition_let`,
`graph_constraint_reference_chains_resolve_to_the_composition_let`, and
`sample_scope_and_graph_producer_resolve_to_a_prior_let`; the body/WITH test
also asserts the `WITH s` resolution via `target_of_use`.
`v3_binder_scope.rs` compares the exact `capture_sets()` pairs (including the
retained empty set) and the single partition candidate's
`reference`/`group`/`binding` identity/`span`. `v3_relation_source_use.rs`
pins the exact alias frame and asserts the valid body source resolves to its
LET while the forbidden producer source is the only rejection.

### TA-14: non-empty carrier rejects and traversal count anchor

Before: `NonEmptyBindersV1::try_new` / `NonEmptyRelationKeysV1::try_new` empty
rejects had no test, and the relation traversal count compared two derivations
of the same `walk_relation`.

Now: `v3_hir_expr_kernel.rs` adds
`nonempty_carriers_reject_empty_inputs_with_exact_messages` (direct binder
carrier plus the public `group_by`/`order_by` builder) and anchors the relation
visitor count to the independent `b.minted()` counter, matching the sibling
query test.

### Remaining known gaps (not remediated here)

These are lower-value or judgment-dependent and are left as reported caveats:
the presentation/catalog/meta "sound program item" rails still assert integrity
+ node count rather than payload fields (the builders are pass-through
constructors); `check_tree_validates_each_leaf_root_variant_directly` omits the
direct `Query` root (covered elsewhere); several `v3_hir_expr_kernel.rs` shape /
record / decl-rule rails compare counts or selected fields rather than full
payloads; `SourceRegistryDiagnosticV2`'s `Display` bodies and the non-extended
`source_len` accessor are unasserted; and a few section anchors in
`v3_binder_mutants.rs` reference stale `resolve.rs` line numbers. Closing these
means authoring new expectations rather than converting an existing skip.

## Owner test status

`cargo test -p quanta-gqlang-v2-hir -p quanta-gqlang-v2-binder`:
**150 passed, 0 failed**. `cargo fmt --check` and
`cargo clippy --all-targets` are clean for both crates.
