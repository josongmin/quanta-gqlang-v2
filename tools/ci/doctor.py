#!/usr/bin/env python3
"""Fail closed if a local qualification tool differs from the source lock."""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
from typing import Dict, Sequence

ROOT = Path(__file__).resolve().parents[2]
LOCK = ROOT / "tools/ci/toolchain.lock.toml"
RUST_TOOLCHAIN = ROOT / "rust-toolchain.toml"
KEYS = {
    "schema", "rust", "clippy", "public_api_nightly", "public_api_rustc",
    "just", "nextest", "taplo", "typos", "deny", "public_api", "semgrep", "python",
}


class DoctorError(RuntimeError):
    pass


def read_lock(path: Path = LOCK) -> Dict[str, str]:
    values: Dict[str, str] = {}
    for number, raw in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        match = re.fullmatch(r"([a-z_]+)\s*=\s*(\"[^\"\\]*\"|[0-9]+)", line)
        if not match:
            raise DoctorError("invalid tool lock line %d" % number)
        key, literal = match.groups()
        if key not in KEYS or key in values:
            raise DoctorError("unknown or duplicate tool lock key %s" % key)
        values[key] = json.loads(literal) if literal.startswith('"') else literal
    if set(values) != KEYS or values["schema"] != "1":
        raise DoctorError("tool lock schema or keys do not match doctor")
    return values


def rust_toolchain_channel(path: Path = None) -> str:
    path = RUST_TOOLCHAIN if path is None else path
    matches = re.findall(
        r'^\s*channel\s*=\s*"([^"\\]+)"\s*$',
        path.read_text(encoding="utf-8"), re.MULTILINE,
    )
    if len(matches) != 1:
        raise DoctorError("rust-toolchain.toml must contain exactly one plain channel")
    return matches[0]


def version(args: Sequence[str], *, env: Dict[str, str] = None) -> str:
    try:
        result = subprocess.run(
            args, cwd=str(ROOT), env=env, capture_output=True, text=True,
            check=False, timeout=30,
        )
    except (OSError, subprocess.TimeoutExpired) as error:
        raise DoctorError("%s cannot run: %s" % (" ".join(args), error)) from error
    if result.returncode:
        raise DoctorError("%s exited %d: %s" % (
            " ".join(args), result.returncode, result.stderr.strip()
        ))
    lines = result.stdout.strip().splitlines()
    if not lines:
        raise DoctorError("%s produced no version" % " ".join(args))
    return lines[0]


def report() -> Dict[str, str]:
    lock = read_lock()
    configured_rust = rust_toolchain_channel()
    if configured_rust != lock["rust"]:
        raise DoctorError(
            "Rust toolchain sources disagree: rust-toolchain.toml=%r lock=%r"
            % (configured_rust, lock["rust"])
        )
    installed = subprocess.run(
        ("rustup", "toolchain", "list"), cwd=str(ROOT), capture_output=True,
        text=True, check=True, timeout=30,
    ).stdout
    for channel in (lock["rust"], lock["clippy"], lock["public_api_nightly"]):
        if not re.search(r"^" + re.escape(channel) + r"(?:-|\s)", installed, re.MULTILINE):
            raise DoctorError("required rustup toolchain is not installed: " + channel)
    commands = (
        ("rust", ("rustup", "run", lock["rust"], "rustc", "--version")),
        ("cargo", ("rustup", "run", lock["rust"], "cargo", "--version")),
        ("clippy", ("rustup", "run", lock["clippy"], "cargo-clippy", "--version")),
        ("public_api_rustc", ("rustup", "run", lock["public_api_nightly"], "rustc", "--version")),
        ("just", ("just", "--version")),
        ("nextest", ("cargo-nextest", "--version")),
        ("taplo", ("taplo", "--version")),
        ("typos", ("typos", "--version")),
        ("deny", ("cargo-deny", "--version")),
        ("public_api", ("cargo-public-api", "--version")),
        ("semgrep", ("semgrep", "--version")),
    )
    with ThreadPoolExecutor(max_workers=4) as workers:
        observed = dict(zip(
            (key for key, _ in commands),
            workers.map(lambda command: version(command[1]), commands),
        ))
    observed["python"] = ".".join(str(part) for part in sys.version_info[:3])
    expected = {
        "rust": "rustc " + lock["rust"],
        "cargo": "cargo " + lock["rust"],
        "clippy": "clippy 0.1." + lock["clippy"].split(".")[1],
        "public_api_rustc": lock["public_api_rustc"],
        "just": lock["just"],
        "nextest": lock["nextest"],
        "taplo": lock["taplo"],
        "typos": lock["typos"],
        "deny": lock["deny"],
        "public_api": lock["public_api"],
        "semgrep": lock["semgrep"],
        "python": lock["python"],
    }
    for key, want in expected.items():
        actual = observed[key]
        if key in {"rust", "cargo", "clippy"}:
            matches = actual.startswith(want + " ") or actual == want
        else:
            matches = actual == want
        if not matches:
            raise DoctorError("%s version mismatch: expected %r, found %r" % (key, want, actual))
    return observed


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()
    try:
        observed = report()
    except (DoctorError, OSError, subprocess.SubprocessError) as error:
        print("tool doctor: " + str(error), file=sys.stderr)
        return 2
    if args.json:
        print(json.dumps(observed, sort_keys=True))
    else:
        print("tool doctor: locked versions verified")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
