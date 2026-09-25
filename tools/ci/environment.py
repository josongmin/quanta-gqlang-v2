"""Remove caller-controlled Rust/Cargo/Nextest build semantics from local gates."""

from __future__ import annotations

import os
import re
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
)
RUN_ID_RE = re.compile(r"[0-9a-f]{32}")


class EnvironmentSanitizationError(RuntimeError):
    pass


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
