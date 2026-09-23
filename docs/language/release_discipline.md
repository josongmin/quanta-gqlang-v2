# Release Discipline

Current posture is a pre-release, read-only v3 frontend with fail-closed
unsupported inputs. This document does not assert a deployed or stable runtime.

Required statements:

1. read-only CLI/diagnostics shell only; no mutating command plane ships here
2. no runtime truth appears in public examples, docs, or CLI smoke output
3. catalog and presentation families carry checked frontend metadata/policy;
   no separate AI exploration API is shipped
4. unsupported product closure claims stay out of release notes and smoke tests
5. the V3 execution document is a frontend producer contract, not a physical
   plan, provider admission, executed result, or consumer qualification

Operator discipline:

1. examples must map to shipped roots only
2. docs must describe frontend-only ownership, not backend availability
3. local `just ci` covers examples, docs, CLI wiring, and owner tests; a clean
   committed-HEAD receipt requires `just qualify-local` separately
4. CLI ships a file-path and inline-query front door; malformed argv must fail closed with non-zero exit
5. the diagnostics stdio front door uses bounded private NDJSON, not LSP or JSON-RPC; unknown or mutating methods must fail closed
6. `examples_index.md` alone lists shipped example paths; README entrypoints
   link to it instead of duplicating the file list
7. a sibling consumer claim needs its own exact source pin and execution proof;
   qglang-local qualification cannot supply it
