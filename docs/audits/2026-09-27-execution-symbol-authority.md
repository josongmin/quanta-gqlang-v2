# E3-6: fail-closed execution symbol projection

Base: `976b7d7e31a6ddc2787dd750374595ab81644e37`.

## Requirement and source boundary

BuiltinIdV2 and OperatorIdV2 have private raw representations. All currently
issued IDs have spellings, pinned by exhaustive Types registry tests. No normal
source input was demonstrated to reach the old fallback. The remaining gap was
failure behavior if that registry-to-spelling contract drifted: Pipeline turned
None into `invalid_closed_builtin` or `invalid_closed_operator`, then issued an
execution DTO and digest containing the invented identity. For builtins the
name is the only encoded call-target identity, so this also collapsed different
unmapped IDs to one successful identity.

Acceptance is exact preservation of owner spellings on success and typed
rejection of missing spellings, propagated through all expression shapes,
relation tables, composition producers/bodies and the compile transaction.
Registry membership remains the Types owner's authority; Pipeline introduces
no spelling allowlist or heuristic and no raw-ID construction seam.

## Change

A shared Option-to-Result conversion rejects absent owner spelling. Call target,
operator, expression shape and expression projectors now return Result. The
relation table cannot finish after a projection error. Existing relation and
composition callers propagate the error through the assembly boundary as
`pipeline.document.unregistered_execution_symbol`, Error/Validation at the
original enclosing authored item's location. No success DTO or digest is issued
on that path. Full expressions or prefixes are not partially returned.

The assembly enum gains one variant/code. Valid source behavior, output schema,
encoding bytes and digest domains are unchanged. This is a structural defense
against owner mapping drift, not evidence of a previously exploitable source
input. The earlier residual disposition in `2026-09-27-blocked-failure-authority.md`
is superseded for E3-6 only.

## Proof and limits

- Committed unit negative control passes None to the exact shared conversion and
  requires the typed error; present names/operators preserve exact spelling.
- Independent literal bridge tests include the new code.
- A committed source regression pins builtin `length` and integer `+` overload
  101 in three contexts: top-level relation, composition query producer, and
  composition relation body. The source passes through the real closed authority,
  semantic transaction, algebra, planning and execution projection.
- Fault injection separately removes builtin ID 14's name and operator ID 101's
  symbol in the actual Types owner. For each fault, a temporary source regression
  requires one document error with exact code, stage, severity, whole-item span,
  empty secondary spans and message in all three contexts.
- Restoring a success-shaped placeholder at the shared conversion makes the
  negative source probe fail at runtime: compilation improperly returned Ok.
  Both faulty fallback runs compiled; compile failures are not mutation kills.
- All temporary fault edits and probes are restored. No extra test feature,
  environment gate or production injection hook is added. Ordinary CI executes
  the committed controls; fault injection is separate owner-local evidence.

The pipeline library suite passed 48 tests before integration. Full qualification
belongs to the eventual clean committed-HEAD receipt. Raw four-run fault logs,
probe script and results are under `target/execution-symbol-audit/` in the owner
worktree. No runtime execution claim is made.
