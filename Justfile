default:
    @just --list

fmt:
    cargo fmt --all --check

taplo:
    taplo fmt --check

clippy:
    cargo clippy --workspace --lib --bins --examples -- -D warnings

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
    python3 tools/architecture/check_architecture.py

semgrep:
    tools/lint/run_semgrep.sh

lint: fmt taplo clippy test arch-lint deny typos

ci: lint semgrep
