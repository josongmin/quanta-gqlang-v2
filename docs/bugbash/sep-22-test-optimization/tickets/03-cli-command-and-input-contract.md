# 03 — CLI Command And Input Contract

Status: `FOCUSED_GREEN (see receipts/03.md)`

Findings owned: `CLI-01`, `CLI-02`.

## Goal

Give advertised command tokens one CLI-owned authority, prove every advertised alias dispatches at
the shipped binary boundary, and cover the existing empty-input guard through both ingress modes.

## Write Set

- `crates/quanta-gqlang-v2-cli/src/lib.rs`
- `crates/quanta-gqlang-v2-cli/src/main.rs`
- `crates/quanta-gqlang-v2-cli/tests/command_doc_gate.rs`
- `crates/quanta-gqlang-v2-cli/tests/v3_cli_fmt_idempotence.rs`
- `crates/quanta-gqlang-v2-cli/tests/open_analysis_contract.rs`

Do not edit frontend-contract, command implementations, language docs, manifests, or workflow files.

## `CLI-01` — Command Authority

Current cause: `CLI_COMMAND_TOKENS_V1`, the literal usage alternation, and the binary dispatch match
encode the same four tokens independently. `contains("fmt")` also accepts `v3-fmt` as proof for
the missing bare token.

Required implementation:

1. Define one CLI-library command specification table mapping exactly
   `query`, `fmt`, `v3-query`, and `v3-fmt` to the two canonical command kinds.
2. Derive exported token iteration and binary command resolution from that table. Do not keep a
   second handwritten dispatch token match.
3. Render the usage alternation from the same ordered table. Preserve the rest of the stable
   key/value usage output.
4. Keep canonical and compatibility aliases behaviorally identical; do not add commands.

If preserving `CLI_COMMAND_TOKENS_V1` is required for source compatibility, derive it from the
same spec definitions or make it a view with a test proving exact equality. A copied four-element
list is not a root fix.

## `CLI-01` Oracles

- Parse only the `<...>` command alternation from the emitted usage line and set-compare it to the
  advertised token set. Assert cardinality, uniqueness, and no extras.
- Invoke all four tokens through the compiled binary with a minimal valid input.
- For each alias pair, assert equal exit code and normalized stdout/stderr contract.
- Keep README documentation coverage, but match tokens as code-delimited command entries or an
  exact parsed command section—not arbitrary substring presence.
- Mutation: delete bare `fmt` from usage or dispatch while retaining `v3-fmt`; the gate must fail.

## `CLI-02` — Empty Ingress

The production trim-empty guard already exists in `parse_invocation`; this finding needs boundary
proof, not a second validation layer.

Add table-driven binary cases:

- inline whitespace-only input;
- zero-byte file;
- whitespace-only file.

Each case asserts exit code `2`, empty stdout, error class `usage`, exact message
`input cannot be empty`, and one usage block. Use a collision-resistant temporary path and cleanup
even on assertion failure through the existing test support/RAII pattern where available.

Positive neighbors: one non-empty inline and one non-empty file continue past usage validation.

Mutation: remove or invert `input.trim().is_empty()`; all empty cases must fail without relying on
a later parser error.

## Exact Verification

```bash
cargo test -p quanta-gqlang-v2-cli --test command_doc_gate
cargo test -p quanta-gqlang-v2-cli --test v3_cli_fmt_idempotence
cargo test -p quanta-gqlang-v2-cli --test open_analysis_contract
cargo test -p quanta-gqlang-v2-cli --all-targets
cargo clippy -p quanta-gqlang-v2-cli --all-targets -- -D warnings
```

## Definition Of Done

- One command spec owns token enumeration, usage rendering, and dispatch resolution.
- Exact set comparison rejects omissions, duplicates, and extras.
- Every token is exercised at the binary boundary.
- Inline and file empty inputs fail at the shared ingress guard with exact transport output.
- No runtime/provider/backend behavior enters the CLI contract.

