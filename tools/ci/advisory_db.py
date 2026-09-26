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
from typing import Mapping

MAX_FETCH_AGE_S = 7 * 24 * 60 * 60
OFFICIAL_ORIGINS = frozenset({
    "https://github.com/RustSec/advisory-db",
    "https://github.com/RustSec/advisory-db.git",
})
ADVISORY_IDENTITY_KEYS = ("path", "origin", "revision", "fetched_at_unix")


class AdvisoryError(RuntimeError):
    pass


def advisory_identity(observed: Mapping[str, object]) -> tuple:
    return tuple(observed.get(key) for key in ADVISORY_IDENTITY_KEYS)


def git(database: Path, *args: str) -> str:
    result = subprocess.run(
        ("git", "-C", str(database), *args),
        capture_output=True, text=True, check=False, timeout=30,
    )
    if result.returncode:
        raise AdvisoryError("cannot inspect advisory DB: " + result.stderr.strip())
    return result.stdout.strip()


def fetch_revision(text: str) -> str:
    candidates = []
    for line in text.splitlines():
        fields = line.split("\t")
        if len(fields) >= 2 and fields[1] != "not-for-merge":
            candidates.append(fields[0])
    if len(candidates) != 1 or not all(
        len(value) == 40 and all(character in "0123456789abcdef" for character in value)
        for value in candidates
    ):
        raise AdvisoryError("advisory DB FETCH_HEAD has no unique fetched revision")
    return candidates[0]


def state(cargo_home: Path, now: float = None) -> dict:
    root = cargo_home / "advisory-dbs"
    if root.is_symlink() or not root.is_dir():
        raise AdvisoryError("cargo-deny advisory DB root is missing or redirected")
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
    revision = git(database, "rev-parse", "HEAD")
    if len(revision) != 40 or any(character not in "0123456789abcdef" for character in revision):
        raise AdvisoryError("cannot identify advisory DB revision")
    try:
        revision_time = int(git(database, "show", "-s", "--format=%ct", "HEAD"))
    except ValueError as error:
        raise AdvisoryError("advisory DB revision has no valid commit time") from error
    revision_age = now - revision_time
    if revision_age < -300 or revision_age > MAX_FETCH_AGE_S:
        raise AdvisoryError("advisory DB revision is stale or future-dated (age_s=%d)" % revision_age)
    if git(database, "status", "--porcelain=v1", "--untracked-files=all"):
        raise AdvisoryError("advisory DB checkout is dirty")
    # Git status may silently skip paths marked assume-unchanged or
    # skip-worktree. Neither flag is valid for a trusted advisory checkout.
    index_flags = git(database, "ls-files", "-v")
    if any(line and line[0] != "H" for line in index_flags.splitlines()):
        raise AdvisoryError("advisory DB index hides tracked files")
    origin = git(database, "remote", "get-url", "origin")
    if origin not in OFFICIAL_ORIGINS:
        raise AdvisoryError("advisory DB origin is not the official RustSec repository")
    fetched_revision = fetch_revision(fetched.read_text(encoding="utf-8"))
    if fetched_revision != revision:
        raise AdvisoryError("advisory DB HEAD does not match the fetched revision")
    return {
        "path": str(database), "origin": origin, "revision": revision,
        "fetched_at_unix": int(fetched.stat().st_mtime), "fetch_age_s": int(age),
        "revision_age_s": int(revision_age),
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
