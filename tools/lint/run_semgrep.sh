#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/../.." && pwd)"
config="${QGLANG_SEMGREP_CONFIG:-$repo_root/.semgrep/qglang-frontend.yml}"
fixtures="$repo_root/tools/lint/tests/fixtures/semgrep"

exec python3 - "$repo_root" "$config" "$fixtures" <<'PY'
from __future__ import annotations

import json
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

import yaml


repo_root = Path(sys.argv[1]).resolve()
config = Path(sys.argv[2]).resolve()
fixtures = Path(sys.argv[3]).resolve()

if shutil.which("semgrep"):
    semgrep = ["semgrep"]
else:
    probe = subprocess.run(
        [sys.executable, "-m", "semgrep", "--version"],
        capture_output=True,
        text=True,
    )
    if probe.returncode != 0:
        print("semgrep is required but not installed", file=sys.stderr)
        raise SystemExit(1)
    semgrep = [sys.executable, "-m", "semgrep"]

loaded = yaml.safe_load(config.read_text(encoding="utf-8"))
configured = [rule["id"] for rule in loaded.get("rules", [])]
if len(configured) != len(set(configured)):
    print("semgrep gate failed: duplicate configured rule IDs", file=sys.stderr)
    raise SystemExit(1)
if not configured:
    print("semgrep gate failed: zero configured rules", file=sys.stderr)
    raise SystemExit(1)

common = ["--config", str(config), "--json", "--metrics=off", "--disable-version-check"]
production_targets = 0
if os.environ.get("QGLANG_SEMGREP_SKIP_PRODUCTION") != "1":
    production = subprocess.run(
        semgrep
        + common
        + [
            "--error",
            "--exclude",
            "tools/lint/tests/fixtures/semgrep/**",
            ".",
        ],
        cwd=repo_root,
        capture_output=True,
        text=True,
    )
    if production.returncode != 0:
        sys.stderr.write(production.stderr)
        sys.stdout.write(production.stdout)
        raise SystemExit(production.returncode)
    production_payload = json.loads(production.stdout)
    production_targets = len(production_payload.get("paths", {}).get("scanned", []))
    if production_targets == 0:
        print("semgrep gate failed: zero production targets", file=sys.stderr)
        raise SystemExit(1)

fixture_destinations = {
    "qglang-v2-no-runtime-sdk-index-imports": "crates/quanta-gqlang-v2-types/src/fixture.rs",
    "qglang-v2-no-legacy-gqlang-imports": "crates/quanta-gqlang-v2-types/src/fixture.rs",
    "qglang-v2-no-host-coupling-tokens": "crates/quanta-gqlang-v2-types/src/fixture.rs",
    "qglang-v2-no-frontend-hub-bypass": "crates/quanta-gqlang-v2-cli/src/fixture.rs",
    "qglang-v2-no-runtime-truth-tokens": "crates/quanta-gqlang-v2-types/src/fixture.rs",
    "qglang-v2-no-silent-err-discard": "crates/quanta-gqlang-v2-types/src/fixture.rs",
    "qglang-v2-no-string-typed-error-results-in-core": "crates/quanta-gqlang-v2-hir/src/fixture.rs",
}
expected_match_counts = {rule_id: 1 for rule_id in configured}
expected_match_counts["qglang-v2-no-runtime-truth-tokens"] = 27

if set(configured) != set(fixture_destinations):
    missing = sorted(set(configured) - set(fixture_destinations))
    stale = sorted(set(fixture_destinations) - set(configured))
    print(
        f"semgrep gate failed: fixture inventory drift; missing={missing}, stale={stale}",
        file=sys.stderr,
    )
    raise SystemExit(1)

executed: set[str] = set()
with tempfile.TemporaryDirectory(prefix="qglang-semgrep-") as temporary:
    temporary_root = Path(temporary)
    positive_root = temporary_root / "positive"
    for rule_id in configured:
        destination = positive_root / fixture_destinations[rule_id]
        destination = destination.with_name(f"{rule_id}.rs")
        destination.parent.mkdir(parents=True, exist_ok=True)
        fixture = fixtures / f"{rule_id}.rs"
        if not fixture.is_file():
            print(f"semgrep gate failed: missing positive fixture {fixture}", file=sys.stderr)
            raise SystemExit(1)
        shutil.copyfile(fixture, destination)

    positive = subprocess.run(
        semgrep + common + ["."],
        cwd=positive_root,
        capture_output=True,
        text=True,
    )
    if positive.returncode != 0:
        sys.stderr.write(positive.stderr)
        raise SystemExit(positive.returncode)
    positive_payload = json.loads(positive.stdout)
    matches_by_fixture = {rule_id: set() for rule_id in configured}
    match_counts_by_fixture = {rule_id: 0 for rule_id in configured}
    for entry in positive_payload.get("results", []):
        fixture_id = Path(entry["path"]).stem
        raw_check_id = entry["check_id"]
        normalized = [rule_id for rule_id in configured if raw_check_id.endswith(rule_id)]
        if fixture_id not in matches_by_fixture or len(normalized) != 1:
            print(
                f"semgrep gate failed: unexpected positive result path={entry['path']} "
                f"check_id={raw_check_id}",
                file=sys.stderr,
            )
            raise SystemExit(1)
        matches_by_fixture[fixture_id].add(normalized[0])
        match_counts_by_fixture[fixture_id] += 1
    for rule_id, matched in matches_by_fixture.items():
        if matched != {rule_id}:
            print(
                f"semgrep gate failed: fixture {rule_id} matched {sorted(matched)}",
                file=sys.stderr,
            )
            raise SystemExit(1)
        if match_counts_by_fixture[rule_id] != expected_match_counts[rule_id]:
            print(
                f"semgrep gate failed: fixture {rule_id} produced "
                f"{match_counts_by_fixture[rule_id]} matches; expected "
                f"{expected_match_counts[rule_id]}",
                file=sys.stderr,
            )
            raise SystemExit(1)
        executed.update(matched)

    clean_root = temporary_root / "clean-control"
    clean_file = clean_root / "crates/quanta-gqlang-v2-hir/src/clean.rs"
    clean_file.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(fixtures / "clean.rs", clean_file)
    clean = subprocess.run(
        semgrep + common + ["."],
        cwd=clean_root,
        capture_output=True,
        text=True,
    )
    if clean.returncode != 0:
        sys.stderr.write(clean.stderr)
        raise SystemExit(clean.returncode)
    clean_matches = json.loads(clean.stdout).get("results", [])
    if clean_matches:
        print("semgrep gate failed: clean control produced findings", file=sys.stderr)
        raise SystemExit(1)

unexecuted = sorted(set(configured) - executed)
if unexecuted:
    print(f"semgrep gate failed: unexecuted rules={unexecuted}", file=sys.stderr)
    raise SystemExit(1)

print(
    "semgrep gate passed: "
    f"configured={len(configured)} executed={len(executed)} "
    f"production_targets={production_targets}"
)
PY
