# B-F3: one string escape authority; B-lane residual disposition

Base: `f01ba38aa53736da79247970f5108e812fae90df`.

## Finding and acceptance

The archived Lane B report identified a scanner/decoder synchronization risk.
The current scanner validated escape spelling and Unicode scalars, but the
independent decoder silently ignored unknown escapes, malformed Unicode and
truncated escape tails. Scanner admission made those decoder cases unreachable
from ordinary source lexing; there was no demonstrated normal-input silent drop.
A later vocabulary change could nevertheless admit text that the decoder erased.

Acceptance: one escape grammar shared by the allocation-free admission scan and
decoding; typed rejection instead of dropped content, even when the private
decoder is tested without a preceding scanner; unchanged byte spans and valid
payloads; token allowance charged before allocating/copying decoded payload.

## Implementation

`walk_string` owns ordinary character emission, all seven short escapes, Unicode
hex syntax/scalar validation and unterminated-string rejection. `scan_string`
uses a no-op emitter. After token admission, `decode_string` uses an emitter into
the bounded-capacity result string and propagates the same typed failure.
The independent decoder escape table and Unicode digit-buffer allocation are
removed. Invalid partial content never becomes a token. The input is visited
twice as before; no timing or memory benchmark improvement is claimed.

## Evidence

- Literal expected payloads cover all short escapes, direct multibyte text,
  empty strings and Unicode scalar boundaries 0, 7f, 80, d7ff, e000 and 10ffff.
- Eleven invalid forms independently require exact code, message and byte span
  from both scanner and decoder, including an initial multibyte prefix. Cases
  cover unknown ASCII/multibyte escapes, malformed Unicode syntax, seven digits,
  nonhex digits, surrogate/out-of-range scalars, absent braces/quotes and a
  dangling escape. Decoder rejection does not rely on prior scanner execution.
- The existing payload-allocation control proves successful allocation is
  observable and token-budget refusal precedes copying/decoding a long payload.
- Four compiling mutants restore the old lossy decoder (only its signature is
  adapted to Result), change newline to carriage return, drop the Unicode
  scalar, or allocate the payload before the token charge. Every mutant fails
  a selected runtime assertion. None is counted from a compile failure.
- Restored focused runs: 7 Surface library tests, 14 format-edge tests and
  31 formatter hellgate tests pass. Full qualification is a separate clean-HEAD
  receipt. Raw mutation logs: owner worktree `target/string-escape-audit/`.

## Remaining B-lane claims

| ID | Current-source disposition |
| --- | --- |
| B-F1 | Root dispatch, relation-stage dispatch and bare-alias restrictions are different grammar contexts. Their vocabularies are not required to be equal. Parsing and formatting already share query::is_reserved. Literal controls preserve find/sample/cross as bare aliases; existing explicit-reserved-alias rails cover all restricted names. Blindly taking the keyword union would change valid source grammar. Future grammar changes must explicitly decide each affected role. No current misparse was demonstrated. |
| B-F2 | Documented the actual composition prefix rule: immediately after `let name =`, unparenthesized from/sample/find selects composition. Parentheses admit these spellings as value symbols. The new test checks literal symbol payloads and canonical text, plus typed ParseRejected for raw declarations that render to the ambiguous prefix. No grammar behavior changes. |
| B-F3 | Structurally closed by the shared walker and rejection controls above. |
| B-F4 | Raw HIR remains intentionally constructible; the checked source transaction enforces HIR integrity before Sema. This is documented in frontend-owner-boundaries.md and protected by HIR integrity and facade privacy rails. It is not a second closed-compilation authority. |
| B-F5 / A2-6 | The formatter precedence table can describe raw HIR operators not admitted by the lexer. Checked formatting rejects unrepresentable trees. No source admission or executable semantics are inferred from a precedence-table entry. Optional cleanup, not an observed failure. |

The archived B report is a historical, partial snapshot audit; its test counts
and uninspected-file notes do not establish current qualification. This report
reconciles its named findings only and does not claim every frontend behavior is
proved merely because these tests pass.
