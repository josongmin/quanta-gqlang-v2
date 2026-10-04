# GQLang v3 frontend engine status

Authority: **code**. This note is a living index, not a proof receipt.
Audited 2026-10-03 against the local checkout.

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

## Code holes (still true)

| Surface | Code |
| --- | --- |
| All authored `query.result_shape` forms | Parse/format live; `materialize_query` typed RED |
| `admit_show_planned_source_v2` | Always `IncompleteSourceDiagnostics` before lexer |
| Open analysis / rename without catalog | Incomplete DTO, never an execution document |
| SHOW lexical | Staged APIs live; not an execution grant |

## Stale claims this note supersedes

- `hir/src/build.rs` used to say the v3 parser did not exist. Parser is live.
- Sema allowlist comments used to say unsupported root families were RED.
  All current `ProgramItemHirV1` variants have owner paths.

## Proof that is code, not README

Facade `conformance` `every_family_compiles_through_the_facade_with_declared_disposition`
runs `compile_source_document_v3` for 19 registered source families, including
Rename Command. Separate facade rails retain preview/apply intent and reject
unsupported result shapes; they establish no runtime mutation authority.
