# ADR-0003: Frontend/downstream integration and proof boundary

- Status: Accepted
- Date: 2026-09-27
- Scope: public frontend integration, downstream admission, and cross-repository proof

## Context

The Sep-25 integration packet mixed durable QGLang decisions with transient
Semantica work orders, absolute checkout paths, source SHAs, run IDs, and
point-in-time status. The QGLang implementation and contract portions have
since landed in the owning source, contract, and tests. Remaining installed
process and product proof belongs to the downstream repository.

Keeping that packet as a live QGLang plan creates a second status authority and
contradicts [ADR-0001](0001-frontend-campaign-consolidation.md), which retired
`docs/plans/` in favor of current source, contracts, executable tests, and
source-bound receipts.

## Decision

### Supported V3 relations share one downstream owner

Composition bodies and standalone admitted relations share the bounded
runtime-owned relation compiler/evaluator. It consumes sealed operator,
expression, polarity and correlation facts rather than reparsing source or
using HIR/operator spelling as execution authority. The admitted RepoPath
subset is distinct from proposed general typed multi-column join.

EXISTS/NOT EXISTS preserve the exact outer environment and right predicate,
left multiplicity and stable order. Right duplicates do not multiply left rows.
Multiple terms retain flat authored-order term/right-child pairing. Failure
cannot become empty right input or successful absence. Unsupported operators
reject before producer acquisition.

The original request root owns memory permits and cumulative work across
producer, nested evaluation and output. Plan positions are private execution
identities; source-stage provenance keys may repeat. Receipts bind actual
plan/input authority/output, with downstream versioning if those bytes change.

### CHECK work and process proof retain their original owners

Canonical count/encode work runs in the existing Foundation owner and serving
control. Native Match checks count, bounded encoding, length, copy/hash work and
digest before transport. Typed work refusal stays non-success. Historical
Foundation37 and Native Match23 passes are proof at their original source.

Graph/CHECK fixtures materialize source before daemon startup and use the same
configured/attached root. Linux Graph success and non-Linux refusal are separate
selectors. Signed Linux CHECK requires its release profile, installed trust,
immutable slot, approval/key and active generation. Generic artifacts,
readiness, a scripted daemon or integrated fixtures do not prove actual queries.

Original-session admission, borrowed-to-owned handoff and curated SDK type
exports remain downstream contracts. Missing wiring is repaired there, without
another frontend engine or default session.

The Oct-4/Oct-5 repair packet and Oct-6 process continuation are retired.
Unfinished acceptance remains in [remaining work](../work/remaining.md).

### The frontend contract is the only public downstream seam

`quanta-gqlang-v2-frontend-contract` is the product-facing integration boundary.
Downstream code consumes its public DTOs and closed compilation entry points;
it does not depend on owner HIR, recreate frontend semantics, or retain a legacy
adapter as a compatibility facade.

`classify_single_source_root_v3` is a parse-level routing aid. It does not issue
an execution document or grant runtime authority. Only the closed source
compiler can issue the source-bound execution projection described by the
[frontend execution V3 contract](../contracts/frontend-execution-v3.md).

### Frontend identity and runtime identity remain distinct

The source digest identifies source bytes. Contract and qualification digests
also cover the public execution projection, including source identity and
authority-sensitive fields. Therefore, two compilations are comparable only
when their source bytes, `SourceId`, authority projection, and public DTO are
the same. Same bytes under different authority are not expected to produce the
same contract or qualification digest.

Runtime generation, provider selection, leases, and budget are outside the
frontend digest. The downstream admission owner must bind those values to the
accepted frontend artifact. Frontend digest equality alone cannot establish
runtime identity or permission to execute.

### Representation does not imply runtime support

The frontend may represent constructs that have no standalone executable
payload or no admitted downstream implementation. Downstream must reject those
constructs with a typed disposition before effects unless a separately
specified and tested runtime contract admits them.

Composition likewise preserves bindings, order, and body identity without
claiming that every represented family is executable. New runtime support
requires an explicit contract change plus producer and consumer tests; an
adapter-side interpretation is not sufficient.

### Proof is source-bound and owner-specific

A clean committed-HEAD QGLang qualification proves the frontend producer at
that exact source. It does not prove the downstream admission owner, SDK,
installed process, deployment, or product behavior.

Cross-repository qualification must record one fixed source tuple and the
participating artifact identities. Where a claim crosses owner, SDK, and
installed-process boundaries, the evidence must also record selected,
executed, passed, skipped, and leaked counts. Manual runs or receipts from
different digests cannot be combined into one product proof.

Downstream release blockers and product receipts are maintained in the
downstream repository or its active tracker. A point-in-time downstream status
does not become current QGLang documentation.

### Retire the Sep-25 integration packet

Delete `docs/plans/bugbash-09-25-integration/`, including its closed producer
ticket and its downstream work orders. Git history retains the scheduling and
handoff detail.

This deletion does not declare unexecuted downstream product proof complete.
It removes a stale, wrong-owner status surface after preserving the durable
boundary and evidence rules here.

## Consequences

- QGLang has one durable integration decision and one normative V3 contract.
- Historical SHAs, checkout paths, run IDs, and worker assignments no longer
  appear as current architecture or qualification state.
- A frontend change can be qualified independently without overstating product
  readiness.
- A downstream product claim still requires downstream-owned evidence bound to
  the exact producer and consumer sources.

## Reopen conditions

Revisit this decision only if one of these changes:

1. the public frontend facade is versioned or replaced;
2. runtime identity becomes part of the frontend-owned contract;
3. a currently representable construct gains an explicit downstream execution
   contract; or
4. the repositories adopt a shared, machine-verifiable cross-repository
   qualification authority.
