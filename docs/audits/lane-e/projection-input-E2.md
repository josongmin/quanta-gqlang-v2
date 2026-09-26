# Lane E-2 projection input — DTO / digest / encoding inventory

Companion to `lane-E2-facade-front-doors.md`. `SN/` = snapshot root
`docs/audits/lane-b-snapshot-20260927-010019` @ `eaeb68ea2711d411616511713fb016d828cd80be`.
All line references are inside `SN/`. Source of structural facts:
`SN/crates/quanta-gqlang-v2-pipeline/src/execution.rs` (E), `.../compile.rs` (C),
`.../diagnostic.rs` (D), facade `SN/crates/quanta-gqlang-v2-frontend-contract/src/lib.rs` (F).
Constructor visibility: "none" = no public constructor exists (unforgeable from outside);
`pub(crate)` = internal only.

## 1. Entry points (F:16-22)

| Symbol | Kind | Semantics | Notes |
| --- | --- | --- | --- |
| `analyze_source_document_v2` | fn | open-authority analysis; never promotes to closed | C:688-723 |
| `classify_single_source_root_v3` | fn | parse-only, exactly-one-item root family | C:592-610 |
| `compile_source_document_v3` | fn | sole closed entry issuing `ExecutionDocumentV3` | C:618-648 |
| `format_source_document_v2` | fn | parse-budget formatter, no semantic traversal | C:751-780 |

## 2. Assembled-document DTOs (F:25-43)

| Type | Shape | Constructors | Mutability | Evidence |
| --- | --- | --- | --- | --- |
| `CompiledSourceDocumentV3` | struct, private fields | `pub(crate)` (built by compile entry) | accessors only: `source_id/items/item/rule_program/diagnostics/execution` | C:191-229 |
| `CompiledDocumentV2` | struct | `try_new` internal | accessors | C:115-189 (not facade-exported) |
| `CompiledItemV2` | enum Complete/NoPlan/BlockedByDiagnostics | none (variant ctor, data-only) | read-only match | C:232-276 |
| `PlannedItemV2` | enum 7 families | none | `item_id/plan/rule_member` | C:278-371 |
| `RuleProgramMemberV2` | struct | `new` private (probe d11) | `program_id/member_item_id` | C:373-401 |
| `CompiledRuleProgramV2` | struct | internal | `program_id/members/plan/stratification` | C:403-436 |
| `AnalysisDocumentV2` | struct | internal | `source_id/order/items/item/rule_program/unresolved/status/diagnostics` | C:440-547 |
| `AnalysisStatusV2` | enum | — | `keyword` | C:450-472 |
| `AnalysisItemV2` | enum | none | match | C:549-558 |
| `UnresolvedAnalysisFactV2` | Copy struct | none | `code/location` | C:560-580 |
| `ExecutionDocumentV3` | struct | `pub(crate) new` derives all 3 digests (E:78-91) | accessors only incl. schema versions | E:69-149 |
| `ExecutionItemV3` | struct | internal projection | `item_id/family/relation/composition/plan` | E:180-228 |
| `ExecutionPlanV3` | struct | internal | `authority/determinism/evidence/effect/policy_family` | E:231-259 |
| `ExecutionDigestV3` | `([u8;32])` tuple, field private | **none** (no `From<[u8;32]>`, no Default, no serde) | `as_bytes/to_hex` | E:32-47 |
| `ExecutionRelationV3` / `ExecutionRelationalOperatorV3` | structs | internal | accessors | E:620-699 |
| `ExecutionSchemaV3` / `ExecutionFieldV3` / `ExecutionOrderKeyV3` | structs | internal | accessors | E:702-767 |
| `ExecutionExpressionV3` / `ExecutionExpressionShapeV3` (16 tags) | struct/enum | internal | accessors | E:770-848 |
| `ExecutionLiteralV3` | enum Int/Float/Bool/String/Null | internal only | match | E:851-858 |
| `ExecutionFiniteFloatV3` | `(u64)` tuple, field private | **none** (built once from `TypedLiteralFactV2`, E:1731) | `bits/get` | E:861-872 |
| `ExecutionOperatorV3`, `ExecutionCallTargetV3`, `ExecutionSourceV3`, `ExecutionBindingV3`, `ExecutionCorrelationV3` | structs/enums | internal | accessors | E:878-979 |
| `ExecutionCompositionV3` / `...BindingV3` / `...ProducerV3` / `...BodyV3` | struct/enums | internal; projection fails closed (`ExecutionCompositionProjectionFailureV3`) | accessors | E:982-1075+ |
| Graph vocabulary (Kind/Direction/ReferenceRole/ReferenceKind/Capability/RelationShape/DedupIdentity/WitnessTieBreak/Target/Reference/Query) | enums + `ExecutionGraphQueryV3` | internal | accessors | E:~1080-1250 |

## 3. Authority / diagnostic / budget vocabulary (F:49-61; defined in sema, re-exported via pipeline lib.rs:59-70)

| Type | Notes |
| --- | --- |
| `ClosedFrontendAuthorityBuilderV2` | the only authority *input* transaction: `try_new` + `add_relation/add_graph_reference/add_pack/add_catalog_asset/add_lockfile` + `finish` (sealing) |
| `ClosedFrontendAuthorityV2` | private fields (probe d08); read accessors |
| `OpenAnalysisAuthorityV2` | `try_standard_v2` / `try_empty_v2`; partial authority → typed `Incomplete`, never defaults |
| `FrontendBudgetV2` | private fields (probes d03/d12); `standard_v2` |
| `FrontendDiagnosticV2` | **public `new`** (E2-06); accessors `stage/code/severity/primary/secondary/message` |
| `FrontendFailureV2` | **public `one`/`try_new`** (rejects empty); `diagnostics` |
| `FrontendTransactionErrorV2` | enum Authority/Document; no flattening |
| Input DTOs (`FrontendRelationInputV2`, `FrontendGraphReferenceInputV2`, `FrontendPackInputV2`, `FrontendCatalogAssetInputV2`, `FrontendLockfileInputV2`, `FrontendCallableParameterInputV2`, field/enum vocab) | host-supplied authority inputs; pub fields by design (builder inputs) |
| `LocatedSpanV2`, `SpanV1`, `SourceIdV2`, `SemanticItemIdV2`, `SourceQualifiedNodeIdV2`, `CheckedLocatedSpanV2` | inert closure types; `LocatedSpanV2` has pub fields (inert), `CheckedLocatedSpanV2` private (probe d06) |

## 4. Digest inventory

| Digest | Domain string | Schema | Inputs (owner: pipeline, single `pub(crate)` ctor E:78-91) | Stability |
| --- | --- | --- | --- | --- |
| source | `qglang.execution.source-bytes.v3` | 3 (E:1909) | exact source UTF-8 bytes | independent of DTO revisions; equal bytes ⇒ equal digest (same across SourceIds) |
| contract | `qglang.execution.contract.v5` | 5 (E:1915) | `U32BE(SourceId.raw)` + source-ordered execution items (plan-required only) | changes with SourceId, DTO projection, authority-sensitive fields |
| qualification | `qglang.execution.qualification.v5` | 5 | source_id + raw source_digest + raw contract_digest | binds both legs; changes iff either leg or source id changes |

Golden vectors: `SN/crates/quanta-gqlang-v2-pipeline/tests/fixtures/execution-v3-vectors.tsv`
(7 columns: case, profile, source_id, source, 3 hex digests; 15 rows pinned at
`v3_compiled_document_assembly.rs:2049`). Consumers of the fixture: producer test
(`v3_compiled_document_assembly.rs:2001-2052`, all rows), facade consumer (2 composition rows,
`tools/frontend_external_consumer/src/main.rs:316-392`, `QGLANG_V3_VECTOR_PATH` override supported).
Provenance: producer-derived (E2-01/E2-14); independent evidence limited to SHA-256 empty vector
(E:2677-2681), independent graph-substream encoder (E:3119+), per-field mutation rails (E:2667+).

## 5. Canonical encoding framing (E:1900-2650; doc `frontend-execution-v3.md:85-168`)

- Primitives: `STR = U64BE(len) || bytes` (put_str E:2637), `SEQ = U64BE(count) || elems`
  (put_usize E:2641), `OPT = 0 | 1||x` (inline matches), `U8/U32BE/U64BE/I128BE` fixed (put_u8/u32 E:2646-2650).
- Presence framing for optional payloads: `U8(0)` absent; payload itself begins `U8(1)` —
  `put_relation` E:2247-2249, `put_composition` E:2024 (no second presence byte).
- Tag families: first-listed-variant = 0, ordered mapping not sorting (doc :110-138); impl:
  `family_tag` E:1970+, `relation_authority_tag` E:2317-2326, etc.
- Top-level orders: item (E:1933-1948), node (E:2258-2315), source (E:2272-2290),
  schema/order keys (E:2454+, doc :104), composition + bindings + producer/body (E:2024-2095,
  doc :106), graph query + targets (E:2097-2140, doc :106), value types (E:2474+, doc :108),
  literals (E:2433+, doc :140-148), expression shapes 0-15 (E:2316+, doc :150-168).
- Hash: SHA-256 via pinned `sha2 = "=0.10.9"` (pipeline Cargo.toml:22); wrapper E:2655-2659.
- Excluded by construction: display/debug formatting, hash-map order, pointer width, runtime IDs,
  provider, generation, lease, budget (encoder has no such inputs — static).

## 6. Front-door stream/exit discipline (for cross-checking)

| Door | stdout | stderr | exit codes |
| --- | --- | --- | --- |
| CLI (`main.rs`) | usage (help), query body, fmt output | class/message line-protocol errors | 0 ok/complete; 1 io/command/incomplete/error; 2 usage |
| stdio server | NDJSON responses (one per line, flushed) | terminal io_error payload only | 0 clean EOF; 1 any per-line error |
