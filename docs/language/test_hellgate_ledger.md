# Test hellgate ledger

This ledger maps each DSL surface and root family in `quanta-gqlang-v2` to the
proof rails that guard it. It is the precise companion to the
[Proof rails section](./README.md#proof-rails-test-hellgates) of the language
README.

**Scope.** syntax, sema, normalized IR, the CLI front door, and the LSP stdio
front door. Everything here proves frontend determinism and fail-closed
behavior only.

**Out of scope.** Runtime execution, provider/registry routing, result counts,
host DTO materialization, mutable edits, and backend closure. No rail in this
ledger asserts any of these as real.

## Rail kinds

- **Binary e2e** — spawns the real shipped binary
  (`CARGO_BIN_EXE_quanta-gqlang-v2-cli` / `...-lsp`), feeds front-door bytes, and
  asserts stdout / stderr / output channel / process exit code. Only a spawned
  process can pin an exit code or a channel.
- **Contract-only** — an in-crate library test against the public API. Pins
  typed shapes, diagnostic codes, normalization, and fail-closed branches. Makes
  no claim about exit codes or output channels.

A surface marked "— (contract-only)" in the Binary e2e column is intentionally
**not** proven at the binary; its only rail is the library contract.

## CLI front door

| Surface / root family | Binary e2e rail (file) | Contract-only rail (file) | Notes |
| --- | --- | --- | --- |
| Shipped positive corpus (`parse` / `sema` / `query` / `fmt` / `headless` matrix over the 17 examples) | `crates/quanta-gqlang-v2-cli/tests/examples_binary_frontdoor_matrix.rs` | `crates/quanta-gqlang-v2-cli/tests/examples_core_smoke.rs`, `examples_declaration_a_smoke.rs`, `examples_declaration_b_smoke.rs`, `examples_surface_smoke.rs` | Binary matrix is the round-trip rail; the `*_smoke.rs` set adds per-family library coverage. |
| Release-docs / shipped-example listing invariants | `crates/quanta-gqlang-v2-cli/tests/examples_release_smoke.rs` | — | Asserts the language constraint substrings and that all 17 example paths are listed in `README.md` and `docs/language/README.md`. |
| Usage / help channel + exit-code contract | `crates/quanta-gqlang-v2-cli/tests/cli_help_usage_contract.rs` | — | help → stdout / exit 0; usage error → stderr / exit 2; canonical usage block echoed. |
| Empty / whitespace-only input | `crates/quanta-gqlang-v2-cli/tests/cli_empty_input_frontdoor.rs` | — | Blank document fails closed (exit 2, usage), never reaches a command. |
| Transport parity (inline argv ≡ `--file`) | `crates/quanta-gqlang-v2-cli/tests/cli_transport_parity.rs` | — | Byte-identical stdout and same exit code for both transports. |
| `doctor` three-way classification (parse-error / sema-error / no-result) | `crates/quanta-gqlang-v2-cli/tests/doctor_frontdoor_hellgate.rs` | `crates/quanta-gqlang-v2-cli/tests/doctor_shell.rs` | Binary rail promotes the three lanes (incl. the previously binary-untested `sema-error`) to real exit codes / channels. |
| Negative-DSL fail-closed (malformed roots) | `crates/quanta-gqlang-v2-cli/tests/cli_negative_dsl_hellgate.rs` | `crates/quanta-gqlang-v2-syntax/tests/parser_conformance.rs` (diagnostic codes) | Binary rail: every malformed root exits 1; library rail pins the parser diagnostic per root. |
| Intra-root variant front door (SELECT operators / negative-int literal, EXPORT kinds, SAMPLE modes round-trip; negative-`LIMIT` fail-closed) | `crates/quanta-gqlang-v2-cli/tests/dsl_variant_frontdoor.rs` | `crates/quanta-gqlang-v2-syntax/tests/select_filter_matrix.rs`, `interactive_variant_matrix.rs` | Promotes the per-variant matrices to the binary: each headline variant survives the full `parse → sema → IR → render` stack and round-trips through `fmt`; the `LIMIT` non-negative guard fails closed (exit 1, command channel, `invalid_limit_value`). |
| `headless` read surface | — (contract-only) | `crates/quanta-gqlang-v2-cli/tests/cli_headless_read.rs` | `headless` is also exercised by the binary matrix corpus; this file is the dedicated library rail. |
| Zero-query / no-result shell | — (contract-only) | `crates/quanta-gqlang-v2-cli/tests/zero_query.rs`, `zero_query_handshake.rs` | No-result classification also asserted via `doctor` binary rail above. |
| `USE PACK` meta root | — (contract-only) | `crates/quanta-gqlang-v2-cli/tests/use_pack_cli.rs` | |
| Explain / path-witness UX | — (contract-only) | `crates/quanta-gqlang-v2-cli/tests/explain_path_ux.rs` | Presentation-only; no runtime path execution. |

## LSP stdio front door

| Surface | Binary e2e rail (file) | Contract-only rail (file) | Notes |
| --- | --- | --- | --- |
| Read-only method matrix + same-session recovery (`hover` / `completion` / `symbols` / `diagnostics` / `zero_query` / `quickfix`) | `crates/quanta-gqlang-v2-lsp/tests/stdio_read_only.rs` | `crates/quanta-gqlang-v2-lsp/tests/read_only_lsp.rs`, `read_only_handshake.rs` | Binary rail proves session recovers after fail-closed frames. |
| Stdio framing edges (EOF, glued frames, incomplete-at-EOF) | `crates/quanta-gqlang-v2-lsp/tests/stdio_frame_edges.rs` | — | Locks `extract_next_request_frame_v1` byte-boundary behavior at the binary. |
| JSON escape / surrogate-pair rejection | `crates/quanta-gqlang-v2-lsp/tests/stdio_json_escape_negative.rs` | — | Drives the hand-rolled `parse_unicode_escape_v1` error branches. |
| Multi-statement first-statement contract | `crates/quanta-gqlang-v2-lsp/tests/stdio_multi_statement_contract.rs` | — | `hover` / `symbols` summarize only the first statement of a document. |
| Negative-DSL fail-closed | `crates/quanta-gqlang-v2-lsp/tests/stdio_negative_dsl_hellgate.rs` | — | Each malformed root fails closed at the spawned binary. |
| Command-plane barrier | `crates/quanta-gqlang-v2-lsp/tests/stdio_command_plane_hellgate.rs` | `crates/quanta-gqlang-v2-lsp/tests/edit_command_handshake.rs` | Mutating / command-plane methods always `command_plane_forbidden`, exit 1; barrier is mutation-specific (not catch-all); session not poisoned. |
| Read-only authority (quickfix metadata-only + zero-query authority) | `crates/quanta-gqlang-v2-lsp/tests/stdio_readonly_authority_hellgate.rs` | `crates/quanta-gqlang-v2-lsp/tests/autocomplete_quickfix.rs` | Quickfix returns metadata only; zero-query path executes no query. |
| Autocomplete / catalog completion | — (contract-only) | `crates/quanta-gqlang-v2-lsp/tests/autocomplete_handshake.rs`, `catalog_autocomplete_handshake.rs`, `catalog_completion.rs` | Completion shapes pinned at library level only. |

## Syntax

| Surface | Binary e2e rail (file) | Contract-only rail (file) | Notes |
| --- | --- | --- | --- |
| Positive root decoding (all root families) | — (exercised via CLI binary corpus) | `crates/quanta-gqlang-v2-syntax/tests/parser_conformance.rs` | Library rail decodes every shipped root; the binary corpus matrix re-proves it through the CLI. |
| SELECT filter / literal / `LIMIT` variant matrix (all six comparison operators, integer literal incl. negative, `LIMIT` non-negative + overflow guards) | `crates/quanta-gqlang-v2-cli/tests/dsl_variant_frontdoor.rs` (subset, via CLI) | `crates/quanta-gqlang-v2-syntax/tests/select_filter_matrix.rs` | Every `SelectFilterOpV1` / `SelectLiteralAstV1` value decodes through `parse_document_v1`, re-derives its canonical query, and is a `format_document_v1` fixed point; negative/overflow `LIMIT` pins the fail-closed branches. |
| Interactive variant matrix (EXPORT kinds, SAMPLE modes, COMPARE metrics) | `crates/quanta-gqlang-v2-cli/tests/dsl_variant_frontdoor.rs` (subset, via CLI) | `crates/quanta-gqlang-v2-syntax/tests/interactive_variant_matrix.rs` | Every `ExportKindV1` / `SampleRetrievalModeV1` / `CompareMetricV1` value decodes through both the `decode_*_intent_v1` API and the front door, is a format fixed point, and unknown values fail closed. |
| Multi-statement recovery | — (contract-only) | `crates/quanta-gqlang-v2-syntax/tests/multi_statement.rs` | Each line is its own root; recovery reports all malformed statements. |
| Declaration-family negatives (VIEW / RULE / MODEL / PACK / PROFILE / GATE) | `crates/quanta-gqlang-v2-cli/tests/cli_negative_dsl_hellgate.rs` (subset, via CLI) | `crates/quanta-gqlang-v2-syntax/tests/declaration_conformance.rs` | Library rail pins fail-closed branches at `parse_document_v1`. |
| Query formatting (`fmt`) | `crates/quanta-gqlang-v2-cli/tests/examples_binary_frontdoor_matrix.rs` (`fmt` round-trip) | `crates/quanta-gqlang-v2-syntax/tests/query_formatting.rs` | `format_document_v1` idempotence proven both ways. |

## Sema

| Surface | Binary e2e rail (file) | Contract-only rail (file) | Notes |
| --- | --- | --- | --- |
| Typed sema conformance | — (exercised via CLI `sema` corpus) | `crates/quanta-gqlang-v2-sema/tests/sema_conformance.rs` | CLI `sema` binary matrix re-proves typed decoding end-to-end through the front door. |
| Pack-trust handshake | — (contract-only) | `crates/quanta-gqlang-v2-sema/tests/pack_trust_handshake.rs` | |

## Normalized IR

| Surface | Binary e2e rail (file) | Contract-only rail (file) | Notes |
| --- | --- | --- | --- |
| SELECT vertical slice | — (exercised via CLI `query` corpus) | `crates/quanta-gqlang-v2-ir/tests/j01_select_vertical_slice.rs` | Table-only row shape kept; non-table widening rejected fail-closed. |
| FIND / CHECK / EXPLAIN handshakes | — (contract-only) | `crates/quanta-gqlang-v2-ir/tests/j02a_find_handshake.rs`, `j02b_check_handshake.rs`, `j02c_explain_handshake.rs` | |
| Public planning IR (`required_capabilities`, `truth_expectation`, `bridge_mode`, export mode) | — (contract-only) | `crates/quanta-gqlang-v2-ir/tests/public_planning_ir.rs` | Roots needing planning metadata fail closed when it is absent. |
| Lowering-ready normalization | — (contract-only) | `crates/quanta-gqlang-v2-ir/tests/lowering_ready_ir.rs` | Preserves planning metadata rather than dropping it; lowering-ready, not lowered. |
| Degraded-reason authority | — (contract-only) | `crates/quanta-gqlang-v2-ir/tests/degraded_reason.rs` | Single `DegradedReasonV1` authority; codes round-trip and stay unique. Presentation-only, not invented runtime truth. |
| IR round-trip | — (contract-only) | `crates/quanta-gqlang-v2-ir/tests/ir_roundtrip.rs` | |
| Fact schema / rowset guard / public fact schema | — (contract-only) | `crates/quanta-gqlang-v2-ir/tests/fact_schema_rowset_guard.rs`, `public_fact_schema.rs`, `fact_model_handshake.rs` | Typed host-ingested vocabulary, not analyzer-computed truth. |
| Model / pack / gate handshake | — (contract-only) | `crates/quanta-gqlang-v2-ir/tests/model_pack_gate_handshake.rs` | |
| Metric inventory | — (contract-only) | `crates/quanta-gqlang-v2-ir/tests/metric_inventory.rs` | `metric.<name>` typed vocabulary; external findings are frontend data nouns. |
| AI profile IR contract | — (contract-only) | `crates/quanta-gqlang-v2-ir/tests/ai_profile_contract.rs` | Same-intent replay; AI lane cannot emit final policy truth. |

## Composition gate

The entire rail set above is composed and gated by `just ci`.
