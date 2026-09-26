# D-04: total document membership rejection

Base: `dbadf778b4bc8dd6e1062fe34e3ef57e0a364833`.

The existing test-only membership seam reproduced D-04: retaining an authored
rule identity while clearing all semantic items caused Logic to return an empty
successful rule contract. The exact one-test regression failed with "got 0
accepted member(s)". Normal semantic assembly prevents that malformed state;
this is a downstream validation contract defect, not evidence of an exploitable
source-language ingress.

The cause was that rejection depended on finding a surviving item's checked
location. Sema now retains a byte-zero checked location from the transaction's
original extended source registry, through declaration and final assembly.
Logic's membership anchor is total, so a missing authored identity always
returns `logic.rule_program.missing_member`. A genuinely empty membership still
returns an empty rule contract.

Planning also consumes that retained anchor for itemless-document diagnostics.
Previously it created a new one-source, zero-length registry merely to mint a
diagnostic span. The public source ID and byte span looked correct, but the
checked registry identity differed from the original document. A separate exact
regression fails against that old implementation and passes with the retained
anchor. No replacement registry or new panic path remains in this fallback.

Owner regressions pin source identity, byte-zero span, original registry identity,
typed rejection/no-plan code, and empty-document positive controls. Sema tests
zero and nonzero source lengths with distinct source IDs and no items. The Logic
adapter suite passes 26 tests. The two baseline regressions each selected and
failed exactly one test; the first abbreviated Logic selector selected zero and
is explicitly excluded from evidence. Raw baseline outputs are retained under
`target/document-anchor-audit/` in the owner worktree.

The new `SemanticDocumentV2::source_anchor()` accessor is an internal-stage
carrier accessor, not a new facade export. Execution DTO fields, source syntax,
digest schemas and valid nonempty-document behavior are unchanged. Full clean
qualification is recorded separately from these focused reproductions.
