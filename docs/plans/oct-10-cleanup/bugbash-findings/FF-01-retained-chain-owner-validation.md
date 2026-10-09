# FF-01 — request owner validates only the chain head

**Final triage:** [not in the active repair queue](FINAL-TRIAGE.md). The selective-sink API counterexample is real; the current Semantica sink's same-budget acceptance does not establish a reachable production failure. Retain for host-contract review.

- **Classification:** confirmed public API invariant failure under a selective host sink; deployed reachability unverified.
- **Owner:** HIR request-storage custody (`crates/quanta-gqlang-v2-hir/src/request_storage.rs`).
- **Affected contract:** a later stage must continue under the issued root for **all** retained reservations. The file's own full-chain check says a head-only check can admit foreign retained custody (`request_storage.rs:137-150,326-338`).

## Evidence and counterexample

`FrontendRequestMaterializationRootV3::from_issued_sink_v3` is public (`request_storage.rs:60-69`). `FrontendRetainedStoragePermitV3::validate_owner_v3` checks source identity and `root.sink.owns_permit_v3(self.permit)` only for the current head (`:172-185`). `try_extend_v3` calls that method, reserves through the supplied root, then links the previous chain behind the new head (`:189-212`). In contrast, `validate_issued_sink_v3` walks every retained permit (`:140-150`). Pipeline source check and Sema declaration use `validate_owner_v3`, and declaration allocation can call `try_extend_v3` (`crates/quanta-gqlang-v2-pipeline/src/compile/request_owner.rs:260-289`; `crates/quanta-gqlang-v2-sema/src/document/declaration.rs:121-129,340-352`).

An independent fixture compiled the **original** HIR source via `#[path]`. Sink A issued two permits. Sink B recognized only the newest A permit (not the older A permit) and issued new B permits. With identical source and source ID:

1. `chain.validate_issued_sink_v3(&b)` returned `ForeignRequestRoot`.
2. `chain.validate_owner_v3(&root_b, id, source)` returned `Ok(())`.
3. `chain.try_extend_v3(&root_b, id, source, 1)` returned `Ok(())` and linked B's permit ahead of A's two permits.
4. `chain.validate_owner_v3(&root_b, id, source)` still returned `Ok(())`, while `chain.validate_issued_sink_v3(&b)` still returned `ForeignRequestRoot`.

Run: `rustc --edition=2021 -Awarnings /tmp/qglang-ff01-owner-chain.rs -o /tmp/qglang-ff01-owner-chain && /tmp/qglang-ff01-owner-chain`; exit `0`; stdout `reproduced: mixed-issuer chain admitted by validate_owner and try_extend; full-chain validation rejects`.

## Impact and repair criterion

The public API can admit a later-stage value whose earlier reservation is outside the currently supplied sink's accepted ledger. The fixture demonstrates the validation inconsistency, **not** a proven production host exploit: no downstream sink was shown to recognize only a chain head, and a sink that recognizes all permits from one stable issuer does not trigger it.

Reject this counterexample before any new reservation. Preserve same-source, same-issuer continuation and last-drop release. Avoid rescanning a potentially long retained chain on every extension without bounding the resulting repeated work; a stable issuer witness carried by the chain is a possible design, subject to compatibility and host-authority review. Add a repository regression with two admissible sink views and inspect downstream sink implementations before making a runtime impact claim.

## Verification state

- Reproduced against the original source file in an external Rust fixture, no repository source edits.
- Repository regression, focused Cargo tests, full `just ci`, consumer build, runtime and performance measurement: **NOT RUN** in this lane.
