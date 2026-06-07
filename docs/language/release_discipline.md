# Release Discipline

Release posture for this repo is parser-stable, read-only, and fail-closed.

Required statements:

1. read-only shell only; no mutating command plane ships from this repo
2. no runtime truth appears in public examples, docs, or CLI smoke output
3. experimental catalog and experimental AI-adjacent surfaces are metadata-only
4. unsupported product closure claims stay out of release notes and smoke tests

Operator discipline:

1. examples must map to shipped roots only
2. docs must describe frontend-only ownership, not backend availability
3. release smoke covers examples, docs, and CLI wiring together
4. CLI ships a file-path and inline-query front door; malformed argv must fail closed with non-zero exit
5. LSP ships a stdio JSON front door; unknown or mutating methods must fail closed
6. docs and README must enumerate the shipped example families that actually exist in `examples/`
