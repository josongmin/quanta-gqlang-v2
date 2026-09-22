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
