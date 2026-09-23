# Claude Code instructions

@AGENTS.md

- `AGENTS.md` is the shared startup contract. Do not import README files, language references, plans, or receipts here.
- Use the context routes in `AGENTS.md`; imports expand into startup context, so task-specific material stays on demand.
- Path-specific Rust and documentation rules live under `.claude/rules/` and load only for matching files.
- Generated instruction files are managed by `tools/prompt-manager/`; edit its sources and run `just pm-sync`.
