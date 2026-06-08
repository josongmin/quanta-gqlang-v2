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
