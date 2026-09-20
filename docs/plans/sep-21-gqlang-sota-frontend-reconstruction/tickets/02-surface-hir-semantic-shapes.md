# 02 — Surface And HIR Semantic Shapes

Status: `VERIFIED`

Decision authority: [00 — Packet Scope And Pinned Decisions](./00-packet-scope-and-pinned-decisions.md)

## Findings Closed

F01's producer mismatch, F09, F11, F33, F34, and F35.

## Goal

Make the source-shaped tree unambiguous and lossless so downstream semantics do
not reconstruct intended stages from string tags or synthetic nodes.

## Current-Head Reuse/Delete/Build Audit

| Class | Assets | Concrete action |
|---|---|---|
| `KEEP` | `HirBuilderV1` ID allocation, byte spans, `CheckedHirV1::check_tree`, visitor traversal, precedence-climbing parser | extend these canonical implementations; do not add a parallel AST, builder, integrity pass, or parser |
| `MIGRATE` | `RelationExprKindV1`, query stage parser, lexer string path, formatter, structural test helpers | replace ambiguous carriers/emitters in place while retaining one source tree |
| `DELETE` | `CompositionOpV1`, `CompositionV1`, `_keys`, Filter-wrapped JOIN, direction-as-alias encoding, permissive unknown escapes | remove after every builder/visitor/formatter caller is exhaustive over the typed variants |
| `NEW` | explicit inner/cross join HIR, typed group/order keys, aggregate-star carrier, independent structural projection, `ParseBudgetV2` | add only these missing contracts |

Existing `v3_item_identity.rs`, `v3_visit_traversal.rs`, `v3_surface_query.rs`,
and `v3_surface_proptest.rs` are updated, not recreated. Escape/precedence cases
already living in `v3_format_edges.rs`, `v3_format_hellgate.rs`, and
`v3_mutation_rails.rs` move to the new independent oracle files; obsolete
V1-shape assertions are deleted in the same change.

## Write Set

1. `crates/quanta-gqlang-v2-hir/src/relation.rs`
2. `crates/quanta-gqlang-v2-hir/src/expr.rs`
3. `crates/quanta-gqlang-v2-hir/src/item.rs`
4. `crates/quanta-gqlang-v2-hir/src/build.rs`
5. `crates/quanta-gqlang-v2-hir/src/visit.rs`
6. `crates/quanta-gqlang-v2-hir/src/integrity.rs`
7. `crates/quanta-gqlang-v2-hir/src/span.rs`
8. `crates/quanta-gqlang-v2-hir/src/lib.rs`
9. `crates/quanta-gqlang-v2-surface/src/query.rs`
10. `crates/quanta-gqlang-v2-surface/src/expr.rs`
11. `crates/quanta-gqlang-v2-surface/src/family.rs`
12. `crates/quanta-gqlang-v2-surface/src/lexer.rs`
13. `crates/quanta-gqlang-v2-surface/src/format.rs`
14. `crates/quanta-gqlang-v2-surface/src/budget.rs`
15. `crates/quanta-gqlang-v2-surface/src/cursor.rs`
16. `crates/quanta-gqlang-v2-surface/src/parser.rs`
17. `crates/quanta-gqlang-v2-surface/src/diag.rs`
18. `crates/quanta-gqlang-v2-surface/src/shape.rs`
19. `crates/quanta-gqlang-v2-surface/src/unit.rs`
20. `crates/quanta-gqlang-v2-surface/src/lib.rs`
21. the exact HIR/surface test files listed below
22. `docs/plans/sep-21-gqlang-sota-frontend-reconstruction/receipts/02.md`
23. `crates/quanta-gqlang-v2-surface/tests/v3_family_edges.rs`
24. `crates/quanta-gqlang-v2-surface/tests/v3_mutation_rails.rs`
25. owner-local `hir/tests/support/mod.rs` or `surface/tests/support/mod.rs`
    only when at least two touched tests consume the extracted helper

## Implementation

1. Replace `CompositionOpV1 { symbol }` with typed Join/GroupBy/OrderBy
   structures.
2. Represent JOIN `on` directly; remove outer-Filter encoding.
3. Remove `_keys` synthetic sources from group/order parsing.
4. Change generic call arguments to a closed `Expr | Star` carrier so
   `count(*)` is never represented as a symbol. Syntax allows `Star` only as
   the sole argument; ticket 04 resolves the callee and rejects star for a
   non-aggregate builtin or predicate.
5. Separate rule quantification from relational EXISTS subqueries.
6. Make USE PACK lockfile required and restrict its activation type.
7. Restrict composition producer/body through dedicated enums.
8. Decode string bodies on UTF-8 boundaries.
9. Reject unknown escapes; support the explicitly documented closed table.
10. Emit expressions with parent precedence and associativity.
11. Provide an internal/test-support, visitor-derived id/span-free structural
    witness for round trips. It exhaustively encodes every semantic field of all
    nine `ProgramItemHirV1` variants plus nested value, Bool, relation, result
    shape, alias, order direction, binder, and CallArg Star/Expr payloads; only
    IDs, spans, and syntax-only redundant `Paren` wrappers are omitted. Required
    grouping still changes the nested operator tree and therefore the witness;
    explicit-paren source preservation is covered separately by exact formatter
    cases. Mutation pairs prove each previously omitted semantic field changes
    the witness. It is not public syntax IR and cannot be consumed by production
    stages.
12. Add `ParseBudgetV2` and charge tokens, document items, expression
    depth/nodes, collection width, and projection width.
13. Extend the existing integrity implementation with a document-level check
    that shares one minted-ID set and source bound across every item. Do not
    loop over `check_tree` with a fresh set per root.
14. Add HIR-owned `SourceIdV2` and `LocatedSpanV2`; validate a located span
    against the source registered for that ID. Modules and sema reuse this
    carrier for cross-document diagnostics.

## Required Type Changes

Replace the open relation composition carrier inside the existing canonical
HIR. Raw HIR is not facade API, so its existing internal `*V1` root names are
migrated in place; a second V2 tree or V1/V2 conversion layer is forbidden.
New payloads make invalid JOIN states unrepresentable:

```rust
enum RelationExprKindV1 {
    Source(RelationSourceV1),
    Filter(RelationFilterV1),
    Project(ProjectionV1),
    Join(JoinHirV1),
    GroupBy(GroupByHirV1),
    OrderBy(OrderByHirV1),
}

enum JoinHirV1 {
    Inner {
        left: Box<RelationExprHirV1>,
        right: Box<RelationExprHirV1>,
        on: BoolExprHirV1,
    },
    Cross {
        left: Box<RelationExprHirV1>,
        right: Box<RelationExprHirV1>,
    },
}

enum CallArgHirV1 {
    Expr(ValueExprHirV1),
    Star { span: SpanV1 },
}
```

`CallExprV1` does not expose a raw public `Vec<CallArgHirV1>`. A private checked
`CallArgsHirV1` carrier admits exactly either zero-or-more `Expr` arguments or
one sole `Star`; mixed/multiple Star states cannot be built through public HIR
constructors. Every parser/builder ingress uses that carrier.

`PathAccessV1` likewise stores a private-backed `NonEmptyPathSegmentsV1` and is
created through `HirBuilderV1::try_path`. A zero-segment path is the base value,
not a distinct HIR state, and rejects before an ID-bearing path node is minted.
`ProjectionV1` uses the equivalent checked `NonEmptyProjectionItemsV1` carrier
and `try_project`; the parser and programmatic HIR therefore admit the same
minimum cardinalities.

`GroupByHirV1` and `OrderByHirV1` do not expose raw public `Vec` key fields.
Each uses one private, checked `NonEmptyRelationKeysV1` carrier (or an
equivalent private field plus fallible constructor) so a zero-key stage cannot
be built programmatically. Ordered keys retain direction as a typed enum.
`CallArgHirV1::Star` participates in integrity span validation and visitor
coverage even though it has no separately minted node ID.

`| join <source> on <predicate>` produces `JoinHirV1::Inner` and requires
`on`. `| cross join <source>` produces `JoinHirV1::Cross` and has no `on`
field.
No other join kind or UNION production is added in this packet. Sema validates
the same invariant; HIR construction preserves authored intent.

`BoolExprKindV1` uses separate variants and payloads for:

1. logical rule quantifier binders/body
2. relational EXISTS/NOT EXISTS subquery
3. scalar/builtin calls

`LogicalQuantifierHirV1` contains a non-empty binder list of
`LogicalBinderV1 { name, span }`, quantifier kind, and Bool body; binder spans
must not be discarded. `RelationExistsHirV1` contains polarity plus a relation query and
cannot carry a result shape. Both forms parse wherever Bool syntax is accepted;
ticket 04 rejects a logical quantifier outside a rule and a relation-existence
node inside a rule. `not exists` is one explicit polarity value, not a generic
`Not(Exists(...))` rewrite.

Do not reuse one ambiguous node and ask later consumers to guess its family.
The grammar is exact: rule quantifiers require at least one binder and `in`;
relational existence is `exists (<query>)` or `not exists (<query>)`, where the
nested body starts with `from`. A quantifier in query context and an existence
subquery in rule context are typed context errors rather than reinterpreted.

Surface entry points are pinned:

```rust
parse_source_document_v2(
    source_id: SourceIdV2,
    source: &str,
    budget: &ParseBudgetV2,
)
    -> Result<ParsedDocumentV2, SurfaceDiagnosticV2>;
format_source_document_with_parse_budget_v2(
    source_id: SourceIdV2,
    source: &str,
    budget: &ParseBudgetV2,
)
    -> Result<String, SurfaceDiagnosticV2>;
check_document_v2(document: &HirDocumentV1)
    -> Result<CheckedHirDocumentV2<'_>, HirIntegrityDiagnosticV2>;
```

`SourceIdV2` is allocated by the owning frontend transaction/source registry;
surface must not hard-code, hash, or globally mint it. `ParsedDocumentV2` owns
one `HirDocumentV1 { source_id, source_len, items }`;
the HIR checker consumes that HIR-owned view, so HIR never depends on surface
and callers cannot supply a mismatched second source length.

These are owner-crate APIs. Ticket 07's facade-level
`format_source_document_v2(SourceIdV2, &str, &FrontendBudgetV2)` projects the
public budget to `ParseBudgetV2` and calls the second function; surface must not
depend on sema. Formatting retains the caller-owned source identity for every
diagnostic. No formatter layer invents source ID `0`; only an explicitly named
legacy single-source wrapper may do so under the restriction below.

Existing public `lex`, expression/predicate/program parsers, document parser,
and formatter entry points become bounded convenience wrappers over the same
implementation using `ParseBudgetV2::standard_v2()`. They are not an unmetered
second path. Custom-budget entry points share the same lexer, cursor, builder,
and parser functions. Legacy single-source wrappers may use the explicitly
documented standalone `SourceIdV2::from_raw(0)` because their return types do
not expose a document identity; every document/transaction API accepts a
caller-owned non-implicit source ID.

## Parse Budget Accounting

1. `ParseBudgetV2` is immutable configuration with private positive fields;
   `ParseBudgetCounterV2` is one per document and is never reset by nested
   parsers.
2. Token count includes the terminal EOF token. Charge before appending token
   `limit + 1`.
   Identifiers, strings, and numeric literals are scanned to byte bounds first;
   the counter charges the token before allocating/copying its owned payload.
   An over-limit token therefore cannot allocate proportional to that rejected
   token's payload.
3. Document items and expression nodes are cumulative per document. Expression
   nodes include every value and Bool HIR node; charge immediately before the
   canonical builder mints the node.
   Single-item and multi-item entry points both charge one document item; no
   wrapper may bypass this dimension.
4. Expression depth migrates the existing cursor recursion-depth guard into the
   shared counter. Exactly depth 64 succeeds under the standard profile; depth
   65 rejects before descent.
5. Collection width is per collection, checked before appending element
   `limit + 1`, for call arguments, list elements, record fields, path segments,
   quantifier binders, result-shape fields, family reference lists, facets, and
   composition bindings.
6. Projection width is the stricter per-projection dimension and is charged in
   addition to collection width; both counters must admit the element.
7. Every exhaustion emits `surface.resource_exhausted` with stable
   `dimension`, `limit`, `observed`, and exact triggering span. No allocation
   proportional to the rejected element occurs before the charge.
8. Exact-limit tests construct a budget at the boundary and one lower; they do
   not depend on the standard profile's large constants.

## Pinned String Contract

The source escape table is closed:

| Escape | Value |
|---|---|
| `\"` | quotation mark |
| `\\` | reverse solidus |
| `\n` | line feed |
| `\r` | carriage return |
| `\t` | horizontal tab |
| `\b` | backspace |
| `\f` | form feed |
| `\u{H}` through `\u{HHHHHH}` | Unicode scalar value |

Reject an unknown escape, empty/braceless Unicode escape, non-hex digit, value
above `U+10FFFF`, and surrogate `U+D800..U+DFFF`. Formatter emits printable
Unicode directly, escapes quote/reverse-solidus and the six named controls, and
uses lowercase `\u{...}` for other control code points. It never emits or
accepts UTF-16 surrogate pairs.

## Step-By-Step Procedure

1. Add new HIR types and exhaustive visitor support without changing the parser.
   Group/order non-empty keys and logical binders are private checked carriers;
   Star and every binder/key span participate in integrity traversal.
2. Add builder methods for every new type and unit-test their IDs/spans.
3. Migrate JOIN parsing, add the exact `cross join` production, reject ordinary
   JOIN without `on`, and delete the Filter(Compose) construction.
4. Migrate GROUP/ORDER parsing and delete `_keys` source synthesis.
5. Add aggregate-star parsing only in the syntactically valid call position.
6. Split quantifier/subquery syntax carriers, require binder-plus-`in` for rule
   quantifiers, and parse `exists (from ...)` / `not exists (from ...)` into the
   dedicated relational carrier.
7. Tighten USE PACK and composition types.
   `CompositionProducerV1::Sample` and `CompositionBodyV1::Explain` must carry
   variant-specific `SampleHirV1` and `ExplainHirV1` nodes (or equivalently
   closed payloads with their own id/span), not a general
   `PresentationHirV1`; constructing SHOW/META/etc. in those positions must be
   impossible in the type system.
8. Update formatter dispatch for every new HIR variant.
9. Replace string lexing with UTF-8 slice/char iteration while retaining byte
   spans.
10. Implement a closed escape decoder and matching formatter encoder.
11. Add precedence-aware value and Bool emitters.
12. Add the independent structural projection and migrate round-trip tests.
13. Thread `ParseBudgetV2` through every source/document parser entry point and
    return a typed surface resource diagnostic on exhaustion.
14. Delete `CompositionOpV1`, `_keys`, and old formatter helpers in the same
    change; do not leave dual representations.
15. Run the existing-test disposition above: update/move an assertion before
    adding a semantically equivalent one, then delete the obsolete V1 case.
16. Extend the existing integrity visitor to share ID/span/source-bound state
    across document roots and add a cross-root duplicate-ID RED. Ticket 04's
    owned `CheckedSyntaxDocumentV2` consumes this proof into an owned carrier.
17. Update every current bounded/unbounded surface entry point to delegate to
    the same budgeted implementation; migrate the existing cursor depth guard
    rather than adding a second depth owner.
18. Add parsed-kind assertions for FIND `Flow`, `PointsTo`, and `Summary` to
    the existing family hellgate suite. These are missing branch oracles, not a
    reason to add a duplicate family test file.

## Test Files

Create or update:

1. `[UPDATE]` `crates/quanta-gqlang-v2-hir/tests/v3_hir_expr_kernel.rs` — typed relation-stage shapes and IDs
2. `[UPDATE]` `crates/quanta-gqlang-v2-hir/tests/v3_item_identity.rs` — document integrity and cross-root ID checks
3. `[UPDATE]` `crates/quanta-gqlang-v2-hir/tests/v3_visit_traversal.rs` — exhaustive new variants
4. `[UPDATE]` `crates/quanta-gqlang-v2-surface/tests/v3_surface_query.rs` — inner/cross/group/order/existence shapes
5. `[UPDATE]` `crates/quanta-gqlang-v2-surface/tests/v3_format_edges.rs` — UTF-8 and closed escape matrix
6. `[UPDATE]` `crates/quanta-gqlang-v2-surface/tests/v3_format_hellgate.rs` — programmatic precedence and independent projection
7. `[UPDATE]` `crates/quanta-gqlang-v2-surface/tests/v3_surface_expr_edges.rs` — quantifier/subquery/star grammar
8. `[UPDATE]` `crates/quanta-gqlang-v2-surface/tests/v3_surface_proptest.rs` — Unicode and structural properties
9. `[ADD]` `crates/quanta-gqlang-v2-surface/tests/v3_surface_budget.rs` — no existing budget owner file
10. `[UPDATE]` `crates/quanta-gqlang-v2-hir/tests/v3_item_hellgate.rs` — migrated relation, USE PACK, and composition carriers
11. `[UPDATE]` `crates/quanta-gqlang-v2-surface/tests/v3_family_hellgate.rs` — migrated USE PACK/composition plus exact FIND kind assertions
12. `[UPDATE]` `crates/quanta-gqlang-v2-surface/tests/v3_surface_unit.rs` — remove zero-binder optional-`in` quantifier assumptions
13. `[UPDATE]` `crates/quanta-gqlang-v2-surface/tests/v3_document_hellgate.rs` — bounded document API and single source-length authority

Delete superseded assertions from `v3_mutation_rails.rs` only after their
stronger semantic oracle is present. Record every moved/deleted test function
in the receipt. All other current tests calling migrated public parse/format
entry points are mechanical call-site migrations inside this ticket's declared
test write set; they are not copied into new files. Do not bulk-delete surface
mutation/hellgate suites: the audited surface and HIR responsibilities are
nearly disjoint.

## Audited Existing-Test Disposition

Perform this cleanup only after the new owner behavior is green:

1. Delete `nested_item_ids_are_globally_unique_through_one_builder` because
   `composition_nests_query_producer_and_find_body` retains the stronger sound
   tree/count assertion. Trim only the compile-time `is_binary_plus` assertion
   from `v3_hir_expr_kernel`, retaining both integrity checks.
2. Delete `find_through_and_sanitizers_empty_reject` only after retaining the
   stronger message-and-span hellgate case. Remove the three byte-identical
   round-trip inputs already covered by the stronger format helper.
3. Replace five hand-written `SurfaceItemV1 -> ProgramItemHirV1` matches with
   `SurfaceItemV1::into_program_item()`.
4. Delete `lex_is_deterministic`; remove all `is_ok() || is_err()` tautologies
   while retaining the parser/lexer call so panic-freedom remains tested.
   Replace formatter self-comparison and third-pass assertions with the
   internal visitor-derived structural witness.
5. Preserve these non-duplicates: duplicate endpoint clause, duplicate list
   clause, empty-list-before-next-clause, full PROFILE/GATE message pins,
   per-family duplicate-clause sites, and the distinct record-literal versus
   result-shape duplicate-field rails.
6. Normalize lint reason text only in touched test files. Integration tests are
   separate crates, so a support module cannot centralize crate-level lint
   attributes.

## Exact RED Cases

```text
from findings f | join repos r on f.repo = r.id
from findings f | cross join repos r
from findings f | join repos r
let x = "한글🙂é"
let x = "\q"
let x = "\u{}"
let x = "\u{d800}"
programmatic Mul(Add(a,b),c)
programmatic Not(And(a,b))
compile-fail: USE PACK with lockfile=None
compile-fail: composition with META/SHOW as producer or SAMPLE as body
exists (1 = 1)
from users u | where exists (from sessions s | where s.user_id = u.id)
rule r : x if exists (from users u)
```

The two invalid-state compile-fail cases are doctests or an owner-local
compile-fail harness; they are not runtime constructors for states that the
new public types deliberately cannot represent. Expected after repair: inner
JOIN has a direct `on`; explicit CROSS has none;
the no-ON inner JOIN rejects; Unicode is byte-identical after
round trip; unknown escape and invalid programmatic item shapes reject; both
Unicode-escape negatives reject; both programmatic expression trees preserve
structure after format/reparse.

## Exact Verification

```bash
cargo test -p quanta-gqlang-v2-hir
cargo test -p quanta-gqlang-v2-surface
cargo test -p quanta-gqlang-v2-surface --test v3_surface_proptest
cargo clippy -p quanta-gqlang-v2-hir -p quanta-gqlang-v2-surface --all-targets -- -D warnings
git diff --check
```

## Forbidden Shortcuts

1. teaching algebra to recognize both old and new JOIN shapes
2. preserving `_keys` as a fallback
3. comparing formatter output to itself as the structural oracle
4. replacing invalid escapes silently
5. changing byte spans to character offsets
6. counting a limit only after allocating the unbounded collection
7. keeping the existing recursion-depth constant as a second counter
8. exporting the structural witness as a production AST/IR
9. preserving both V1 and V2 raw HIR hierarchies

## RED

1. normal JOIN parses to `Filter(Compose)`
2. non-ASCII string bytes are corrupted
3. `\q` loses its backslash
4. `Mul(Add(a,b),c)` formats to a different parse tree
5. programmatic HIR can encode lockless USE PACK and illegal composition slots

## Tests

1. exact inner JOIN/explicit CROSS/Group/Order HIR shape tests, including
   missing-ON and CROSS-with-ON negatives
2. Unicode, emoji, combining-character property tests
3. every valid escape and every invalid escape
4. generated programmatic expression trees across all precedences
5. parse-format-parse structural projection equality
6. invalid UsePack/composition construction tests
7. visitor exhaustiveness compile rail for every new variant
8. exact-limit and one-over-limit tests for every parse-owned budget dimension
9. rule-quantifier versus relational-existence context and ambiguity matrix
10. cross-root duplicate ID, per-root out-of-bounds span, and valid multi-item
    document integrity tests
11. wrong-source and out-of-range `LocatedSpanV2` tests
12. parsed FIND kind pins for Flow, PointsTo, and Summary

## Definition Of Done

1. no semantic operator in relation HIR is an open string
2. no parser-only structural invariant is bypassable through public constructors
3. formatter round-trip is proved independently of formatter text equality
4. HIR integrity checking is mandatory input to ticket 04
5. exactly one raw HIR tree, call-argument carrier, recursion counter, and
   document source-length authority remain

## Current-Head Final Audit Addendum — 2026-09-21

The adversarial re-audit found and closed four issues not fully represented by
the original receipt:

1. `SurfaceDiagnosticV2` is now a concrete source-qualified diagnostic, and
   both bounded parse and format entry points accept/retain caller-owned
   `SourceIdV2` on failure.
2. HIR integrity traversal now validates every id-less auxiliary span carried
   by path segments, callees, aliases, record/shape field names, declaration
   and rule names, graph references, legality/presentation references, catalog
   names, and composition names/bindings. Value-payload and graph-reference OOB
   rails prove both traversal classes.
3. Raw empty projection and empty path states are closed with private-backed
   non-empty carriers plus fallible builders. Exact empty-state rejects were
   added instead of parser-only assumptions.
4. The specified duplicate `find_through_and_sanitizers_empty_reject` was
   deleted only after its stronger message/span rail was confirmed present.

Join operands were reviewed but not narrowed: the pinned HIR definition in
this ticket intentionally permits relation-expression operands on both sides,
while the authored grammar currently produces a source on the right. This is
not a duplicate carrier or an implementation gap; later sema/algebra legality
owns restrictions on programmatically composed relation trees.

Current owner proof after these repairs:

```text
cargo test -p quanta-gqlang-v2-hir --all-targets: 69 passed
cargo test -p quanta-gqlang-v2-surface --all-targets: 204 passed
cargo test -p quanta-gqlang-v2-types --all-targets: 54 passed (try_path consumer migration)
```

The strict three-crate Clippy command is rerun after the final source change and
recorded in receipt 02. Broad workspace compilation remains intentionally red
at unmigrated ticket-04+ consumers of the atomic HIR cutover; it is not evidence
against these owner-local results and is not claimed green here.

## Dependencies

Requires 01. Tickets 02-08 are one unreleasable atomic cutover stack because
the canonical HIR carrier changes every downstream exhaustive consumer. Ticket
03's types work may be prepared independently, but its modules implementation
starts after 02; no intermediate 02-07 revision is a releasable workspace
contract.
