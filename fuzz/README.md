# Surface fuzzing

`surface_document` exercises parser and formatter totality on byte-derived
UTF-8 text up to 4096 bytes. Successful parses must produce integrity-checked
HIR, the same visitor-derived structure after formatting, and a stable canonical
format; rejects must carry source-bounded typed
diagnostics. This is an exploratory rail outside `just ci` and outside the
clean-HEAD qualification receipt.

Run a bounded local investigation with the pinned installed nightly toolchain:

```sh
just fuzz-smoke
```

For a solo macOS checkout, run this once per week from the repository root
with a local scheduler such as launchd. The recipe requires `cargo-fuzz 0.13.1`
and `nightly-2026-09-23`, runs offline for 60 seconds, and sets a five-second
per-input timeout. Its growing corpus, crash artifacts, and bounded log stay
under `target/fuzz-smoke/`; it copies only tracked seeds from `fuzz/corpus/`.
Keep the scheduler's working directory at this checkout and retain the exit
status. Missing tools, seeds, or dependencies fail the run. Fuzzing is absent
from `just ci` because its coverage-guided result is nondeterministic.

The checked-in corpus is seed input, not expected output. Keep generated
`fuzz/artifacts/` and coverage data local. Minimize a failure, inspect the
semantic expectation independently, then add a permanent `.qglang`/`.expect`
pair under the surface specification corpus or a focused owner regression.
Do not treat an unfinished fuzz run as qualification evidence.
