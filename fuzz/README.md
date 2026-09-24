# Surface fuzzing

`surface_document` exercises parser and formatter totality on byte-derived
UTF-8 text up to 4096 bytes. Successful parses must produce integrity-checked
HIR, the same visitor-derived structure after formatting, and a stable canonical
format; rejects must carry source-bounded typed
diagnostics. This is an exploratory rail outside `just ci` and outside the
clean-HEAD qualification receipt.

Run a bounded local investigation with the installed nightly toolchain:

```sh
cargo +nightly fuzz run surface_document -- -max_total_time=60
```

The checked-in corpus is seed input, not expected output. Keep generated
`fuzz/artifacts/` and coverage data local. Minimize a failure, inspect the
semantic expectation independently, then add a permanent `.qglang`/`.expect`
pair under the surface specification corpus or a focused owner regression.
Do not treat an unfinished fuzz run as qualification evidence.
