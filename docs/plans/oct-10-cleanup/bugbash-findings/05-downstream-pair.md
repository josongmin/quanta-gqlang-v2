# Semantica 소비자 pair 감사 (lane 09)

- QGLang 입력: `25fad76cf5b4510244157c2e009b4dc37aa958ff`, tree `99501ba1aabfa37361fccfb7dee388cd91055ee1`.
- Semantica 최초 관찰 입력: `58cfa983cf9785ab1752f4d75011fefdfba1285f`의 `/Users/songmin/Documents/code-new/semantica-codegraph-v2`. 조사 중 다른 작업자가 문서를 커밋하여 HEAD가 `170736bc5512f5d766306ccb5756aa87f783a0ef`로 이동했다. 두 SHA에서 `Cargo.toml`과 `Cargo.lock`은 이 감사자가 수정하지 않았다.

## DP-01 — locked consumer dependency graph 불일치 (통합 차단, 재현)

- caller → owner: Semantica `packages/analysis/quanta-v2/crates/quanta-gqlang-v2-exec-contract/Cargo.toml:19-21,35-38`의 `v3-execution` feature가 sibling QGLang 공개 facade에 path 의존한다. QGLang facade `crates/quanta-gqlang-v2-frontend-contract/Cargo.toml:16`은 이번 주부터 새 `quanta-gqlang-v2-execution-contract` crate를 직접 의존하고 Pipeline도 같은 crate에 의존한다.
- 예상: 고정된 QGLang source closure를 사용하는 Semantica의 locked dependency graph가 새 내부 crate를 포함하고, `cargo metadata --locked --offline --format-version 1`이 lock 변경 없이 종료한다.
- 실제: Semantica `Cargo.lock:6830-6834`의 facade 항목에는 Pipeline만 있다. QGLang의 새 `quanta-gqlang-v2-execution-contract` package 항목은 그 lockfile에 없다. Semantica root에서 위 명령을 실행하면 **exit 101**과 `the lock file ... needs to be updated but --locked was passed`로 종료한다. `--no-deps` metadata는 exit 0이어서 전체 dependency closure를 확인하는 대체 검사가 될 수 없다.
- 판정: **현재 downstream locked-metadata 통합 실패는 확인됐다.** 새 crate 누락이 그 원인과 일치하지만, lockfile의 다른 변경 요인까지 분리한 것은 아니다. 소비자 Rust 컴파일·V3 vector 실패를 이 결과만으로 주장하지 않는다.
- HEAD 이동 재검증: 새 `170736bc`는 문서 커밋이며 `Cargo.lock`의 facade 항목은 여전히 Pipeline만 열거한다. 같은 full locked metadata 명령도 다시 **exit 101**과 동일한 lock 갱신 필요 오류를 냈다. 관찰 실패는 새 HEAD에서도 지속된다.
- 소유자/조치: Semantica E13 소비자 pair에서 정확한 QGLang SHA·전체 source path·feature·lock을 하나로 고정한다. 해당 checkout의 다른 작업자 문서를 건드리지 않는 격리 환경에서 lock 재생성/차이를 검토하고, `cargo metadata --locked --offline`, 실제 consumer compile, facade 계약 vector를 순서대로 실행한다. 새 package를 lock에 수동으로 끼워 넣거나 QGLang facade 밖 내부 crate를 소비자에 직접 노출하지 않는다.

## 증거 경계

- 실행: QGLang manifest/lock과 Semantica manifest/lock의 정적 대조, Semantica root의 full `cargo metadata --locked --offline --format-version 1` exit 101, `--no-deps` exit 0, 실행 후 Semantica manifests/lock 변경 없음.
- 미실행: consumer Cargo compile/tests, V3 vector, runtime generation/provider/lease/grant admission, 설치 제품, hosted CI. QGLang `25fad76`의 로컬 1,529/1,529 영수증은 이 pair를 증명하지 않는다.
