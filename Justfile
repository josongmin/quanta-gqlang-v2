default:
    @just --list

# ---- static leaves (each runs exactly one formatter/linter command) ----

fmt:
    cargo fmt --all --check

taplo:
    taplo fmt --check

clippy-msrv:
    CARGO_TARGET_DIR=target/ci-msrv cargo clippy --workspace --all-targets --locked --offline -- -D warnings

clippy-current:
    python3 -I -S tools/ci/python_gate.py tools/ci/clippy_gate.py

typos:
    typos .

pm-check:
    python3 -I -S tools/ci/python_gate.py tools/prompt-manager/pm.py check

pm-test:
    PYTHONDONTWRITEBYTECODE=1 python3 -I -S tools/ci/python_gate.py tools/ci/unittest_gate.py tools/prompt-manager/tests 'test_*.py'

pm-sync:
    python3 -I -S tools/ci/python_gate.py tools/prompt-manager/pm.py sync

pm-status:
    python3 -I -S tools/ci/python_gate.py tools/prompt-manager/pm.py status

# ---- test leaves (each runs exactly one test command) ----

nextest:
    python3 -I -S tools/ci/python_gate.py tools/ci/nextest_gate.py

doctest:
    python3 -I -S tools/ci/python_gate.py tools/ci/doctest_gate.py

frontdoor-parity:
    python3 -I -S tools/ci/python_gate.py tools/test_frontdoor_parity.py

# ---- policy leaves (each runs exactly one policy command) ----

arch-lint:
    PYTHONDONTWRITEBYTECODE=1 python3 -I -S tools/ci/python_gate.py tools/ci/unittest_gate.py tools/architecture/tests 'test_architecture.py'
    python3 -I -S tools/ci/python_gate.py tools/architecture/check_architecture.py

external-consumer:
    python3 -I -S tools/ci/python_gate.py tools/test_frontend_external_consumer.py

external-consumer-preflight:
    python3 -I -S tools/ci/python_gate.py tools/test_frontend_external_consumer.py --preflight

deny:
    python3 -I -S tools/ci/python_gate.py tools/ci/deny_gate.py

advisory-refresh:
    cargo deny fetch
    python3 -I -S tools/ci/python_gate.py tools/ci/advisory_db.py

semgrep:
    PYTHONDONTWRITEBYTECODE=1 python3 -I -S tools/ci/python_gate.py tools/ci/unittest_gate.py tools/lint/tests 'test_semgrep_non_vacuity.py'
    python3 -I -S tools/ci/python_gate.py tools/lint/semgrep_gate.py

semantic-honesty:
    PYTHONDONTWRITEBYTECODE=1 python3 -I -S tools/ci/python_gate.py tools/ci/unittest_gate.py tools/lint/tests 'test_semantic_honesty.py'
    python3 -I -S tools/ci/python_gate.py tools/lint/semantic_honesty.py

qualifier-custody:
    PYTHONDONTWRITEBYTECODE=1 python3 -I -S tools/ci/python_gate.py tools/ci/unittest_gate.py tools/qualification/tests 'test_*.py'

doctor:
    python3 -I -S tools/ci/python_gate.py tools/ci/doctor.py

ci-rail-test:
    PYTHONDONTWRITEBYTECODE=1 python3 -I -S tools/ci/python_gate.py tools/ci/unittest_gate.py tools/ci/tests 'test_*.py'

# ---- composition groups (one composition path per command) ----
# Each full gate leaf appears in exactly one explicit serial group body. The
# offline external-consumer preflight runs before any build or test. The old
# `lint` recipe (which nested test + frontdoor-parity inside lint) is removed.

static:
    just fmt
    just taplo
    just clippy-current
    just clippy-msrv
    just typos
    just pm-check
    just pm-test

test:
    just nextest
    just doctest
    just frontdoor-parity

policy:
    just ci-rail-test
    just qualifier-custody
    just arch-lint
    just external-consumer
    just deny
    just semgrep
    just semantic-honesty

ci:
    python3 -I -S tools/ci/python_gate.py tools/ci/run_local.py

# Pinned local qualification: one locked, stable-path managed worktree/cache;
# clean-tree and exact HEAD/tree checks; then `just ci` and a local receipt.
# All generated state stays under target/qualification/ (gitignored).
qualify-local:
    bash tools/qualification/qualify-local.sh
