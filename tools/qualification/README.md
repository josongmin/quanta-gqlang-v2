# Local qualification cache

`just qualify-local` qualifies a clean committed HEAD in a detached, managed
worktree at `target/qualification/managed-checkout`. Its `target` symlink points
to `target/qualification/managed-build-cache`. Both paths remain fixed between
runs, so Rust artifacts that embed `CARGO_MANIFEST_DIR` cannot be reused from a
different checkout path. Git/Cargo still recheck changed source and toolchains.

The driver takes a non-blocking OS file lock, pins HEAD and tree, refuses a
dirty caller or managed checkout, and checks both again after `just ci`.
Tracked file bytes, symlink targets, and executable bits are compared directly
with HEAD blobs, independently of Git index flags such as `assume-unchanged`. It
refuses an unknown worktree, marker, or redirected cache; it never resets or
deletes a dirty checkout. The per-SHA `receipt.json` is schema 3: it retains
`sha`, `tree`, `cmds`, `exit`, and `duration_s`, and records tool versions, input
hashes, the observed post-run dirty state (`false` is required for success), per-recipe durations,
current-run nextest counts with zero skipped/ignored cases, advisory DB revision
and fetch time, and a hash of the streamed `ci.log`. A warm cache is build
acceleration, **not** proof from a prior SHA: `just ci` still executes on every
qualification. Old `target/qualification/<sha>/build` directories from the
former cold-cache driver are not automatically removed.

Nextest inventory and terminal event streams are parsed and reconciled
fail-closed during the run. The qualification receipt archives both raw streams
as `nextest-list.json` and `nextest-events.jsonl`, and checks their SHA-256 values
against same-run gate evidence before accepting success. Direct `just ci` also
rechecks both streams after the policy group, so a later gate cannot silently
replace them. The Nextest component
can therefore be independently replayed from the receipt. Other gates still
have their own evidence boundaries; the receipt is not a complete replay of
every command in `just ci`.

Local gate subprocesses use bounded process groups. Front-door parity gives
its Cargo build 600 seconds and each shipped binary 20 seconds; the external
consumer gate gives Cargo calls 300 seconds and other calls 30 seconds.
`qualify-local` gives `just ci` 1800 seconds. A timeout terminates the spawned
process group, fails closed (qualification exit 2), and still records the
post-run source check and receipt. These are hang limits, not performance
targets or a substitute for recording actual gate durations.
An interrupted gate also sends `SIGKILL`, reaps its direct child, and waits a
bounded second for the process group to disappear before the interrupt is
propagated. If the group remains observable, cleanup fails closed instead of
claiming that every descendant was reaped.

`just ci` runs doctor, external-consumer preflight, static, test, and policy
groups serially, and each group's leaves are an explicit serial waterfall.
Before running groups it rejects any indexed path excluded by a repository
`.gitignore`, including force-added files. This check uses repository ignore
files only, not personal global or `.git/info/exclude` rules. Put local audit
snapshots and repro build trees under the ignored `.audit/` directory; keep
durable audit reports and source provenance under `docs/audits/`. Ignoring a
path does not remove it from Git: existing generated files must be removed
from the index while preserving the local evidence when it is still needed.
It removes caller-controlled Rust/Cargo/Nextest/property-test settings and
`QGLANG_*` test-child selectors, then restores only the locked Rust toolchain
and current run identities before launching groups. Python gates start with
`-I -S`; pinned user-site packages enter `sys.path` only after interpreter
startup, so caller `PYTHONPATH` and startup hooks cannot preempt a gate.
The full gate uses an owned Cargo home under `target/ci/` with no caller Cargo
config. It shares offline registry/git caches and copies the advisory database.
It hashes every tracked and non-ignored untracked source path, its change time,
and containing-directory change times before the first group and rejects any
between-group source drift, including an edit reverted before a group ends.
The preflight checks both the external-consumer fixture and
repository workspace locked dependency graphs against the pinned toolchain;
it does not compile or execute either graph. A missing crate exits before the
expensive gates. Populate the local Cargo cache for the missing graph with
`CARGO_NET_OFFLINE=false cargo fetch --locked --manifest-path tools/frontend_external_consumer/Cargo.toml`
or `CARGO_NET_OFFLINE=false cargo fetch --locked --manifest-path Cargo.toml`. If the
dated public-API toolchain alone is missing sources, use
`CARGO_NET_OFFLINE=false RUSTUP_TOOLCHAIN=nightly-2026-09-23 cargo fetch --locked --manifest-path Cargo.toml`,
then run `just ci` again. The full external gate consumes a single-use stamp
bound to that CI run ID, both Cargo.lock hashes, the toolchain, and gate source;
standalone `just external-consumer` performs its own preflight. The preflight
is not evidence that the external consumer gate passed.

`just advisory-refresh` explicitly refreshes cargo-deny's active RustSec DB.
The routine `just deny` gate requires a fetch within seven days and runs with
network disabled. It also requires the official DB commit time to be within
seven days, so touching an old `FETCH_HEAD` cannot make an old revision fresh.
It rejects a dirty database or hidden index paths, requires checked-out HEAD to
match the fetched revision, requires the official RustSec advisory DB origin,
and proves that identity did not change across the scan. Qualification accepts
only same-run deny evidence and records its hash, origin, revision, and fetch
time in the receipt. `~/.cargo/advisory-dbs` (plural) is cargo-deny 0.19.7's active DB
directory; the older singular `advisory-db` directory is not this gate's DB.

`qualify-local` always emits Just recipe timestamps and durations in its log,
and a successful receipt requires the exact `ci`/group/leaf timing inventory.
The nextest evidence additionally records list, run, and event parsing durations
so warm-cache delays can be attributed without changing test selection or counts.
For the same trail from a direct local gate, use
`JUST_TIME=true JUST_TIMESTAMP=true just ci`. A failed or timed-out gate has no
automatic retry. Preserve the first failing leaf and exit code; run the leaf
alone once after a concrete environment change, then run the full gate once to
establish a new result. A binary timeout only proves that the process did not
finish within its bound. It does not by itself identify dyld, host load, or
application code as the cause.

Run
`PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tools/qualification/tests`
for the quick custody and exact-receipt tests. The separate Cargo rebuild smoke
is run with:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.qualification.tests.test_qualify_local.QualificationDriverTest.smoke_changed_source_rebuilds_at_the_same_manifest_path
```

It is excluded from routine `just ci` because it performs two real Rust builds.

Observed on the implementation commit `ee8a4b6ee5204b0cf227974baa6624c7bb6c6969`
with `JUST_TIME=true` on 2026-09-23: the first empty-cache `just ci` took 231 s;
the same-SHA warm run took 137 s (both exit 0). Clippy and nextest reused
artifacts; front-door/Cargo lock contention varied between runs. The external
consumer gate now keeps a separate fixture and build cache at the fixed
`target/external-consumer-gate/` path. It locks this path, checks an owner
marker, restores drifted fixture bytes without rewriting unchanged files, and
refuses redirected or unowned cache state. Preserving unchanged mtimes keeps
warm Cargo artifacts reusable. The locked build executes only the exact absolute
binary path emitted by Cargo for that fixture and rejects missing, redirected,
duplicate, relative, or cache-external artifacts. Cargo parses the complete
dependency tables before the gate accepts the facade-only path seam; the
positive program reads selected digests from the producer-owned V3 vector TSV.
The curated public roots and privacy probes are pinned in
`tools/test_frontend_external_consumer.py` and checked against the current
facade. The gate reconciles selected and executed probe counts. The negative
probes are stable Cargo bins checked in one `--keep-going`
JSON run, then reconciled by exact target source, error code, and diagnostic
text. Its public-API probe uses a target directory isolated by the pinned dated
nightly. The standalone
gate compares its own input bytes and Git status shape, but cannot detect an
in-place content edit to every already-dirty foreign file. Only
`just qualify-local` attests a clean exact committed source snapshot. Both
caches are per-checkout; moving or sharing them between checkout paths is not
supported because Rust artifacts may embed source paths.
These two samples are evidence of reuse, not a guaranteed runtime or a reason
to skip any gate. The small Cargo smoke separately proves that switching a
source commit at the stable checkout path rebuilds a changed Rust crate.
