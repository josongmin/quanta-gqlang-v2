# Target Architecture

> 2026-09-23 execution facade supersession: the V2 compiler/analysis architecture below remains the semantic owner map, but its source execution API listing is historical. Current producer contract: [`docs/contracts/frontend-execution-v3.md`](../../contracts/frontend-execution-v3.md), implemented under [Ticket 10a](./tickets/10a-v3-frontend-execution-contract.md). The source entry alone seals `ExecutionDocumentV3`; resolved compilation has no executable digest.

Status: `PINNED FOR IMPLEMENTATION`

## Ownership Rules

### HIR

Owns source-shaped syntax, node identity, spans, and lossless formatting input.
It does not own name resolution, types, algebra, recursion legality, or plans.
It also owns `SourceIdV2` and `LocatedSpanV2 { source_id, span }`. Bare
`SpanV1` is valid inside one registered source; cross-document authority and
diagnostics use the located form so byte offsets cannot be attributed to the
wrong file.

Raw HIR remains one internal tree. Existing `*V1` HIR names are migrated in
place because raw HIR is not exported by the curated V2 facade. A version
suffix does not justify a parallel `*V2` syntax tree or a conversion layer.

### Modules

Owns module/import resolution, typed catalog namespaces, pack manifests, lock
verification inputs, duplicate policy, and source-rich module diagnostics.

### Sema

Owns the one document-scoped semantic transaction:

1. consume checked syntax
2. predeclare document symbols
3. resolve catalog/module/relation/predicate references
4. bind scopes in evaluation order
5. type expressions and relations
6. aggregate builtin effects
7. emit sealed semantic contracts

No downstream crate may repeat these decisions from raw strings or HIR shape.

### Algebra

Owns logical relational normalization from typed relation contracts. Public
nodes are valid by construction. It does not resolve names or type expressions.

### Logic

Owns graph-query boundedness, witness contracts, recursive-program SCC and
stratification, and semi/anti semantics. It consumes resolved IDs and typed
polarity, never callee strings.

### Planning

Owns host-agnostic truth/evidence/effect/result/capability classification from
verified algebra/logic/item contracts. It is not a physical optimizer.

### Pipeline

Owns orchestration only. It parses, invokes the semantic transaction, lowers the
sealed contracts, and assembles `CompiledDocumentV2`. It must not contain local
type walkers, name heuristics, or catalog mutation policy.

### Resource Budgets

Sema authority owns the public immutable `FrontendBudgetV2` configuration.
Pipeline projects it once per document into private owner-local counters; no
remaining-count state crosses crate boundaries and no dimension has two charge
owners.

| Dimension | Canonical charge owner |
|---|---|
| frame bytes | diagnostics stdio transport |
| source bytes | pipeline source ingress; CLI/service may mirror the same absolute pre-allocation rejection |
| tokens, document items, expression depth/nodes, collection/projection width | surface |
| semantic steps | sema |
| module graph nodes/edges/steps | modules |
| authority bytes and single authority-string bytes | modules/types owner of each sealed admission session |
| rule graph nodes/edges/steps, witness depth/count | logic |

The authority byte ceiling is per sealed admission session, not a global sum:
module authority, relation authority, and per-document catalog validation each
have exactly one non-resettable counter. The worst-case admitted bytes across
those three simultaneously resident domains is therefore three ceilings; host
concurrency/memory policy remains outside this frontend. Profile equality proves
the ceiling used by each proof, not aggregate byte usage.

Algebra and planning operate over already bounded sealed inputs and may not
recharge source/expression dimensions. Pipeline creates exactly one counter per
applicable owner and invokes that owner once. Mirrored byte prechecks are not
cumulative counters and must produce the same limit/code as pipeline.

### Frontend Contract

Owns the intentionally small downstream API. CLI and the diagnostics stdio
service consume only this crate.

## Final Dependency DAG

```text
consumer -> direct internal dependencies

hir                 -> none
surface             -> hir
binder              -> hir
types               -> hir
modules             -> hir + types
sema                -> hir + binder + types + modules
algebra             -> sema + types
logic               -> sema + types
planning            -> sema + algebra + logic + types
pipeline            -> surface + sema + algebra + logic + planning + modules
frontend-contract   -> pipeline
cli                 -> frontend-contract
diagnostics-stdio   -> frontend-contract
conformance         -> frontend-contract
```

Development-only exceptions must be explicit in the architecture guard. Normal
dependencies may not point opposite this graph.

## Semantic State Machine

```text
SourceDocument
  -> ParsedDocumentV2
  -> CheckedSyntaxDocumentV2
  -> DeclaredDocumentV2
  -> ResolvedDocumentV2
  -> SemanticDocumentV2
  -> LoweredDocumentV2     // exact-key assembly by SemanticItemIdV2
  -> CompiledDocumentV2
```

Each transition is fallible and typed. Only the final state is externally
servable. No `Option` means "the stage was skipped".

## Authority Model

```rust
pub struct ClosedFrontendAuthorityV2 {
    modules: ValidatedModuleAuthorityPartsV2,
    relations: RelationCatalogV2,
    graph_references: GraphReferenceCatalogV2,
    budgets: FrontendBudgetV2,
}
```

`ValidatedModuleAuthorityPartsV2` is one sealed modules-owned result containing
the resolved graph, external catalog, validated pack authority, exact host
source registry, and its `AuthorityBudgetProfileV2`. `ResolvedGraphV2` remains
the single owner of current-module identity; the closed authority exposes it by
delegation and does not store a second copy. `ValidatedPackAuthorityV2` owns a
validated graph plus the lock proof verified against that exact graph, so
independently validated graph/lock values cannot be mixed during authority
construction. Modules depends on the canonical `types` crate for catalog
signatures; it must not define an opaque or parallel type-tag universe.

`AuthorityBudgetProfileV2` is a types-owned immutable checked value containing
the exact module-graph node/edge/step and authority-byte/string dimensions.
`ModuleBudgetV2` is derived from that profile; it is not an independent public
configuration. `ValidatedModuleAuthorityPartsV2` and `RelationCatalogV2` each
retain the exact profile under which their untrusted authority inputs were
admitted. `FrontendBudgetV2` projects the same profile canonically.
`ClosedFrontendAuthorityV2::try_new` rejects unless all retained profiles equal
that projection. A proof built with a larger authority allowance therefore
cannot be paired later with a smaller frontend budget.

Construction is fallible. No raw `BTreeMap` or default-empty authority is a
semantic compile input.

Editor analysis uses a different type:

```rust
pub struct OpenAnalysisAuthorityInputV2 {
    optional_modules: Option<ValidatedModuleAuthorityPartsV2>,
    optional_relations: Option<RelationCatalogV2>,
    optional_graph_references: Option<GraphReferenceCatalogV2>,
    host_sources: SourceRegistryViewV2,
    budgets: FrontendBudgetV2,
}
pub struct OpenAnalysisAuthorityV2 { /* checked incomplete authority */ }
pub struct AnalysisDocumentV2 { /* may retain unresolved refs */ }
```

`AnalysisDocumentV2` cannot be converted implicitly into `CompiledDocumentV2`.

## Typed Semantic Contracts

Required core outputs and ownership:

1. `SemanticItemIdV2` — sema, canonical per-item join key
2. `ResolvedCallTargetV2::{Builtin, Predicate}` — types, reused without alias
3. `TypedExprV2` and Bool-proven `TypedBoolExprV2` — sema
4. `ExpressionEffectsV2` — types
5. `RelationOutputContractV2` — sema, retained unchanged by algebra
6. `ResolvedGraphQueryV2`, `ResolvedCheckV2`, `ResolvedPresentationV2` — sema
7. `LoweredRelationItemV2` — algebra, retaining `SemanticItemIdV2`
8. `ResolvedRuleV2` — sema producer; `RuleProgramContractV2` — logic consumer,
   both retaining item IDs. Sema's `RuleCallPolarityV2` is the canonical edge
   polarity input; logic maps it to its internal recursion model.
9. `GraphQueryContractV2` — logic, retaining the resolved graph item ID
10. `CheckContractV2` and `PresentationContractV2` — pipeline assembly

`SourceQualifiedNodeIdV2` is the only expression identity. Parallel names such
as `TypedExprIdV2`, `PredicateIdV2`, and `ResolvedCallV2` are forbidden. Every
document assembly compares exact expected and produced item-ID sets; positional
zip is forbidden.

Expression effects retain:

1. purity
2. evaluation determinism
3. monotonicity
4. aggregation class
5. semantic cost class

## Relational Stage Contract

Source HIR has six explicit relation-stage variants for source-reachable
syntax: source, filter, project, join (inner or explicit cross), group-by, and
order-by. The
grammar does not acquire union, limit, semi-join, anti-join, or HAVING merely to
make every algebra operator source-reachable. Typed relation IR may add derived
aggregate and correlated-subquery forms. Checked logical algebra retains its
existing 12-operator vocabulary, including programmatic-only union/limit/apply
forms. A string operator tag is not a valid semantic carrier at any boundary.

Every typed stage carries its output:

1. row schema
2. relation source identities
3. multiplicity/cardinality
4. partition keys
5. ordering
6. expression effects

Composition consumes this output contract directly. It never reconstructs row
types from raw HIR.

## Rule Contract

1. A source `rule <name> : <head-term> if <body>` predeclares predicate `<name>`.
2. Current grammar is unary unless the syntax ticket explicitly expands arity.
3. Variables present in the head are rule variables.
4. Quantifier variables are scoped explicitly.
5. Other free value symbols reject.
6. Predicate dependencies require call syntax and resolve by predicate ID.
7. Bare predicate symbols reject unless zero-arity predicates are introduced.
8. Positive, negated, and aggregated polarity comes from typed expression nodes.
9. All document rules form one program before Tarjan/stratification.

## Planning Contract

Planning cannot import raw HIR. Its entry point accepts only sealed inputs:

```rust
pub enum SemanticPlanInputV2 {
    Relational(LoweredRelationItemV2),
    Graph(GraphQueryContractV2),
    Check(CheckContractV2),
    Presentation(PresentationContractV2),
    Rule(RuleProgramContractV2),
}
```

`AuthorityLaneV2` and `EvaluationDeterminismV2` are separate. A nondeterministic
builtin must not be hidden behind a lane named deterministic.

One document-wide `RuleProgramContractV2` produces one
`CompiledRuleProgramV2` and one semantic plan. Source rule items retain checked
member references; they do not duplicate that plan. Program member identity is
validated as an exact set independently from per-item plan keys.

## Public API

The facade exports source/document entry points and stable DTOs only:

```rust
compile_source_document_v2
analyze_source_document_v2
format_source_document_v2
ClosedFrontendAuthorityV2
ClosedFrontendAuthorityBuilderV2
OpenAnalysisAuthorityV2
FrontendBudgetV2
CompiledDocumentV2
AnalysisDocumentV2
SemanticPlanV2
FrontendDiagnosticV2
FrontendFailureV2
FrontendAuthorityErrorV2
FrontendTransactionErrorV2
```

It does not export raw HIR builders, unchecked type/algebra constructors,
normalizer internals, or crate-wide globs.

## Deleted Seams

The cutover removes or makes crate-private:

1. `compile_program_structural`
2. `compile_item_structural`
3. `compile_item_with_schema`
4. public `compile_item_with_context`
5. raw-HIR `classify_item`
6. `env_closed` skip semantics
7. `root_symbol_name` dependency inference
8. raw-HIR `producer_output_relation` reconstruction
9. string-tagged `CompositionOpV1`
10. frontend-contract glob re-exports

Compatibility wrappers that preserve these behaviors are explicitly forbidden.
