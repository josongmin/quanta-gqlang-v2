# P3 formatter production closure — 2026-09-27

The adversarial audit of `9bd67dd` established two production defects: reserved
relation aliases lost their necessary `as`, and successful source formatting
could emit text rejected by the same caller budget. These are separate from the
earlier witness and deep-child test-oracle fixes.

The relation renderer now shares the parser's reserved-word predicate and emits
explicit `as` for those aliases. Ordinary bare-alias canonical text is preserved.
All 21 reserved spellings are pinned across direct, inner/CROSS join RHS,
nested EXISTS, inline VIEW, composition producer and composition body positions.
Each fixture has independently expected canonical bytes, checked-formatter
acceptance and an independent visitor witness comparison after re-parsing.

The surface bounded source formatter re-parses canonical output using the same
parse budget before returning success. The pipeline delegates to that owner and
also checks output source bytes. Generated-output failures retain their typed
code/resource count but anchor on the original source ID and whole document
span. Input failures retain their original exact diagnostic. No second parser or
estimated token counter is introduced. Validation adds one bounded linear parse
pass; infallible HIR renderers continue to render without an output-budget check.

CLI output includes a transport newline only when it fits the source-byte cap.
An exact-cap document is emitted without that extra byte, so successful stdout
remains valid input to the same file front door. Overflow exits with a resource
error and empty stdout; no success-shaped artifact is emitted.

Eight new regression tests cover both production defects, exact/cap+1 boundaries,
source attribution and actual CLI behavior. The owner-focused checks passed.
Five isolated compiling negative controls also failed by executed assertions:
removing alias `as`, output parse validation, diagnostic re-anchoring, output
byte validation, and restoring an unconditional exact-cap CLI newline. Temporary
mutations were restored and were never applied to the shared source checkout.

Full local CI evidence is recorded separately under
`target/sep27-p3-format-closure/`. Only a completed, source-stable CI receipt can
establish repository-wide local completion. Dirty source is not a clean committed
HEAD qualification; no downstream execution or hosted CI claim is made here.
