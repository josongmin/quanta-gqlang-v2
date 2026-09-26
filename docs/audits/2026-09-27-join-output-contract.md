# D-02: join frame and computed row contracts

Base: `676009d2a77cc21ac8e6ee98cf43c54d36a86e76`.

## Acceptance and reproduction

The remaining D-02 row/cardinality claims concern Sema sealing and Algebra
construction, consumed by planning and execution projection. Acceptance requires
exact operand-derived Join row/columns/provenance, Bag cardinality, empty
partition and ordering, and agreement between computed output field types and
their already checked expressions. No type inference or name-based recovery
belongs at this boundary. Malformed contracts must return the existing typed
constructor failures. Source syntax, valid output and facade API stay unchanged.

Three new tests each selected and failed one test against the base production
implementation: Sema Join accepted Set output; Algebra Join accepted the same;
Algebra Project accepted a changed scalar field type with identical column
origins. These are reproduced internal-constructor defects, not demonstrated
source-language exploits. Normal Join production obtains its row from the types
owner, fixes Bag cardinality and clears partition/order. The types owner derives
Join rows from the ordered child rows and computed fields from exact expression
facts. Feature-gated output controls rebuild a generically valid row-aligned
contract without changing its column origins, so rejection cannot be attributed
to generic row alignment.

## Structural change

Sema's `has_exact_join_frame` is shared by Sema stage sealing and Algebra node
construction. It compares the complete ordered fields and columns, the exact
source-set union, Bag cardinality and empty partition/order. Iterator equality
checks lengths as well as values; the set union comparison allocates no new set.
The work is linear in the operand/output widths and source counts.

Algebra's computed-row check compares exact expression origins and exact known
value types against output fields. Project, Group and Aggregate use the same
predicate. Unknown types reject rather than acting as wildcards. This consumes
existing types-owned facts; it does not run inference again. Tests change a
scalar type and required/optional shape independently while leaving column
identities intact. Grouped sum, scalar count and ordinary projection remain
accepted; the explicit Group child is tested separately from its Aggregate.

The join matrix covers inner, cross and self-join, each against Set, Sequence,
changed scalar type, changed optionality, retained partition and retained order.
The mutation experiment independently removes each new Join check and each of
the three computed-row call sites. All seven mutants compiled and failed the
selected assertion (one selected, one failed each); compile failures were not
counted. The original implementations were restored afterward. Raw baseline and mutation logs are retained
under `target/join-output-audit/` in the owner worktree; clean qualification is
recorded separately from these focused checks.

## Semi/Anti correlation disposition

D-02's correlation statement identifies a different owner, not a reproduced
source acceptance defect. Binder owns complete lexical capture sets. Sema's
`validate_capture_vector` compares the whole ordered witness to Binder's capture
set for the exact EXISTS identity, verifies each binding owner/ordinal and its
resolution, and checks the exact nested HIR root before producing the typed
EXISTS fact. Its owner matrix rejects dropped, duplicate, reordered, foreign,
wrong-owner/ordinal, wrong-root and missing-set witnesses.

Algebra's `exists_leaf` consumes that typed fact and its registered nested root,
checks duplicate/foreign references, and lowers both together in the same
post-order work item. The checked node constructor preserves the left output
frame; it is not a second lexical binding authority. Existing correlated
EXISTS/NOT EXISTS and nested-EXISTS tests pin capture identities and keep the
levels separate. The dropped/swapped capture fixtures deliberately replace the
already sealed authority through a feature-only test seam; Algebra preserves
that supplied authority. They do not demonstrate a normal producer bypass.

Accordingly, D-02's Project, Join and computed-row constructor gaps are addressed
by the two follow-up changes. Correlation correctness remains owned and tested
at Binder/Sema plus exact Algebra retention; it is not claimed to be independently
re-derived by `CheckedAlgebraNodeV2::new`. The constructor documentation states
that boundary explicitly. This disposition does not claim all possible frontend
defects have been excluded.
