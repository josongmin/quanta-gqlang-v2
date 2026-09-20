# Sep-21 Frontend Reconstruction Execution Ledger

Frozen on `2026-09-21` before product edits.

## Source Freeze

- Base SHA: `e896a11024bd5a1e1ef03c094a70032f4e6c765f`
- Branch: `main`
- Upstream: `origin/main`
- Product-tree state at freeze: no tracked modifications
- Packet-owned untracked tree:
  `docs/plans/sep-21-gqlang-sota-frontend-reconstruction/`
- External concurrent untracked tree, read-only for this packet:
  `docs/bugbash/sep-21-test-improvement/`
- External concurrent untracked audit, read-only for this packet:
  `docs/test-suite-audit.md`
- Integrator: `/root`
- Execution protocol: `ss`; checkpoint review protocol: `rr`
- Commit policy: implementation remains uncommitted unless the user explicitly
  authorizes commits. Receipts therefore identify the frozen base plus the
  scoped working-tree fingerprint until a commit SHA exists.

### Integration history after the original freeze

- Original audit and implementation base:
  `e896a11024bd5a1e1ef03c094a70032f4e6c765f`.
- First externally created integration commit:
  `96eab779bfce0a5178dbbd48e86bd457158db228`, subject `done`.
- `96eab779` contains the verified Ticket 01/02 stack and plan/bugbash material,
  but it is not a clean Ticket 01/02 boundary: it also contains Ticket 03-owned
  destructive type/module edits. Those edits were not covered by receipts
  01/02 and must be audited by receipt 03 against the original `e896a110...`
  source, not treated as trusted baseline behavior.
- Second externally created integration commit:
  `fcf106b56e794eb38f74c52b3bd2c1cf34f77fdc`, subject `done`.
- `fcf106b` is an atomic-red partial Ticket 03 snapshot: HIR was green, types did
  not compile, modules had no Ticket 03 source delta, formatting failed, and no
  receipt 03 existed. It is an integration state only, not a verified ticket
  boundary or releasable candidate.
- Preservation: no reset, amend, or history rewrite was performed.
- Current work continues from `fcf106b` with Ticket 03 dirty changes owned by
  `/root` for HIR/types and `/root/ticket03_modules_impl` for modules.
- Evidence rule: receipts 01/02 retain their original frozen-source evidence;
  their schema backfills below do not extend that evidence. Receipt 03 must
  separately enumerate inherited contamination in `e896a110..96eab779` and
  active repair in `96eab779..final`, while comparing preserved algorithms and
  tests to `e896a110`.

The external bugbash tree and root test-suite audit may inform verification, but
no packet worker may edit, move, delete, stage, or claim ownership of them.

## Accepted Decisions

All twenty decisions in
[00 — Packet Scope And Pinned Decisions](tickets/00-packet-scope-and-pinned-decisions.md)
are accepted without replacement. In particular:

1. `quanta-gqlang-v2-sema` is the only new semantic owner.
2. The V2 cutover is breaking and atomic; no compatibility facade is allowed.
3. Closed compilation and open analysis remain distinct result types.
4. Ticket 01 mechanically renames the current private flat-JSON transport to
   `quanta-gqlang-v2-diagnostics-stdio`; ticket 08 rewrites framing to bounded
   UTF-8 NDJSON. This packet does not call it LSP or add JSON-RPC behavior.
5. Rule arity remains unary.
6. Numeric semantics remain `i128` plus finite `f64`.
7. Tickets 01-09 may not widen language, runtime, provider, SDK, optimizer, or
   physical-planning scope independently.

## Ownership And Dependency State

`write set` points to the authoritative enumerated section in each ticket; the
short scope is an additional collision check, not a substitute for that list.

| Ticket | Owner | State | Dependency state | Short write set |
|---|---|---|---|---|
| 00 | `/root` | `ACCEPTED` | base frozen | packet decisions and this ledger only |
| 01 | `/root/ticket01_impl` | `VERIFIED` | 00 satisfied; RR re-review found no remaining scope blocker | workspace manifests/lock, empty sema crate, architecture/Semgrep/CI gates, mechanical `lsp` to `diagnostics-stdio` move, receipt 01 |
| 02 | `/root/ticket02_prep` | `VERIFIED` | RR re-review clean; owner-local proof green | surface and HIR owners/tests named in ticket 02, receipt 02 |
| 03 | `/root` + `/root/ticket03_modules_impl` | `ACTIVE` | 02 verified; HIR/types focused proof observed; modules, final RR, and receipt 03 still open | HIR source-registry extension, types and modules owners/tests named in ticket 03, receipt 03 |
| 04 | `/root` | `QUEUED` | waits for 02 and 03 | sema, binder, integrator-owned pipeline seam, legacy API inventory, receipt 04 |
| 05 | `/root/algebra_impl` | `QUEUED` | waits for 04 | algebra owner/tests plus integrator-owned pipeline seam, receipt 05 |
| 06 | `/root/logic_impl` | `QUEUED` | waits for 04 | logic owner/tests plus integrator-owned pipeline seam, receipt 06 |
| 07 | `/root` | `QUEUED` | waits for 05 and 06 | planning, final document assembly, curated facade, integrator seams, receipt 07 |
| 08 | `/root/frontdoors_impl` | `QUEUED` | waits for 07 | CLI, diagnostics stdio, legacy seam deletion, public docs, integrator manifests, receipt 08 |
| 09 | `/root` | `QUEUED` | waits for 08 | conformance, truth sync, transition deletion, final gates and closeout receipt |

## Integration Rules

1. `/root` is the only writer for root `Cargo.toml`, `Cargo.lock`, shared
   pipeline/facade manifests, and any file marked `integrator only` after
   ticket 01.
2. A worker owns only its ticket's enumerated write set. Cross-owner defects
   reopen the owning ticket; consumers do not patch around them.
3. Ticket 01 is serialized. Ticket 03 types RED preparation may overlap ticket
   02, but modules implementation follows ticket 02's shared HIR/location cut.
   Tickets 05 and 06 may implement in parallel only after ticket 04 passes its
   checkpoint. Tickets 02-08 are one unreleasable atomic cutover stack.
4. Broad Cargo verification is serialized by `/root`. Workers run only focused
   owner-local checks that do not compete for the shared target directory.
5. Every checkpoint records the diff scope, owner/consumer trace, duplicate
   owner scan, focused tests, failures, repairs, and residual risk in that
   ticket's receipt.
6. The repository does not provide the Semantica-specific persona/session
   scripts referenced by the generic `rr` workflow. Reviews therefore apply
   `rr`'s diff-first, live-source, sibling-universe, adversarial, and evidence
   rules directly and record the missing scripts as repository-inapplicable;
   no substitute authority file is invented.

## State Transitions

Allowed transitions are:

```text
QUEUED -> READ_ONLY_PREP -> PREP_REVIEWED -> ACTIVE -> REVIEW -> REPAIR -> VERIFIED
                                                      REVIEW ------------> VERIFIED
```

`VERIFIED` requires the ticket definition of done and a completed receipt.
Only ticket 09 may mark the packet `REPO_LOCAL_CLOSED` (hosted CI unavailable)
or `CLOSED_BY_09` (hosted CI green). A verified hosted failure blocks both.

## 2026-09-21 Implementation-Aware Plan Re-Audit

The final multi-angle audit found that the original 04-09 prose was not
executable against the active 02/03 cutover. The ticket-local corrective
sections are normative. Concrete repairs:

1. tickets 04-06 now stop at owner-only `FOCUSED_GREEN`; every deferred
   consumer compiler failure is recorded and ticket 07 is the first pipeline
   integration gate;
2. `SemanticItemIdV2` is the sole document item join key and all lowerer output
   sets are checked for exact equality before assembly;
3. types reuse `SourceQualifiedNodeIdV2`, `ResolvedCallTargetV2`, and
   `ResolvedPredicateIdV2`; the plan no longer invents parallel IDs;
4. open authority input/absence rules and fallible standard construction are
   explicit; ticket 07 provides a facade-only closed-authority builder;
5. correlated semi/anti lowering retains an explicit correlation set;
6. authority byte limits are documented honestly as per sealed admission
   session, not a global total implied by profile equality;
7. diagnostics stdio is pinned to UTF-8 NDJSON, the stale LSP paths are removed,
   and CLI/service parity moves to a dependency-neutral conformance harness;
8. ticket 09 migrates the proof ledger before deletion, separates closed/open
   matrices, and refuses closure without a clean committed candidate SHA.

No ticket 04 implementation starts until ticket 03's final RR and receipt are
complete. These plan corrections do not advance any implementation state.
