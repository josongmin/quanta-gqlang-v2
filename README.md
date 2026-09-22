# quanta-gqlang-v2

`quanta-gqlang-v2`는 public language frontend owner repo다.

범위:

1. syntax
2. sema
3. normalized IR
4. CLI
5. diagnostics stdio transport

비범위:

1. runtime substrate
2. `quanta-gqlang-v2-exec-*`
3. SDK/runtime wire
4. `quanta-index-sdk`

Sibling consumer seam:

1. downstream repos may reference this repo only through the host-agnostic
   frontend contract crate: `quanta-gqlang-v2-frontend-contract`
2. semantica/sdk consumers must not depend on deleted legacy
   `quanta-gqlang-v2-{syntax,sema,ir}` crates
3. semantica runtime DTOs and backend wiring remain downstream-owned

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
5. composition family (`LET`/`WITH`, root들의 조합): `examples/composition/let_with_find.qglang`

CLI front door:

1. supported read-only subcommands: `query`, `fmt` (aliases `v3-query`, `v3-fmt`)
2. inline argv tail: `cargo run -p quanta-gqlang-v2-cli -- query 'from files f | where f.size > 0 | project f.id'`
3. file-path input: `cargo run -p quanta-gqlang-v2-cli -- query --file examples/show/capabilities.qglang`
4. malformed argv, unreadable files, and command errors fail closed with non-zero exit

Diagnostics stdio front door:

1. one UTF-8 JSON object per newline (NDJSON)
2. supported read-only methods are declared by this exact-set block:

<!-- supported-diagnostics-methods:start -->
- `v3_diagnostics`
<!-- supported-diagnostics-methods:end -->

3. mutating or unknown methods fail closed
4. binary entry: `cargo run -p quanta-gqlang-v2-diagnostics-stdio`
5. request fields are exactly `id`, `method`, `source_id`, and `source`; duplicate
   or unknown fields fail closed
6. this is a private NDJSON protocol, not JSON-RPC or LSP; it does not implement
   `initialize` or document lifecycle methods

Repo-owned front-door e2e:

1. CLI shipped-example matrix: `query` and `fmt` stay deterministic on the
   frontend-owned example corpus under `examples/`
2. CLI bad argv, unreadable file, parse-error, and no-result shell behavior stay
   fail-closed
3. diagnostics stdio analysis stays read-only, mutating methods stay forbidden, malformed
   JSON stays fail-closed, and the same stdio session must recover after those
   failures
4. these rails prove frontend determinism and fail-closed behavior only; they do
   not prove runtime execution or backend materialization

로컬 전체 검증은 `just ci` 하나다. 앞선 focused 검증을 모두 다시
실행하므로 `just nextest`, `just lint`, `just semgrep`를 순서대로 실행한 뒤
`just ci`를 실행하지 않는다.

Focused 진입점은 원인 확인용이다:

1. `just lint` — format, TOML, MSRV/current Clippy
2. `just test` — workspace 및 rustdoc tests
3. `just boundary` — shipped front doors, architecture, external facade consumer
4. `just policy` — supply-chain, typos, Semgrep

### Proof rails

The repo-owned e2e boundary is proven by two kinds of rails. Binary-only rails
spawn the real CLI/diagnostics-stdio binaries and assert stdout/stderr/exit code,
open-analysis status, NDJSON recovery, and bounded-input behavior. Contract-only
rails are in-crate library tests for syntax, sema, and IR shapes.
The scope is syntax/sema/normalized-IR/CLI/diagnostics stdio only; runtime execution, provider
routing, result counts, host DTOs, and backend materialization are out of scope
and are never asserted as real. The full surface-to-rail mapping is the coverage
ledger at [docs/language/test_hellgate_ledger.md](docs/language/test_hellgate_ledger.md),
and the language boundary is documented in
[docs/language/README.md](docs/language/README.md). The whole set is gated by
`just ci`.
