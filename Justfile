default:
    @just --list

fmt:
    cargo fmt --all --check

taplo:
    taplo fmt --check

clippy:
    cargo clippy --workspace --all-targets -- -D warnings

clippy-current-policy:
    cargo +1.95.0 clippy --workspace --all-targets -- -D clippy::unused_result_ok -D warnings

nextest:
    cargo nextest run --workspace --no-tests pass

doc-test:
    RUSTDOCFLAGS="-D warnings" cargo test --workspace --doc

frontdoor-parity:
    python3 tools/test_frontdoor_parity.py

test:
    just nextest
    just doc-test
    just frontdoor-parity

deny:
    cargo deny check bans licenses sources advisories

typos:
    typos .

arch-lint:
    python3 tools/architecture/tests/test_architecture.py
    python3 tools/architecture/check_architecture.py

external-consumer:
    python3 tools/test_frontend_external_consumer.py

semgrep:
    python3 tools/lint/tests/test_semgrep_non_vacuity.py
    tools/lint/run_semgrep.sh

lint: fmt taplo clippy clippy-current-policy

boundary: arch-lint external-consumer

policy: deny typos semgrep

ci: lint test boundary policy
