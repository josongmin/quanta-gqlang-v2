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

### Re-freeze after external integration

- Current base SHA: `96eab779bfce0a5178dbbd48e86bd457158db228`
- Observed commit subject: `done`
- Attribution: external actor committed the previously verified Ticket 01/02
  stack plus packet/bugbash documentation while Ticket 03 was active.
- Preservation: no reset, amend, or history rewrite was performed.
- Active dirty ownership after re-freeze: Ticket 03 HIR/types work, generated
  `Cargo.lock`, and Ticket 03/04 contract amendments by `/root`.
- Evidence rule: receipts 01/02 retain their original frozen-base evidence;
  receipt 03 and later must identify this re-frozen base and a fresh dirty-tree
  fingerprint. The new commit is integration state, not proof that hosted CI ran.

The external bugbash tree and root test-suite audit may inform verification, but
no packet worker may edit, move, delete, stage, or claim ownership of them.

## Accepted Decisions

All twenty decisions in
[00 — Packet Scope And Pinned Decisions](tickets/00-packet-scope-and-pinned-decisions.md)
are accepted without replacement. In particular:

1. `quanta-gqlang-v2-sema` is the only new semantic owner.
2. The V2 cutover is breaking and atomic; no compatibility facade is allowed.
3. Closed compilation and open analysis remain distinct result types.
4. The current private flat-JSON transport is renamed mechanically to
   `quanta-gqlang-v2-diagnostics-stdio`; this packet does not call it LSP or add
   JSON-RPC behavior.
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
| 03 | `/root/ticket03_impl` | `ACTIVE` | 02 verified; contract under adversarial consumer audit | HIR source-registry extension, types and modules owners/tests named in ticket 03, receipt 03 |
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
Only ticket 09 may mark the packet closed.
