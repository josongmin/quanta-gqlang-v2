# P9 — tools / Python rails: final supplement record

Scope: `tools/test_frontdoor_parity.py`, `tools/test_frontend_external_consumer.py`,
`tools/architecture/tests/test_architecture.py`, `tools/ci/tests/test_gates.py`,
`tools/lint/tests/test_semantic_honesty.py`, `tools/lint/tests/test_semgrep_non_vacuity.py`,
`tools/prompt-manager/tests/test_pm.py`, `tools/qualification/tests/*.py`.

This is the third pass: the first deep audit (8 parallel read-only subagents + AST scans +
mutation probes) is committed, and the triage assignments TA-03/TA-04 are already repaired in
the base commit. This record covers the **final supplement round** on the current tree and
closes the remaining mechanically-safe held items. `just ci` / `just qualify-local` were not run.

## Supplement round (3 changes, all oracle-preserving)

| # | file | gap closed | verification |
|---|---|---|---|
| S-1 | `tools/qualification/tests/test_frontdoor_parity.py` | The e2e parity gate can only reach `blocked`/error item envelopes, so `complete` and `not_applicable` schema branches were never parsed. New unit case feeds a synthetic success envelope with `item[0].status=complete` (+ `kind` + full `plan.*`/`cost.*`) and `item[1].status=not_applicable` (+ `no_plan_reason`), then requires `CLI output schema drift` for a missing `kind`, missing `no_plan_reason`, and a **partial** plan block. A complete item with no plan block is asserted accepted (matches `v3_plan.rs: if let Some(plan)`). | `test_frontdoor_parity.py` 7/7 OK (was 6). |
| S-2 | `tools/qualification/tests/test_qualify_local.py` | `source_hashes == {}` was a vacuous oracle because the fixture repo had no hashed file, so a constant `{}` implementation passed. The fixture now commits a `Cargo.lock` and the receipt must equal the exact `{"Cargo.lock": sha256(...)}` mapping. | `test_qualify_local.py` 17/17 OK; a monkeypatched constant `source_hashes` now **fails** the test. |
| S-3 | `tools/qualification/tests/test_external_consumer_cache.py` + `tools/test_frontend_external_consumer.py` | (a) `offline_dependency_preflight`'s guard chain only exercised the `git status` branch; added `Cargo.lock`-hash-drift and fingerprint-drift cases. (b) `repo_toolchain_channel()` returned `None` on a missing/unparseable `rust-toolchain.toml`, silently unpinning the fixture toolchain despite the module docstring; it now raises `InfrastructureError`, with a unit test for missing file, missing channel, and the accepted pinned case. | `test_external_consumer_cache.py` 22/22 OK; external-consumer gate still `contract_violations=0 case_mismatches=0`. |

## Standing evidence (unchanged this round)

- Earlier committed fixes: mutation-applied guards in `test_semantic_honesty.py`; non-vacuity
  counters and the unmanaged-rule negative case in `test_pm.py`; inventory pinning, `compiled`
  positive control, span-anchor case, and fixture-lock hash case in
  `test_external_consumer_cache.py`; stuck-child reaping and the `ps` oracle in
  `test_process_guard.py`; expanded nextest/deny evidence matrices, `tool_versions` value check,
  and spawn-failure receipt key-set in `test_qualify_local.py`; doctest validator assertion,
  `[profile.default]` scoping, `python` lock check, composition exclusivity, atomic consume, and
  leaked positive leg in `test_gates.py`; rule-unique marker, registry-source isolation, and
  human-format failure exit in `test_architecture.py`; empty-workspace guard and
  `negative_executed == negative_selected` in the external-consumer gate.
- TA-03: `ps` child probe accepts only a zombie (`exit 0`, state `Z`) or a verified absent process
  (`exit != 0`, empty stdout/stderr, `os.kill(pid, 0)` raises `ProcessLookupError`).
- TA-04: `smoke_changed_source_rebuilds_at_the_same_manifest_path` stays out of `test_*.py`
  discovery; routine selection is 54 without it, and the documented explicit command passes.

## Remaining held items (owner/integrator decision, not mechanically safe)

- `test_architecture.py` valid-DAG fixtures derive their expected edge counts from the checker's
  own `FINAL_*` constants (self-referential); pinning literals is a policy-ratchet decision.
- `test_architecture.py` valid fixture declares no external dependency / internal feature, so
  allowlist over-strictness has no positive control.
- `test_external_consumer_cache.py` now pins compiler-mutation JSON error codes
  (`E0599`, `E0308`, `E0599`, `E0004`) as well as non-zero exit. The synthetic probe
  remains separate from the shipped `src/main.rs` semantic program.
- `test_frontend_external_consumer.py` source-shape regexes ignore non-canonical `pub use`/`pub mod`
  forms, the positive program's imports are not bound to `PINNED_ALLOWLIST`, and the immutability
  fingerprint omits facade oracle inputs (misattribution risk for concurrent editors).
- `test_frontdoor_parity.py` still derives the expected item key set from the payload's own
  `item_count`; a zero-item success envelope is contract-legal, so this is not a schema fail-open
  and value-level `source_id`/`unresolved_count` checks were left out deliberately.

## Final results (current tree)

| Rail | Result |
|---|---|
| `tools/qualification/tests test_*.py` | 54/54 OK (frontdoor 7 + process_guard 8 + cache 22 + qualify_local 17) |
| `tools/test_frontdoor_parity.py` | PASS (cases=10) |
| `tools/test_frontend_external_consumer.py` | `contract_violations=0 case_mismatches=0`; positives 1/1, negatives 15/15, public API 102/102, `duplicates=0` |
| lint / prompt-manager / ci / architecture rails | green on the committed base (26 / 7 / 33 / 11) |

The external-consumer run again reported `exit 2 INFRASTRUCTURE` solely through its
concurrent-writer path: other auditors edited many `crates/**` files during the run (see the
gate's own `concurrent writer delta`). Twelve temporary `zz_*`/`P*_*.md` scratch paths from
concurrent workers were also present; they are outside this owner's scope.
