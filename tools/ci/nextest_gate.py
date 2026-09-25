#!/usr/bin/env python3
"""Reconcile the generated nextest inventory with terminal test events."""

from __future__ import annotations

import hashlib
import json
import re
import sys
import time
from pathlib import Path
from typing import Dict, Optional, Set, Tuple

sys.path.insert(0, str(Path(__file__).resolve().parents[2]))
from tools.process_guard import ProcessTimeoutError, run_bounded  # noqa: E402
from tools.ci.evidence import EvidencePathError, write_json_evidence  # noqa: E402
from tools.ci.doctor import DoctorError, read_lock  # noqa: E402
from tools.ci.environment import (  # noqa: E402
    EnvironmentSanitizationError, sanitized_environment,
)

ROOT = Path(__file__).resolve().parents[2]
EVIDENCE = ROOT / "target/ci/nextest-evidence.json"
BASE = ("cargo", "nextest")
TERMINAL = {"ok", "failed", "timeout"}
TEST_THREADS = 4
SELECTION_ARGS = ("--profile", "default", "--ignore-default-filter")


class NextestEvidenceError(RuntimeError):
    pass


def inventory(document: dict) -> Tuple[Set[str], Set[str]]:
    suites = document.get("rust-suites")
    if not isinstance(suites, dict) or type(document.get("test-count")) is not int:
        raise NextestEvidenceError("invalid nextest list schema")
    selected: Set[str] = set()
    skipped: Set[str] = set()
    total = 0
    for suite in suites.values():
        package, binary, cases = (
            suite.get("package-name"), suite.get("binary-name"), suite.get("testcases")
        )
        if not isinstance(package, str) or not isinstance(binary, str) or not isinstance(cases, dict):
            raise NextestEvidenceError("invalid nextest suite schema")
        for case, detail in cases.items():
            name = "%s::%s$%s" % (package, binary, case)
            if name in selected or name in skipped:
                raise NextestEvidenceError("duplicate nextest inventory item: " + name)
            match = detail.get("filter-match") if isinstance(detail, dict) else None
            status = match.get("status") if isinstance(match, dict) else None
            if status == "matches":
                selected.add(name)
            elif status == "mismatch" and match.get("reason") == "ignored":
                skipped.add(name)
            else:
                raise NextestEvidenceError("unexpected nextest filter status for " + name)
            total += 1
    if total != document["test-count"] or not selected:
        raise NextestEvidenceError("nextest inventory is empty or count is inconsistent")
    return selected, skipped


def terminal_events(output: str) -> Tuple[Set[str], Dict[str, str], Set[str]]:
    completed: Set[str] = set()
    unsuccessful: Dict[str, str] = {}
    leaked: Set[str] = set()
    for line in output.splitlines():
        try:
            event = json.loads(line)
        except (ValueError, TypeError) as error:
            raise NextestEvidenceError("non-JSON nextest event") from error
        if event.get("type") != "test":
            continue
        status, name = event.get("event"), event.get("name")
        if status not in TERMINAL:
            continue
        if not isinstance(name, str) or name in completed:
            raise NextestEvidenceError("unnamed or duplicate terminal nextest event")
        completed.add(name)
        if status != "ok":
            unsuccessful[name] = status
        nextest = event.get("nextest") or {}
        if isinstance(nextest, dict) and (
            nextest.get("leaked") is True or nextest.get("status") == "leaky"
        ):
            leaked.add(name)
    return completed, unsuccessful, leaked


def summary_counts(stderr: str) -> Tuple[int, int, int, int]:
    summaries = [line for line in stderr.splitlines() if "Summary [" in line]
    if len(summaries) != 1:
        raise NextestEvidenceError("missing or duplicate nextest run summary")
    run = re.search(r"\b(\d+) tests? run:", summaries[0])
    passed = re.search(r"\b(\d+) passed\b", summaries[0])
    if run is None or passed is None:
        raise NextestEvidenceError("invalid nextest run summary")
    failed = re.search(r"\b(\d+) failed\b", summaries[0])
    leaky = re.search(r"\((\d+) leaky\)", summaries[0])
    return (
        int(run.group(1)), int(passed.group(1)),
        int(failed.group(1)) if failed else 0,
        int(leaky.group(1)) if leaky else 0,
    )


def reported_leaks(stderr: str) -> Set[str]:
    leaks = set()
    pattern = re.compile(r"^\s*LEAK(?:-FAIL)?\s+\[[^]]+\]\s+(\S+)\s+(.+?)\s*$")
    for line in stderr.splitlines():
        match = pattern.match(line)
        if match:
            binary, test = match.groups()
            leaks.add(binary + "$" + test)
    return leaks


def reconcile(
    selected: Set[str], completed: Set[str], leaked: int,
    leaked_tests: Optional[Set[str]] = None,
    unsuccessful: Optional[Dict[str, str]] = None,
    skipped_ignored: Optional[Set[str]] = None,
) -> None:
    missing, extra = selected - completed, completed - selected
    if missing or extra or leaked or unsuccessful or skipped_ignored:
        leaked_names = ",".join(sorted(leaked_tests or set())) or "unknown"
        ignored_names = ",".join(sorted(skipped_ignored or set())) or "none"
        unsuccessful_names = ",".join(
            "%s=%s" % item for item in sorted((unsuccessful or {}).items())
        ) or "none"
        raise NextestEvidenceError(
            "terminal events differ from inventory: missing=%d extra=%d leaked=%d "
            "names=%s ignored=%d ignored_names=%s unsuccessful=%s" % (
                len(missing), len(extra), leaked, leaked_names,
                len(skipped_ignored or set()), ignored_names, unsuccessful_names,
            )
        )


def write_evidence(payload: dict) -> None:
    write_json_evidence(EVIDENCE, payload, owned_root=ROOT / "target", prefix="nextest")


def main() -> int:
    try:
        env = sanitized_environment(preserve_qualification_run_id=True)
        env["CARGO_TARGET_DIR"] = str(ROOT / "target/ci-msrv")
        env["NEXTEST_EXPERIMENTAL_LIBTEST_JSON"] = "1"
        env["RUSTUP_TOOLCHAIN"] = read_lock()["rust"]
        list_started = time.monotonic()
        listed = run_bounded(
            (*BASE, "list", "--workspace", "--locked", "--offline", *SELECTION_ARGS,
             "--message-format", "json"),
            cwd=ROOT, env=env, capture_output=True, text=True, timeout_s=600,
        )
        list_seconds = time.monotonic() - list_started
        sys.stderr.write(listed.stderr or "")
        if listed.returncode:
            return listed.returncode
        selected, skipped = inventory(json.loads(listed.stdout or ""))
        run_started = time.monotonic()
        result = run_bounded(
            (*BASE, "run", "--workspace", "--locked", "--offline", *SELECTION_ARGS,
             "--retries", "0", "--no-tests", "fail",
             "--message-format", "libtest-json-plus", "--color", "never",
             "--status-level", "leak", "--test-threads", str(TEST_THREADS)),
            cwd=ROOT, env=env, capture_output=True, text=True, timeout_s=600,
        )
        run_seconds = time.monotonic() - run_started
        sys.stderr.write(result.stderr or "")
        parse_started = time.monotonic()
        completed, unsuccessful, leaked = terminal_events(result.stdout or "")
        leaked.update(reported_leaks(result.stderr or ""))
        summary_run, summary_passed, summary_failed, summary_leaked = summary_counts(
            result.stderr or ""
        )
        if (
            summary_run != len(selected)
            or summary_run != len(completed)
            or summary_passed != len(completed) - len(unsuccessful)
            or summary_failed != len(unsuccessful)
        ):
            raise NextestEvidenceError(
                "nextest summary differs from terminal events: "
                "run=%d passed=%d failed=%d selected=%d completed=%d unsuccessful=%d" % (
                    summary_run, summary_passed, summary_failed, len(selected),
                    len(completed), len(unsuccessful),
                )
            )
        leak_count = max(len(leaked), summary_leaked)
        parse_seconds = time.monotonic() - parse_started
        evidence = {
            "schema": 1,
            "qualification_run_id": env.get("QGLANG_QUALIFICATION_RUN_ID"),
            "discovered": len(selected) + len(skipped),
            "selected": len(selected),
            "executed": len(completed),
            "skipped_ignored": len(skipped),
            "leaked": leak_count,
            "leaked_tests": sorted(leaked),
            "unsuccessful": dict(sorted(unsuccessful.items())),
            "test_threads": TEST_THREADS,
            "list_duration_s": round(list_seconds, 3),
            "run_duration_s": round(run_seconds, 3),
            "parse_duration_s": round(parse_seconds, 3),
            "summary_run": summary_run,
            "summary_passed": summary_passed,
            "summary_failed": summary_failed,
            "inventory_sha256": hashlib.sha256((listed.stdout or "").encode()).hexdigest(),
            "events_sha256": hashlib.sha256((result.stdout or "").encode()).hexdigest(),
            "exit": result.returncode,
        }
        write_evidence(evidence)
        reconcile(
            selected, completed, leak_count, leaked, unsuccessful,
            skipped_ignored=skipped,
        )
        if result.returncode:
            return result.returncode
        print("nextest gate: discovered=%d executed=%d skipped=%d leaked=0 list=%.3fs run=%.3fs parse=%.3fs" % (
            evidence["discovered"], evidence["executed"], evidence["skipped_ignored"],
            list_seconds, run_seconds, parse_seconds,
        ))
        return 0
    except (
        DoctorError, EnvironmentSanitizationError, EvidencePathError,
        NextestEvidenceError, ProcessTimeoutError,
        OSError, ValueError,
    ) as error:
        print("nextest gate: " + str(error), file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
