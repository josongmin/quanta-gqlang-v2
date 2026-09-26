# QGLang frontend execution contract V3

Status: producer contract for the breaking, pre-release qglang/Semantica source-pin cutover. The producer is qglang's checked Pipeline. Runtime admission and execution are not part of this contract.

## Public boundary

`compile_source_document_v3(SourceIdV2, &str, &ClosedFrontendAuthorityV2) -> Result<CompiledSourceDocumentV3, FrontendTransactionErrorV2>` is the only closed source entry that issues an `ExecutionDocumentV3`. The result retains the source-ordered compiled item dispositions and exposes one immutable execution document. No public constructor accepts preassembled execution items or a caller-supplied digest. `compile_resolved_document_v2` can check a sealed semantic document without original bytes, but its result exposes **no** execution document or source/qualification digest. Open analysis is never promoted to closed execution.

`classify_single_source_root_v3(SourceIdV2, &str, &FrontendBudgetV2)` is a
canonical parse-only routing aid. It requires exactly one source item and
returns only its closed root-family enum; it does not resolve semantic catalogs,
issue an execution document, or grant host/runtime authority. A downstream
consumer may use it to reject an unsupported family before constructing closed
authority, but every supported family must still pass
`compile_source_document_v3` before execution.

`SourceIdV2`, `SemanticItemIdV2`, and `SourceQualifiedNodeIdV2` are transaction-local frontend identities. Their raw values and `SourceIdV2` are encoded deliberately. Contract and qualification digests are deterministic fingerprints of the projected DTO and source bytes with that source ID, **not** unique transaction-instance IDs or fingerprints of the entire host authority. Equal source bytes under different source IDs have equal source digests but different contract and qualification digests; a repeated compile with the same ID, bytes, and DTO has the same digests. They are not cross-transaction semantic-equivalence cache keys. Runtime admission must bind the selected document and item to its own source/authority closure, generation, provider, lease, and budget as applicable. Formatting-equivalent text has no promised contract-digest equality.

The document carries source-ordered **plan-required items**. Non-plan declarations are absent; their checked `NoPlanReasonV2` remains on the compiled result. `item(id)` is a document-local exact-key lookup: an absent or unequal raw `SemanticItemIdV2` returns no item, but a separately compiled document may reuse the same transaction-local raw ID. Lookup alone does not prove provenance. Each item carries frontend plan authority, determinism, evidence, effect, and policy family. The V3 execution payload is complete for a top-level relation and for the immutable composition binding/body handoff. Graph, CHECK, presentation, and meta-required-certified currently carry their checked family/policy but no standalone executable payload in this DTO. Rule members and the document-wide rule program are not V3 execution items. These families are **not** implicitly executable by a runtime; the Semantica admission owner must either add a separately specified payload/cutover or reject them with an explicit typed reason before effects. The producer never silently drops a plan-required item to make a consumer succeed.

Composition bindings retain source order, exact binding identity, producer item/family, checked output schema/relation when present, exact WITH target, and body item/family/relation. SAMPLE retains its optional resolved scope. Graph producers and bodies retain the complete logic-checked query: kind, ordered role-qualified targets, positive depth, optional direction, summary facets, and the sealed witness policy (relation shape, dedup identity, witness cap, path-length cap, and deterministic tie-break). Scoped SAMPLE, graph, and multi-binding forms may be represented by the frontend; representation does not grant runtime support. Semantica owns support/admission decisions.

The literal field of `ExecutionExpressionShapeV3::Literal` is `ExecutionLiteralV3`, projected only from `TypedLiteralFactV2`. Its variants are signed `Int(i128)`, normalized finite `Float(ExecutionFiniteFloatV3)`, `Bool(bool)`, UTF-8 `String`, and `Null`. The checked finite-float owner normalizes negative zero to positive zero and rejects non-finite values. For the authored `-2^127` boundary, the types owner uses `DeferredI128MinMagnitude` only for the positive-magnitude operand while recording `i128::MIN` on the unary root. Sema materializes that root as the canonical literal at the authored root identity; the sentinel operand remains a semantic proof node but is unreachable from, and therefore absent from, the execution expression closure. The public float wrapper exposes bits but cannot be constructed from arbitrary non-finite bits by a downstream caller. No literal text is reparsed or used as an execution value.

## Minimal facade use

A host supplies real, checked authority before source compilation. This example
registers the canonical workspace-files relation; it does not read files or
execute the query. All imported types come from the facade:

```rust
use quanta_gqlang_v2_frontend_contract::{
    compile_source_document_v3, ClosedFrontendAuthorityBuilderV2,
    FrontendAuthorityLocationV2, FrontendAuthoritySourceV2, FrontendBudgetV2,
    FrontendLanguageVersionV2, FrontendRelationInputV2, SourceIdV2,
};

fn main() {
    let host = SourceIdV2::from_raw(1);
    let location = FrontendAuthorityLocationV2 {
        source_id: host,
        start: 0,
        end: 1,
    };
    let mut builder = ClosedFrontendAuthorityBuilderV2::try_new(
        FrontendBudgetV2::standard_v2().expect("standard budget"),
        vec![FrontendAuthoritySourceV2 {
            source_id: host,
            source_len: 64,
        }],
        "app".to_owned(),
        location,
        FrontendLanguageVersionV2 {
            major: 1,
            minor: 0,
            patch: 0,
        },
    ).expect("builder starts");
    builder.add_relation(FrontendRelationInputV2::canonical_workspace_files(
        location, location,
    ));
    let authority = builder.finish().expect("authority seals");
    let compiled = compile_source_document_v3(
        SourceIdV2::from_raw(9),
        "from files f | project f.path as path",
        &authority,
    ).expect("source compiles");
    let execution = compiled.execution();
    let selected = execution.items().first().expect("one plan-required item");
    assert!(execution.item(selected.item_id()).is_some());
    assert!(selected.relation().is_some());
    let _frontend_identity = execution.qualification_digest();
}
```

The host selects by exact item identity, not a second name lookup, and must
not reimplement the digest encoder. The facade-only external-consumer program
at `tools/frontend_external_consumer/src/main.rs` is the compiled integration
reference. Selecting an item is not runtime admission: the downstream owner
must bind its own generation/provider/budget and reject unsupported payloads
before effects.

## Canonical digest encoding

All three digests are SHA-256 over distinct canonical byte streams. `STR(x)` means `U64BE(length of UTF-8 bytes) || UTF-8 bytes`; `SEQ(xs)` means `U64BE(count) || each element in declared order`; `OPT(x)` means one byte `0` if absent or `1 || x` if present. `U8`, `U32BE`, `U64BE`, and `I128BE` have fixed widths; signed integers use two's-complement big-endian bytes. Lengths come from the frontend's bounded checked transaction and are encoded as `U64BE`. No display/debug formatting, hash-map order, host pointer width, runtime ID, provider, generation, lease, or budget enters these streams.

The V3 Rust DTO uses source-byte digest schema 3 and contract/qualification
digest schema 5. Schema 5 is the explicit incompatible contract-encoding
revision that added scoped SAMPLE targets, complete Composition graph queries,
and their sealed witness policy. The unchanged source-byte stream remains on
schema/domain 3; revised contract streams use the `.v5` suffix so older bytes
cannot collide. The document exposes all three schema versions through
read-only accessors, so a downstream receipt never has to infer the digest
algorithm from the Rust DTO name or from a hash value:

| Digest | Exact top-level stream |
| --- | --- |
| `source_digest` | `STR("qglang.execution.source-bytes.v3") || U32BE(3) || U64BE(source byte length) || exact source UTF-8 bytes` |
| `contract_digest` | `STR("qglang.execution.contract.v5") || U32BE(5) || U32BE(SourceId.raw) || SEQ(source-ordered execution items)` |
| `qualification_digest` | `STR("qglang.execution.qualification.v5") || U32BE(5) || U32BE(SourceId.raw) || raw 32-byte source_digest || raw 32-byte contract_digest` |

An item is encoded in this order: `ItemId`, family tag, plan authority tag, determinism tag, evidence tag, effect tag, policy-family tag, optional relation, optional composition. `ItemId = U32BE(source ID) || U32BE(HIR item ID)`; `NodeKey = U32BE(source ID) || U32BE(node ID)`; `Binding = U32BE(source ID) || U32BE(owner node) || U64BE(ordinal) || STR(name)`.

The nested encoding is recursive and uses the following exact field order. `Relation = U8(1) || ItemId || Node || SEQ(expressions)`; each expression is `NodeKey || ValueType || ExpressionShape`. `Node = algebra-family tag || OPT(join-mode tag) || stage NodeKey || OPT(Source) || Schema || SEQ(predicate NodeKeys) || SEQ(expression NodeKeys) || SEQ(Correlation) || SEQ(partition-input Bindings) || SEQ(child Nodes)`. `Source = relation-authority tag || source-kind tag || OPT(STR(namespace)) || STR(name)`. `Correlation = Binding || reference NodeKey`. `Schema = SEQ(fields) || cardinality tag || SEQ(partition-key NodeKeys) || SEQ(order keys)`; a field is `STR(name) || ValueType` and an order key is `NodeKey || direction tag || null-order tag || ValueType`.

`Composition = U8(1) || SEQ(bindings) || WITH Binding || body ItemId || STR(body-family keyword) || OPT(body Relation) || Body`. Each composition binding is `Binding || producer ItemId || STR(producer-family keyword) || OPT(Schema) || OPT(Relation) || Producer`. `Producer` is `U8(0)` for Relation, `U8(1) || STR(query text) || sample-mode tag || U32BE(limit) || OPT(GraphTarget)` for Sample, or `U8(2) || GraphQuery` for Graph. `Body` is `U8(0)` for Relation, `U8(1) || GraphQuery` for Graph, `U8(2)` for Check, or `U8(3)` for Explain. `GraphQuery = graph-kind tag || SEQ(graph-role tag || GraphTarget) || U32BE(depth) || OPT(graph-direction tag) || SEQ(STR(summary facet)) || relation-shape tag || dedup-identity tag || U32BE(max witnesses) || OPT(U32BE(max path length)) || witness-tie-break tag`. `GraphTarget` is `U8(0) || graph-reference-kind tag || STR(spelling) || OPT(STR(entity subtype))` for a host selector or `U8(1) || Binding || graph-capability tag` for a prior Composition binding. The top-level optional relation/composition use `U8(0)` absent or the corresponding payload beginning with `U8(1)` present; no second presence byte is inserted.

`ValueType` tags are: `0 || scalar tag`; `1` Null; `2 || SEQ(ValueType)` Tuple; `3 || SEQ(STR(field name) || ValueType)` Record; `4 || ValueType` Optional; `5 || ValueType || cardinality tag` Collection; `6 || SEQ(STR(variant tag) || ValueType)` Union; `7` Unknown. An operator is `STR(symbol) || U32BE(overload ID)`; the spelling is a checked display witness, never reparsed. A call target is `U8(0) || STR(name)` Builtin, `U8(1) || STR(name)` Predicate, or `U8(2) || STR(fingerprint)` HostPredicate.

`ExecutionFamilyV3::RuleProgram` (item-family tag 6) is reserved vocabulary: the current producer emits no execution item with that family. Its presence in the encoding table does not grant rule-program execution support.

Within each tag family below, the first listed variant has tag `0`, the next tag `1`, and so on. This is an explicit ordered mapping, not a sorting rule:

| Tag family | Variants in tag order |
| --- | --- |
| item family | Relation, Graph, Check, Presentation, Composition, MetaRequireCertified, RuleProgram |
| plan authority | FrontendChecked, CertifiedRequirementValidated |
| determinism | Deterministic, NonDeterministic |
| evidence | None, SourceLocation, WitnessPayload, ProofTree |
| effect | ReadOnly, ProofEvidenceOnly, ExportOnly, MutationForbidden |
| policy family | Relational, Graph, Check, Presentation, RuleProgram, MetaRequireCertified, Composition |
| graph direction | Forward, Backward, Both, Undirected |
| graph reference role | Source, Sink, Anchor, Reference, Pointee, Callable, Via, Through, Sanitizer, Scope, Window, Frontier |
| graph reference kind | Entity, Callable, Scope, Window, Frontier, Sanitizer, Pattern, Snapshot, ExportSource |
| graph capability | Entity, Scope |
| graph relation shape | Acyclic, Cyclic |
| graph dedup identity | Endpoints, NodeSequence, NodeSet |
| graph witness tie-break | ShortestThenLexical, Lexical |
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

Builtin names and operator spellings come from their closed owner registries. If an issued identity has no registered spelling, projection returns `pipeline.document.unregistered_execution_symbol` at the enclosing authored item and issues no execution document. Placeholder symbol strings are not part of the execution contract.

The checked owner enforces finite float and exact source membership before encoding; public DTO fields and document constructor remain private. `crates/quanta-gqlang-v2-pipeline/src/execution.rs` is the producer implementation of these rules, not a second source of policy.

## Shared conformance vectors

The TSV is a pinned producer-output regression artifact, not an independent
implementation of the byte contract. Independent expected streams live in
`crates/quanta-gqlang-v2-pipeline/src/execution_digest_oracle.rs`: synthetic
relation/composition fixtures build complete source, contract, and qualification
streams from the literal tags and fixed-width framing specified above, without
reading DTO fields or calling production writer/tag helpers. These fixtures cover
nested payload encoding; they do not establish source acceptance or downstream
runtime admission. The payload-free whole-stream fixture in `execution.rs` also
pins independently computed SHA-256 values.

`crates/quanta-gqlang-v2-pipeline/tests/fixtures/execution-v3-vectors.tsv` pins named source/authority cases and expected three lower-case hexadecimal digests. A same-byte/different-`SourceId` pair pins source equality and contract/qualification inequality. A byte-only change pins source/qualification inequality without assuming semantic-equivalence. The same source and ID under `records_host` versus `records_host_string_id` pins projected host field-schema sensitivity; `workspace_files_canonical` versus `workspace_files_host` pins relation-authority sensitivity. The composition pair changes SAMPLE limit. Encoder owner tests separately pin binding identity/order and schema field mutations; the TSV does not claim an authored binding-name/order pair. Typed literals pin `i128::MIN`, normalized zero, finite float, bool, non-ASCII string, and null. An unequal raw item ID returns no item; same-raw-ID cross-document selection is **not** rejected by lookup and needs downstream admission binding. Changing only Semantica data generation must leave all three frontend digests unchanged and change the admitted runtime identity; that final pairwise assertion belongs in Semantica.

The TSV authority profiles use `FrontendBudgetV2::standard_v2()`, host source registration `SourceIdV2(1)` with length 64, selected module `app` version `1.0.0`, and checked host location `(1, 0..1)`. `minimal` has no relation or graph sources. `records_host` adds `app::records` (lookup `records`, Host authority, `host` provenance namespace, one `id:Int` field, Bag cardinality) and the checked graph selectors `a`, `b` (Entity), `scope` (Scope). `records_host_string_id` changes only that field to `id:String`; graph selectors remain the same. `workspace_files_canonical` uses the facade builder's `canonical_workspace_files` relation; `workspace_files_host` uses that same schema with Host authority. Composition rows use `minimal`. The TSV records exact one-line source UTF-8 text, source ID, profile, and digests, so a sibling can compile through the facade rather than reimplement this encoder.

The V3 producer may be called qualified only after owner tests, facade-only consumer/privacy proof, conformance, and a clean committed-source local qualification receipt pass on one exact qglang SHA. Consumer build and end-to-end service execution require a separately recorded exact Semantica SHA and dependency pin; producer-local GREEN does not imply either.
