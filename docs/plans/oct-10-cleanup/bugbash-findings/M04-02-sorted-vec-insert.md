# M04-02: View dependency 정렬 Vec의 역순 삽입

- 판정: `CONFIRMED_DEFECT` (소스상 이차 작업). QGLang `25fad76cf5b4510244157c2e009b4dc37aa958ff`, tree `99501ba1aabfa37361fccfb7dee388cd91055ee1`.
- 소유자: `crates/quanta-gqlang-v2-modules/src/catalog/view_topology.rs:238-271`. 같은 validator의 공개 요청 경로에서 도달한다.
- 독립 반례: 서로 다른 유효 dependency `E`개를 `(source_asset_id, target_asset_id)`의 역순으로 준다. 매번 `binary_search_by`가 삽입 위치 0을 반환하고 `Vec::insert`가 기존 원소를 전부 민다. 총 이동 횟수는 `E(E−1)/2`다. `E=4`에서 6회, `E=4096`에서 8,386,560회, `E=50,000`에서 1,249,975,000회다. 마지막 수치는 typed 입력의 연산 계산이지 실제 파싱 문서에서 그 크기에 도달했다는 측정은 아니다.
- 기대/실제: canonical dependency 순서와 authored 순서의 duplicate/kind 첫 오류를 보존하면서 선형 수집과 한 번의 정렬에 준하는 작업을 기대한다. 실제는 edge당 한 번의 예산 charge로 이차 이동을 수행한다.
- 보완 수락: authored 순서로 validation 및 duplicate/first-error를 결정할 별도 인덱스를 사용하고, 유효 edge는 모은 뒤 한 번 정렬한다. 역순·중복·동일 위치·kind conflict·exact/short 예산의 작은 독립 오라클을 추가한다. 지연/RSS 개선은 동일 SHA·workload의 비교 측정으로만 주장한다.
- 현재 검증: [레인 보고](02-modules-logic.md)의 관련 집중 테스트는 통과했다. 이차 이동을 거부하는 성능 회귀, 수정 후 전체 CI와 제품 측정은 `NOT_RUN`.
