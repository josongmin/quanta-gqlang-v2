# QGLang frontend execution contract V3

Status: producer contract for the breaking, pre-release qglang/Semantica source-pin cutover. The producer is qglang's checked Pipeline. Runtime admission and execution are not part of this contract.

## Public boundary

`compile_source_document_v3(SourceIdV2, &str, &ClosedFrontendAuthorityV2) -> Result<CompiledSourceDocumentV3, FrontendTransactionErrorV2>` is the only closed source entry that issues an `ExecutionDocumentV3`. The result retains the source-ordered compiled item dispositions and exposes one immutable execution document. No public constructor accepts preassembled execution items or a caller-supplied digest. `compile_resolved_document_v2` can check a sealed semantic document without original bytes, but its result exposes **no** execution document or source/qualification digest. Open analysis is never promoted to closed execution.

`SourceIdV2`, `SemanticItemIdV2`, and `SourceQualifiedNodeIdV2` are transaction-local frontend identities. Their raw values and `SourceIdV2` are encoded deliberately. Contract and qualification digests are therefore valid for exact transaction-bound admission, not cross-transaction semantic-equivalence caching. Equal source bytes under different source IDs have equal source digests but different contract and qualification digests. Formatting-equivalent text has no promised contract-digest equality.

The document carries source-ordered executable items. Non-plan declarations are absent; their checked `NoPlanReasonV2` remains on the compiled result. Item lookup is by exact `SemanticItemIdV2`; a foreign or absent identity returns no item. Each item carries frontend plan authority, determinism, evidence, effect, and policy family. The V3 execution payload is complete for a top-level relation and for the immutable composition binding/body handoff. Graph, CHECK, presentation, and meta-required-certified currently carry their checked family/policy but no standalone executable payload in this DTO. Rule members and the document-wide rule program are not V3 execution items. These families are **not** implicitly executable by a runtime; the Semantica admission owner must either add a separately specified payload/cutover or reject them with an explicit typed reason before effects. The producer never silently drops a plan-required item to make a consumer succeed.

Composition bindings retain source order, exact binding identity, producer item/family, checked output schema/relation when present, producer kind, exact WITH target, and body item/family/relation/kind. Scoped SAMPLE, graph, and multi-binding forms may be represented by the frontend; representation does not grant runtime support. Semantica owns support/admission decisions.

The literal field of `ExecutionExpressionShapeV3::Literal` is `ExecutionLiteralV3`, projected only from `TypedLiteralFactV2`. Its variants are signed `Int(i128)`, normalized finite `Float(ExecutionFiniteFloatV3)`, `Bool(bool)`, UTF-8 `String`, and `Null`. The checked finite-float owner normalizes negative zero to positive zero and rejects non-finite values. `DeferredI128MinMagnitude` projects to `i128::MIN`. The public float wrapper exposes bits but cannot be constructed from arbitrary non-finite bits by a downstream caller. No literal text is reparsed or used as an execution value.

## Canonical digest encoding

All three digests are SHA-256 over distinct canonical byte streams. `STR(x)` means `U64BE(length of UTF-8 bytes) || UTF-8 bytes`; `SEQ(xs)` means `U64BE(count) || each element in declared order`; `OPT(x)` means one byte `0` if absent or `1 || x` if present. `U8`, `U32BE`, `U64BE`, and `I128BE` have fixed widths; signed integers use two's-complement big-endian bytes. Lengths come from the frontend's bounded checked transaction and are encoded as `U64BE`. No display/debug formatting, hash-map order, host pointer width, runtime ID, provider, generation, lease, or budget enters these streams.

Schema version is `U32BE(3)`. Domain strings include the `.v3` suffix:

| Digest | Exact top-level stream |
| --- | --- |
| `source_digest` | `STR("qglang.execution.source-bytes.v3") || U32BE(3) || U64BE(source byte length) || exact source UTF-8 bytes` |
| `contract_digest` | `STR("qglang.execution.contract.v3") || U32BE(3) || U32BE(SourceId.raw) || SEQ(source-ordered execution items)` |
| `qualification_digest` | `STR("qglang.execution.qualification.v3") || U32BE(3) || U32BE(SourceId.raw) || raw 32-byte source_digest || raw 32-byte contract_digest` |

An item is encoded in this order: `ItemId`, family tag, plan authority tag, determinism tag, evidence tag, effect tag, policy-family tag, optional relation, optional composition. `ItemId = U32BE(source ID) || U32BE(HIR item ID)`; `NodeKey = U32BE(source ID) || U32BE(node ID)`; `Binding = U32BE(source ID) || U32BE(owner node) || U64BE(ordinal) || STR(name)`.

The nested encoding is recursive and uses the following exact field order. `Relation = U8(1) || ItemId || Node || SEQ(expressions)`; each expression is `NodeKey || ValueType || ExpressionShape`. `Node = algebra-family tag || OPT(join-mode tag) || stage NodeKey || OPT(Source) || Schema || SEQ(predicate NodeKeys) || SEQ(expression NodeKeys) || SEQ(Correlation) || SEQ(partition-input Bindings) || SEQ(child Nodes)`. `Source = relation-authority tag || source-kind tag || OPT(STR(namespace)) || STR(name)`. `Correlation = Binding || reference NodeKey`. `Schema = SEQ(fields) || cardinality tag || SEQ(partition-key NodeKeys) || SEQ(order keys)`; a field is `STR(name) || ValueType` and an order key is `NodeKey || direction tag || null-order tag || ValueType`.

`Composition = U8(1) || SEQ(bindings) || WITH Binding || body ItemId || STR(body-family keyword) || OPT(body Relation) || Body`. Each composition binding is `Binding || producer ItemId || STR(producer-family keyword) || OPT(Schema) || OPT(Relation) || Producer`. `Producer` is `U8(0)` for Relation, `U8(1) || STR(query text) || sample-mode tag || U32BE(limit)` for Sample, or `U8(2) || graph-kind tag` for Graph. `Body` is `U8(0)` for Relation, `U8(1) || graph-kind tag` for Graph, `U8(2)` for Check, or `U8(3)` for Explain. The top-level optional relation/composition use `U8(0)` absent or the corresponding payload beginning with `U8(1)` present; no second presence byte is inserted.

`ValueType` tags are: `0 || scalar tag`; `1` Null; `2 || SEQ(ValueType)` Tuple; `3 || SEQ(STR(field name) || ValueType)` Record; `4 || ValueType` Optional; `5 || ValueType || cardinality tag` Collection; `6 || SEQ(STR(variant tag) || ValueType)` Union; `7` Unknown. An operator is `STR(symbol) || U32BE(overload ID)`; the spelling is a checked display witness, never reparsed. A call target is `U8(0) || STR(name)` Builtin, `U8(1) || STR(name)` Predicate, or `U8(2) || STR(fingerprint)` HostPredicate.

Within each tag family below, the first listed variant has tag `0`, the next tag `1`, and so on. This is an explicit ordered mapping, not a sorting rule:

| Tag family | Variants in tag order |
| --- | --- |
| item family | Relation, Graph, Check, Presentation, Composition, MetaRequireCertified, RuleProgram |
| plan authority | FrontendChecked, CertifiedRequirementValidated |
| determinism | Deterministic, NonDeterministic |
| evidence | None, SourceLocation, WitnessPayload, ProofTree |
| effect | ReadOnly, ProofEvidenceOnly, ExportOnly, MutationForbidden |
| policy family | Relational, Graph, Check, Presentation, RuleProgram, MetaRequireCertified, Composition |
| algebra family | Scan, Filter, Project, Join, Group, Aggregate, Sort, SemiJoin, AntiJoin |
| relation authority | Host, CanonicalWorkspaceFiles, CatalogView, CompositionBinding |
| source kind | Relation, View, Composition |
| join mode | Inner, Cross |
| scalar type | Bool, Int, Float, String, Duration, Location, Path, RepoPath, EntityRef, SymbolRef, RuleRef, SourceScope, ProofTree |
| cardinality | Set, Bag, Sequence |
| sort direction | Asc, Desc |
| null order | First, Last |
| sample mode | Lexical, Semantic, Hybrid |
| graph kind | Reachability, Flow, Slice, Taint, Path, PointsTo, Summary |
| logical quantifier | Exists, Forall |
| relation-exists polarity | Exists, NotExists |

For an expression, `NodeKey || ValueType || ExpressionShape` is encoded in its checked expression-table order. `ExpressionShape::Literal` is shape tag `U8(0)` followed by the typed literal payload:

| Literal | Encoding following shape tag `0` |
| --- | --- |
| Int | `U8(0) || I128BE(value)` |
| Float | `U8(1) || U64BE(normalized finite IEEE-754 bits)` |
| Bool | `U8(2) || U8(0 or 1)` |
| String | `U8(3) || STR(value)` |
| Null | `U8(4)` |

The remaining expression shapes use these exact tags and payloads:

| Tag | Shape payload after tag |
| --- | --- |
| 1 | Binding |
| 2 | base NodeKey, `SEQ(STR(path segment))` |
| 3 | unary Operator, operand NodeKey |
| 4 | binary Operator, left NodeKey, right NodeKey |
| 5 | CallTarget, `SEQ(OPT(argument NodeKey))` |
| 6 | parenthesized NodeKey |
| 7 | `SEQ(STR(field name) || NodeKey)` record |
| 8 | `SEQ(NodeKey)` list |
| 9 | Boolean-value NodeKey |
| 10 | comparison Operator, left NodeKey, right NodeKey |
| 11 | AND Operator, left NodeKey, right NodeKey |
| 12 | OR Operator, left NodeKey, right NodeKey |
| 13 | NOT Operator, operand NodeKey |
| 14 | quantifier-kind tag, `SEQ(Binding)`, body NodeKey |
| 15 | relation-exists polarity tag, relation-root NodeKey, `SEQ(Correlation)` |

The checked owner enforces finite float and exact source membership before encoding; public DTO fields and document constructor remain private. `crates/quanta-gqlang-v2-pipeline/src/execution.rs` is the producer implementation of these rules, not a second source of policy.

## Shared conformance vectors

`crates/quanta-gqlang-v2-pipeline/tests/fixtures/execution-v3-vectors.tsv` pins named source/authority cases and expected three lower-case hexadecimal digests. A same-byte/different-`SourceId` pair pins source equality and contract/qualification inequality. A byte-only change pins source/qualification inequality without assuming semantic-equivalence. Binding and authority/schema mutations pin contract/qualification inequality. Typed literals pin `i128::MIN`, normalized zero, finite float, bool, non-ASCII string, and null. Foreign item lookup is a producer negative; the Semantica admission owner additionally proves effect-free typed rejection. Changing only Semantica data generation must leave all three frontend digests unchanged and change the admitted runtime identity; that final pairwise assertion belongs in Semantica.

The TSV authority profiles use `FrontendBudgetV2::standard_v2()`, host source registration `SourceIdV2(1)` with length 64, selected module `app` version `1.0.0`, and checked host location `(1, 0..1)`. `minimal` has no relation or graph sources. `records_host` adds `app::records` (lookup `records`, Host authority, `host` provenance namespace, one `id:Int` field, Bag cardinality) and the checked graph selectors `a`, `b` (Entity), `scope` (Scope); graph selectors are irrelevant to the listed relation rows but part of the frozen authority fixture. `workspace_files_canonical` uses the facade builder's `canonical_workspace_files` relation; `workspace_files_host` uses that same schema with Host authority. The literal and record rows use `records_host`; composition rows use `minimal`. The TSV records exact one-line source UTF-8 text, source ID, profile, and digests, so a sibling can compile through the facade rather than reimplement this encoder.

The V3 producer may be called qualified only after owner tests, facade-only consumer/privacy proof, conformance, and a clean committed-source local qualification receipt pass on one exact qglang SHA. Consumer build and end-to-end service execution require a separately recorded exact Semantica SHA and dependency pin; producer-local GREEN does not imply either.
