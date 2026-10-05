# quanta-gqlang-v2 docs

현재 공개 표면과 검증 안내:

- [최신 세션 코드 감사·잔여 액션](audits/2026-10-05-current-source-audit-action-plan.md):
  완료된 producer 정리, 소비자 코드 잔여, source별 증거와 설치 전제
- [GQLang·Semantica substrate 감사](audits/2026-10-06-substrate-ownership-and-usecase-audit.md):
  frontend/native/SDK·Index·filesystem owner, 중복·별도 계약,
  최종 수리 6건·중복 제거 7건, 구현별 ingress·계약·의존 정책, 통합/삭제/최소 API 선택과 checkpoint gate
- [Frontend/소비자 계약 계획](audits/2026-10-04-frontend-consumer-contract-plan.md):
  현재 판정과 원래 수리의 acceptance; 이전 source 기록은 이력

- [시스템 구성·사용 사례·현재 증거](../README.md): 두 진입 경로와 로컬 개발 전제조건
- [언어/owner stack](language/README.md): 지원 family, open/closed authority,
  CLI 및 diagnostics 경계
- [V3 frontend 실행 계약](contracts/frontend-execution-v3.md): qglang producer
  DTO, digest, 지원하지 않는 실행 payload
- [예제 목록](language/examples_index.md) ·
  [릴리스 주장 기준](language/release_discipline.md) ·
  [테스트 계약 맵](language/test_hellgate_ledger.md)
- [AI 탐색 프로필 상태](language/ai-profile.md): 현재 별도 공개 API 없음
- [아키텍처 결정 기록](adr/README.md): 폐기된 캠페인에서 보존한 현재 결정과
  권위 경계

현재 구현·지원 범위는 위 문서와 코드에서 확인하고, clean committed HEAD 검증
방법은 [local qualification 안내](../tools/qualification/README.md)를 따른다. 폐기된
실행 캠페인의 종합 결과는
[ADR-0001](adr/0001-frontend-campaign-consolidation.md), 테스트 감사에서 확정된
oracle·gate 결정은
[ADR-0002](adr/0002-test-authority-and-adversarial-oracles.md)에 있다.
