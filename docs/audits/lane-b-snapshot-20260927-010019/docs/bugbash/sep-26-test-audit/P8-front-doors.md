# P8: front-door findings resolved in the working tree

## TA-01: error diagnostic identity

Before: `binary_contract.rs` accepted any nonempty diagnostics array for the malformed-source response.

Now: `semantic_diagnostics_are_a_successful_protocol_response` runs the shipped diagnostics process and pins one diagnostic's stage, code, severity, message, source ID, and span. The focused binary test passed. A temporary wrong-code conversion in `handlers/v3_diagnostics.rs` made that test fail; the mutation was restored.

## TA-05: canonical CLI output

Before: messy-input tests checked only nonempty output and `fmt(fmt(x)) == fmt(x)`, which an identity formatter could satisfy.

Now: `messy_declaration_has_canonical_binary_output_v1` requires the shipped `v3-fmt` process to convert `let  x =  1 +  2*3` to `let x = 1 + 2 * 3`. The focused binary test passed. A temporary validated-identity formatter made it fail; the mutation was restored. The existing bare-`fmt` rail compares its output with `v3-fmt` on a messy input.

TA-02 was rejected: a combined wrong-type `source` and unknown `extra` request reports the same wrong-type error as the request without `extra`; `extra` alone reports an unknown field.
