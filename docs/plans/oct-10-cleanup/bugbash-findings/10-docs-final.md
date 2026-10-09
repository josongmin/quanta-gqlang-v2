# Final documentation and evidence readback

- QGLang source HEAD `25fad76cf5b4510244157c2e009b4dc37aa958ff`, tree `99501ba1aabfa37361fccfb7dee388cd91055ee1`. The shared checkout contains another writer's uncommitted documentation consolidation. This readback did not modify it.
- The committed-HEAD defects [DOC-01](DOC-01-qg-diag-routing.md) and [DOC-02](DOC-02-ticket-denominator.md) remain valid for that HEAD. The current working-tree overlay redirects QG-DIAG to accepted ADR-0008 and labels the unfinished set as 18. These are **uncommitted candidate corrections**, not a new committed-source verdict.

## Current overlay checks

| Check | Executed result | Limit |
| --- | --- | --- |
| Local Markdown file and heading links under `docs/` | Final readback: 47 Markdown files, 184 local links; 0 unresolved paths or headings using a GitHub-style heading slug approximation | This is a syntactic local-path check, not semantic correctness or a GitHub render check. External URLs and sibling-repo source identity were not verified. |
| Unfinished ticket denominator | `docs/work/remaining.md` has 18 distinct active ID rows, no duplicate ID; `docs/README.md` says 18 | Acceptance state of each downstream ID was not independently executed in this documentation lane. |
| Retired plan recovery | All 11 deleted plan paths are readable from `25fad76` via `git cat-file -e`; ADR-0001 documents an exact `git show` recovery command | Recoverability does not prove the shortened replacements preserve every operational detail inline. |
| Weekly inventory denominator | `git rev-list --first-parent` returned 216 commits; per-commit `git diff-tree --no-commit-id --name-only -r` union returned 340 paths | A plain `git log --name-only` union returned 339 because it omits a merge-path contribution; use the documented first-parent/diff-tree method for this inventory. |
| Dirty tracked diff hygiene | `git diff --check` exit 0 | New untracked files are outside `git diff --check`. |

No new confirmed documentation defect was established in this pass. The proposed ADR-0009 and rewrites remain another writer's uncommitted work. Re-run the link, 18-ID and source/receipt checks after that writer integrates them and the resulting SHA is known. Neither the existing clean-HEAD local CI receipt nor this static readback qualifies the dirty overlay, hosted CI, or downstream product behavior.
