default:
    @just --list

# ---- static leaves (each runs exactly one formatter/linter command) ----

fmt:
    cargo fmt --all --check

taplo:
    taplo fmt --check

clippy-msrv:
    cargo clippy --workspace --all-targets -- -D warnings

clippy-current:
    cargo +1.95.0 clippy --workspace --all-targets -- -D clippy::unused_result_ok -D warnings

typos:
    typos .

# ---- test leaves (each runs exactly one test command) ----

nextest:
    cargo nextest run --workspace --no-tests pass

doctest:
    RUSTDOCFLAGS="-D warnings" cargo test --workspace --doc

frontdoor-parity:
    python3 tools/test_frontdoor_parity.py

# ---- policy leaves (each runs exactly one policy command) ----

arch-lint:
    python3 tools/architecture/tests/test_architecture.py
    python3 tools/architecture/check_architecture.py

external-consumer:
    python3 tools/test_frontend_external_consumer.py

deny:
    cargo deny check bans licenses sources advisories

semgrep:
    python3 tools/lint/tests/test_semgrep_non_vacuity.py
    tools/lint/run_semgrep.sh

# ---- composition groups (one composition path per command) ----
# Each leaf is a dependency of exactly one group; each group is a
# dependency of exactly `ci`. The old `lint` recipe (which nested
# test + frontdoor-parity inside lint) is removed.

static: fmt taplo clippy-msrv clippy-current typos

test: nextest doctest frontdoor-parity

policy: arch-lint external-consumer deny semgrep

ci: static test policy

# Pinned local qualification: clean-tree check, HEAD SHA pin, toolchain
# record, then `just ci`, writing target/qualification/<sha>/receipt.json.
# The receipt directory lives under target/ (gitignored) so it stays uncommitted.
qualify-local:
    bash tools/qualification/qualify-local.sh
