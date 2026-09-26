# prompt-manager

This tool keeps persistent agent context small, scoped, and reproducible.

Loading model:

- `AGENTS.md`: shared always-on repository router.
- `CLAUDE.md`: compact Claude bootstrap importing `@AGENTS.md` once.
- `.cursor/rules/*.mdc`: Cursor rules attached by file glob.
- `.claude/rules/*.md`: Claude rules attached by `paths` frontmatter.
- `docs/` and detailed contracts: on demand through the routes in `AGENTS.md`.

Codex receives the shared engineering criteria in `AGENTS.md` and is routed to the Rust source rule for Rust changes. Cursor and Claude attach their generated Rust rules by file scope. Do not assume that those client rule directories load in Codex. This accounts for repository-managed context only; user/global rules, ancestor instructions, memory, tools, and conversation context are outside these size totals.

Edit only `sources/`, `targets.json`, or the manager itself. The root instruction files and scoped rules are generated outputs.

Commands:

```bash
just pm-sync
just pm-check
just pm-status
python3 tools/prompt-manager/pm.py ratchet --update
```

`sync` renders all outputs in memory, validates budgets and the shrink-only ratchet, then atomically replaces each changed file. `check` fails on drift, retired files, invalid scoping metadata, missing or wrong-case literal scope paths, invalid imports, budget growth, changed startup-group members, or moved target outputs. Literal scope paths are checked against directory entry names even on case-insensitive filesystems; wildcard matching remains client-owned. Imports resolve relative to the importing output and must name managed targets within every context group containing the importer. Unbudgeted imports, importers without a context group, and cycles are rejected. Markdown code examples do not expand imports. The ratchet records output identities, group members, and sizes. `ratchet --update` can lower existing ceilings; it never raises one or silently changes an identity.

`required_imports` declares the exact ordered target imports for a bootstrap. Claude must import `agents` once; omission, duplicate imports, and a code-quoted import fail `check`. Adding useful instructions may require an explicit reviewed edit of `context-size-ratchet.json` in the same change as the sources and generated outputs. Record the reason and resulting sizes; keep `targets.json` hard limits unchanged unless separately justified. Do not raise ceilings to hide drift or a failing check.

## Engineering criteria

The shared prompt requires consumer and owner tracing, contract invariants, typed failure behavior, compatibility review, deterministic outputs, bounded resource use on affected hot paths, and discriminating verification. Rust rules add transaction-local identity and producer/consumer provenance constraints from the frontend contract. These are decision criteria, not a requirement to build a new framework, run every test, or benchmark every edit.

The September 26, 2026 baseline expansion adds these engineering criteria and the explicit Codex Rust route. It intentionally updates the size ratchet alongside the sources and generated files while retaining the original hard limits: 4,000 bytes for shared startup, 5,500 for Claude startup, and 2,000 per Rust rule. It does not represent a measured improvement in agent behavior.

Apply the criteria to the changed behavior. For a transformation bug, demonstrate the expected semantics independently of the implementation. For an encoding change, review the byte contract and schema compatibility. For a performance change, compare the same workload and environment across recorded source revisions. A stronger engineering prompt establishes expectations; a claim of SOTA performance still needs task-specific comparative evidence.

## Verification boundaries

Run `just pm-test` and `just pm-check` after manager or target changes. These commands verify generation, scope metadata, literal paths, import accounting, and size limits. They do not launch Cursor or Claude Code, measure model adherence, or establish agent performance.

For behavior evaluation, replay these tasks in fresh client sessions:

| Task | Expected trace and result |
| --- | --- |
| Make a narrow Rust change | Preserve frontend ownership; select the affected owner rail; load relevant context. |
| Change a public frontend contract | Consult `docs/contracts/frontend-execution-v3.md`; preserve the host-agnostic public seam. |
| Change repository instructions | Edit manager sources or targets, sync generated outputs, and check drift. |
| Audit test coverage | Consult the hellgate ledger; distinguish focused proof from repository qualification. |
| Add runtime/provider wiring | Identify downstream ownership; avoid placing runtime wiring in the frontend repository. |
| Analyze a query without required external authority | Preserve typed `Incomplete`; do not fabricate a closed plan or runtime capability. |
| Fix a transformation with self-derived test expectations | Supply an independent semantic expectation that rejects the old defect. |
| Optimize an affected hot path | Bound the relevant resource growth; substantiate improvement with a comparable workload and source revisions. |
| Work in a checkout with concurrent edits | Inspect relevant changes, preserve unrelated ownership, and bind proof to the actual tested source. |
| Report completion from an old receipt | Identify stale evidence and unresolved qualification; do not promote it to current proof. |

Record the client version, model identifier, source HEAD and dirty diff, prompt content digests, task text, tool trace, final diff, verification results, latency, and tokens when available. Compare prompt revisions on the same cases and source snapshot with repeated runs. Keep actual client attachment evidence separate from model behavior scores; static PASS alone cannot establish either. See [OpenAI evaluation best practices](https://developers.openai.com/api/docs/guides/evaluation-best-practices).

Reference basis:

- [OpenAI prompt and skill guidance](https://developers.openai.com/blog/rethinking-skills-and-prompts-for-gpt-6-astra): keep `AGENTS.md` current, task-relevant, and free of mandatory full-document preloads.
- [Cursor Project Rules](https://cursor.com/docs/rules): use focused `.cursor/rules/*.mdc` files with explicit attachment metadata.
- [Claude Code memory](https://code.claude.com/docs/en/memory): keep startup instructions concise; imports still consume startup context, while path-scoped rules load conditionally.
