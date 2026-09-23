# quanta-gqlang-v2 docs

현재 공개 표면과 검증 안내:

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

`bugbash/`는 감사 시점의 기록이며 현재 구현 증거는 아니다. 현재 구현·지원 범위는
위 문서와 코드에서 확인하고, clean committed HEAD 검증 방법은
[local qualification 안내](../tools/qualification/README.md)를 따른다. 폐기된 실행
캠페인의 종합 결과는 [ADR-0001](adr/0001-frontend-campaign-consolidation.md)에 있다.
