# 14 Horn Rule Semantics And Recursive Predicate Lowering

> Historical 2026-06-24 snapshot. “Current Main” below refers to that dated
> source, not today's repository. Use the [current language authority](../../../language/README.md).

Status: `Implemented On Current Main`

## Current Main Receipt

As of `2026-06-24`, source horn-rule lowering is live on the public pipeline:

1. `compile_rule` lowers source horn `rule` through `lower_source_rule` into
   `RecursiveProgramV1`
2. unsupported arbitrary rule heads reject with
   `logic.unsupported_rule_head`
3. body literal dependency extraction is explicit and deterministic
4. horn `rule` compiles as a frontend rule artifact rather than stalling at
   parser/HIR presence
5. proof rails:
   `v3_normalize_item_decl_rule.rs`,
   `v3_logic_recursion.rs`,
   `audit_edge_cases.rs`
6. verified with:
   `cargo test -p quanta-gqlang-v2-pipeline`,
   `cargo test --workspace`,
   `cargo clippy --workspace --all-targets`

Pinned public restriction:

1. current head only accepts rule heads whose predicate root is a symbol/path
   root; broader arbitrary scalar heads are an explicit fail-closed reject, not
   an implicit TODO

## Goal

Turn source horn `rule` into a real frontend semantic lane by lowering it from
shared HIR into recursive predicate/program structures consumable by logic.

## Why

The pre-closeout gap had three disconnected facts:

1. the surface grammar accepts horn `rule`
2. shared HIR has a `RuleHirV1`
3. logic already owns recursive predicate semantics via `RuleDefV1` /
   `RecursiveProgramV1`

But the public pipeline still lacks the bridge between them. Parser/HIR support
therefore overstates actual rule support.

## Deliverables

1. pinned semantic contract for source horn `rule`
2. source `RuleHirV1` -> recursive predicate lowering
3. rule-head interpretation policy:
   - explicit predicate symbol ownership
   - explicit argument/term shape
   - explicit rejection for unsupported arbitrary-head shapes
4. body predicate dependency extraction from binder/type outputs
5. polarity extraction:
   - positive
   - negated
   - aggregated
6. module/schema-aware predicate reference binding where applicable
7. recursive/stratified rule-set construction from source rules
8. explicit split between horn `rule` and catalog `ruledef`
9. proof rails for positive recursion, stratified negation, and fail-closed
   unsupported rule heads

## Pinned Semantics

1. source horn `rule` is not just a generic value/bool convenience carrier; it
   must lower into explicit predicate semantics or reject
2. parser/HIR presence is not support unless the public pipeline reaches logic
3. rule head and body dependencies must be derived deterministically from
   frontend-owned semantics
4. catalog `ruledef` and horn `rule` remain distinct language families
5. recursion legality remains owned by `05`; this ticket owns only the source
   lowering into that logic contract

## Non-Goals

1. this ticket does not redefine stratification legality:
   ticket `05`
2. this ticket does not define non-relational compiled-artifact closure:
   ticket `15`
3. this ticket does not define semantica runtime execution of rules

## Scenarios Unlocked

1. a source recursive horn rule lowers into `RuleDefV1` and stratifies
2. a source horn rule with negated dependency surfaces explicit polarity to
   logic
3. an unsupported arbitrary scalar head rejects with a typed diagnostic instead
   of pretending to be a recursive predicate
4. a rule family can compile to a frontend artifact even when it is not yet
   servable

## Definition Of Done

1. public horn `rule` no longer stops at parser/HIR support
2. source horn rules can lower into recursive predicate semantics or reject
   explicitly
3. rule recursion rails are source-driven, not hand-built only from logic test
   fixtures
4. horn `rule` and catalog `ruledef` semantics are both explicit and
   non-overlapping

## Dependencies

1. requires `00`
2. depends on `02`, `03`, `05`, `06`, `11`, and `12`
3. closes before `15` may claim standalone horn-rule artifact closure

## Likely Touch Points

1. `crates/quanta-gqlang-v2-hir/src/unit.rs`
2. `crates/quanta-gqlang-v2-surface/src/unit.rs`
3. `crates/quanta-gqlang-v2-binder/src/resolve.rs`
4. `crates/quanta-gqlang-v2-logic/src/recursion.rs`
5. `crates/quanta-gqlang-v2-pipeline/src/compile.rs`

## Mandatory Worker Outputs

1. one explicit source horn-rule lowering contract
2. one deterministic body-dependency extraction path into logic
3. typed rejects for unsupported source rule heads/bodies

## Proof Rails

1. replace hand-built-only rule recursion coverage with at least one source
   horn-rule end-to-end rail
2. un-ignore and satisfy the horn-rule side of
   `crates/quanta-gqlang-v2-pipeline/tests/v3_normalize_item_decl_rule.rs`
   or replace it with the correct explicit artifact rail if servability stays
   downstream
3. add one fail-closed rail for unsupported arbitrary-head rule shapes

## Reject Or Escalate

1. treating `RuleHirV1` presence as equivalent to recursive rule support
2. conflating horn `rule` with catalog `ruledef`
3. inventing runtime-native rule execution DTOs in this repo
