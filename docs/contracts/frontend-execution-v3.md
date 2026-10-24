# QGLang frontend execution contract V3

Status: producer contract for the breaking, pre-release qglang/Semantica source-pin cutover. The producer is qglang's checked Pipeline. Runtime admission and execution are not part of this contract.

## Contract ownership

`quanta-gqlang-v2-execution-contract` owns the Execution DTO declarations,
canonical encoding/digests, and projection from sealed Algebra/Logic/Planning/Sema
facts. Private DTO fields and constructors stay with their projector. It has no
Pipeline dependency and no normal Surface/runtime/provider dependency.
`quanta-gqlang-v2-pipeline` owns parsing, stage orchestration, diagnostics and
request permit chains, and consumes the internal contract producer bridge.
`quanta-gqlang-v2-frontend-contract` remains the sole downstream seam: it imports
compiler entries from Pipeline and Execution roots directly from execution-contract.

The internal `producer` module is not a facade export. Its move-only projection
buffer constructs items from checked stage facts and seals its own Vec; it accepts
no caller-built ExecutionItem Vec or supplied digest. The scalar projection uses
a transparent move-only wrapper and the original two-slot reservation before seal.
No second item buffer is introduced; exact/short reservation and final-drop
regressions remain required. Direct dependency on an owner crate is outside the
supported downstream contract.

The physical move changes Rust type definition paths and requires a source-pinned
consumer rebuild. Public facade names/signatures, source schema3, contract/qualification
schema8, field ordering and shared golden vectors are unchanged. Paired downstream
source pins and consumer proof must be refreshed separately.

## Public boundary

`compile_source_document_v3(SourceIdV2, &str, &ClosedFrontendAuthorityV2) -> Result<CompiledSourceDocumentV3, FrontendTransactionErrorV2>` is the direct closed source entry that issues an `ExecutionDocumentV3`. The result retains the source-ordered compiled item dispositions and exposes one immutable execution document. `compile_source_document_with_lexical_owner_v1` performs the same closed compile while retaining a source-bound permit for lexer and top-level parser storage in `AdmittedLexicalCompiledSourceDocumentV1`; its permit does **not** cover nested HIR, semantic, planning, or execution allocations. Neither entry grants downstream runtime admission or complete frontend memory qualification. No public constructor accepts preassembled execution items or a caller-supplied digest. `compile_resolved_document_v2` can check a sealed semantic document without original bytes, but its result exposes **no** execution document or source/qualification digest. Open analysis is never promoted to closed execution.

The facade also exports the two [bounded scalar request entries](#bounded-scalar-request-entry).
They issue the same V3 projection inside `AdmittedScalarExecutionDocumentV3`
for their finite Source→Project subset, retaining request-owned storage.
Lexical-only ownership and scalar execution ownership cover different storage;
neither represents complete frontend or runtime memory qualification.

`classify_single_source_root_v3(SourceIdV2, &str, &FrontendBudgetV2)` is a
canonical parse-only routing aid. It requires exactly one source item and
returns only its closed root-family enum; it does not resolve semantic catalogs,
issue an execution document, or grant host/runtime authority. A downstream
consumer may use it to reject an unsupported family before constructing closed
authority, but every supported family must still pass
`compile_source_document_v3` before execution.

`census_source_lexical_materialization_v3` reports only lexer token storage and
decoded payload demand. It does not account for HIR, semantic, planning, or
retained execution objects and grants no compile or runtime admission.
`FrontendLexicalHostCeilingV1` is a finite value whose issuer the frontend does
not authenticate. The downstream host must install its own ceiling before
issuing `FrontendLexicalMaterializationRootV1` for an exact source invocation.
`classify_single_source_root_with_lexical_owner_v1` keeps that root live while
classifying. `admit_show_syntax_v2` retains parser and SHOW HIR storage under
the same root, but its result is only a syntax-stage owner. A lexical census
alone cannot admit parser storage; a SHOW syntax owner cannot be treated as a
semantic plan or execution grant. The facade-only external consumer exercises
the positive, short-budget, foreign-source, and non-SHOW rejection paths.

`admit_show_planned_source_v2` is **not** a live one-shot compile. After the
source-byte cap check it always returns `IncompleteSourceDiagnostics` and never
invokes the lexer (preissued diagnostic debit is missing). The Pipeline owner
has the live staged continuation `admit_show_syntax_v2` →
`admit_show_source_declaration_v2` → `admit_show_plans_v2` →
`admit_show_execution_v2` on one retained source/root. Only the syntax entry and
syntax owner are exported by the downstream facade. Later Pipeline owner
entries are not a second public downstream seam, and an issued frontend DTO
still requires downstream runtime admission.

`parse_source_document_with_request_owner_v3(SourceIdV2, &str,
&FrontendBudgetV2, &FrontendRequestMaterializationRootV3)` is a partial parser
entry. The pipeline checks the public source-byte cap and projects parser
limits from the same checked frontend budget. Its opaque
`ParsedRequestSourceV3` retains request-root reservations for lexical storage,
parser/HIR items, and nested constructors. Parser temporary storage is charged
before use and released before the owner returns.
`check_source_document_with_request_owner_v3` consumes that owner, checks the
parser-issued HIR under the exact same source and root, and returns an opaque
`CheckedRequestSourceV3`. Its integrity scratch is reserved before use and
released when the check finishes. Neither stage issues an execution document
or runtime authority. Diagnostic, semantic, planning, and execution storage
still require producer admission.

The host implements `FrontendRequestStorageSinkV3` against its issued request
ledger. Each reservation must include its concrete boxed permit carrier.
`FrontendRequestMaterializationRootV3` checks the exact source invocation.
`FrontendRetainedStoragePermitV3` asks the host to authenticate it before extending its
reservations. Equal source bytes, source IDs, or numeric limits do not
authorize a foreign request root. The facade exposes the opaque parsed and
checked stages, the bounded scalar compile entry described below, and the
named opaque storage permit returned by the root.
The generic payload owner and producer mapping operation are private to owner
crates; naming a retained permit does not construct a parsed or checked stage.
Permits remain live until their payload
owner drops, and linked reservations are released iteratively with bounded
stack use. The facade consumer tests source-byte refusal, foreign and short
roots, and release after the checked owner drops.
`validate_minimum_reservation_v3` authenticates the permit and checks its
current stage's logical bytes against producer-derived demand. Earlier linked
reservations and permit-carrier overhead cannot satisfy the new stage's demand.

Producer-only staging also retains declared syntax, a Modules session,
predeclared catalog assets, the finite builtin inventory, and Binder
declarations and source bindings. Source binding admission supports declaration
values, bare relations, and source-backed projects including inline Views;
calls and richer relation/rule families return a typed preflight refusal.
Those stages validate the exact source and request root;
their nested producers retain independently issued permits. The Modules
session identity retains its permit through every shared proof, and the
predeclared backing and returned proofs share catalog storage custody.
Dropping the original caller permit does not release live producer storage.
Binder declaration carriers also authenticate their own original request,
including empty declaration tables; an authentic foreign root and caller
permit cannot rebind the declaration owner. Request-only frame and duplicate
indices use admitted Vec storage without relying on private standard-library
BTree node layouts.
Binder source resolution also admits one Composition LET whose Query producer
and Query body each contain one Source alias followed by a one-field Project
of that alias's one-segment path. The body Source must name that exact LET;
self-reference, a different WITH target, rebinding, cross-query alias capture,
and adjacent shapes return typed preflight refusal. Its lexical identities and
ordered outputs match ordinary Binder resolution. The admitted result retains
the resolution reservation, uses two prepared scope frames and Vec-backed
relation evidence, and exposes the exact LET identity through a borrowed
reader. The scalar-specific source entries continue to accept only a bare Query.
The source-family continuation described below also carries this exact one-LET
shape through materialization, catalog, algebra, planning and execution seal.

The producer-only Types continuation also accepts this exact one-LET shape.
Sema checks the Binder LET identity, host scalar fields, and the producer/body
field join before reserving storage. Types retains four expression facts and
four relation rows in bounded Vec tables. Its owner test compares each node
with the ordinary Type kernel and observes logical Vec/string/record births
against the reserved upper. This is not a whole-allocator heap bound. The
source-family materializer admits this exact Composition shape; richer
Composition families remain typed refusals. The scalar-specific public compile
entry remains Query-only.
The facade exposes the partial parser and HIR-check stages. Later retained
carriers feed the producer-owned continuation described below; their constructors
and stage mapping hooks remain outside the facade.

Producer-only body staging supports one bare Query or inline View over a host
Source followed by a Project of scalar, one-segment field paths. Types, topology, and body
materialization consume opaque stage owners. Type admission authenticates the
parent, Modules session, Binder result, and builtin inventory before reserving
new storage. The Types owner recomputes demand from checked input and retains
both its permit and builtin custody; readers borrow facts without extracting
the whole cloneable fact store. Other shapes return typed refusal. Request-only
expression stores and single-origin provenance avoid tree-node allocations.
Sema retains a Modules session witness and seals the final catalog only when
the original root, source registry, exact item/expression membership, and issuing
validator session agree. Query has zero catalog assets; View has one.

The finite continuation consumes these owners through a borrowed normalized
Project(Scan) algebra view and fixed-size relational plan. It reuses the ordinary
Algebra contract checks and Planning result/cost/policy constructors, preserving
expression order, effects, capability, and the authored item's checked location.
The scalar algebra and plan observations create no new heap payload and retain
all earlier reservations. Composition policy storage must be admitted separately
before its canonical retention is constructed. Neither path requires a private
BTree layout assumption or new normal crate dependency.
The read views convey no detachable semantic authority. These intermediate
stages remain outside the facade. The bounded scalar source entry composes them
with execution projection and the document seal; this does not certify general
execution or full serving memory custody.
Producer reservations cover logical payload and the documented finite capacity
bounds; allocator metadata and process-wide heap usage are not measured by them.

### Bounded scalar request entry

`compile_scalar_source_with_request_owner_v3(SourceIdV2, &str,
&ClosedFrontendAuthorityV2, &FrontendRequestMaterializationRootV3)` accepts one
bare Query in the scalar Source→Project or Source→Filter→Project subset. It consumes the parser-issued
owner through checking, declarations, Modules/Binder/Types, semantic body,
catalog, normalized algebra, planning, execution projection, and the document
seal. Other root families return typed `UnsupportedRoot`; unsupported relation
and expression shapes return the owning stage's typed refusal. No fallback to
the unrestricted compiler occurs. Inline View staging remains producer-only.

The finite Filter has one checked scalar field/literal comparison and retains
its Bool predicate, Binder alias, typed expression closure, and inherited Scan
schema through the same request owners. The sealed relation contains the
existing Project, Filter, and Scan operator variants; filtering is not discarded
at admission. Other relation trees and predicate grammars remain typed refusals.
Source rows may include scalar fields, `Optional(String)`, and
`Collection(String, Sequence)` while selected Project fields remain scalar.
The full Scan and retained binding-record schemas preserve those field types,
names, order, and cardinality rather than flattening or omitting them.

Execution reserves the additional Filter operator/predicate/closure and each
nested Optional/Collection child before its first DTO birth. Optional and
Collection children use a boxed one-element array so the pinned stable compiler
can create them through fallible, exact-capacity Vec storage without unsafe code.
Their public borrowed value reads and canonical ValueType tags/bytes are
unchanged; Rust DTO constructors supply `Box::new([child])`. The one-LET
Composition subset retains its existing scalar-only source contract.

`ParsedRequestSourceV3::root_family_v3` classifies the retained parser owner
without another parse or reservation.
`compile_scalar_parsed_source_with_request_owner_v3(SourceIdV2, &str,
&ClosedFrontendAuthorityV2, &FrontendRequestMaterializationRootV3,
ParsedRequestSourceV3)` consumes that same owner and continues the bounded
compile. The parser's checked budget must equal the authority's checked budget;
a mismatch returns `ParsedBudgetMismatch` before further reservations. Source
and issued-root identity remain subject to the checking stage's validation.

The final `AdmittedScalarExecutionDocumentV3` keeps the original source custody
and the reservations backing its execution payload attached. Consumed semantic
and planning payloads release their independent reservations when they drop;
this does not release live execution storage. Its DTO backing and constructors are
private. It exposes source/item identities, family, expression count, digests,
source-byte comparison, and original-root/issued-sink validation. The three
digests use the existing V3 encoding and match ordinary compilation of the same
source/authority; this owner does not introduce a wire schema version.

`into_shared_with_request_owner_v3` authenticates the original root and reserves
shared-owner storage before allocating. `SharedAdmittedScalarExecutionDocumentV3`
clones share the immutable allocation and permits without deep-cloning the DTO.
`shares_payload_v3` and equality compare that allocation's identity, not semantic
equivalence between two compilations. Issued-sink validation authenticates every
linked retained permit iteratively. Reservations survive dropping the caller or
an earlier shared clone, and release when the final owner drops.

The shared owner offers a borrowed `AdmittedCanonicalFilesScalarReadV3` only
when its exact sealed Scan→Project payload carries canonical workspace-files
authority and the supported RepoPath field/binding shape. Its accessors expose
item identity, source namespace/name, input/output field names and cardinalities,
and source-local binding identity. A same-spelling ordinary Host relation does
not yield this observation. The read cannot outlive the shared owner and carries
no runtime generation, provider, lease, or execution grant. Downstream must bind
the retained owner to its independently issued authority and request ledger.

The facade exports the producer-owned
`CANONICAL_WORKSPACE_FILES_PROVENANCE_NAMESPACE_V2`,
`CANONICAL_WORKSPACE_FILES_QUALIFIED_NAME_V2`, and
`CANONICAL_WORKSPACE_FILES_FIELD_NAME_V2` constants for those canonical names.
They share the Types owner's definitions through the existing stage direction;
the names alone confer no canonical source or runtime authority.

The facade-only consumer pins result/digest parity, exact canonical fields,
foreign-root and foreign-ledger refusal, no-spare shared-allocation refusal,
adjacent root refusal, and clone/last-drop custody. Compile-fail probes reject
backing access, deep cloning of the move-only owner, public construction fields,
and detaching the borrowed read from its owner.

### Bounded one-LET source continuation

`compile_source_parsed_with_request_owner_v3(SourceIdV2, &str,
&ClosedFrontendAuthorityV2, &FrontendRequestMaterializationRootV3,
ParsedRequestSourceV3)` consumes the same parser-issued owner. It accepts either
the scalar Query subset or one Composition LET whose producer and body are
single-field Project(Scan) Queries over the same checked Binder binding. It
returns `AdmittedSourceExecutionDocumentV3::{Scalar, OneLet}`; a Composition is
never published as a scalar capability. Sample, additional LETs, a foreign WITH
binding and adjacent shapes retain the owning stage's typed refusal. This entry
does not introduce another parser, semantic engine or runtime grant.

The one-LET owner seals both relation projections and the canonical Composition
envelope after reserving their combined demand. Its source/contract/qualification
digests match ordinary compilation of the same checked source. Sharing retains
the original permits; foreign roots/sinks are rejected and the final shared
drop releases custody. `SharedAdmittedOneLetExecutionDocumentV3` lends
`AdmittedOneLetCompositionReadV3`, binding identities and the finite RepoPath
Project(Scan) observations required by its current downstream consumer. These
observations do not authorize a provider, generation or runtime request.

### Borrowed relation observations

`SharedAdmittedScalarExecutionDocumentV3::relation_read_v3()` returns
`ExecutionRelationReadV3<'_>`. The complete operator, expression, source, schema
and value-type closure is observed through private-field read wrappers and
borrowed shape enums. `ExecutionRelationV3::read_v3()` lets the ordinary DTO path
enter the same downstream compiler. No read accessor returns a cloneable DTO,
mutable backing, `Deref` target or conversion back into an owned DTO.

Copying a read wrapper copies its borrow; it does not copy payload or retain a
new independent allocation. Nested reads cannot outlive their original owner.
String and identity observations are inert data; a consumer remains responsible
for admitting any owned physical-plan projection and work it performs. The old
generic `with_sealed_relation_v3` callback is removed because a caller could
return `relation.clone()` from it. This pre-release API cutover must ship with
the downstream consumer conversion and source pin; it changes no wire encoding,
digest domain or golden vector.

All authored `query.result_shape` forms (leaf, record, and nested record/list)
parse and format, but
`sema::materialize_query` rejects `result_shape.is_some()` with a typed
“not yet available” failure. Closed compile of that construct is therefore
RED until a later owner cut. FIND/CHECK payloads on the V3 DTO are frontend
contracts, not solver execution.

`SourceIdV2`, `SemanticItemIdV2`, and `SourceQualifiedNodeIdV2` are transaction-local frontend identities. Their raw values and `SourceIdV2` are encoded deliberately. Contract and qualification digests are deterministic fingerprints of the projected DTO and source bytes with that source ID, **not** unique transaction-instance IDs or fingerprints of the entire host authority. Equal source bytes under different source IDs have equal source digests but different contract and qualification digests; a repeated compile with the same ID, bytes, and DTO has the same digests. They are not cross-transaction semantic-equivalence cache keys. Runtime admission must bind the selected document and item to its own source/authority closure, generation, provider, lease, and budget as applicable. Formatting-equivalent text has no promised contract-digest equality.

The document carries source-ordered **plan-required items**. Non-plan declarations are absent; their checked `NoPlanReasonV2` remains on the compiled result. `item(id)` is a document-local exact-key lookup: an absent or unequal raw `SemanticItemIdV2` returns no item, but a separately compiled document may reuse the same transaction-local raw ID. Lookup alone does not prove provenance. Each item carries frontend plan authority, determinism, evidence, effect, and policy family. The V3 execution payload is complete for a top-level relation, standalone Graph, CHECK, Presentation, Rename Command, and the immutable Composition binding/body handoff. Graph carries `ExecutionGraphQueryV3`; CHECK carries `ExecutionCheckV3` projected from the coherent `CheckContractV2`, including rule catalog identity, optional scope/pattern selectors, exact nested FIND item identity and query, confirmation, emission and detection modes. Presentation carries `ExecutionPresentationKindV3` projected from the coherent `PresentationContractV2`, preserving all six disjoint verb payloads. `MetaRequireCertified` carries planning authority but no standalone execution payload. Rule members and the document-wide rule program are not V3 execution items. Declaration and Catalog items remain checked non-plan dispositions. An absent family payload never grants runtime execution; Semantica admission must reject unsupported families before effects. The producer never silently drops a plan-required item to make a consumer succeed.

### Command contract boundary

The only source-level Command is Rename. Its canonical syntax is `rename <closed-symbol-selector> to "<new-name>" | phase=preview | snapshot="<expected-snapshot>"`; apply uses `phase=apply` and adds `| preview_token="<issued-token>"` after the snapshot. Clause order is fixed. Empty names, snapshots, and tokens, unknown selectors or operations, and invalid phase/token combinations reject.

`ClosedFrontendAuthorityBuilderV2::add_symbol_target(FrontendSymbolTargetInputV3)` supplies each selector's exact repository path, symbol ID, and half-open byte span from a host-owned closed symbol catalog. Duplicate selectors and invalid identity reject. Resolution uses that catalog; a source spelling/name alone never grants a target. The execution DTO exposes `ExecutionItemV3::command()` with a typed Rename phase, exact target, replacement name, expected snapshot, and optional preview token. Preview is `ReadOnly` with `SymbolRenamePreview`; apply is `MutationRequiresGrant` with `SymbolRenameApply`. Both use `CommandReceipt` as the result kind. No other mutation operation is issued.

These fields are source intent and frontend projection, not execution authority. The runtime must verify the selected source and target against its issued snapshot and closed catalog, and validate the apply token against its own issued preview, snapshot, target, and grant before effects. A source-authored `preview_token` is untrusted input. A stale target, mismatched snapshot, missing grant, or token mismatch must fail closed without mutation. The producer implements no preview/apply effect.

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


`ExecutionDocumentV3::matches_source_bytes_v3()` checks source identity using
this document's canonical source-digest encoder. Hosts can use it before
issuing a source-bound receipt without recompiling or duplicating the encoder.
A matching digest grants no runtime authority; the host must also verify the
selected plan and its own generation/provider/budget. This helper adds no DTO
fields and does not change any digest schema.

## Public DTO field map

This is the field inventory for the facade's `Execution*V3` payload. Each row
lists stored fields in the order exposed by accessors; the last column names
the exact contract-stream clause below. The stream order is authoritative when
it differs from accessor order. `ItemId` and `NodeKey` are source-qualified;
strings and sequence elements keep their checked, authored order.

| DTO | Stored fields or variant payloads | Contract-stream clause |
| --- | --- | --- |
| `ExecutionDocumentV3` | `source_id`, `items`, `source_digest`, `contract_digest`, `qualification_digest` | `source_id` and `items` enter `contract_digest`; raw source enters `source_digest`; the two digests and `source_id` enter `qualification_digest`. Digests are outputs, not recursively encoded fields. |
| `ExecutionItemV3` | `item_id`, `family`, `relation`, `graph`, `check`, `presentation`, `composition`, `command`, `plan` | `ItemId`, family and the five plan tags, then optional Relation, Composition, Graph, CHECK, Presentation, Command in that order. |
| `ExecutionPlanV3` | `authority`, `determinism`, `evidence`, `effect`, `policy_family` | Five plan tags in that order; detailed semantic plan policy is outside V3. |
| `ExecutionSymbolTargetV3` / `ExecutionRenameCommandV3` | target: `selector`, `repo_path`, `symbol_id`, `start_byte`, `end_byte`; command: `target`, `new_name`, `phase`, `expected_snapshot`, `preview_token` | Command encodes `phase`, then target's five fields, then `new_name`, `expected_snapshot`, optional `preview_token`. |
| `ExecutionCatalogIdentityV3` | `module`, `name` | `CatalogIdentity`; CHECK fixes the Rule namespace, EXPORT fixes Profile, and EXPLAIN Rule fixes Rule. |
| `ExecutionNestedGraphV3` / `ExecutionCheckV3` | nested: `item_id`, `query`; CHECK: `rule`, `scope`, `pattern`, `nested_graph`, `confirm`, `emit`, `mode` | CHECK clause, including the nested Graph's distinct `ItemId`. |
| `ExecutionPresentationKindV3` | EXPLAIN `target`, `evidence`, `truth`; SHOW `category`; DESCRIBE `target`; SAMPLE `query_text`, `mode`, `limit`, `scope`; COMPARE `baseline`, `candidate`, `metric`; EXPORT `source`, `format`, `profile` | Presentation verb tag followed by exactly the selected variant's fields. The other variants supply no fields. |
| `ExecutionRelationV3` | `item_id`, `root`, `expressions` | Relation clause: `ItemId`, Node, ordered expression table. |
| `ExecutionRelationalOperatorV3` | `family`, `join_mode`, `stage`, `source`, `schema`, `predicates`, `expressions`, `correlations`, `existential_terms`, `partition_input`, `children` | Node clause in this order; `existential_terms` has a sequence only for ExistsFilter. |
| `ExecutionExistentialTermV3` | `expression`, `polarity`, `correlations` | ExistentialTerm clause. |
| `ExecutionSchemaV3` / `ExecutionFieldV3` / `ExecutionOrderKeyV3` | schema: `fields`, `cardinality`, `partition_keys`, `order`; field: `name`, `value_type`; order key: `expression`, `direction`, `null_order`, `value_type` | Schema, field and order-key clauses. |
| `ExecutionExpressionV3` / `ExecutionExpressionShapeV3` / `ExecutionLiteralV3` | expression: `id`, `value_type`, `shape`; shape and literal carry their selected enum variant's fields | ExpressionShape and literal tag tables below; no unselected variant contributes bytes. |
| `ExecutionOperatorV3` | `symbol`, `overload_id` | Operator clause; semantic dispatch uses the separate closed descriptor lookup. |
| `ExecutionSourceV3` | `authority`, `kind`, `namespace`, `name`, `composition_binding` | Source clause encodes the first four. The exact `composition_binding` slot is encoded by the enclosing Composition binding sequence; a consumer must match it there. |
| `ExecutionBindingV3` / `ExecutionCorrelationV3` | binding: `source_id`, `owner_node`, `ordinal`, `name`; correlation: `binding`, `reference` | Binding and Correlation clauses. |
| `ExecutionCompositionV3` | `bindings`, `with_binding`, `body_item_id`, `body_family`, `body_relation`, `body` | Composition clause in this order. |
| `ExecutionCompositionBindingV3` | `binding`, `producer_item_id`, `producer_family`, `schema`, `relation`, `producer` | One ordered Composition binding clause per producer. |
| `ExecutionCompositionProducerV3` / `ExecutionCompositionBodyV3` | producer: Relation, Sample (`query_text`, `mode`, `limit`, `scope`), or Graph (`query`); body: Relation, Graph (`query`), Check, or Explain | Producer and Body tagged clauses; payload-free variants contribute only their tag. |
| `ExecutionGraphQueryV3` | `kind`, `references`, `depth`, `direction`, `summary_facets`, `relation_shape`, `dedup_identity`, `max_witnesses`, `max_path_length`, `witness_tie_break` | GraphQuery clause in this order. |
| `ExecutionGraphReferenceV3` / `ExecutionGraphTargetV3` | reference: `role`, `target`; target: Host (`kind`, `spelling`, `entity_subtype`) or Composition (`binding`, `capability`) | Graph role/target clauses. |
| `ExecutionValueTypeV3` / `ExecutionRecordFieldV3` / `ExecutionUnionVariantV3` | selected type variant; record field: `name`, `value_type`; union variant: `tag`, `value_type` | ValueType tag table, including ordered record fields and union variants. |

`ExecutionDocumentV3::item()` is a document-local lookup, not a stored field.
The three `*_schema_version()` accessors return stream constants, not DTO
fields. `ExecutionFiniteFloatV3::get()` derives a value from its encoded finite
bits; `ExecutionOperatorDescriptorV3` is a lookup result and adds no bytes.
Every enum tag and expression payload is specified below. This mapping is a
frontend encoding contract; downstream admission must validate its own host
generation and supported capabilities before execution.

## Canonical digest encoding

All three digests are SHA-256 over distinct canonical byte streams. `STR(x)` means `U64BE(length of UTF-8 bytes) || UTF-8 bytes`; `SEQ(xs)` means `U64BE(count) || each element in declared order`; `OPT(x)` means one byte `0` if absent or `1 || x` if present. `U8`, `U32BE`, `U64BE`, and `I128BE` have fixed widths; signed integers use two's-complement big-endian bytes. Lengths come from the frontend's bounded checked transaction and are encoded as `U64BE`. No display/debug formatting, hash-map order, host pointer width, runtime ID, provider, generation, lease, or budget enters these streams.

The V3 Rust DTO uses source-byte digest schema 3 and contract/qualification
digest schema 8. Schema 8 adds the exact Rename Command target and intent
after the existing Relation, Composition, Graph, CHECK and Presentation payloads. The unchanged source-byte stream remains on
schema/domain 3; revised contract streams use the `.v8` suffix so older bytes
cannot collide. The document exposes all three schema versions through
read-only accessors, so a downstream receipt never has to infer the digest
algorithm from the Rust DTO name or from a hash value:

| Digest | Exact top-level stream |
| --- | --- |
| `source_digest` | `STR("qglang.execution.source-bytes.v3") || U32BE(3) || U64BE(source byte length) || exact source UTF-8 bytes` |
| `contract_digest` | `STR("qglang.execution.contract.v8") || U32BE(8) || U32BE(SourceId.raw) || SEQ(source-ordered execution items)` |
| `qualification_digest` | `STR("qglang.execution.qualification.v8") || U32BE(8) || U32BE(SourceId.raw) || raw 32-byte source_digest || raw 32-byte contract_digest` |

An item is encoded in this order: `ItemId`, family tag, plan authority tag, determinism tag, evidence tag, effect tag, policy-family tag, optional relation, optional composition, optional standalone graph, optional CHECK, optional Presentation, optional Command. Each optional payload starts with `U8(0)` absent or `U8(1)` followed by its payload. Relation and Composition already include their own `U8(1)` presence marker. `ItemId = U32BE(source ID) || U32BE(HIR item ID)`; `NodeKey = U32BE(source ID) || U32BE(node ID)`; `Binding = U32BE(source ID) || U32BE(owner node) || U64BE(ordinal) || STR(name)`.

The nested encoding is recursive and uses the following exact field order. `Relation = U8(1) || ItemId || Node || SEQ(expressions)`; each expression is `NodeKey || ValueType || ExpressionShape`. `Node = algebra-family tag || OPT(join-mode tag) || stage NodeKey || OPT(Source) || Schema || SEQ(predicate NodeKeys) || SEQ(expression NodeKeys) || SEQ(Correlation) || [SEQ(ExistentialTerm) if family is ExistsFilter] || SEQ(partition-input Bindings) || SEQ(child Nodes)`. `ExistentialTerm = expression NodeKey || relation-exists polarity tag || SEQ(Correlation)`; its index matches the corresponding child after the input child, in authored order. `Source = relation-authority tag || source-kind tag || OPT(STR(namespace)) || STR(name)`. `Correlation = Binding || reference NodeKey`. `Schema = SEQ(fields) || cardinality tag || SEQ(partition-key NodeKeys) || SEQ(order keys)`; a field is `STR(name) || ValueType` and an order key is `NodeKey || direction tag || null-order tag || ValueType`.

The node's `stage` key is source-stage provenance, not a unique operator ID. A filter stage with ordinary and multiple existential conjuncts emits a residual `Filter` above an `ExistsFilter`; both carry that stage key. Consumers distinguish operator nodes by tree position and family. An existential term's polarity is the effective filter polarity, including an enclosing direct `not`; its referenced `RelationExists` expression retains its authored inner polarity.

`Composition = U8(1) || SEQ(bindings) || WITH Binding || body ItemId || STR(body-family keyword) || OPT(body Relation) || Body`. Each composition binding is `Binding || producer ItemId || STR(producer-family keyword) || OPT(Schema) || OPT(Relation) || Producer`. `Producer` is `U8(0)` for Relation, `U8(1) || STR(query text) || sample-mode tag || U32BE(limit) || OPT(GraphTarget)` for Sample, or `U8(2) || GraphQuery` for Graph. `Body` is `U8(0)` for Relation, `U8(1) || GraphQuery` for Graph, `U8(2)` for Check, or `U8(3)` for Explain. `GraphQuery = graph-kind tag || SEQ(graph-role tag || GraphTarget) || U32BE(depth) || OPT(graph-direction tag) || SEQ(STR(summary facet)) || relation-shape tag || dedup-identity tag || U32BE(max witnesses) || OPT(U32BE(max path length)) || witness-tie-break tag`. `GraphTarget` is `U8(0) || graph-reference-kind tag || STR(spelling) || OPT(STR(entity subtype))` for a host selector or `U8(1) || Binding || graph-capability tag` for a prior Composition binding. The top-level optional relation/composition use `U8(0)` absent or the corresponding payload beginning with `U8(1)` present; no second presence byte is inserted.

The standalone Graph payload is `GraphQuery`. Command is `phase tag (preview=0, apply=1) || STR(selector) || STR(repo_path) || STR(symbol_id) || U64BE(start_byte) || U64BE(end_byte) || STR(new_name) || STR(expected_snapshot) || OPT(STR(preview_token))`. The source digest binds authored bytes and the contract digest binds the resolved exact target. `CatalogIdentity = STR(module) || STR(local name)`; the enclosing CHECK rule or Presentation EXPLAIN/EXPORT variant fixes its Rule/Profile namespace. `CHECK = CatalogIdentity(rule) || OPT(GraphTarget scope) || OPT(GraphTarget pattern) || OPT(ItemId || GraphQuery nested FIND) || confirm-mode tag || emit-mode tag || detection-mode tag`. The nested `ItemId` is distinct from the CHECK item ID. `Presentation` begins with a verb tag: EXPLAIN `0 || (0 || CatalogIdentity(rule) | 1 || GraphTarget(entity)) || evidence-mode tag || truth-mode tag`; SHOW `1 || category tag`; DESCRIBE `2 || GraphTarget`; SAMPLE `3 || STR(query text) || sample-mode tag || U32BE(positive limit) || OPT(GraphTarget scope)`; COMPARE `4 || GraphTarget(baseline) || GraphTarget(candidate) || metric tag`; EXPORT `5 || GraphTarget(source) || format tag || OPT(CatalogIdentity(profile))`. Within each mode/category/metric/format tag family, variants are encoded in Rust declaration order starting at zero. The source digest binds authored bytes; catalog and graph reference identities must also be bound by host admission to its trusted snapshot.

`ValueType` tags are: `0 || scalar tag`; `1` Null; `2 || SEQ(ValueType)` Tuple; `3 || SEQ(STR(field name) || ValueType)` Record; `4 || ValueType` Optional; `5 || ValueType || cardinality tag` Collection; `6 || SEQ(STR(variant tag) || ValueType)` Union; `7` Unknown. An operator is `STR(symbol) || U32BE(overload ID)`; the spelling is a checked display witness, never reparsed. A call target is `U8(0) || STR(name)` Builtin, `U8(1) || STR(name)` Predicate, or `U8(2) || STR(fingerprint)` HostPredicate.

`ExecutionFamilyV3::RuleProgram` (item-family tag 6) is reserved vocabulary: the current producer emits no execution item with that family. Its presence in the encoding table does not grant rule-program execution support.

Within each tag family below, the first listed variant has tag `0`, the next tag `1`, and so on. This is an explicit ordered mapping, not a sorting rule:

| Tag family | Variants in tag order |
| --- | --- |
| item family | Relation, Graph, Check, Presentation, Composition, MetaRequireCertified, RuleProgram, Command |
| plan authority | FrontendChecked, CertifiedRequirementValidated |
| determinism | Deterministic, NonDeterministic |
| evidence | None, SourceLocation, WitnessPayload, ProofTree |
| effect | ReadOnly, ProofEvidenceOnly, ExportOnly, MutationForbidden, MutationRequiresGrant |
| policy family | Relational, Graph, Check, Presentation, RuleProgram, MetaRequireCertified, Composition, Command |
| graph direction | Forward, Backward, Both, Undirected |
| graph reference role | Source, Sink, Anchor, Reference, Pointee, Callable, Via, Through, Sanitizer, Scope, Window, Frontier |
| graph reference kind | Entity, Callable, Scope, Window, Frontier, Sanitizer, Pattern, Snapshot, ExportSource |
| graph capability | Entity, Scope |
| graph relation shape | Acyclic, Cyclic |
| graph dedup identity | Endpoints, NodeSequence, NodeSet |
| graph witness tie-break | ShortestThenLexical, Lexical |
| algebra family | Scan, Filter, Project, Join, Group, Aggregate, Sort, SemiJoin, AntiJoin, ExistsFilter |
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

`execution_operator_descriptor_v3(raw_id)` returns the canonical overload's typed
semantic kind, ordered operand domains and result domain. The descriptor is
derived from the existing closed type-checker registry. Unknown IDs return
`Ok(None)`; invalid registry construction returns a typed error. Runtime consumers
dispatch through the semantic kind and validate the exact domains, rather than
interpreting operator spellings or maintaining an ID table. This lookup API adds
no encoded DTO fields and changes none of the digest domains or byte vectors above.

Composition scans also retain `composition_binding()`, the exact sealed Binder
slot used by source resolution. Consumers match this slot against the issued
capability, including source ID, owner and ordinal. The wire encoding stays the
same: the scan name resolves uniquely in the enclosing sealed composition,
whose binding sequence already encodes that exact slot. Host and catalog scans
carry no Composition slot. This accessor retains the producer proof and does
not authorize a new source or infer correlation from source use.

The checked owner enforces finite float and exact source membership before encoding; public DTO fields and document constructor remain private. `crates/quanta-gqlang-v2-execution-contract/src/lib.rs` and its encoding/projection children implement these rules; Pipeline imports that owner.

## Shared conformance vectors

The TSV is a pinned producer-output regression artifact, not an independent
implementation of the byte contract. Independent expected streams live in
`crates/quanta-gqlang-v2-execution-contract/src/digest_oracle.rs`: synthetic
relation/composition fixtures build complete source, contract, and qualification
streams from the literal tags and fixed-width framing specified above, without
reading DTO fields or calling production writer/tag helpers. These fixtures cover
nested payload encoding; they do not establish source acceptance or downstream
runtime admission. The payload-free whole-stream fixture in
`crates/quanta-gqlang-v2-execution-contract/src/tests.rs` also pins independently
computed SHA-256 values.

`crates/quanta-gqlang-v2-pipeline/tests/fixtures/execution-v3-vectors.tsv` pins named source/authority cases and expected three lower-case hexadecimal digests, including standalone Graph, CHECK and each Presentation verb. A same-byte/different-`SourceId` pair pins source equality and contract/qualification inequality. A byte-only change pins source/qualification inequality without assuming semantic-equivalence. The same source and ID under `records_host` versus `records_host_string_id` pins projected host field-schema sensitivity; `workspace_files_canonical` versus `workspace_files_host` pins relation-authority sensitivity. The composition pair changes SAMPLE limit. Encoder owner tests separately pin binding identity/order and schema field mutations; the TSV does not claim an authored binding-name/order pair. Typed literals pin `i128::MIN`, normalized zero, finite float, bool, non-ASCII string, and null. An unequal raw item ID returns no item; same-raw-ID cross-document selection is **not** rejected by lookup and needs downstream admission binding. Changing only Semantica data generation must leave all three frontend digests unchanged and change the admitted runtime identity; that final pairwise assertion belongs in Semantica.

The TSV authority profiles use `FrontendBudgetV2::standard_v2()`, host source registration `SourceIdV2(1)` with length 64, selected module `app` version `1.0.0`, and checked host location `(1, 0..1)`. `minimal` has no relation or graph sources. `records_host` adds `app::records` (lookup `records`, Host authority, `host` provenance namespace, one `id:Int` field, Bag cardinality) and the checked graph selectors `a`, `b` (Entity), `scope` (Scope). `records_host_string_id` changes only that field to `id:String`; graph selectors remain the same. `workspace_files_canonical` uses the facade builder's `canonical_workspace_files` relation; `workspace_files_host` uses that same schema with Host authority. Composition rows use `minimal`. The TSV records exact one-line source UTF-8 text, source ID, profile, and digests, so a sibling can compile through the facade rather than reimplement this encoder.

The V3 producer may be called qualified only after owner tests, facade-only consumer/privacy proof, conformance, and a clean committed-source local qualification receipt pass on one exact qglang SHA. Consumer build and end-to-end service execution require a separately recorded exact Semantica SHA and dependency pin; producer-local GREEN does not imply either.


## Relational Limit source contract

The existing relation pipeline admits `| limit <u64>` as a relational stage. Zero
retains no rows; it is not the positive `u32` SAMPLE presentation limit. Parser,
HIR depth/integrity/formatting, Binder, Types, Sema, normalized Algebra and Planning
carry the same authored stage position. The checked Limit has one child, no
expression/predicate/source payload, and exactly preserves its child's row,
columns, provenance, cardinality, ordering and effects. Planning classifies a
count at most one as scalar-or-singleton and other counts as bounded.

`ExecutionRelationalOperatorV3::limit_count()` exposes only the sealed count.
`AlgebraFamilyV2::Limit` appends tag 10 to the existing vocabulary. A present count
uses a disjoint `255` extension marker and eight big-endian bytes immediately
after the family tag, before the existing `0/1` join-mode tag. An absent count has
no extension bytes, retaining every existing non-Limit node's canonical encoding
and the schema-8 golden domain. This is one unambiguous extension of the same
canonical encoder; no alternate digest, parser, provider grant or runtime authority
is introduced. Runtime admission must require a count exactly on Limit nodes.

This source contract does not expand a bounded request subset by itself. Its
existing Source/Project-only producers continue to refuse a Limit root until the
same request owner admits its actual semantic, plan and execution births. Runtime
selection, typed Host producer/generation/provider custody, V1 caller retirement,
and compiler/test/runtime qualification are separate coupled gates.

The existing relational order key retains source `nulls first` or `nulls last`
independently of `asc`/`desc`. An omitted policy remains `Last`. The canonical
formatter emits `nulls first` and omits the default `nulls last`, preserving old
canonical source and execution bytes for the default. HIR, semantic order keys,
algebra, and the existing execution Schema order all carry the same policy;
no downstream runtime reinterpretation supplies a missing policy. This is a
source contract extension, not a runtime/provider qualification receipt.
