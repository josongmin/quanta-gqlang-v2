# Sep 26 transport byte-boundary challenge

Base: `78f477056cfbc726441d298fe243d3d34efb566f` plus the previous
uncommitted audit improvements. This challenge targets the public diagnostics
transport's byte budgets and EOF behavior, then checks representative HIR
payload regressions through the surface consumer. It is a separate evidence
set from the earlier audit receipts.

## Three demonstrated gaps

Each production mutation compiled and passed the diagnostics-stdio package
suite before its targeted oracle was added:

| Mutation | New independent expectation |
|---|---|
| Source cap counts Unicode scalars instead of decoded UTF-8 bytes | A 17-byte/12-scalar source, 18-byte exact control and 19-byte rejection at cap 18; raw and JSON-escaped representations of the same decoded source produce identical responses apart from ID; a following empty-source request succeeds |
| Response cap counts Unicode scalars instead of serialized UTF-8 bytes | Literal Unicode ID/method error response is exactly 116 bytes and 101 scalars; cap 116 admits exact wire bytes, cap 115 rejects and the session emits the exact bounded ASCII fallback |
| Oversized final frame silently disappears at EOF | A 257-byte frame at cap 256, without a terminal newline, produces exactly one frame-limit error; exercised alone and after a valid request, with ordinary and one-byte reads |

These are regression-detection gaps. Normal production code already enforces
the correct behavior, so this change adds tests rather than altering transport
semantics. The EOF mutation's package run included the two new UTF-8 tests;
they do not cover frame EOF behavior and correctly did not kill that mutation.

The source cap is over the decoded source, while the response cap is over the
serialized response bytes. Neither is a JSON wire character count. The EOF
case matters because overflow clears the frame buffer; the retained overflow
flag must independently trigger its error when the reader ends.

## Controls and final validation

The three target tests reject their respective compiling mutations after the
patch. Representative HIR Int/String payload erasure and EXPLAIN evidence
defaulting are challenged through existing surface exact-render assertions.
The string mutation survives HIR/Types owner suites and the non-string literal
render test, but `string_escapes_round_trip` rejects it. The non-string test
comment is corrected to name its actual scope; no duplicate builder rail is
added when the consumer already rejects the regression.

`target/sep26-byte-boundary-audit/` preserves individual commands, source and
log hashes, terminal exits and byte restoration:

- `before-results.json` and `overflow-eof-dropped-before.json`: surviving probes.
- `after-results.json`: post-patch mutations and existing consumer controls;
  the first narrow string probe is retained as a survivor, then attributed to
  the string-specific consumer rail.
- `test-authority.json`: new fixture and existing consumer test hashes.
- `hir-string-owner-suite-before.json`: narrow HIR/Types survivor, excluded
  from the three retained gaps because the surface consumer rejects it.
- `ci-result.json` and `ci.log`: final `just ci` on the restored frozen source.

Whole-repository completion requires the actual CI command to exit 0, matching
before/after source snapshots, and reconciled current-run selected/executed/pass
counts. Earlier full CI results do not qualify this later patch. Unrelated
prompt-manager changes in the shared checkout are preserved. The receipt is
dirty-source local verification, not clean committed-HEAD or hosted CI proof.
