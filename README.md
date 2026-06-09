# quanta-gqlang-v2

`quanta-gqlang-v2`는 public language frontend owner repo다.

범위:

1. syntax
2. sema
3. normalized IR
4. CLI
5. LSP

비범위:

1. runtime substrate
2. `quanta-gqlang-v2-exec-*`
3. SDK/runtime wire
4. `quanta-index-sdk`

언어 릴리스 표면:

1. parser-stable read-only shell
2. frontend-only typed sema / IR
3. no runtime truth claims in public examples or docs
4. experimental catalog / AI-adjacent surfaces stay explicitly non-runtime

주요 문서:

1. [docs/language/README.md](docs/language/README.md)
2. [docs/language/release_discipline.md](docs/language/release_discipline.md)
3. [docs/language/examples_index.md](docs/language/examples_index.md)

예제:

1. query roots: `examples/select/basic.qglang`, `examples/find/taint.qglang`, `examples/check/hybrid_rule.qglang`, `examples/explain/why_no_result.qglang`
2. interactive roots: `examples/show/capabilities.qglang`, `examples/describe/rule_detail.qglang`, `examples/sample/retrieval.qglang`, `examples/export/sarif.qglang`, `examples/compare/semantic_delta.qglang`
3. declaration roots: `examples/view/catalog_users.qglang`, `examples/rule/shell_exec.qglang`, `examples/model/embeddings_code.qglang`, `examples/pack/security_default.qglang`, `examples/profile/strict_review.qglang`, `examples/gate/release_gate.qglang`
4. meta roots: `examples/use_pack/secure_pack.qglang`, `examples/require_certified/strict_only.qglang`

CLI front door:

1. supported read-only subcommands: `parse`, `sema`, `query`, `fmt`, `doctor`, `headless`
2. inline argv tail: `cargo run -p quanta-gqlang-v2-cli -- query SELECT id FROM catalog.files LIMIT 5`
3. file-path input: `cargo run -p quanta-gqlang-v2-cli -- headless --file examples/show/capabilities.qglang`
4. malformed argv, unreadable files, and command errors fail closed with non-zero exit

LSP front door:

1. stdio JSON requests only
2. supported read-only methods: `hover`, `completion`, `symbols`, `diagnostics`, `zero_query`, `quickfix`
3. mutating or unknown methods fail closed
4. binary entry: `cargo run -p quanta-gqlang-v2-lsp`

Repo-owned front-door e2e:

1. CLI shipped-example matrix: `parse`, `sema`, `query`, `fmt`, and `headless`
   stay deterministic on the frontend-owned example corpus
2. CLI bad argv, unreadable file, parse-error, and no-result shell behavior stay
   fail-closed
3. LSP stdio methods stay read-only, mutating methods stay forbidden, malformed
   JSON stays fail-closed, and the same stdio session must recover after those
   failures
4. these rails prove frontend determinism and fail-closed behavior only; they do
   not prove runtime execution or backend materialization

검증 진입점:

1. `just nextest`
2. `just doc-test`
3. `just lint`
4. `just semgrep`
5. `just ci`

### Proof rails

The repo-owned e2e boundary is proven by two kinds of rails. Binary-only rails
spawn the real CLI/LSP binary and assert stdout/stderr/exit code (usage, empty
input, transport parity, doctor classification, negative-DSL, and the LSP stdio
framing / escape / multi-statement / command-plane / read-only-authority gates).
Contract-only rails are in-crate library tests for syntax, sema, and IR shapes.
The scope is syntax/sema/normalized-IR/CLI/LSP only; runtime execution, provider
routing, result counts, host DTOs, and backend materialization are out of scope
and are never asserted as real. The full surface-to-rail mapping is the coverage
ledger at [docs/language/test_hellgate_ledger.md](docs/language/test_hellgate_ledger.md),
and the language boundary is documented in
[docs/language/README.md](docs/language/README.md). The whole set is gated by
`just ci`.
