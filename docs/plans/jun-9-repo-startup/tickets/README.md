# Jun 9 Repo Startup Ticket Pack

Status: `Done`

## What This Packet Is

This packet is not a generic cleanup note.

It is the repo-local startup and recovery packet for `quanta-gqlang-v2` after
the current base narrowed or deleted parts of the original local ticket scope.

The packet keeps two truths separate:

1. original ticket authority still says `E05E`, `E06C`, `E11C`, and `E12C`
   belong to this repo
2. the current local base ships a narrower read-only shell contract

This packet chooses to recover the original local ticket scope inside this repo
without changing cross-repo ownership.

## Source Anchors

Local repo anchors:

1. [language/README](../../../language/README.md)
2. [language/release_discipline](../../../language/release_discipline.md)
3. [Justfile](../../../../Justfile)

Original ticket anchors in the sibling RFC packet:

1. [E05E](../../../../../semantica-codegraph-v2/docs/rfcs/2026-06-05-rfc-qglang-v2-semantic-host-dx-platform/tickets-v2/external/E05E-ai-exploration-profile-working-set-and-reconfirmation-semantics.md)
2. [E06C](../../../../../semantica-codegraph-v2/docs/rfcs/2026-06-05-rfc-qglang-v2-semantic-host-dx-platform/tickets-v2/external/E06C-public-planning-and-lowering-ready-ir.md)
3. [E11C](../../../../../semantica-codegraph-v2/docs/rfcs/2026-06-05-rfc-qglang-v2-semantic-host-dx-platform/tickets-v2/external/E11C-metric-inventory-and-external-findings-surface.md)
4. [E12C](../../../../../semantica-codegraph-v2/docs/rfcs/2026-06-05-rfc-qglang-v2-semantic-host-dx-platform/tickets-v2/external/E12C-explain-and-path-witness-ux.md)
5. [dispatch/INDEX](../../../../../semantica-codegraph-v2/docs/rfcs/2026-06-05-rfc-qglang-v2-semantic-host-dx-platform/tickets-v2/dispatch/INDEX.md)

## Closure Snapshot

1. local docs say this repo currently ships a read-only shell only
2. local docs say experimental catalog and AI-adjacent surfaces are metadata-only
3. the named ticket proof rails are present in this repo:
   - `ai_profile_contract`
   - `public_planning_ir`
   - `metric_inventory`
   - `explain_path_ux`
4. the repo-local composition rails are green:
   - `cargo clippy --workspace --tests -- -D warnings`
   - `just ci`
5. Wave 3 cleanup is landed:
   - `read_only_analysis.rs` owns shared LSP read-only interpretation
   - root-local test helpers moved out of `syntax/src/lib.rs`
   - crate-root test-time allow blocks are removed

## Historical Note

1. per-ticket `Current Base` sections below capture the startup baseline before the recovery landed

## Canonical Waves

1. Wave 0
   - [00-packet-scope-and-repo-boundary](./00-packet-scope-and-repo-boundary.md)
2. Wave 1
   - [01-e05e-ai-profile-working-set-and-reconfirmation-recovery](./01-e05e-ai-profile-working-set-and-reconfirmation-recovery.md)
   - [02-e06c-public-planning-and-lowering-ready-ir-recovery](./02-e06c-public-planning-and-lowering-ready-ir-recovery.md)
   - [03-e11c-metric-inventory-and-external-findings-recovery](./03-e11c-metric-inventory-and-external-findings-recovery.md)
   - [04-e12c-explain-path-witness-and-why-no-result-ux-recovery](./04-e12c-explain-path-witness-and-why-no-result-ux-recovery.md)
3. Wave 2
   - [05-repo-local-serial-integration-and-green-rails](./05-repo-local-serial-integration-and-green-rails.md)
4. Wave 3
   - [06-lsp-read-only-interpreter-extraction](./06-lsp-read-only-interpreter-extraction.md)
   - [07-ir-facade-and-lint-test-root-cleanup](./07-ir-facade-and-lint-test-root-cleanup.md)

## Worker Contract

1. source first, docs second
2. this packet edits `quanta-gqlang-v2` only
3. do not treat the current narrowed local docs as authority to silently cancel
   `E05E/E06C/E11C/E12C`
4. restore named proof rails as separate test targets; do not fold them into a
   generic workspace-only test story
5. keep frontend-only ownership, read-only shell behavior, and no-runtime-truth
   claims intact
6. `05` owned repo-local glue and the final `just ci` claim
7. `06` and `07` were post-closure cleanup tickets, not prerequisites for local
   ticket recovery
