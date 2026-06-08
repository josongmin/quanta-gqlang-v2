# Repo Startup Worker Start Here

Status: `Done`

## Current Routing Snapshot

Current closure truth in this repo:

1. local public docs describe a read-only shell only
2. local public docs still allow catalog and AI-adjacent metadata surfaces, but
   not runtime truth claims
3. the original local RFC tickets for `E05E`, `E06C`, `E11C`, and `E12C`
   remain pointed at this repo
4. the named proof rails for those four tickets are present and green
5. the repo-local composition rails are:
   - `cargo clippy --workspace --tests -- -D warnings`
   - `just ci`

## Historical Note

1. the individual ticket `Current Base` sections are the startup baseline, not the post-closeout state

## Hard Routing Rules

1. edit only `quanta-gqlang-v2`
2. no runtime substrate, provider registry, or SDK backend wiring belongs here
3. no Semantica DTO or host-truth payload may leak into public frontend IR
4. restored AI and catalog surfaces remain typed frontend metadata, not backend
   availability claims
5. degraded reasons are presentation-only; do not invent runtime truth
6. do not touch the serial glue files from Wave 1 tickets
7. do not claim repo-local closure before `05` is green
8. do not claim packet closure from `06` or `07`; they are cleanup only

## Parallel Waves

### Wave 0

- `00-packet-scope-and-repo-boundary`

### Wave 1

- `01-e05e-ai-profile-working-set-and-reconfirmation-recovery`
- `02-e06c-public-planning-and-lowering-ready-ir-recovery`
- `03-e11c-metric-inventory-and-external-findings-recovery`
- `04-e12c-explain-path-witness-and-why-no-result-ux-recovery`

### Wave 2

- `05-repo-local-serial-integration-and-green-rails`

### Wave 3

- `06-lsp-read-only-interpreter-extraction`
- `07-ir-facade-and-lint-test-root-cleanup`

## Common Failure Modes

1. keeping the current narrowed base and pretending the original ticket scope no
   longer exists
2. restoring symbols but not restoring the named proof rails
3. editing `lib.rs`, `server.rs`, `doctor.rs`, `main.rs`, or `Cargo.toml` from
   the parallel Wave 1 tickets
4. reintroducing duplicate degraded-reason strings instead of using the shared
   enum authority
5. letting cleanup tickets widen into new product claims or backend wiring
