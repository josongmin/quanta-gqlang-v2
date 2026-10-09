# DP-01: Semantica locked consumer dependency closure 실패

- 판정: `DOWNSTREAM_BLOCKER` (locked metadata 실패 확인, 단일 원인 미분리).
- 입력: QGLang `25fad76cf5b4510244157c2e009b4dc37aa958ff`; Semantica 최초 `58cfa983cf9785ab1752f4d75011fefdfba1285f`, 재검증 `170736bc5512f5d766306ccb5756aa87f783a0ef`. Semantica manifest/lock은 이 감사에서 수정하지 않았다.
- 도달 경로: Semantica `packages/analysis/quanta-v2/crates/quanta-gqlang-v2-exec-contract/Cargo.toml:19-21,35-38` → QGLang 공개 facade. 현재 QGLang facade `crates/quanta-gqlang-v2-frontend-contract/Cargo.toml:16`은 새 execution-contract crate를 직접 의존한다. Semantica `Cargo.lock:6830-6834`의 facade dependencies는 Pipeline만 열거하고 새 `quanta-gqlang-v2-execution-contract` package는 없다.
- 기대/실제: 정확한 source closure의 `cargo metadata --locked --offline --format-version 1`은 lock 변경 없이 성공해야 한다. 두 Semantica SHA에서 모두 **exit 101**, `the lock file ... needs to be updated but --locked was passed`가 재현됐다. `--no-deps` 성공은 전체 dependency graph의 통과 증거가 아니다.
- 영향/한계: consumer locked 빌드의 선행 단계가 막혀 있다. 새 crate 누락과 일치하지만 다른 lock 변경 원인도 있을 수 있다. Rust consumer compile, V3 vector, runtime/installed 동작은 이 실패로 판정하지 않았다.
- 소유자/보완 수락: Semantica E13. 다른 작업자 WIP가 없는 격리 checkout에서 QGLang source SHA·전체 path closure·features를 고정하고 lock diff를 검토한다. locked metadata → consumer compile → 계약 vector 순서로 독립 증거를 남긴다. QGLang 내부 crate를 소비자 공개 의존성으로 우회하지 않는다.
- 원본 명령·상태: [레인 보고](05-downstream-pair.md). 후속 소비자/제품 검증 `NOT_RUN`.
