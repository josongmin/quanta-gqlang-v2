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

For leaf timings on the next run, use `JUST_TIME=true just qualify-local`. Run
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
marker, restores the positive fixture before each run, and refuses redirected
or unowned cache state. Cargo parses the complete dependency tables before
the gate accepts the facade-only path seam; the positive program reads selected
digests from the producer-owned V3 vector TSV. Its public-API probe reuses the
same checkout's Cargo target. The standalone gate compares its own input bytes
and Git status shape, but cannot detect an in-place content edit to every
already-dirty foreign file. Only `just qualify-local` attests a clean exact
committed source snapshot. Both caches are per-checkout; moving or sharing them between checkout
paths is not supported because Rust artifacts may embed source paths.
These two samples are evidence of reuse, not a guaranteed runtime or a reason
to skip any gate. The small Cargo smoke separately proves that switching a
source commit at the stable checkout path rebuilds a changed Rust crate.
