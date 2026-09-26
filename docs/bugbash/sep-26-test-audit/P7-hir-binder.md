# P7: HIR and binder proof gaps resolved in the working tree

## TA-09: auxiliary-span visitor coverage

Before: exact auxiliary-span order was asserted for catalog payloads only; other tests counted visited nodes without proving the auxiliary callbacks.

Now: `v3_visit_traversal.rs` traces exact auxiliary spans across value paths/calls/records and call-star, logical binders, relation source/projection aliases, result shape, declarations/rules, graph references, legality, all presentation variants, standalone SAMPLE/EXPLAIN, catalog, and composition. An out-of-bounds presentation reference must fail HIR integrity. The traversal target passed 14 tests. Temporarily removing the relation source-name callback made its focused test fail; the mutation was restored. This is branch-focused coverage, not a claim that every possible HIR shape is enumerated.

## TA-10: immediate noncapturing barrier

Before: the existing barrier test placed `NonCapturing` on a middle frame, leaving the top-frame early guard untested.

Now: `v3_binder_mutants.rs` pairs a directly noncapturing child (no outer shadow) with a capturing child (exact `OuterShadow` report and outer span). The focused binder test passed. Temporarily deleting the early guard made it fail; the mutation was restored.
