# Lane E — projection map

Contract: `source fact/field | transform owner | destination | intentionally omitted? | semantic/digest coverage | evidence`.
All references are inside snapshot `docs/audits/lane-b-snapshot-20260927-010019`. Sources: E-1 inline audit, E-2 subagent report + `projection-input-E2.md`.

| Source fact/field | Transform owner | Destination | Intentionally omitted? | Semantic/digest coverage | Evidence |
|---|---|---|---|---|---|
| Raw source bytes | pipeline `digest_source_bytes` | `ExecutionDocumentV3.source_digest` (schema 3) | no | full source bytes, length-framed, domain `qglang.execution.source-bytes.v3` | execution.rs:1917–1924 |
| DTO canonical encoding | pipeline `digest_contract` | `contract_digest` (schema 5) | rule program excluded (see below) | item_id, family, plan×5, relation?, composition? per item; domain `qglang.execution.contract.v5` | execution.rs:1927–1953 |
| source+contract binding | pipeline `digest_qualification` | `qualification_digest` (schema 5) | no | domain, version, source_id, both digests | execution.rs:1956–1968 |
| `SemanticPlanV2` authority lane | pipeline `project_plan` | `ExecutionPlanV3.authority` | no | enum copy; digest via authority_tag | execution.rs:577–583, 1982–1987 |
| plan determinism/evidence/effect | pipeline `project_plan` | `ExecutionPlanV3.{determinism,evidence,effect}` | no | enum copies; digest tags | execution.rs:584–599 |
| plan policy family | pipeline `project_plan` | `ExecutionPlanV3.policy_family` | policy payloads (Relational params etc.) dropped | family tag only in digest | execution.rs:600–608 |
| sealed algebra node tree | pipeline `project_node`/`project_relation_root` | `ExecutionRelationV3.root` | no | full operator tree incl. join mode, predicates, correlations, schema | execution.rs:1277–1303 |
| document expression table | pipeline `close_expression_ids` + `project_expression` | `ExecutionRelationV3.expressions` | expressions unreachable from the item's operator tree | BTreeSet closure (stable order); dangling/foreign fail closed | execution.rs:1283–1297, 1363–1383 |
| `TypedLiteralFactV2` | pipeline `project_expression_shape` | `ExecutionLiteralV3` | unknown-cause of editor `Unknown` type | i128::MIN materialized; float via `to_bits()` (−0/NaN preserved); string/bool/null copied | execution.rs:1724–1740 |
| `ScalarTyV1` | pipeline `project_scalar` | `ExecutionScalarTypeV3` | no | 1:1 enum copy | execution.rs:328–344 |
| resolved composition bindings | pipeline `project_composition` + producer/body projections | `ExecutionCompositionV3` | no | length- and kind-checked against sealed bindings; drift → typed assembly error | execution.rs:1415–1430; compile.rs:1676–1692 |
| graph query contract (`GraphQueryContractV2`) | pipeline `project_graph_query` | `ExecutionGraphQueryV3` | witness details per logic owner | carried as payload inside relation/composition projections | execution.rs:1532; compile.rs:1412–419 |
| plan-required item identity/family | pipeline `execution_identity` + `ExecutionItemV3::new` | `ExecutionDocumentV3.items` | `RuleMember` items excluded (members live on `CompiledRuleProgramV2`) | source-ordered; digest via item loop | compile.rs:1043–1057, 1600–1612 |
| document-wide rule program (plan + stratification) | pipeline `compile_rule_program` | `CompiledDocumentV2.rule_program` | **yes — not an ExecutionDocumentV3 item; outside contract_digest** | retained proof, not re-charged; digest coverage gap recorded as E1-02 | compile.rs:1488–1519; execution.rs:1927–1953 |
| NoPlanReason classification | pipeline `run_item` matches on `SemanticItemV2` | `CompiledItemV2::NoPlan` | no execution payload | classification adapter over owner reasons (MetaActivation/CatalogDeclaration/ValueDeclaration) | compile.rs:1453–1484 |
| diagnostics (stage/code/severity/span) | pipeline `diagnostic.rs` sort/canonicalize; stdio handler serializes | `FrontendDiagnosticV2` wire rows | notes flattened only at stdio JSON boundary per E-2 | stable code + checked location preserved; sort key determinism → lane F | compile.rs:851–890; handlers/v3_diagnostics.rs |
| facade re-exports | frontend-contract lib.rs | public seam | owner internals (HIR/Sema/Algebra/Planning/Pipeline types beyond curated DTOs) unreachable | curated allowlist, no glob | frontend-contract/src/lib.rs:16–66 |

Per-field DTO/digest inventory detail (E-2): see `projection-input-E2.md`.
