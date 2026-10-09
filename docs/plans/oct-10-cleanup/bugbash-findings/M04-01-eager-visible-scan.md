# M04-01: 로컬 View edge의 불필요한 visible 전체 검색

- 판정: `CONFIRMED_DEFECT` (소스상 이차 작업). QGLang `25fad76cf5b4510244157c2e009b4dc37aa958ff`, tree `99501ba1aabfa37361fccfb7dee388cd91055ee1`.
- 소유자: `crates/quanta-gqlang-v2-modules/src/catalog/view_topology.rs:189-197`. 호출자는 `DocumentCatalogValidatorV2::validate_view_topology_with_request_owner_v3` (`authority/request_catalog.rs:208-252`).
- 독립 반례: 로컬 View A→B edge `E`개와 B와 무관한 visible binding `B`개를 둔다. `predeclared.get`이 로컬 B를 찾았어도 `visible.visible_bindings().iter().find(...)`가 먼저 실행된다. 각 edge에서 최대 B개 비교 후 `local_target.or_else(...)`가 로컬 결과를 선택한다. `E×B` 비교가 edge당 한 번의 `charge_edge` 밖에서 발생한다.
- 기대/실제: 로컬 또는 external hit면 visible lookup 0회여야 한다. 실제는 local hit에서도 visible 순회가 수행된다. 격리 계산의 `E=4, B=3`은 12회 비교; `E=B=4096`이면 16,777,216회다. 이 수치는 연산 횟수이며 Rust 지연/RSS 측정은 아니다.
- 보완 수락: local → external → visible 우선순위와 wrong-kind 진단 위치를 유지하며 visible 조회를 필요할 때만 한다. 로컬 hit에서 visible scan 0회를 관측하는 독립 회귀와 missing/visible hit의 동일 진단을 추가한다. 대표 입력의 실제 시간·allocation을 별도 기록한다.
- 현재 검증: Modules/Logic focused owner 테스트는 [레인 보고](02-modules-logic.md)에 기록됐다. 해당 테스트 통과는 이 불필요한 비교를 거부하지 않는다. 수정·전체 `just ci`·성능 측정은 `NOT_RUN`.
