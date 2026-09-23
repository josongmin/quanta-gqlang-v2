# prompt-manager

This tool keeps persistent agent context small, scoped, and reproducible.

Loading model:

- `AGENTS.md`: shared always-on repository router.
- `CLAUDE.md`: compact Claude bootstrap importing `@AGENTS.md` once.
- `.cursor/rules/*.mdc`: Cursor rules attached by file glob.
- `.claude/rules/*.md`: Claude rules attached by `paths` frontmatter.
- `docs/` and detailed contracts: on demand through the routes in `AGENTS.md`.

Edit only `sources/`, `targets.json`, or the manager itself. The root instruction files and scoped rules are generated outputs.

Commands:

```bash
just pm-sync
just pm-check
just pm-status
python3 tools/prompt-manager/pm.py ratchet --update
```

`sync` renders all outputs in memory, validates budgets and the shrink-only ratchet, then writes atomically. `check` fails on drift, retired files, invalid scoping metadata, missing imports, or budget growth. `ratchet --update` can lower existing ceilings; it never raises one.

Reference basis:

- [OpenAI prompt and skill guidance](https://developers.openai.com/blog/rethinking-skills-and-prompts-for-gpt-6-astra): keep `AGENTS.md` current, task-relevant, and free of mandatory full-document preloads.
- [Cursor Project Rules](https://cursor.com/docs/rules): use focused `.cursor/rules/*.mdc` files with explicit attachment metadata.
- [Claude Code memory](https://code.claude.com/docs/en/memory): keep startup instructions concise; imports still consume startup context, while path-scoped rules load conditionally.
