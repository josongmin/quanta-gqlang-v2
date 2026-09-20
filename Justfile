default:
    @just --list

fmt:
    cargo fmt --all --check

taplo:
    taplo fmt --check

clippy:
    cargo clippy --workspace --all-targets -- -A clippy::allow_attributes -A clippy::duplicated_attributes -D warnings

clippy-current-policy:
    cargo +1.95.0 clippy --workspace --all-targets -- -A clippy::allow_attributes -A clippy::duplicated_attributes -D clippy::unused_result_ok -D warnings

nextest:
    cargo nextest run --workspace --no-tests pass

doc-test:
    cargo test --workspace --doc

test:
    just nextest
    just doc-test

deny:
    cargo deny check bans licenses sources advisories

typos:
    typos .

arch-lint:
    python3 tools/architecture/tests/test_architecture.py
    python3 tools/architecture/check_architecture.py

semgrep:
    python3 tools/lint/tests/test_semgrep_non_vacuity.py
    tools/lint/run_semgrep.sh

lint: fmt taplo clippy clippy-current-policy test arch-lint deny typos

ci: lint semgrep
