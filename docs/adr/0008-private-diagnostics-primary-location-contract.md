# ADR-0008: Private diagnostics primary-location contract

- Status: Accepted
- Date: 2026-10-07
- Scope: QG-DIAG contract decision; existing V2 NDJSON wire

## Context

The frontend semantic diagnostic retains secondary locations and typed authority
failures. Its private stdio wire exposes a smaller projection. That difference
alone does not establish a consumer defect or require another wire version.

The reachable process path is `main` → `run_stdio_front_door_v2` →
`DiagnosticsServerV2::dispatch` → `handle_v3_diagnostics_v2` →
`analyze_source_document_v2`. The repository's actual parity consumer and NDJSON
tests require the exact seven-field diagnostic projection.

## Decision

Keep the existing V2 wire and document its narrower contract in the
[language reference](../language/README.md#cli-and-diagnostics-stdio).

- Diagnostics contain stage, code, severity, message and the primary source ID
  with a half-open byte span. Secondary locations remain internal.
- A semantic error is an analysis result. An authority failure remains the
  protocol `authority_error` category and human-readable message; it is not a
  successful diagnostic response or a structured authority identity.
- A demonstrated client requirement for more fields requires a versioned
  producer/consumer change with bounded-response and process oracles.

QG-DIAG is resolved as a contract decision. No wire or executable behavior changes.
This decision does not establish downstream runtime or installed qualification.

## Existing consumer and proof owners

- [Parity consumer](../../tools/test_frontdoor_parity.py) validates exact response
  and diagnostic key sets.
- [NDJSON contract](../../crates/quanta-gqlang-v2-diagnostics-stdio/tests/ndjson_contract.rs)
  rejects unsupported request fields and checks the diagnostic projection.
- [Binary contract](../../crates/quanta-gqlang-v2-diagnostics-stdio/tests/binary_contract.rs)
  exercises the real stdio process.

The source readback establishes the decision's consumer basis. Current execution
results must be recorded separately; source presence is not a new passing receipt.
