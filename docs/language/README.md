# QGLang v2 Language Surface

`quanta-gqlang-v2` ships a parser-stable frontend contract for a read-only shell.

Public guarantees:

1. parser-stable root decoding across syntax, sema, and normalized IR
2. read-only CLI/LSP affordances only
3. no runtime truth in public language docs, examples, or smoke tests
4. experimental catalog and AI-adjacent surfaces remain typed frontend metadata

Shipped root families:

1. core query roots: `SELECT`, `FIND`, `CHECK`, `EXPLAIN`
2. interactive roots: `SHOW`, `DESCRIBE`, `SAMPLE`, `COMPARE`, `EXPORT`
3. declaration roots: `VIEW`, `RULE`, `MODEL`, `PACK`, `PROFILE`, `GATE`
4. meta roots: `USE PACK`, `REQUIRE CERTIFIED`

Composition family (clause-level, not one of the 17 roots): `LET` bindings
consumed by a body that names them via a trailing `WITH`. `LET`/`WITH` groups
existing roots into a working-set composition; per the corpus decision it is
covered by dedicated composition rails, not the 17-root example matrix.

Shipped example families:

1. core: `examples/select/basic.qglang`, `examples/find/taint.qglang`, `examples/check/hybrid_rule.qglang`, `examples/explain/why_no_result.qglang`
2. interactive: `examples/show/capabilities.qglang`, `examples/describe/rule_detail.qglang`, `examples/sample/retrieval.qglang`, `examples/export/sarif.qglang`, `examples/compare/semantic_delta.qglang`
3. declarations: `examples/view/catalog_users.qglang`, `examples/rule/shell_exec.qglang`, `examples/model/embeddings_code.qglang`, `examples/pack/security_default.qglang`, `examples/profile/strict_review.qglang`, `examples/gate/release_gate.qglang`
4. meta: `examples/use_pack/secure_pack.qglang`, `examples/require_certified/strict_only.qglang`

The composition family ships `examples/composition/let_with_find.qglang`
separately — it is covered by the dedicated composition rails rather than the
17-root example matrix above.

Read-only binary fronts:

1. CLI: `parse`, `sema`, `query`, `fmt`, `explain`, `doctor`, `headless`, `zero-query` (nullary read-only starter scaffold)
2. LSP stdio JSON methods: `hover`, `completion`, `symbols`, `diagnostics`, `zero_query`, `quickfix`, `explain`
3. malformed argv, malformed JSON, and any non-allowlisted method fail closed
   (the stdio dispatch is a closed 7-method allowlist; every other method —
   mutating or simply unknown — is rejected with `method_not_allowed`)

Repo-local front-door proof rails:

1. shipped example docs must round-trip through the actual CLI binary for
   `parse`, `sema`, `query`, `fmt`, and `headless`
2. CLI fail-closed behavior must cover bad argv, unreadable files, and explicit
   parse/sema/no-result shell classification
3. LSP stdio must prove read-only methods, non-allowlisted-method rejection,
   malformed JSON rejection, and same-session recovery after fail-closed responses
4. these rails are frontend-only proof; they do not claim runtime execution,
   provider routing, or backend result materialization

Typed frontend surfaces (recovered local tickets):

These are typed frontend metadata only — read-only, presentation-level, and never
runtime truth claims. Degraded reasons are presentation-only and share a single
authority enum; they are not invented runtime truth.

1. AI exploration profile (`E05E`): `AiExplorationPlanV1`, `AiObjectiveV1`,
   `ExplorationBudgetV1`, and `WorkingSetV1` are an explicit profile over the same
   DSL. Reconfirmation is same-intent replay through `ci_gate` or `lint`, and the
   AI lane cannot emit final policy truth directly. See [ai-profile.md](./ai-profile.md).
2. Public planning IR (`E06C`): planning carries explicit `required_capabilities`,
   `truth_expectation`, `bridge_mode`, and export mode; roots that require planning
   metadata fail closed when it is absent, and lowering-ready normalization
   preserves planning metadata rather than dropping it.
3. Metric inventory and external findings (`E11C`): metric identifiers
   (`query_result_count`, `external_finding_count`, …) are a typed, host-ingested
   vocabulary and external findings are public frontend data nouns — not
   analyzer-computed truth. The inventory is host-contract data; it has no DSL
   grammar position and is deliberately not surfaced as completion.
4. Explain and path-witness UX (`E12C`): explain presentation, path-witness
   rendering, degraded badges (via the shared degraded-reason authority), and
   why-no-result guidance are presentation-only.

Out of scope:

1. runtime execution substrate
2. provider registry routing
3. product closure claims about backend materialization

Unsupported claims:

1. unsupported result counts, fix generation, or execution success promises
2. unsupported host DTO publication through frontend IR
3. unsupported assertions that examples prove end-to-end runtime behavior

## Proof rails (test hellgates)

The language surface is guarded by a set of repo-owned proof rails. These rails
fall into two kinds, and the distinction is load-bearing:

- **Binary-only proof** spawns the *real* shipped CLI / LSP binary, feeds it the
  exact front-door bytes (argv tail, `--file`, or stdio frames), and asserts the
  observable contract: stdout, stderr, output channel, and process exit code.
  Only a spawned process can pin an exit code or a channel, so these rails lock
  behavior the library alone cannot express.
- **Contract-only proof** is an in-crate library test. It pins typed shapes,
  diagnostic codes, normalization, and fail-closed branches against the public
  API, but it does **not** spawn a process and therefore makes no claim about
  exit codes or output channels.

Both kinds are frontend-only. They prove that the `parser-stable`, `read-only`
shell decodes roots deterministically and fails closed on malformed input. They
deliberately assert **no runtime truth**: runtime execution, provider/registry
routing, result counts, host DTO materialization, mutable edits, and backend
closure are never asserted as real. The `experimental` catalog and AI-adjacent
surfaces stay typed frontend metadata, and anything marked `unsupported` above is
never proven end-to-end here.

Binary-only rails (spawn the real binary, assert stdout/stderr/exit code):

- Shipped positive corpus — CLI `parse` / `sema` / `query` / `fmt` / `headless`
  matrix over the frontend-owned examples:
  `crates/quanta-gqlang-v2-cli/tests/examples_binary_frontdoor_matrix.rs`
- CLI usage / help channel + exit-code contract (help → stdout/exit 0, usage
  error → stderr/exit 2):
  `crates/quanta-gqlang-v2-cli/tests/cli_help_usage_contract.rs`
- CLI empty / whitespace-only front door (blank document fails closed, exit 2):
  `crates/quanta-gqlang-v2-cli/tests/cli_empty_input_frontdoor.rs`
- CLI transport parity (inline argv ≡ `--file`, byte-identical stdout):
  `crates/quanta-gqlang-v2-cli/tests/cli_transport_parity.rs`
- CLI `doctor` three-way classification (parse-error / sema-error / no-result)
  promoted to the binary:
  `crates/quanta-gqlang-v2-cli/tests/doctor_frontdoor_hellgate.rs`
- CLI negative-DSL fail-closed at the binary (each malformed root, exit 1):
  `crates/quanta-gqlang-v2-cli/tests/cli_negative_dsl_hellgate.rs`
- LSP stdio framing edges (EOF, glued frames, incomplete-at-EOF):
  `crates/quanta-gqlang-v2-lsp/tests/stdio_frame_edges.rs`
- LSP JSON escape / surrogate-pair negative paths:
  `crates/quanta-gqlang-v2-lsp/tests/stdio_json_escape_negative.rs`
- LSP multi-statement first-statement contract (`hover` / `symbols` summarize the
  first statement only):
  `crates/quanta-gqlang-v2-lsp/tests/stdio_multi_statement_contract.rs`
- LSP negative-DSL fail-closed at the binary:
  `crates/quanta-gqlang-v2-lsp/tests/stdio_negative_dsl_hellgate.rs`
- LSP command-plane barrier (non-allowlisted methods always `method_not_allowed`,
  exit 1, session recovers):
  `crates/quanta-gqlang-v2-lsp/tests/stdio_command_plane_hellgate.rs`
- LSP read-only authority (quickfix metadata-only + zero-query authority, no
  query is executed):
  `crates/quanta-gqlang-v2-lsp/tests/stdio_readonly_authority_hellgate.rs`
- LSP stdio read-only method matrix + same-session recovery:
  `crates/quanta-gqlang-v2-lsp/tests/stdio_read_only.rs`

Contract-only rails (in-crate library tests):

- Syntax positive conformance (root decoding):
  `crates/quanta-gqlang-v2-syntax/tests/parser_conformance.rs`
- Syntax multi-statement recovery:
  `crates/quanta-gqlang-v2-syntax/tests/multi_statement.rs`
- Syntax declaration-family negatives (VIEW / RULE / MODEL / PACK / PROFILE /
  GATE fail-closed): `crates/quanta-gqlang-v2-syntax/tests/declaration_conformance.rs`
- Sema typed conformance: `crates/quanta-gqlang-v2-sema/tests/sema_conformance.rs`
- IR vertical slices and planning rails: `select_vertical_slice.rs`,
  `public_planning_ir.rs`, and `degraded_reason.rs` under
  `crates/quanta-gqlang-v2-ir/tests/`.

The full surface-to-rail mapping (including the contract-only-only surfaces) is
the coverage ledger: [test_hellgate_ledger.md](./test_hellgate_ledger.md). The
whole set is gated by `just ci`.
