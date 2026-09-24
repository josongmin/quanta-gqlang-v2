#!/usr/bin/env python3
"""Identify the active cargo-deny RustSec database and bound fetch age."""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import time
from pathlib import Path

MAX_FETCH_AGE_S = 7 * 24 * 60 * 60


class AdvisoryError(RuntimeError):
    pass


def state(cargo_home: Path, now: float = None) -> dict:
    root = cargo_home / "advisory-dbs"
    databases = sorted(root.glob("advisory-db-*"))
    if len(databases) != 1 or databases[0].is_symlink() or not databases[0].is_dir():
        raise AdvisoryError("expected exactly one owned cargo-deny advisory DB in %s" % root)
    database = databases[0]
    fetched = database / ".git/FETCH_HEAD"
    if not fetched.is_file() or fetched.is_symlink():
        raise AdvisoryError("advisory DB fetch evidence is missing")
    now = time.time() if now is None else now
    age = now - fetched.stat().st_mtime
    if age < -300 or age > MAX_FETCH_AGE_S:
        raise AdvisoryError("advisory DB fetch is stale or future-dated (age_s=%d)" % age)
    result = subprocess.run(
        ("git", "-C", str(database), "rev-parse", "HEAD"),
        capture_output=True, text=True, check=False, timeout=30,
    )
    revision = result.stdout.strip()
    if result.returncode or len(revision) != 40:
        raise AdvisoryError("cannot identify advisory DB revision")
    return {
        "path": str(database), "revision": revision,
        "fetched_at_unix": int(fetched.stat().st_mtime), "fetch_age_s": int(age),
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()
    cargo_home = Path(os.environ.get("CARGO_HOME", str(Path.home() / ".cargo")))
    try:
        observed = state(cargo_home)
    except (AdvisoryError, OSError, subprocess.SubprocessError) as error:
        print("advisory DB: " + str(error), file=sys.stderr)
        return 2
    if args.json:
        print(json.dumps(observed, sort_keys=True))
    else:
        print("advisory DB: revision=%s fetch_age_s=%d" % (
            observed["revision"], observed["fetch_age_s"]
        ))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
