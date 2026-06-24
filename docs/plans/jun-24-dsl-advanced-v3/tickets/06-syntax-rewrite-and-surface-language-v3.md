# 06 Syntax Rewrite And Surface Language v3

## Goal

Extend the surface grammar so the public language can express the HIR and
algebra cleanly.

## Why

Current parser is line-oriented and root-keyword dispatched. That is a good fit
for the current shell, but the wrong fit for advanced expression/query syntax.

Current head already ships `quanta-gqlang-v2-surface` as the owner lane. This
ticket extends that parser/formatter surface; it does not authorize a second
public grammar implementation.

## Deliverables

1. precedence-aware expression parser
2. query grammar supporting:
   - aliases
   - joins
   - subqueries
   - grouping
   - aggregates
   - sorting
   - quantified existence forms
   - path-pattern / recursive forms where admitted by `05`
3. declaration/rule grammar updated to embed shared expressions
4. readable compositional query surface for derived columns, filters, grouping,
   ordering, nesting, and pipeline-like transforms
5. compat parsing policy decided explicitly

## Recommended Parser Shape

1. keep top-level root discrimination only where it remains semantically useful
2. parse expressions with Pratt or precedence-climbing machinery
3. keep spans exact for LSP diagnostics and formatting
4. surface sugar may improve readability, but it must lower into the same HIR
   and must not create a second semantic language
5. nested/path forms must follow lexical binder-visible scope rules only

## Semantic Boundary

1. syntax accepts structural forms only
2. alias binding, symbol resolution, overload choice, and type legality stay
   owned by `02`/`03`
3. parser recovery must not silently encode `v2` subset semantics
4. surface syntax must not smuggle in alternate scoping rules that bypass the
   binder

## Scenarios Unlocked

1. `SELECT repo, count(*) FROM findings GROUP BY repo ORDER BY count(*) DESC`
2. `WHERE EXISTS (...)`
3. richer rule predicates and parameterized view bodies
4. nested or shape-like result projections without stringly post-processing
5. readable path-pattern or recursive query surface without inventing a second
   semantic language

## Definition Of Done

1. new syntax maps to HIR first, not directly to legacy typed carriers
2. precedence/associativity rails are green
3. formatter can round-trip the new grammar canonically
4. readability sugar does not fork semantic lowering or diagnostics

## Dependencies

1. requires `00`
2. depends on `01`, `02`, and `03`
3. closes with semantic targets from `04` and `05`

## Likely Touch Points

1. `crates/quanta-gqlang-v2-surface/src/parser.rs`
2. `crates/quanta-gqlang-v2-surface/src/lexer.rs`
3. `crates/quanta-gqlang-v2-surface/src/expr.rs`
4. `crates/quanta-gqlang-v2-surface/src/query.rs`
5. `crates/quanta-gqlang-v2-surface/src/unit.rs`
6. `crates/quanta-gqlang-v2-surface/src/format.rs`

## Mandatory Worker Outputs

1. one precedence-aware grammar for advanced expressions and query forms
2. exact span behavior for diagnostics and formatting
3. compatibility decision written explicitly for any retained legacy parse path

## Proof Rails

1. extend `crates/quanta-gqlang-v2-surface/tests/v3_surface_query.rs`
2. extend `crates/quanta-gqlang-v2-surface/tests/v3_surface_expr_kernel.rs`
3. extend `crates/quanta-gqlang-v2-surface/tests/v3_format_hellgate.rs`
4. extend `crates/quanta-gqlang-v2-surface/tests/v3_surface_hellgate.rs`
5. add at least one negative rail proving parser recovery does not smuggle in
   `v2` subset semantics

## Reject Or Escalate

1. binder or overload semantics in parser code:
   belongs to `02`/`03`
2. alternate hidden path scoping:
   forbidden by `D25`
3. readability sugar that lowers to a second semantic language
