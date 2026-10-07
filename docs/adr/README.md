# Architecture decision records

Durable architectural decisions live here. Execution plans, worker prompts,
wave ledgers, and source-bound campaign receipts do not define current behavior.

- [ADR-0001: Consolidate retired frontend campaigns into current authority](0001-frontend-campaign-consolidation.md)
- [ADR-0002: Test authority and adversarial oracle design](0002-test-authority-and-adversarial-oracles.md)
- [ADR-0003: Frontend/downstream integration and proof boundary](0003-frontend-downstream-integration-boundary.md)
- [ADR-0004: Sealed owner facts and bounded frontend projection](0004-sealed-owner-facts-and-bounded-projection.md)
- [ADR-0005: Substrate reuse and consumer projection ownership](0005-substrate-reuse-and-consumer-projection.md)
- [ADR-0006: Code-query evaluation and dependency adoption boundary](0006-code-query-evaluation-boundary.md)
- [ADR-0007: Typed code-query contract proposal (Proposed)](0007-typed-code-query-contract-proposal.md)
- [ADR-0008: Private diagnostics primary-location contract](0008-private-diagnostics-primary-location-contract.md)

Current language behavior and proof remain owned by the
[language reference](../language/README.md),
[frontend execution contract](../contracts/frontend-execution-v3.md), and
[test hellgate ledger](../language/test_hellgate_ledger.md).

[Remaining work](../work/remaining.md) contains only unfinished acceptance.
[ADR-0001](0001-frontend-campaign-consolidation.md#2026-10-07-consolidation)
maps retired document/ticket groups to their authority and Git recovery baseline.
ADR status records a decision; it does not qualify runtime execution.
