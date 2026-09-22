#!/usr/bin/env python3
"""External-consumer gate for the pinned qglang-v3 frontend facade (lane W4-H).

The gate proves the facade is a *sufficient and minimal* seam for a sibling
repository. It never modifies tracked checkout files; the public-API probe
reuses the checkout's gitignored Cargo target:

1. materialize ``tools/frontend_external_consumer/`` at one locked, owned path
   under this checkout's ignored ``target/`` directory, rewrite its single
   ``path = ...`` dependency to an absolute path into this repository, copy
   ``Cargo.lock`` for offline resolution, and reuse only that checkout's
   fixture build cache;

2. assert the manifest shapes (fixture normal deps == {facade}; facade normal
   deps == {pipeline}) and the facade source shape (zero ``^pub use .*::\\*``,
   zero ``^pub mod``, curated allowlist == the pinned literal);

3. run the positive program (``cargo run --quiet``) and require exit 0 plus the
   exact marker ``external-consumer positive: ok``;

4. run privacy probes as complete ``src/main.rs`` programs through ``cargo
   check --quiet``. The allowlist and manifest checks already prove that
   unlisted root symbols, public modules, and owner crates are unreachable;
   these probes cover the separate property that selected fields and
   constructors stay private. Two struct-literal diagnostics carry no E-code,
   so those map entries require that no coded error appears;

5. diff ``cargo public-api -p quanta-gqlang-v2-frontend-contract --simplified``
   against the pinned allowlist literal (extra/missing names). This invocation
   runs in the repository checkout and reuses only that checkout's Cargo target;
   the fixture keeps its separate target;

6. assert the checkout is untouched: ``Cargo.lock`` sha256, the byte content of
   every path the gate owns, and ``git status --porcelain`` are compared
   before/after. A delta under a gate-owned path is a contract violation; a
   delta that only touches foreign paths is reported as a concurrent writer and
   makes the run unjudgeable (exit 2) instead of being blamed on the gate.

Exit codes: 0 = every expectation matched; 1 = the gate found a contract or
case violation; 2 = the gate could not judge (infrastructure failure: the
repository tree does not compile, a case failed for a non-fixture reason, or a
required tool is missing). Exit 2 is never a pass.

Standard library only: the repository python is 3.9, which has no ``tomllib``,
so the small manifest reader below handles exactly the subset of TOML the
repository manifests use (``[section]`` headers, ``key = value`` entries,
inline tables).

The cargo toolchain is pinned to the repository's ``rust-toolchain.toml``
channel via ``RUSTUP_TOOLCHAIN`` because the fixture lives outside the
checkout, where that file would otherwise not apply and the expected
diagnostic codes would drift with the machine default toolchain.
"""

from __future__ import annotations

import argparse
import fcntl
import hashlib
import os
import re
import shutil
import subprocess
import sys
from pathlib import Path
from typing import Dict, IO, List, Optional, Sequence, Tuple

REPOSITORY_ROOT = Path(__file__).resolve().parents[1]
FIXTURE_DIR = REPOSITORY_ROOT / "tools" / "frontend_external_consumer"
NEGATIVES_DIR = FIXTURE_DIR / "negatives"
FACADE_DIR = REPOSITORY_ROOT / "crates" / "quanta-gqlang-v2-frontend-contract"
FACADE_MANIFEST = FACADE_DIR / "Cargo.toml"
FACADE_SOURCE = FACADE_DIR / "src" / "lib.rs"
LOCKFILE = REPOSITORY_ROOT / "Cargo.lock"
CACHE_ROOT = REPOSITORY_ROOT / "target" / "external-consumer-gate"

FIXTURE_PACKAGE = "quanta-gqlang-v2-frontend-external-consumer"
FACADE_PACKAGE = "quanta-gqlang-v2-frontend-contract"
PIPELINE_PACKAGE = "quanta-gqlang-v2-pipeline"
FACADE_DEP_LINE = 'path = "../../crates/quanta-gqlang-v2-frontend-contract"'
FACADE_OWNER_CRATE = "quanta_gqlang_v2_pipeline"

POSITIVE_MARKER = "external-consumer positive: ok"

EXIT_OK = 0
EXIT_GATE_FAILED = 1
EXIT_INFRASTRUCTURE = 2

# The pinned curated surface (W4-C / F-01): three entry points, the
# assembled-document DTOs, the planning DTOs, the authority/budget/diagnostic
# vocabulary, and the inert closure types. Public-API over-claiming is a
# violation in both directions (extra and missing).
PINNED_ALLOWLIST: Tuple[str, ...] = (
    "analyze_source_document_v2",
    "compile_source_document_v3",
    "format_source_document_v2",
    "AnalysisDocumentV2",
    "AnalysisItemV2",
    "AnalysisStatusV2",
    "CompiledSourceDocumentV3",
    "CompiledItemV2",
    "CompiledRuleProgramV2",
    "ExecutionAuthorityV3",
    "ExecutionBindingV3",
    "ExecutionCallTargetV3",
    "ExecutionCardinalityV3",
    "ExecutionCompositionBindingV3",
    "ExecutionCompositionBodyV3",
    "ExecutionCompositionProducerV3",
    "ExecutionCompositionV3",
    "ExecutionCorrelationV3",
    "ExecutionDeterminismV3",
    "ExecutionDigestV3",
    "ExecutionDocumentV3",
    "ExecutionEffectV3",
    "ExecutionEvidenceV3",
    "ExecutionExistsPolarityV3",
    "ExecutionExpressionShapeV3",
    "ExecutionExpressionV3",
    "ExecutionFamilyV3",
    "ExecutionFiniteFloatV3",
    "ExecutionFieldV3",
    "ExecutionGraphKindV3",
    "ExecutionItemV3",
    "ExecutionJoinModeV3",
    "ExecutionLiteralV3",
    "ExecutionLogicalQuantifierV3",
    "ExecutionNullOrderV3",
    "ExecutionOperatorV3",
    "ExecutionOrderKeyV3",
    "ExecutionPlanV3",
    "ExecutionPolicyFamilyV3",
    "ExecutionRecordFieldV3",
    "ExecutionRelationAuthorityV3",
    "ExecutionRelationV3",
    "ExecutionRelationalOperatorV3",
    "ExecutionScalarTypeV3",
    "ExecutionSampleModeV3",
    "ExecutionSchemaV3",
    "ExecutionSortDirectionV3",
    "ExecutionSourceKindV3",
    "ExecutionSourceV3",
    "ExecutionUnionVariantV3",
    "ExecutionValueTypeV3",
    "PlannedItemV2",
    "RuleProgramMemberV2",
    "UnresolvedAnalysisFactV2",
    "OPEN_AUTHORITY_INCOMPLETE_CODE",
    "NoPlanReasonV2",
    "SemanticPlanV2",
    "ClosedFrontendAuthorityBuilderV2",
    "ClosedFrontendAuthorityV2",
    "FrontendAuthorityErrorV2",
    "FrontendAuthorityLocationV2",
    "FrontendAuthoritySourceV2",
    "FrontendBudgetV2",
    "FrontendCallableParameterInputV2",
    "FrontendCatalogAssetInputV2",
    "FrontendCatalogPayloadV2",
    "FrontendDiagnosticV2",
    "FrontendFailureV2",
    "FrontendGraphReferenceInputV2",
    "FrontendLanguageVersionV2",
    "FrontendLockfileInputV2",
    "FrontendPackInputV2",
    "FrontendRelationAuthorityV2",
    "FrontendRelationCardinalityV2",
    "FrontendRelationFieldInputV2",
    "FrontendRelationInputV2",
    "FrontendScalarTypeV2",
    "FrontendSeverityV2",
    "FrontendStageV2",
    "FrontendTransactionErrorV2",
    "GraphReferenceKindV2",
    "OpenAnalysisAuthorityV2",
    "SemanticItemIdV2",
    "AlgebraFamilyV2",
    "CheckedLocatedSpanV2",
    "LocatedSpanV2",
    "SourceIdV2",
    "SourceQualifiedNodeIdV2",
    "SpanV1",
)

# name -> (expected compiler code, expected message needle). The public-name
# allowlist and manifest topology are the SSOT for root/module/crate reachability;
# these fixtures cover privacy, which that SSOT cannot express.
NEGATIVE_CASES: Dict[str, Tuple[str, str]] = {
    # Privacy probes on curated roots (read, write, struct literal,
    # private associated function). An empty expected code pins a rustc
    # diagnostic that carries no E-code at all (rustc 1.81 reports a struct
    # literal over private fields as an uncoded `error:`), which the gate treats
    # as "no coded error may appear".
    "d01_read_diagnostic_stage": ("E0616", "field `stage` of struct `FrontendDiagnosticV2` is private"),
    "d02_write_diagnostic_severity": (
        "E0616",
        "field `severity` of struct `FrontendDiagnosticV2` is private",
    ),
    "d03_literal_budget_field": (
        "",
        "cannot construct `FrontendBudgetV2` with struct literal syntax due to private fields",
    ),
    "d04_read_analysis_items": ("E0616", "field `items` of struct `AnalysisDocumentV2` is private"),
    "d05_read_plan_authority_lane": ("E0616", "field `authority_lane` of struct `SemanticPlanV2` is private"),
    "d06_read_checked_span_location": (
        "E0616",
        "field `location` of struct `CheckedLocatedSpanV2` is private",
    ),
    "d07_read_unresolved_code": ("E0616", "field `code` of struct `UnresolvedAnalysisFactV2` is private"),
    "d08_read_closed_authority_modules": (
        "E0616",
        "field `modules` of struct `ClosedFrontendAuthorityV2` is private",
    ),
    "d09_literal_plan_field": (
        "",
        "cannot construct `SemanticPlanV2` with struct literal syntax due to private fields",
    ),
    "d10_private_plan_from_parts": ("E0624", "associated function `from_parts` is private"),
    "d11_private_rule_member_new": ("E0624", "associated function `new` is private"),
    "d12_read_budget_max_source_bytes": (
        "E0616",
        "field `max_source_bytes` of struct `FrontendBudgetV2` is private",
    ),
    "d13_read_execution_composition_bindings": (
        "E0616",
        "field `bindings` of struct `ExecutionCompositionV3` is private",
    ),
    "d14_import_retired_compiled_item_v1": ("E0432", "no `CompiledItemV1` in the root"),
    "d15_private_execution_document_new": ("E0624", "associated function `new` is private"),
}

ERROR_CODE_RE = re.compile(r"error\[(E\d{4})\]")
PUB_USE_RE = re.compile(r"pub use\s+([A-Za-z0-9_]+)::(?:\{([^}]*)\}|([A-Za-z0-9_]+))\s*;", re.DOTALL)
GLOB_RE = re.compile(r"^pub use .*::\*", re.MULTILINE)
MODULE_RE = re.compile(r"^pub mod", re.MULTILINE)
FIXTURE_ANCHOR = "--> src/main.rs:"


class InfrastructureError(RuntimeError):
    """The gate cannot judge: the failure is not the fixture's own surface."""


def sha256_of(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def run(command: Sequence[str], cwd: Path, env: Dict[str, str]) -> subprocess.CompletedProcess:
    try:
        return subprocess.run(
            list(command),
            cwd=str(cwd),
            env=env,
            check=False,
            capture_output=True,
            text=True,
        )
    except OSError as error:
        raise InfrastructureError("%s is not runnable: %s" % (command[0], error))


def git_status_bytes() -> bytes:
    result = subprocess.run(
        ["git", "status", "--porcelain"],
        cwd=str(REPOSITORY_ROOT),
        check=False,
        capture_output=True,
    )
    if result.returncode != 0:
        raise InfrastructureError("git status --porcelain failed in the checkout")
    return result.stdout


# Paths this gate is allowed to exist in: a change to any of them is the gate's
# own doing (or cargo's, through the lock file or the target directory) and is a
# contract violation. Anything else that moves during a run is a concurrent
# writer in a shared checkout: reported, never attributed to the gate.
GATE_WRITABLE_PREFIXES = (
    "Cargo.lock",
    "target/",
    "tools/frontend_external_consumer/",
    "tools/test_frontend_external_consumer.py",
)


def gate_writable_fingerprint() -> Dict[str, str]:
    """Content hash of every path this gate owns or must not change."""
    paths = [LOCKFILE, REPOSITORY_ROOT / "tools" / "test_frontend_external_consumer.py"]
    paths.extend(sorted(path for path in FIXTURE_DIR.rglob("*") if path.is_file()))
    return {
        str(path.relative_to(REPOSITORY_ROOT)): sha256_of(path)
        for path in paths
        if path.is_file()
    }


def status_delta(before: bytes, after: bytes) -> List[str]:
    before_lines = set(before.decode("utf-8", "replace").splitlines())
    after_lines = set(after.decode("utf-8", "replace").splitlines())
    return sorted(before_lines.symmetric_difference(after_lines))


def delta_path(status_line: str) -> str:
    rest = status_line[3:].strip() if len(status_line) > 3 else status_line.strip()
    if " -> " in rest:
        rest = rest.split(" -> ")[-1]
    return rest.strip().strip('"')


def manifest_dependencies(text: str, section: str) -> List[str]:
    """Return the dependency key order of one manifest section."""
    names: List[str] = []
    current: Optional[str] = None
    for raw_line in text.splitlines():
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        if line.startswith("[") and line.endswith("]"):
            current = line[1:-1].strip()
            continue
        if current != section:
            continue
        if "=" not in line:
            continue
        key = line.split("=", 1)[0].strip().strip('"')
        if key:
            names.append(key)
    return names


def facade_allowlist(text: str) -> List[str]:
    """Names re-exported from the facade's sole owner crate, in source order."""
    names: List[str] = []
    for match in PUB_USE_RE.finditer(text):
        if match.group(1) != FACADE_OWNER_CRATE:
            continue
        if match.group(2) is not None:
            for part in match.group(2).split(","):
                part = part.strip()
                if part:
                    names.append(part)
        elif match.group(3) is not None:
            names.append(match.group(3))
    return names


def facade_owner_crates(text: str) -> List[str]:
    return sorted({match.group(1) for match in PUB_USE_RE.finditer(text)})


def repo_toolchain_channel() -> Optional[str]:
    manifest = REPOSITORY_ROOT / "rust-toolchain.toml"
    if not manifest.is_file():
        return None
    match = re.search(r'^\s*channel\s*=\s*"([^"]+)"', manifest.read_text(), re.MULTILINE)
    return match.group(1) if match else None


def cargo_env(target_dir: Path, channel: Optional[str]) -> Dict[str, str]:
    env = dict(os.environ)
    env["CARGO_TARGET_DIR"] = str(target_dir)
    env["CARGO_NET_OFFLINE"] = "true"
    env["CARGO_TERM_COLOR"] = "never"
    if channel:
        env["RUSTUP_TOOLCHAIN"] = channel
    return env


def tool_version(executable: str, cwd: Path, env: Dict[str, str]) -> str:
    result = run([executable, "--version"], cwd=cwd, env=env)
    if result.returncode != 0:
        raise InfrastructureError("%s --version failed: %s" % (executable, result.stderr.strip()))
    version = result.stdout.strip()
    prefix = executable + " "
    return version[len(prefix):] if version.startswith(prefix) else version


def lock_cache(cache_root: Path, repo_root: Path) -> IO[str]:
    if cache_root.is_symlink():
        raise InfrastructureError("external-consumer cache root must not be a symlink")
    cache_root.mkdir(parents=True, exist_ok=True)
    lock_path = cache_root / ".lock"
    if lock_path.is_symlink():
        raise InfrastructureError("external-consumer cache lock must not be a symlink")
    lock = lock_path.open("a+")
    try:
        try:
            fcntl.flock(lock.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError as error:
            raise InfrastructureError("another external-consumer gate owns the fixture cache") from error
        marker = cache_root / ".owner"
        expected = "%s\n%s\n" % (repo_root.resolve(), cache_root.resolve())
        if marker.is_symlink():
            raise InfrastructureError("external-consumer cache owner must not be a symlink")
        if marker.exists():
            if not marker.is_file() or marker.read_text() != expected:
                raise InfrastructureError("external-consumer cache belongs to another checkout")
        else:
            foreign = sorted(path.name for path in cache_root.iterdir() if path.name != ".lock")
            if foreign:
                raise InfrastructureError("unowned external-consumer cache contains %s" % foreign)
            marker.write_text(expected)
        unexpected = sorted(
            path.name for path in cache_root.iterdir()
            if path.name not in {".lock", ".owner", "fixture", "build"}
        )
        if unexpected:
            raise InfrastructureError("external-consumer cache contains unexpected paths %s" % unexpected)
        build = cache_root / "build"
        if build.is_symlink() or (build.exists() and not build.is_dir()):
            raise InfrastructureError("external-consumer build cache is redirected")
        return lock
    except (OSError, InfrastructureError):
        lock.close()
        raise


def materialize_fixture(cache_root: Path) -> Path:
    fixture = cache_root / "fixture"
    if fixture.is_symlink():
        raise InfrastructureError("external-consumer fixture must not be a symlink")
    fixture.mkdir(exist_ok=True)
    unexpected = sorted(
        path.name for path in fixture.iterdir()
        if path.name not in {"Cargo.toml", "Cargo.lock", "src"}
    )
    if unexpected:
        raise InfrastructureError("external-consumer fixture contains unexpected paths %s" % unexpected)
    src = fixture / "src"
    if src.is_symlink():
        raise InfrastructureError("external-consumer fixture src must not be a symlink")
    src.mkdir(exist_ok=True)
    if [path.name for path in src.iterdir() if path.name != "main.rs"]:
        raise InfrastructureError("external-consumer fixture src contains unexpected paths")
    for path in (fixture / "Cargo.toml", fixture / "Cargo.lock", src / "main.rs"):
        if path.is_symlink() or (path.exists() and not path.is_file()):
            raise InfrastructureError("external-consumer fixture file is redirected: %s" % path)
    manifest = fixture / "Cargo.toml"
    text = (FIXTURE_DIR / "Cargo.toml").read_text()
    if text.count(FACADE_DEP_LINE) != 1:
        raise InfrastructureError(
            "the fixture manifest must carry exactly one %r dependency line" % FACADE_DEP_LINE
        )
    manifest.write_text(
        text.replace(FACADE_DEP_LINE, 'path = "%s"' % (FACADE_DIR,))
    )
    shutil.copyfile(str(LOCKFILE), str(fixture / "Cargo.lock"))
    (src / "main.rs").write_bytes((FIXTURE_DIR / "src" / "main.rs").read_bytes())
    return fixture


def main(argv: Optional[Sequence[str]] = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--verbose", action="store_true", help="print every case and its verdict")
    args = parser.parse_args(argv)

    def say(message: str) -> None:
        print(message, flush=True)

    def trace(message: str) -> None:
        if args.verbose:
            say(message)

    violations: List[str] = []
    mismatches: List[str] = []
    infrastructure: List[str] = []
    counts: Dict[str, int] = {
        "positive_selected": 1,
        "positive_executed": 0,
        "positive_marker": 0,
        "negative_selected": len(NEGATIVE_CASES),
        "negative_executed": 0,
        "code_matched": 0,
        "needle_matched": 0,
        "uncoded_expected": sum(1 for code, _ in NEGATIVE_CASES.values() if not code),
        "globs": 0,
        "pub_mods": 0,
        "allowlist_observed": 0,
        "allowlist_pinned": len(PINNED_ALLOWLIST),
        "fixture_deps": 0,
        "facade_normal_deps": 0,
        "facade_extra_deps": 0,
        "facade_missing_deps": 0,
        "public_api_observed": 0,
        "public_api_extra": 0,
        "public_api_missing": 0,
    }
    cache_lock: Optional[IO[str]] = None
    fixture: Optional[Path] = None
    positive_main: Optional[bytes] = None
    cargo_version = "unknown"
    rustc_version = "unknown"
    before_lock = ""
    before_status = b""
    before_fingerprint: Dict[str, str] = {}

    try:
        # ---- stage 0: repository shape ------------------------------------
        before_lock = sha256_of(LOCKFILE)
        before_status = git_status_bytes()
        before_fingerprint = gate_writable_fingerprint()

        fixture_text = (FIXTURE_DIR / "Cargo.toml").read_text()
        fixture_deps = manifest_dependencies(fixture_text, "dependencies")
        counts["fixture_deps"] = len(fixture_deps)
        if fixture_deps != [FACADE_PACKAGE]:
            violations.append(
                "fixture normal deps must be exactly [%s], got %s" % (FACADE_PACKAGE, fixture_deps)
            )
        for extra_section in ("dev-dependencies", "build-dependencies"):
            if manifest_dependencies(fixture_text, extra_section):
                violations.append("fixture must declare no [%s]" % extra_section)

        facade_text = FACADE_MANIFEST.read_text()
        facade_deps = manifest_dependencies(facade_text, "dependencies")
        counts["facade_normal_deps"] = len(facade_deps)
        facade_extra = sorted(set(facade_deps) - {PIPELINE_PACKAGE})
        facade_missing = sorted({PIPELINE_PACKAGE} - set(facade_deps))
        counts["facade_extra_deps"] = len(facade_extra)
        counts["facade_missing_deps"] = len(facade_missing)
        if facade_extra or facade_missing:
            violations.append(
                "facade normal deps must be exactly [%s]: extra=%s missing=%s "
                "(the five transitional owner edges must be deleted in the facade cut)"
                % (PIPELINE_PACKAGE, facade_extra, facade_missing)
            )

        source_text = FACADE_SOURCE.read_text()
        globs = GLOB_RE.findall(source_text)
        modules = MODULE_RE.findall(source_text)
        counts["globs"] = len(globs)
        counts["pub_mods"] = len(modules)
        if globs:
            violations.append("facade source keeps %d crate-wide glob re-export(s)" % len(globs))
        if modules:
            violations.append("facade source declares %d public module(s)" % len(modules))

        observed_allowlist = facade_allowlist(source_text)
        counts["allowlist_observed"] = len(observed_allowlist)
        if sorted(observed_allowlist) != sorted(PINNED_ALLOWLIST):
            extra = sorted(set(observed_allowlist) - set(PINNED_ALLOWLIST))
            missing = sorted(set(PINNED_ALLOWLIST) - set(observed_allowlist))
            violations.append(
                "facade source allowlist drifted from the pinned surface: extra=%s missing=%s"
                % (extra, missing)
            )
        owner_crates = facade_owner_crates(source_text)
        if owner_crates != [FACADE_OWNER_CRATE]:
            violations.append("facade source re-exports owner crates %s" % owner_crates)

        if violations:
            for violation in violations:
                say("contract violation: %s" % violation)
        say(
            "stage 0: fixture_deps=%d facade_normal_deps=%d globs=%d pub_mods=%d allowlist=%d violations=%d"
            % (
                counts["fixture_deps"],
                counts["facade_normal_deps"],
                counts["globs"],
                counts["pub_mods"],
                counts["allowlist_observed"],
                len(violations),
            )
        )

        # ---- stage 1/2: isolated fixture at a stable, owned path ----------
        channel = repo_toolchain_channel()
        cache_lock = lock_cache(CACHE_ROOT, REPOSITORY_ROOT)
        fixture = materialize_fixture(CACHE_ROOT)
        env = cargo_env(CACHE_ROOT / "build", channel)
        # Stage 3 runs at this checkout's stable manifest path, so its Cargo
        # target can be reused without cross-checkout stale-path artifacts.
        # Pin its toolchain too: an inherited RUSTUP_TOOLCHAIN can otherwise
        # override rust-toolchain.toml and change the observed public API.
        public_api_env = cargo_env(REPOSITORY_ROOT / "target", channel)
        cargo_version = tool_version("cargo", fixture, env)
        rustc_version = tool_version("rustc", fixture, env)
        trace("locked fixture cache: %s" % CACHE_ROOT)
        trace("toolchain: %s / %s" % (cargo_version, rustc_version))

        positive_main = (fixture / "src" / "main.rs").read_bytes()

        result = run(["cargo", "run", "--quiet"], cwd=fixture, env=env)
        counts["positive_executed"] = 1
        if result.returncode != 0:
            if FIXTURE_ANCHOR not in result.stderr:
                raise InfrastructureError(
                    "the repository tree does not compile for the fixture:\n%s"
                    % result.stderr.strip()
                )
            violations.append("positive program failed to compile")
            say("positive: exit=%d" % result.returncode)
            say(result.stderr.strip())
        elif result.stdout.strip() != POSITIVE_MARKER:
            violations.append(
                "positive program printed %r, expected %r" % (result.stdout.strip(), POSITIVE_MARKER)
            )
        else:
            counts["positive_marker"] = 1
        say("stage 1: positive exit=%d marker=%d" % (result.returncode, counts["positive_marker"]))

        unregistered = sorted(
            path.stem for path in NEGATIVES_DIR.glob("*.rs") if path.stem not in NEGATIVE_CASES
        )
        absent = sorted(name for name in NEGATIVE_CASES if not (NEGATIVES_DIR / (name + ".rs")).is_file())
        if unregistered:
            violations.append("negative fixtures missing from the expected-code map: %s" % unregistered)
        if absent:
            violations.append("expected-code map names without a fixture: %s" % absent)

        for name, (expected_code, expected_needle) in sorted(NEGATIVE_CASES.items()):
            path = NEGATIVES_DIR / (name + ".rs")
            if not path.is_file():
                continue
            (fixture / "src" / "main.rs").write_bytes(path.read_bytes())
            result = run(["cargo", "check", "--quiet"], cwd=fixture, env=env)
            counts["negative_executed"] += 1
            observed_codes = sorted(set(ERROR_CODE_RE.findall(result.stderr)))
            if expected_code:
                code_ok = result.returncode != 0 and expected_code in observed_codes
            else:
                # No E-code is expected for this diagnostic: any coded error is
                # a mismatch, and the needle below carries the assertion.
                code_ok = result.returncode != 0 and not observed_codes
            needle_ok = result.returncode != 0 and expected_needle in result.stderr
            if result.returncode == 0:
                violations.append("negative %s compiled: the leak is reachable" % name)
            elif FIXTURE_ANCHOR not in result.stderr:
                raise InfrastructureError(
                    "negative %s failed for a non-fixture reason:\n%s" % (name, result.stderr.strip())
                )
            if code_ok:
                counts["code_matched"] += 1
            else:
                mismatches.append(
                    "%s: expected %s, observed %s"
                    % (
                        name,
                        expected_code or "no error code",
                        ",".join(observed_codes) or "none",
                    )
                )
            if needle_ok:
                counts["needle_matched"] += 1
            else:
                first_line = result.stderr.strip().splitlines()
                mismatches.append(
                    "%s: expected message %r, observed %r"
                    % (name, expected_needle, first_line[0] if first_line else "")
                )
            trace(
                "privacy probe %-27s exit=%d expected=%s observed=%s"
                % (
                    name,
                    result.returncode,
                    expected_code or "uncoded",
                    ",".join(observed_codes) or "none",
                )
            )

        say(
            "stage 2: negative selected=%d executed=%d code_matched=%d needle_matched=%d mismatches=%d"
            % (
                counts["negative_selected"],
                counts["negative_executed"],
                counts["code_matched"],
                counts["needle_matched"],
                len(mismatches),
            )
        )

        # ---- stage 3: public-API allowlist diff ----------------------------
        result = run(
            ["cargo", "public-api", "-p", FACADE_PACKAGE, "--simplified"],
            cwd=REPOSITORY_ROOT,
            env=public_api_env,
        )
        if result.returncode != 0:
            raise InfrastructureError(
                "cargo public-api failed (exit %d):\n%s" % (result.returncode, result.stderr.strip())
            )
        observed_public: List[str] = []
        for line in result.stdout.splitlines():
            line = line.strip()
            prefix = "pub use %s::" % FACADE_PACKAGE.replace("-", "_")
            if line.startswith(prefix):
                observed_public.append(line[len(prefix):].strip())
        counts["public_api_observed"] = len(observed_public)
        public_extra = sorted(set(observed_public) - set(PINNED_ALLOWLIST))
        public_missing = sorted(set(PINNED_ALLOWLIST) - set(observed_public))
        counts["public_api_extra"] = len(public_extra)
        counts["public_api_missing"] = len(public_missing)
        if public_extra:
            violations.append("public API over-claims names outside the allowlist: %s" % public_extra)
        if public_missing:
            violations.append("public API is missing allowlisted names: %s" % public_missing)
        say(
            "stage 3: public_api observed=%d pinned=%d extra=%d missing=%d"
            % (counts["public_api_observed"], counts["allowlist_pinned"], len(public_extra), len(public_missing))
        )
    except InfrastructureError as error:
        infrastructure.append(str(error))
        say("infrastructure failure: %s" % error)
    except OSError as error:
        infrastructure.append("external-consumer filesystem failure: %s" % error)
        say("infrastructure failure: %s" % infrastructure[-1])
    finally:
        if fixture is not None and positive_main is not None:
            try:
                (fixture / "src" / "main.rs").write_bytes(positive_main)
            except OSError as error:
                infrastructure.append("could not restore the cached positive fixture: %s" % error)
        if cache_lock is not None:
            cache_lock.close()

    # ---- stage 4: the checkout is untouched --------------------------------
    lock_unchanged = "unknown"
    status_unchanged = "unknown"
    gate_files_unchanged = "unknown"
    concurrent_paths: List[str] = []
    try:
        lock_unchanged = "unchanged" if sha256_of(LOCKFILE) == before_lock else "CHANGED"
        gate_files_unchanged = (
            "unchanged" if gate_writable_fingerprint() == before_fingerprint else "CHANGED"
        )
        after_status = git_status_bytes()
        if after_status == before_status:
            status_unchanged = "unchanged"
        else:
            delta = status_delta(before_status, after_status)
            concurrent_paths = [
                line
                for line in delta
                if not delta_path(line).startswith(GATE_WRITABLE_PREFIXES)
            ]
            status_unchanged = "CHANGED(concurrent)" if concurrent_paths else "CHANGED"
    except (InfrastructureError, OSError) as error:
        infrastructure.append("repository immutability check failed: %s" % error)
    if lock_unchanged == "CHANGED":
        violations.append("Cargo.lock changed during the gate run")
    if gate_files_unchanged == "CHANGED":
        violations.append("the gate modified its own fixture or script during the run")
    if status_unchanged == "CHANGED":
        violations.append("git status --porcelain changed under a path the gate owns")
    elif concurrent_paths:
        infrastructure.append(
            "a concurrent writer changed the shared checkout while the gate ran; "
            "immutability of the run cannot be judged: %s" % concurrent_paths
        )

    # ---- count block -------------------------------------------------------
    say("")
    say("external-consumer gate count block (W4-C contract, lane W4-H)")
    say(
        "positive        selected=%d executed=%d marker=%d"
        % (counts["positive_selected"], counts["positive_executed"], counts["positive_marker"])
    )
    say(
        "negative        selected=%d executed=%d"
        % (counts["negative_selected"], counts["negative_executed"])
    )
    say(
        "compile_fail    selected=%d executed=%d code_matched=%d needle_matched=%d uncoded_expected=%d"
        % (
            counts["negative_selected"],
            counts["negative_executed"],
            counts["code_matched"],
            counts["needle_matched"],
            counts["uncoded_expected"],
        )
    )
    say(
        "facade_source   globs=%d pub_mods=%d allowlist_observed=%d allowlist_pinned=%d"
        % (counts["globs"], counts["pub_mods"], counts["allowlist_observed"], counts["allowlist_pinned"])
    )
    say(
        "facade_manifest fixture_deps=%d facade_normal_deps=%d extra=%d missing=%d"
        % (
            counts["fixture_deps"],
            counts["facade_normal_deps"],
            counts["facade_extra_deps"],
            counts["facade_missing_deps"],
        )
    )
    say(
        "public_api      observed=%d pinned=%d extra=%d missing=%d"
        % (
            counts["public_api_observed"],
            counts["allowlist_pinned"],
            counts["public_api_extra"],
            counts["public_api_missing"],
        )
    )
    say(
        "repo_immutability cargo_lock=%s gate_files=%s git_status=%s"
        % (lock_unchanged, gate_files_unchanged, status_unchanged)
    )
    if concurrent_paths:
        say("concurrent writer delta: %s" % concurrent_paths)
    say("toolchain       cargo=%s rustc=%s channel=%s" % (cargo_version, rustc_version, repo_toolchain_channel()))

    for mismatch in mismatches:
        say("case mismatch: %s" % mismatch)
    for violation in violations:
        say("contract violation: %s" % violation)
    for failure in infrastructure:
        say("infrastructure failure: %s" % failure)

    if infrastructure:
        say(
            "verdict=INFRASTRUCTURE exit=%d contract_violations=%d case_mismatches=%d"
            % (EXIT_INFRASTRUCTURE, len(violations), len(mismatches))
        )
        return EXIT_INFRASTRUCTURE
    if violations or mismatches:
        say(
            "verdict=FAIL exit=%d contract_violations=%d case_mismatches=%d"
            % (EXIT_GATE_FAILED, len(violations), len(mismatches))
        )
        return EXIT_GATE_FAILED
    say("verdict=PASS exit=%d contract_violations=0 case_mismatches=0" % EXIT_OK)
    return EXIT_OK


if __name__ == "__main__":
    sys.exit(main())
