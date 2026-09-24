# Local qualification cache

`just qualify-local` qualifies a clean committed HEAD in a detached, managed
worktree at `target/qualification/managed-checkout`. Its `target` symlink points
to `target/qualification/managed-build-cache`. Both paths remain fixed between
runs, so Rust artifacts that embed `CARGO_MANIFEST_DIR` cannot be reused from a
different checkout path. Git/Cargo still recheck changed source and toolchains.

The driver takes a non-blocking OS file lock, pins HEAD and tree, refuses a
dirty caller or managed checkout, and checks both again after `just ci`. It
refuses an unknown worktree, marker, or redirected cache; it never resets or
deletes a dirty checkout. The per-SHA `receipt.json` keeps the existing
`sha`, `tree`, `cmds`, `exit`, and `duration_s` schema. A warm cache is build
acceleration, **not** proof from a prior SHA: `just ci` still executes on every
qualification. Old `target/qualification/<sha>/build` directories from the
former cold-cache driver are not automatically removed.

Local gate subprocesses use bounded process groups. Front-door parity gives
its Cargo build 600 seconds and each shipped binary 20 seconds; the external
consumer gate gives Cargo calls 300 seconds and other calls 30 seconds.
`qualify-local` gives `just ci` 1800 seconds. A timeout terminates the spawned
process group, fails closed (qualification exit 2), and still records the
post-run source check and receipt. These are hang limits, not performance
targets or a substitute for recording actual gate durations.
An interrupted gate also terminates and reaps its process group before the
interrupt is propagated.

`just ci` runs the external-consumer Cargo offline dependency preflight before
the build and test groups. It checks both the external-consumer fixture and
repository workspace locked dependency graphs against the pinned toolchain;
it does not compile or execute either graph. A missing crate exits before the
expensive gates. Populate the local Cargo cache for the missing graph with
`CARGO_NET_OFFLINE=false cargo fetch --locked --manifest-path tools/frontend_external_consumer/Cargo.toml`
or `CARGO_NET_OFFLINE=false cargo fetch --locked --manifest-path Cargo.toml`,
then run `just ci` again. The preflight is not evidence that the external
consumer gate passed; the full gate still runs in `just policy`.

`qualify-local` always emits Just recipe timestamps and durations in its log.
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
Its public-API probe reuses the same checkout's Cargo target. The standalone
gate compares its own input bytes and Git status shape, but cannot detect an
in-place content edit to every already-dirty foreign file. Only
`just qualify-local` attests a clean exact committed source snapshot. Both
caches are per-checkout; moving or sharing them between checkout paths is not
supported because Rust artifacts may embed source paths.
These two samples are evidence of reuse, not a guaranteed runtime or a reason
to skip any gate. The small Cargo smoke separately proves that switching a
source commit at the stable checkout path rebuilds a changed Rust crate.
