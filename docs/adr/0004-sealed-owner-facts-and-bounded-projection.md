# ADR-0004: Sealed owner facts and bounded frontend projection

- Status: Accepted
- Date: 2026-10-07
- Scope: completed Sep-26/Sep-27 frontend remediation and Oct-4/Oct-5 producer maintenance
- Consolidation baseline: `68ff02ef443fcb5aecccb885adad2cb24af8e719`

## Context

Lane reports, repeated findings, remediation plans and scenario checklists
retained incompatible snapshots after their repairs reached source and owner
tests. Their finding identifiers describe historical investigations, not a
current defect census. Preserve the resulting invariants here; current API
definitions remain in the [owner-boundary contract](../contracts/frontend-owner-boundaries.md),
[planning-result contract](../contracts/planning-results.md), and
[V3 execution contract](../contracts/frontend-execution-v3.md).

## Decision

### Consume facts from the stage that can issue them

- Surface scan/decode share one checked escape walker. Unsupported escapes fail;
  decoding cannot silently omit a character. Allocation accounting uses decoded
  payload length rather than raw escape spelling length.
- Binder distinguishes a Composition relation-source reference from an outer
  row-expression capture. Scan authority does not invent correlation.
- Types owns checked value facts, overload identity and transparent-parenthesis
  behavior, including the exact negative `i128::MIN` boundary.
- Modules owns sealed import/catalog resolution. Binder's inert input snapshots
  do not mint a Modules proof. Sema's declared state is not resolved evidence.
- Rule literal classification and polarity consume typed, position-sensitive
  facts. Composed effects do not grant literal position to calls inside opaque
  scalar wrappers. Logic validates the exact retained dependency and location.
- Sema seals exact ordered Join fields, columns and provenance, Bag cardinality,
  and empty partition/order. Algebra reuses that frame predicate and checks
  Project/Group/Aggregate output types against checked expressions.
- Planning owns non-relational result and cost policy. Shared capability names
  do not erase the distinction between rule fixpoints and bounded graph traversal.

### A session cache retains evidence, not guessed authority

Algebra indexes permitted nested roots once per owner in an immutable document
session. Discovery and lowering debit the same receipt. Membership checks still
compare the complete sealed tree after identity lookup. Absent owners and failed
discovery cannot publish a partial index. Rejection does not refund cumulative
work. This removes repeated discovery; it asserts no measured latency gain.

### Projection preserves failure and identity

Blocked item/rule carriers retain `FrontendFailureV2` rather than rebuilding
non-emptiness from a vector and inventing a fallback diagnostic. Pipeline owns
its stable sort and adjacent full-struct dedup policy; equal-key secondary-span
ties retain input order. Reporting stage does not determine the code prefix.

Execution projection rejects a missing Types-owned builtin/operator spelling
with `pipeline.document.unregistered_execution_symbol`. It cannot issue a DTO
or digest containing a fabricated identity. Successful projection bytes and
the digest domains remain governed by the versioned contract.

The qualification digest binds source and the execution projection. It is not
a digest of every compiled-document field. Rule-program representation outside
that projection does not become runtime execution support.

### Resource authority follows the request and owning stage

Sema owns the frontend profile; Pipeline exhaustively projects Surface parse
dimensions. Source bytes, transport frames, semantic traversal, diagnostics and
response limits retain their distinct owners. Owner-local convenience defaults
are not another shipped closed-compilation profile.

Producer maintenance extracted typed preparation, encoding, projection and
candidate owners, preserved request-root scratch accounting, and attached
current/MSRV and size/dependency checks to the existing CI composition. No
reverse Pipeline-to-facade dependency or wholesale DTO move is introduced.

## Executable evidence

The [test hellgate ledger](../language/test_hellgate_ledger.md) routes the current
regressions. In particular:

| Historical work | Retained owner/oracle |
| --- | --- |
| TA-01/05/06/07/08/09/10 and RA-01–05 | Exact process diagnostics/output; callable effects; EXPORT identity/use; complete composition payload; HIR spans; Binder barriers; independent item lookup |
| IR polarity, integer and catalog findings | Sema/Types/Logic source tests and facade conformance; exact literal position, typed value and catalog-edge identity |
| D-02, D-06 | Join/computed-row rejection matrix and nested ownership discovery/count/changed-tree controls |
| D-05/07 | Independent family result/cost vocabulary in Planning tests |
| E3-5/6, F-01/03/04/09 | Nonempty diagnostic propagation, missing-spelling rejection, bridge parity, facade root classification and independent digest stream/vectors |
| Oct-4/Oct-5 producer maintenance | Request-owned stage/EXISTS scratch regressions, conformance, architecture/size policy and both Clippy profiles |

These are retained proof owners, not a claim that every historical suggestion
was a defect or that old mutation runs qualify this baseline. Exact raw reports,
source pins and logs are recoverable using [ADR-0001](0001-frontend-campaign-consolidation.md).

The 2026-10-07 source recheck at the consolidation baseline retains these
direct implementation authorities:

| Invariant | Owning source |
| --- | --- |
| Shared string scan/decode | [Surface lexer](../../crates/quanta-gqlang-v2-surface/src/lexer.rs): `scan_string` and `decode_string` consume `walk_string` and the decoded byte count |
| Typed rule literal dependency | [Sema rule classifier](../../crates/quanta-gqlang-v2-sema/src/rule.rs) issues `classify_rule_body_calls`; [Logic](../../crates/quanta-gqlang-v2-logic/src/source.rs) validates retained targets/polarities/locations against those facts |
| Exact Join frame | [Sema relation](../../crates/quanta-gqlang-v2-sema/src/relation.rs) owns `has_exact_join_frame`; [Algebra](../../crates/quanta-gqlang-v2-algebra/src/normalize.rs) consumes it |
| One nested-owner discovery | Algebra's `AlgebraSessionV2::validate_nested_owner` installs `discover_owned_relations` only after successful discovery, then checks the complete retained tree |
| Nonempty failure and typed identity refusal | [Sema diagnostic](../../crates/quanta-gqlang-v2-sema/src/diagnostic.rs) seals `FrontendFailureV2`; [Pipeline diagnostic](../../crates/quanta-gqlang-v2-pipeline/src/diagnostic.rs) retains the missing-spelling rejection |
| Finite scalar request ownership | [Scalar compiler](../../crates/quanta-gqlang-v2-pipeline/src/compile/request_owner/scalar_compile.rs) composes checked stages; [execution seal](../../crates/quanta-gqlang-v2-pipeline/src/compile/request_owner/source_body/sealed/execution.rs) retains execution reservations and binds original source digests |

The scalar entries are publicly exported alongside ordinary and lexical-owner
compilation. Their source/storage coverage is specified in the V3 contract;
they do not make the unrelated staged SHOW entries public or qualified.

## Explicit limitations and dispositions

- Authored `result_shape` remains unsupported. Specialized one-shot
  `admit_show_planned_source_v2` retains typed refusal until diagnostic debit is
  issued. Staged SHOW owners do not grant runtime authority.
- Private diagnostics stdio currently exposes primary locations and wraps
  authority failures as `authority_error`; secondary locations and structured
  owner authority codes are not present on that wire. The historical transport
  finding is retained as [QG-DIAG](../work/remaining.md#qg-diag), not recorded as a repair.
- Direct owner APIs consume checked prerequisites. Raw HIR construction,
  contextual keyword roles, reserved execution tags and optional refactors are
  not independently successful closed compilation.
- Completed test work does not establish downstream execution, hosted CI,
  deployment or current clean-HEAD qualification.

## Consequences

Retire the duplicated lane reports, findings, scenario checklist and completed
producer work orders. Reopen an item from a current caller/owner counterexample
or an explicit consumer requirement, rather than an old severity label.
