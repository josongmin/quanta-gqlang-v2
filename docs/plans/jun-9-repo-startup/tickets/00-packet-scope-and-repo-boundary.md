# 00 Packet Scope And Repo Boundary

Status: `Done`

## Goal

Freeze the repo-local execution vocabulary for this packet and separate local
recovery from cross-repo closeout.

## Depends On

1. none

## Unblocks

1. `01-e05e-ai-profile-working-set-and-reconfirmation-recovery`
2. `02-e06c-public-planning-and-lowering-ready-ir-recovery`
3. `03-e11c-metric-inventory-and-external-findings-recovery`
4. `04-e12c-explain-path-witness-and-why-no-result-ux-recovery`
5. `05-repo-local-serial-integration-and-green-rails`

## Repo Boundary

1. `quanta-gqlang-v2` only
2. no code or docs changes in `semantica-codegraph-v2`

## Current Base

1. local docs narrow the shipped story to a read-only shell
2. original RFC tickets still name this repo as owner for `E05E/E06C/E11C/E12C`
3. current local work is a recovery packet, not a new feature plan

## Ticket Anchors

1. [E05E](../../../../../semantica-codegraph-v2/docs/rfcs/2026-06-05-rfc-qglang-v2-semantic-host-dx-platform/tickets-v2/external/E05E-ai-exploration-profile-working-set-and-reconfirmation-semantics.md)
2. [E06C](../../../../../semantica-codegraph-v2/docs/rfcs/2026-06-05-rfc-qglang-v2-semantic-host-dx-platform/tickets-v2/external/E06C-public-planning-and-lowering-ready-ir.md)
3. [E11C](../../../../../semantica-codegraph-v2/docs/rfcs/2026-06-05-rfc-qglang-v2-semantic-host-dx-platform/tickets-v2/external/E11C-metric-inventory-and-external-findings-surface.md)
4. [E12C](../../../../../semantica-codegraph-v2/docs/rfcs/2026-06-05-rfc-qglang-v2-semantic-host-dx-platform/tickets-v2/external/E12C-explain-and-path-witness-ux.md)

## Owner Files

1. this document
2. [README.md](./README.md)
3. [WORKER_START_HERE.md](./WORKER_START_HERE.md)

## Verification Commands

1. `git diff --check -- docs/plans/jun-9-repo-startup`

## Not Done If

1. the packet mixes repo-local recovery with cross-repo closure
2. the packet treats cleanup tickets as prerequisites for local ticket recovery
3. the packet allows Wave 1 tickets to edit the shared serial glue files
