"""Remove caller-controlled Rust/Cargo/Nextest build semantics from local gates."""

from __future__ import annotations

import os
from typing import Iterable, Mapping, Optional


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


def sanitized_environment(
    source: Optional[Mapping[str, str]] = None, *, preserve: Iterable[str] = (),
) -> dict[str, str]:
    env = dict(os.environ if source is None else source)
    retained = {key: env[key] for key in preserve if key in env}
    for key in tuple(env):
        if key in EXACT_KEYS or key.startswith(PREFIXES):
            env.pop(key)
    env.update(retained)
    return env
