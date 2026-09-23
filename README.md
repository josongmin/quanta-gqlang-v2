# quanta-gqlang-v2

`quanta-gqlang-v2`는 QGLang v3 언어 프론트엔드의 owner repo다.

범위:

1. syntax, formatting, HIR, binding, types, modules
2. normalized algebra/logic and frontend planning IR
3. checked source compilation and the host-agnostic V3 execution document
4. read-only CLI and diagnostics stdio transport

비범위:

1. runtime substrate
2. physical plan, provider/generation admission, result materialization
3. `quanta-gqlang-v2-exec-*`, SDK/backend/runtime wire, `quanta-index-sdk`

Sibling consumer seam:

1. downstream repos may reference this repo only through the host-agnostic
   frontend contract crate: `quanta-gqlang-v2-frontend-contract`
2. sibling consumers must not use owner crates such as `surface`, `sema`,
   `hir`, or `pipeline` as a public API
3. semantica runtime DTOs and backend wiring remain downstream-owned

언어 릴리스 표면:

1. pre-release, read-only v3 frontend; no deployed runtime claim
2. closed source compilation produces a source-bound V3 execution document;
   open analysis does not
3. catalog/presentation families describe checked frontend semantics, not
   provider availability or executed results
4. no separate AI exploration API is shipped

## 시스템 구성

두 진입점 모두 `quanta-gqlang-v2-frontend-contract`를 통과한다. Facade가
`pipeline`을 호출하고, pipeline은 `surface`/HIR, binder·types·modules,
algebra·logic, planning의 checked 결과를 조립한다. 각 stage의 책임은
[언어/owner stack](docs/language/README.md)에 정리돼 있다.

| 호출 경로 | 입력 권위 | 출력 | 이 repo가 하지 않는 일 |
|---|---|---|---|
| CLI `query`·diagnostics stdio | 기본 open analysis | 상태·소스 위치가 붙은 diagnostics | 호스트 카탈로그 추정, 질의 실행 |
| 호스트의 `compile_source_document_v3` | caller가 봉인한 closed authority와 원문 bytes | `CompiledSourceDocumentV3`와 불변 `ExecutionDocumentV3` | provider/generation admission, 물리계획, 결과 생성 |

## 사용 사례와 현재 결과

| 목적 | 사용 경로와 확인 가능한 결과 | 한계 |
|---|---|---|
| DSL 작성·정규화 | CLI `fmt`가 현재 예제를 파싱하고 포맷 고정점을 검사 | 의미 검증이나 실행 결과가 아님 |
| 편집기·도구 진단 | CLI `query` 또는 bounded NDJSON `v3_diagnostics`가 typed 진단과 `incomplete`/`error`를 반환 | LSP/JSON-RPC가 아니며 기본 open authority는 호스트 스키마를 제공하지 않음 |
| 호스트 연계 | facade가 checked source를 V3 item·typed literal·세 digest로 투영 | item 표현은 downstream 실행 허가가 아님 |
| 언어 family 회귀 | conformance registry와 owner 테스트가 root family disposition, composition, 거부 경계를 검증 | 데이터 결과·서비스 E2E·배포 증거가 아님 |

현재 증거는 [테스트 계약 맵](docs/language/test_hellgate_ledger.md)과
[V3 digest golden vectors](crates/quanta-gqlang-v2-pipeline/tests/fixtures/execution-v3-vectors.tsv)에 연결된다.
질의 처리량·지연시간의 측정 결과는 이 repo에 없다. 별도의
[qualification 캐시 관측치](tools/qualification/README.md)는 빌드/검증 시간이지
쿼리 성능이 아니다.

주요 문서:

1. [docs/language/README.md](docs/language/README.md)
2. [docs/contracts/frontend-execution-v3.md](docs/contracts/frontend-execution-v3.md)
3. [docs/language/examples_index.md](docs/language/examples_index.md)
4. [docs/language/test_hellgate_ledger.md](docs/language/test_hellgate_ledger.md)
5. [docs/language/release_discipline.md](docs/language/release_discipline.md)

예제:

실제 파일 목록은 [examples index](docs/language/examples_index.md) 한 곳에서
관리한다. 예제의 존재는 실행 가능성이나 결과를 보증하지 않는다.

CLI front door:

1. supported read-only subcommands: `query`, `fmt` (aliases `v3-query`, `v3-fmt`)
2. formatting example: `cargo run -p quanta-gqlang-v2-cli -- fmt --file examples/show/capabilities.qglang`
3. open-analysis example: `cargo run -p quanta-gqlang-v2-cli -- query --file examples/show/capabilities.qglang`
4. `query` does not supply a host catalog or execute a plan; unresolved
   authority reports `analysis.status=incomplete` with exit code 1
5. malformed argv, unreadable files, and command errors fail closed with non-zero exit

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

저장소 전체 로컬 게이트는 `just ci`다. GitHub Actions는 사용하지 않으며
`.github/workflows/`를 두지 않는다. clean committed HEAD의 정확한 소스
qualification 영수증은 `just qualify-local`이 별도로 발행한다. 두 명령 모두
downstream consumer 실행이나 배포를 입증하지 않는다. `just ci`는 focused
검증을 다시 실행하므로 focused 명령을 순서대로 실행한 뒤 중복 실행하지 않는다.

Focused 진입점은 원인 확인용이다:

1. `just static` — format, TOML, MSRV/current Clippy, typos
2. `just test` — workspace, rustdoc, shipped front-door parity
3. `just policy` — qualification custody, architecture, external facade consumer,
   supply-chain, Semgrep

## 로컬 개발 전제조건

`rust-toolchain.toml`은 기본 Rust 1.81과 rustfmt/Clippy를 고정한다.
`just ci`에는 별도 Rust 1.95 Clippy, `just`, `cargo-nextest`, `taplo`,
`typos`, `cargo-deny`, Python 3, Semgrep CLI 또는 Python 모듈이 필요하다.
필요한 실행 명령은 [Justfile](Justfile)이 권위다. 변경 부위의 owner 테스트로
반복하고, 저장소 전체 판정이 필요할 때 `just ci`를 실행한다. clean committed
HEAD의 재현 가능한 영수증이 필요할 때만 `just qualify-local`을 사용한다.

### Proof rails

Proof is split across owner API tests, shipped CLI/diagnostics-stdio process
tests, facade-only external-consumer/privacy probes, and family conformance.
Process tests assert stdout/stderr/exit code, open-analysis status, NDJSON
recovery, and bounded-input behavior; owner API tests do not prove those
process boundaries.
The scope includes the checked V3 frontend execution-document producer, but
not runtime admission, provider routing, result counts, host DTOs, or backend
materialization. The full surface-to-rail mapping is the coverage
ledger at [docs/language/test_hellgate_ledger.md](docs/language/test_hellgate_ledger.md),
and the language boundary is documented in
[docs/language/README.md](docs/language/README.md). The whole set is gated by
`just ci`.
