# GQLang v3 frontend engine status

Authority: **code**. This note is a living index, not a proof receipt.
Source re-read 2026-10-05 against producer `main@1ddeaac` before the
documentation checkpoint. This is a static source inventory, not a new run.
Current maintenance/proof status: [session audit](../audits/2026-10-05-current-source-audit-action-plan.md).

## Scope the code implements

Host-agnostic frontend compiler. Public seam is
`quanta-gqlang-v2-frontend-contract` → `compile_source_document_v3`.

Live stages: `surface` parse → `HirBuilderV1` → `CheckedSyntaxDocumentV2` →
binder + types + modules (`resolve_checked_document_v2`) → algebra
(`lower_root_with_budget`) → logic (`stratify` / Tarjan) → planning
(`plan` / `classify`) → `ExecutionDocumentV3` (three digests).

`unimplemented!` / `todo!` are clippy-denied in src.

## Not this engine

Runtime substrate, physical plan, provider/generation admission, result
materialization, FIND/CHECK **solving**. Those DTO fields are contracts.

## Supported and refused boundaries

| Surface | Code |
| --- | --- |
| All authored `query.result_shape` forms | Parse/format live; `materialize_query` typed RED |
| `admit_show_planned_source_v2` one-shot | Source-cap refusal or `IncompleteSourceDiagnostics` before lexer; diagnostic debit is not issued |
| Open analysis / rename without catalog | Incomplete DTO, never an execution document |
| SHOW facade syntax owner | Live; syntax alone grants no semantic plan or execution |
| SHOW Pipeline staged owners | Syntax → declaration → plan → Execution DTO retains one root; positive/short/foreign/final-drop owner rails exist. These later entries are not facade exports or runtime authority |

## Stale claims this note supersedes

- `hir/src/build.rs` used to say the v3 parser did not exist. Parser is live.
- Sema allowlist comments used to say unsupported root families were RED.
  All current `ProgramItemHirV1` variants have owner paths.

## Proof that is code, not README

Facade `conformance` `every_family_compiles_through_the_facade_with_declared_disposition`
runs `compile_source_document_v3` for 19 registered source families, including
Rename Command. Separate facade rails retain preview/apply intent and reject
unsupported result shapes; they establish no runtime mutation authority.
