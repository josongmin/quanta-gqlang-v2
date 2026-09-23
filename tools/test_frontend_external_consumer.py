#!/usr/bin/env python3
"""External-consumer gate for the pinned qglang-v3 frontend facade (lane W4-H).

The gate proves the facade is a *sufficient and minimal* seam for a sibling
repository. It never modifies tracked checkout files; the public-API probe
reuses the checkout's gitignored Cargo target:

1. materialize ``tools/frontend_external_consumer/`` at one locked, owned path
   under this checkout's ignored ``target/`` directory, rewrite its single
   ``path = ...`` dependency to an absolute path into this repository, copy
   the fixture-owned ``Cargo.lock`` for offline locked resolution, and reuse
   only that checkout's fixture build cache;

2. ask Cargo to parse the complete dependency tables, require exactly one
   unconditional path edge in each manifest (fixture -> facade -> pipeline),
   and assert the facade source shape (zero glob re-exports, zero public
   modules, curated allowlist == the pinned literal);

3. build the positive consumer with Cargo JSON diagnostics, select the binary
   artifact Cargo just reported, and execute that artifact separately. Its
   ``#![deny(unused_imports)]`` plus typed method signatures and exhaustive
   enum matches make rustc type-check its curated API uses; require exit 0
   plus the exact marker ``external-consumer positive: ok``;

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

6. compare ``Cargo.lock`` sha256, gate-owned source bytes, and
   ``git status --porcelain`` before/after. This detects gate-owned writes and
   status-shape changes, not content edits to an already-dirty foreign path;
   clean committed-HEAD qualification attests an exact source snapshot.

Exit codes: 0 = every expectation matched; 1 = the gate found a contract or
case violation; 2 = the gate could not judge (infrastructure failure: the
repository tree does not compile, a case failed for a non-fixture reason, or a
required tool is missing). Exit 2 is never a pass.

Standard library only: the repository Python is 3.9, which has no ``tomllib``.
Cargo's ``read-manifest`` parses the full TOML dependency grammar without
writing a lockfile or requiring a second TOML implementation here.

The cargo toolchain is pinned to the repository's ``rust-toolchain.toml``
channel via ``RUSTUP_TOOLCHAIN`` because the fixture lives outside the
checkout, where that file would otherwise not apply and the expected
diagnostic codes would drift with the machine default toolchain.
"""

from __future__ import annotations

import argparse
import fcntl
import hashlib
import json
import os
import re
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
FIXTURE_LOCKFILE = FIXTURE_DIR / "Cargo.lock"
CACHE_ROOT = REPOSITORY_ROOT / "target" / "external-consumer-gate"
VECTOR_PATH = (
    REPOSITORY_ROOT
    / "crates"
    / "quanta-gqlang-v2-pipeline"
    / "tests"
    / "fixtures"
    / "execution-v3-vectors.tsv"
)
FACADE_PACKAGE = "quanta-gqlang-v2-frontend-contract"
FIXTURE_PACKAGE = "quanta-gqlang-v2-frontend-external-consumer"
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
GLOB_RE = re.compile(r"^[ \t]*pub use .*::\*", re.MULTILINE)
MODULE_RE = re.compile(r"^[ \t]*pub mod", re.MULTILINE)
FIXTURE_ANCHOR = "--> src/main.rs:"


class InfrastructureError(RuntimeError):
    """The gate cannot judge: the failure is not the fixture's own surface."""


def sha256_of(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def write_bytes_if_changed(path: Path, content: bytes) -> None:
    """Keep a warm Cargo fixture stable when its checked source is unchanged."""
    if not path.is_file() or path.read_bytes() != content:
        path.write_bytes(content)


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


def manifest_dependencies(manifest: Path) -> List[Dict[str, object]]:
    """Return every Cargo-parsed dependency, including table and target forms."""
    result = run(
        ["cargo", "read-manifest", "--manifest-path", str(manifest)],
        manifest.parent,
        os.environ.copy(),
    )
    if result.returncode != 0:
        raise InfrastructureError("Cargo could not parse %s:\n%s" % (manifest, result.stderr.strip()))
    package = json.loads(result.stdout)
    dependencies = package.get("dependencies")
    if not isinstance(dependencies, list):
        raise InfrastructureError("Cargo returned no dependency list for %s" % manifest)
    return dependencies


def exact_path_dependency(dependencies: List[Dict[str, object]], name: str, path: Path) -> bool:
    """One default-feature path edge, with no alias, target, or optional form."""
    if len(dependencies) != 1:
        return False
    dependency = dependencies[0]
    raw_path = dependency.get("path")
    return (
        dependency.get("name") == name
        and dependency.get("kind") is None
        and dependency.get("target") is None
        and dependency.get("rename") is None
        and dependency.get("source") is None
        and dependency.get("optional") is False
        and dependency.get("uses_default_features") is True
        and dependency.get("features") == []
        and isinstance(raw_path, str)
        and Path(raw_path).resolve() == path.resolve()
    )


def positive_outcome(
    build: subprocess.CompletedProcess,
    execution: Optional[subprocess.CompletedProcess],
) -> Tuple[str, str]:
    """Classify a fixture build separately from its runtime behavior."""
    if build.returncode != 0:
        if FIXTURE_ANCHOR not in build.stderr:
            return "infrastructure", "the repository tree does not compile for the fixture:\n%s" % build.stderr.strip()
        return "violation", "positive program failed to compile"
    if execution is None:
        return "infrastructure", "the positive program was built but not executed"
    if execution.returncode != 0:
        return "violation", "positive consumer program failed during execution"
    if execution.stdout.strip() != POSITIVE_MARKER:
        return "violation", "positive program printed %r, expected %r" % (
            execution.stdout.strip(),
            POSITIVE_MARKER,
        )
    return "pass", ""


def positive_executable(build: subprocess.CompletedProcess, fixture: Path, target_dir: Path) -> Path:
    """Select the one fixture binary from this successful Cargo build's artifacts."""
    candidates: List[Path] = []
    expected_source = (fixture / "src" / "main.rs").resolve()
    owned_target = target_dir.resolve()
    for line in build.stdout.splitlines():
        try:
            message = json.loads(line)
        except json.JSONDecodeError as error:
            raise InfrastructureError("Cargo build emitted invalid JSON: %s" % error) from error
        if not isinstance(message, dict) or message.get("reason") != "compiler-artifact":
            continue
        target = message.get("target")
        if not isinstance(target, dict) or target.get("name") != FIXTURE_PACKAGE or target.get("kind") != ["bin"]:
            continue
        source = target.get("src_path")
        executable = message.get("executable")
        if not isinstance(source, str) or Path(source).resolve() != expected_source:
            raise InfrastructureError("Cargo reported a fixture binary with the wrong source path")
        if not isinstance(executable, str):
            raise InfrastructureError("Cargo reported a fixture binary without an executable path")
        candidate = Path(executable)
        if not candidate.is_absolute():
            raise InfrastructureError("Cargo reported a relative fixture binary path")
        try:
            resolved_candidate = candidate.resolve(strict=True)
            resolved_candidate.relative_to(owned_target)
        except ValueError as error:
            raise InfrastructureError("Cargo fixture binary is outside the owned build cache") from error
        except (OSError, RuntimeError) as error:
            raise InfrastructureError("Cargo fixture binary is missing or redirected") from error
        relative = None
        relative_root = None
        for root in (target_dir, owned_target):
            try:
                relative = candidate.relative_to(root)
                relative_root = root
                break
            except ValueError:
                continue
        redirected = relative is None or ".." in relative.parts
        current = relative_root
        if relative is not None and current is not None:
            for part in relative.parts:
                current = current / part
                if current.is_symlink():
                    redirected = True
                    break
        if redirected or not resolved_candidate.is_file():
            raise InfrastructureError("Cargo fixture binary is missing or redirected")
        candidates.append(resolved_candidate)
    if len(candidates) != 1:
        raise InfrastructureError("Cargo reported %d fixture binary artifacts, expected one" % len(candidates))
    return candidates[0]


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


def public_api_roots(output: str) -> Tuple[List[str], List[str]]:
    """Split the simplified API into curated re-export roots and forbidden items."""
    crate_name = FACADE_PACKAGE.replace("-", "_")
    crate_root = "pub mod %s" % crate_name
    prefix = "pub use %s::" % crate_name
    observed: List[str] = []
    unexpected: List[str] = []
    for raw_line in output.splitlines():
        line = raw_line.strip()
        if not line or line == crate_root:
            continue
        if line.startswith(prefix):
            name = line[len(prefix):].strip()
            if name:
                observed.append(name)
            else:
                unexpected.append(line)
        else:
            # A direct public fn/type/module/constant is outside the curated
            # re-export-only facade even when all pinned roots remain present.
            unexpected.append(line)
    return observed, unexpected


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
    write_bytes_if_changed(
        manifest,
        text.replace(FACADE_DEP_LINE, 'path = "%s"' % FACADE_DIR).encode(),
    )
    write_bytes_if_changed(fixture / "Cargo.lock", FIXTURE_LOCKFILE.read_bytes())
    write_bytes_if_changed(src / "main.rs", (FIXTURE_DIR / "src" / "main.rs").read_bytes())
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
        "fixture_exact_dep": 0,
        "facade_exact_dep": 0,
        "public_api_observed": 0,
        "public_api_extra": 0,
        "public_api_missing": 0,
        "public_api_direct": 0,
    }
    cache_lock: Optional[IO[str]] = None
    fixture: Optional[Path] = None
    positive_main: Optional[bytes] = None
    cargo_version = "unknown"
    rustc_version = "unknown"
    before_lock = ""
    before_vector = ""
    before_status = b""
    before_fingerprint: Dict[str, str] = {}

    try:
        # ---- stage 0: repository shape ------------------------------------
        before_lock = sha256_of(LOCKFILE)
        before_vector = sha256_of(VECTOR_PATH)
        before_status = git_status_bytes()
        before_fingerprint = gate_writable_fingerprint()

        fixture_deps = manifest_dependencies(FIXTURE_DIR / "Cargo.toml")
        counts["fixture_deps"] = len(fixture_deps)
        counts["fixture_exact_dep"] = int(
            exact_path_dependency(fixture_deps, FACADE_PACKAGE, FACADE_DIR)
        )
        if not counts["fixture_exact_dep"]:
            violations.append(
                "fixture must have exactly one unconditional facade path dependency, got %s"
                % fixture_deps
            )

        facade_deps = manifest_dependencies(FACADE_MANIFEST)
        counts["facade_normal_deps"] = len(facade_deps)
        counts["facade_exact_dep"] = int(
            exact_path_dependency(
                facade_deps,
                PIPELINE_PACKAGE,
                REPOSITORY_ROOT / "crates" / PIPELINE_PACKAGE,
            )
        )
        if not counts["facade_exact_dep"]:
            violations.append(
                "facade must have exactly one unconditional pipeline path dependency, got %s"
                % facade_deps
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
        env["QGLANG_V3_VECTOR_PATH"] = str(VECTOR_PATH)
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

        build = run(
            ["cargo", "build", "--locked", "--message-format=json"],
            cwd=fixture,
            env=env,
        )
        execution = None
        if build.returncode == 0:
            # The JSON artifact path follows Cargo's selected target/config;
            # a fixed build/debug path can run a stale binary instead.
            executable = positive_executable(build, fixture, CACHE_ROOT / "build")
            execution = run([str(executable)], cwd=fixture, env=env)
            counts["positive_executed"] = 1
        outcome, message = positive_outcome(build, execution)
        if outcome == "infrastructure":
            raise InfrastructureError(message)
        if outcome == "violation":
            violations.append(message)
            say(build.stderr.strip() if execution is None else execution.stderr.strip())
        else:
            counts["positive_marker"] = 1
        say(
            "stage 1: build_exit=%d consumer_program_executed=%d marker=%d"
            % (
                build.returncode,
                counts["positive_executed"],
                counts["positive_marker"],
            )
        )

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
            result = run(["cargo", "check", "--locked", "--quiet"], cwd=fixture, env=env)
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
        observed_public, direct_public = public_api_roots(result.stdout)
        counts["public_api_observed"] = len(observed_public)
        counts["public_api_direct"] = len(direct_public)
        public_extra = sorted(set(observed_public) - set(PINNED_ALLOWLIST))
        public_missing = sorted(set(PINNED_ALLOWLIST) - set(observed_public))
        counts["public_api_extra"] = len(public_extra)
        counts["public_api_missing"] = len(public_missing)
        if public_extra:
            violations.append("public API over-claims names outside the allowlist: %s" % public_extra)
        if public_missing:
            violations.append("public API is missing allowlisted names: %s" % public_missing)
        if direct_public:
            violations.append(
                "public API exposes direct items outside the re-export allowlist: %s"
                % direct_public
            )
        say(
            "stage 3: public_api observed=%d pinned=%d extra=%d missing=%d direct=%d"
            % (
                counts["public_api_observed"],
                counts["allowlist_pinned"],
                len(public_extra),
                len(public_missing),
                len(direct_public),
            )
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
                write_bytes_if_changed(fixture / "src" / "main.rs", positive_main)
            except OSError as error:
                infrastructure.append("could not restore the cached positive fixture: %s" % error)
        if cache_lock is not None:
            cache_lock.close()

    # ---- stage 4: the checkout is untouched --------------------------------
    lock_unchanged = "unknown"
    vector_unchanged = "unknown"
    status_unchanged = "unknown"
    gate_files_unchanged = "unknown"
    concurrent_paths: List[str] = []
    try:
        lock_unchanged = "unchanged" if sha256_of(LOCKFILE) == before_lock else "CHANGED"
        vector_unchanged = "unchanged" if sha256_of(VECTOR_PATH) == before_vector else "CHANGED"
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
    if vector_unchanged == "CHANGED":
        infrastructure.append("the shared V3 vector input changed during the gate run")
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
        "facade_manifest fixture_deps=%d fixture_exact=%d facade_deps=%d facade_exact=%d"
        % (
            counts["fixture_deps"],
            counts["fixture_exact_dep"],
            counts["facade_normal_deps"],
            counts["facade_exact_dep"],
        )
    )
    say(
        "public_api      observed=%d pinned=%d extra=%d missing=%d direct=%d"
        % (
            counts["public_api_observed"],
            counts["allowlist_pinned"],
            counts["public_api_extra"],
            counts["public_api_missing"],
            counts["public_api_direct"],
        )
    )
    say(
        "consumer_api   curated_roots=%d positive_programs=%d compiler=external_fixture"
        % (counts["allowlist_pinned"], counts["positive_executed"])
    )
    say(
        "repo_immutability cargo_lock=%s vector_input=%s gate_files=%s git_status=%s"
        % (lock_unchanged, vector_unchanged, gate_files_unchanged, status_unchanged)
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
