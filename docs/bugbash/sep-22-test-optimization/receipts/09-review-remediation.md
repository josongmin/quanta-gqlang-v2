# Receipt 09 — Post-review remediation

This receipt qualifies the committed, clean review fix at
`2acae4a517638e995cfb306cd3bac0e2a2b4376a` (tree
`e95af89e8e2e7a2e162beefa569a749238df892c`). Its base was
`b1b979fe769e41c24f3b9a2bf91aa76bda51af2f`. `git status --porcelain=v1`
was empty after the fix commit and after the commands below. The commit changes
exactly nine paths: six code/test paths and Tickets 02, 04, and 05. This
receipt is a later documentation commit, not part of the tested tree.

## Historical scope correction

Receipts 00–06 describe a 12-leased-file patch on the old `16c97e82` base.
They do **not** describe the whole `d353c45a` commit. `git diff-tree
--no-commit-id --name-only -r d353c45` returns 46 paths: 12 leased
implementation paths, 14 packet ticket/receipt paths, and 20 other paths in
conformance, frontend-contract, pipeline, Sema production/tests, and types.
The 20 other paths were integrated in that mixed commit and require their own
owner evidence; none is qualified as packet-only by Receipts 00–06. Receipts
07–08 are historical observations at their named SHAs, not current-source
qualification.

## Review fixes

- Sema `test_support` and its three forgery seams now compile in unit tests
  as well as under the downstream `test` feature. The Ticket 02 standalone
  selector executes 33 tests, including
  `test_support::tests::every_positive_fixture_variant_resolves`; it selected
  zero before this fix. The feature-gated downstream fixture contract is kept.
- DOC-02 derives the leading `;` byte offset from that row's source and the
  token length. The seven document target tests pass.
- The CLI positive neighbor checks the actual analysis status on stdout and
  requires empty stderr. Ticket 04/05 commands now name the current Justfile
  `static`, `test`, and `ci` groups, including one parity path.

## Commands on clean `2acae4a`

| Command | Result |
|---|---|
| `cargo test -p quanta-gqlang-v2-sema --lib test_support` | exit 0; 33 passed, 17 filtered out |
| `cargo nextest list --workspace` | exit 0 |
| `cargo nextest run --workspace` | exit 0; 1,084 passed, 0 skipped, 89 binaries |
| `just test` | exit 0; nextest 1,084 passed, doctests 5 passed, parity 2 cases passed |
| `just static` | exit 0; fmt, taplo, both Clippy profiles, typos |
| `cargo check --workspace --all-targets` | exit 0 |
| `python3 tools/architecture/tests/test_architecture.py` | exit 0; 11 passed |
| `python3 tools/architecture/check_architecture.py` | exit 0 |
| `python3 tools/test_frontend_external_consumer.py` | exit 0; 1 positive and 14 negative cases |
| `typos docs/bugbash/sep-22-test-optimization` | exit 0 |
| `just --dry-run test` and `just --dry-run ci` | exit 0; parity appears once in each expansion |
| `git diff --check` and `git status --porcelain=v1` | exit 0; no output |

The workspace nextest count remains 1,084 because other workspace packages
already enabled Sema's `test` feature during feature unification. The fixed
gap was the standalone Ticket 02 command selecting zero tests. No mutation
result from Receipts 01–06 is relabeled as a fresh run here. Hosted CI,
`just policy`, `just ci`, review, deploy, and activation were not run or claimed.
