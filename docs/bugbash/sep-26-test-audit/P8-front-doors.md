# P8: current front-door test gap

## TA-01: error diagnostic identity is unchecked

- `crates/quanta-gqlang-v2-diagnostics-stdio/tests/binary_contract.rs:42-61`: `semantic_diagnostics_are_a_successful_protocol_response` now requires a nonempty `diagnostics` array. It does not check count or entries, so a wrong nonempty list still satisfies the test.
- `crates/quanta-gqlang-v2-diagnostics-stdio/src/handlers/v3_diagnostics.rs:13-48` serializes diagnostics from the document failure into an unrestricted `Vec`. The nonempty assertion at `tests/ndjson_contract.rs:149` exercises an incomplete-analysis fixture, not this error path.
- Current binary observation for the existing fixture: one diagnostic, code `surface.parse.unexpected_token`, source ID `1`, span `0..7`. Add those response assertions and verify the test fails if the code changes while the list stays nonempty.

No production response failure was observed. The existing focused binary test passes with this partial oracle.

The former TA-02 combined-invalid-fields claim is withdrawn: the current binary reports `invalid type: sequence, expected a string` both with and without the trailing `extra` field; `extra` alone reports an unknown field.

## TA-05: the CLI formatter has no noncanonical output oracle

- `crates/quanta-gqlang-v2-cli/tests/v3_cli_fmt_idempotence.rs:120-159` feeds messy inputs through the shipped binary and checks nonempty output plus `fmt(fmt(x)) == fmt(x)`. An identity formatter satisfies both. The shipped-example rail also checks only nonempty formatted output; `command_doc_gate.rs:114-134` uses already-canonical `let x = 1`.
- The pipeline library rail pins canonical output (`pipeline/tests/v3_compiled_document_assembly.rs:2302-2311`), but it does not exercise the CLI wrapper at `cli/src/commands/v3_format.rs:19`.
- Current binary observation for `let  x =  1 +  2*3` is `let x = 1 + 2 * 3`. Assert this exact output in one binary test. A CLI wrapper that returns validated input unchanged must fail.
