# D-06: bounded document-session ownership indexing

Base: `5ac954f624f4e789af786174c7eb0105e4065799`.

## Reproduction and acceptance

The pipeline uses one AlgebraSessionV2 for a document but calls nested lowering
once per relational composition producer and once for its body. The old
ownership check searched the owner's earlier producer trees again for each
request. A sequence of P query producers could repeat those searches
quadratically. Discovery was not charged to either downstream traversal ledger.
The baseline owner regression selected one test and failed: after the first
Project(Scan) request, four of six semantic steps remained instead of two,
showing that only lowering had been charged, not the two discovery visits.
This is evidence of missing work accounting, not a latency benchmark.

Acceptance is one discovery per owner in an immutable-document session,
complete exact-tree membership checks after lookup, bounded discovery using the
same document receipt as lowering, preserved transitive EXISTS ownership, and
no partial or absent-owner authority cached after rejection.

## Change and boundary

The session retains an ordered owner-to-root index borrowing exact trees from
its document. Direct inline View / composition roots and transitively registered
EXISTS roots are admitted; arbitrary relation subtrees remain excluded. Each
distinct relation or expression discovery visit consumes the existing
semantic-step or expression-node allowance before expansion. All roots for the
owner are discovered once. Later calls reuse the complete index and still
compare the requested tree with the sealed tree; matching raw keys alone are
not accepted. An absent caller-supplied owner is rejected before cache insertion.
A failed build leaves no partial index and does not refund spent traversal work.

Discovery visits each distinct node once per owner, with ordered-map/set lookup
costs. Repeated membership requests no longer rescan sibling producer trees.
The cache stores borrowed roots rather than cloned trees; transient visited sets
are discarded after construction. Existing source and semantic collection bounds
still limit the input carriers. No timing, peak-memory or downstream-runtime
speedup is asserted here.

The budget behavior intentionally tightens: near-limit nested inputs that
previously paid only for lowering can now reject while paying for discovery.
The existing `algebra.traversal_budget_exceeded` code is retained, with an exact
ownership-discovery message and checked location. Top-level relation lowering
that needs no separate nested ownership lookup keeps its previous accounting.
No public signature, execution DTO field or digest schema changes.

## Proof

- A Project(Scan), Path(Binding) fixture spends exactly two relation and two
  expression discovery visits, plus two relation visits per lowering. A second
  request consumes only lowering allowance; a third exhausts the same ledger.
- Real composition transactions with 1, 4 and 16 query producers each have one
  Scan per producer and one body Scan. Discovery plus all lowerings consume
  exactly 4, 10 and 34 semantic steps, respectively, with zero expression visits.
- One-under relation/expression discovery budgets reject with exact code,
  message and spans `(125,134)` / `(152,153)`, and publish no index.
- Absent owner identities cannot grow the cache. A cached matching identity with
  a changed predicate/output-equivalent tree rejects, as does an ordinary child
  subtree. Both levels of nested EXISTS remain admitted under one owner index.
- Five compiling mutants remove cache reuse, exact-tree comparison, relation
  accounting, expression accounting or absent-owner protection. Each is checked
  against one selected failing assertion; compile failures do not count.

Raw baseline, mutation and restored-run output is under
`target/ownership-index-audit/` in the owner worktree. The clean committed-HEAD
qualification receipt is recorded separately. D-08 and D-09 are separate
optional suggestions and are not claimed fixed by this change.
