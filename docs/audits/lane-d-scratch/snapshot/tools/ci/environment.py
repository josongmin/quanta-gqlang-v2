"""Remove caller-controlled Rust/Cargo/Nextest build semantics from local gates."""

from __future__ import annotations

import os
import re
import shutil
from pathlib import Path
from typing import Mapping, Optional


EXACT_KEYS = {
    "CARGO_BUILD_RUSTC",
    "CARGO_BUILD_RUSTC_WRAPPER",
    "CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER",
    "CARGO_BUILD_RUSTDOC",
    "CARGO_BUILD_RUSTFLAGS",
    "CARGO_BUILD_TARGET",
    "CARGO_ENCODED_RUSTFLAGS",
    "CARGO_INCREMENTAL",
    "CARGO_TARGET_DIR",
    "DOCS_RS",
    "RUSTC",
    "RUSTC_BOOTSTRAP",
    "RUSTC_WRAPPER",
    "RUSTC_WORKSPACE_WRAPPER",
    "RUSTDOC",
    "RUSTDOCFLAGS",
    "RUSTFMT",
    "RUSTFLAGS",
    "RUSTUP_TOOLCHAIN",
    "RUST_TEST_THREADS",
}
PREFIXES = (
    "CARGO_CFG_",
    "CARGO_PROFILE_",
    "CARGO_TARGET_",
    "NEXTEST_",
    "PROPTEST_",
    "QGLANG_",
    "PYTHON",
)
RUN_ID_RE = re.compile(r"[0-9a-f]{32}")


class EnvironmentSanitizationError(RuntimeError):
    pass


def isolated_cargo_home(repo_root: Path, source_home: Path) -> Path:
    """Materialize an offline Cargo home without caller-owned Cargo config."""
    ci_root = repo_root / "target" / "ci"
    if ci_root.is_symlink():
        raise EnvironmentSanitizationError("CI output directory is redirected")
    ci_root.mkdir(parents=True, exist_ok=True)
    destination = ci_root / "cargo-home"
    marker = destination / ".qglang-ci-owned"
    if destination.exists() or destination.is_symlink():
        if destination.is_symlink() or not destination.is_dir() or not marker.is_file() or marker.is_symlink():
            raise EnvironmentSanitizationError("Cargo home path is not owned by this gate")
        shutil.rmtree(destination)
    if source_home.is_symlink() or not source_home.is_dir():
        raise EnvironmentSanitizationError("source Cargo home is missing or redirected")
    destination.mkdir()
    marker.write_text("qglang isolated Cargo home v1\n", encoding="ascii")
    for cache in ("registry", "git"):
        source = source_home / cache
        if source.exists():
            if source.is_symlink() or not source.is_dir():
                raise EnvironmentSanitizationError("Cargo cache is redirected: " + cache)
            (destination / cache).symlink_to(source, target_is_directory=True)
    database = source_home / "advisory-dbs"
    if database.exists():
        if database.is_symlink() or not database.is_dir():
            raise EnvironmentSanitizationError("advisory DB root is redirected")
        shutil.copytree(database, destination / "advisory-dbs", symlinks=False, copy_function=shutil.copy2)
    return destination


def valid_run_id(value: object) -> bool:
    return isinstance(value, str) and RUN_ID_RE.fullmatch(value) is not None


def sanitized_environment(
    source: Optional[Mapping[str, str]] = None,
    *,
    preserve_qualification_run_id: bool = False,
) -> dict[str, str]:
    env = dict(os.environ if source is None else source)
    run_id = env.get("QGLANG_QUALIFICATION_RUN_ID")
    if preserve_qualification_run_id and run_id is not None and not valid_run_id(run_id):
        raise EnvironmentSanitizationError("invalid qualification run identity")
    for key in tuple(env):
        if key in EXACT_KEYS or key.startswith(PREFIXES):
            env.pop(key)
    if preserve_qualification_run_id and run_id is not None:
        env["QGLANG_QUALIFICATION_RUN_ID"] = run_id
    return env
