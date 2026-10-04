#!/usr/bin/env python3
"""Bounded, local-only surface fuzz run with retained corpus and quiet logs."""

from __future__ import annotations

import argparse
import os
import shutil
import signal
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
STATE = ROOT / "target/fuzz-smoke"
NIGHTLY = "nightly-2026-09-23"
CARGO_FUZZ = "cargo-fuzz 0.13.1"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--seconds", type=int, default=60)
    args = parser.parse_args()
    if not 1 <= args.seconds <= 300:
        parser.error("seconds must be between 1 and 300")

    observed = subprocess.run(
        ["cargo", f"+{NIGHTLY}", "fuzz", "--version"], cwd=ROOT,
        capture_output=True, text=True, check=False, timeout=30,
    )
    if observed.returncode or observed.stdout.strip() != CARGO_FUZZ:
        print(f"fuzz smoke: expected {CARGO_FUZZ} on {NIGHTLY}; got {observed.stdout.strip()!r}",
              file=sys.stderr)
        return 2

    corpus = STATE / "corpus/surface_document"
    artifacts = STATE / "artifacts/surface_document"
    corpus.mkdir(parents=True, exist_ok=True)
    artifacts.mkdir(parents=True, exist_ok=True)
    tracked = subprocess.check_output(
        ["git", "ls-files", "-z", "--", "fuzz/corpus/surface_document"], cwd=ROOT,
        timeout=30,
    ).split(b"\0")
    seeds = [os.fsdecode(item) for item in tracked if item]
    if not seeds:
        print("fuzz smoke: no tracked seed corpus", file=sys.stderr)
        return 2
    for relative in seeds:
        source = ROOT / relative
        if not source.is_file() or source.is_symlink():
            print(f"fuzz smoke: invalid seed {relative}", file=sys.stderr)
            return 2
        target = corpus / source.name
        shutil.copyfile(source, target)

    log = STATE / "last.log"
    command = [
        "cargo", f"+{NIGHTLY}", "fuzz", "run", "surface_document", str(corpus), "--",
        f"-max_total_time={args.seconds}", "-timeout=5", "-verbosity=0",
        f"-artifact_prefix={artifacts}/",
    ]
    env = os.environ.copy()
    env["CARGO_NET_OFFLINE"] = "true"
    with log.open("w", encoding="utf-8") as output:
        process = subprocess.Popen(
            command, cwd=ROOT, env=env, stdout=output, stderr=subprocess.STDOUT,
            start_new_session=True,
        )
        try:
            code = process.wait(timeout=args.seconds + 120)
        except subprocess.TimeoutExpired:
            os.killpg(process.pid, signal.SIGTERM)
            try:
                process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                os.killpg(process.pid, signal.SIGKILL)
                process.wait(timeout=10)
            print(f"fuzz smoke: timed out; log: {log}", file=sys.stderr)
            return 2
    if code:
        for line in log.read_text(encoding="utf-8", errors="replace").splitlines()[-20:]:
            print(line, file=sys.stderr)
    print(f"fuzz smoke: exit {code}; corpus files: {len(list(corpus.iterdir()))}; "
          f"log: {log}; artifacts: {artifacts}")
    return code


if __name__ == "__main__":
    raise SystemExit(main())
