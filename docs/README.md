# QGLang documentation

- [잔여 작업](work/remaining.md): 미완료 frontend 계약 검토, downstream 검증·설치 전제와 제안 작업
- [Substrate 병렬 실행 계획](plans/oct-6-substrate-dedup/readme.md): 현재 소스·작성 소유권, 변경 범위, 테스트·완료 조건과 메인 checkpoint
- [아키텍처 결정 기록](adr/README.md): 완료된 조사·수리·티켓의 결정과 증거 경계
- [언어와 owner stack](language/README.md): 현재 지원 범위와 CLI·diagnostics 경계
- [Frontend execution V3](contracts/frontend-execution-v3.md): 공개 DTO·digest·발행 권한
- [Frontend owner 경계](contracts/frontend-owner-boundaries.md) · [Planning 결과](contracts/planning-results.md)
- [예제](language/examples_index.md) · [릴리스 주장 기준](language/release_discipline.md)
- [테스트 계약 맵](language/test_hellgate_ledger.md) · [Local qualification](../tools/qualification/README.md)
- [제안 상태의 typed code-query 계약](adr/0007-typed-code-query-contract-proposal.md) · [미완료 A00–A15](work/remaining.md#proposed-code-query-work)

현재 동작은 소스·계약·owner 테스트가 정의한다. ADR은 결정과 근거를 기록한다.
완료된 감사·계획·티켓·checkpoint 원문은
[ADR-0001의 Git 복구 기준](adr/0001-frontend-campaign-consolidation.md#recover-original-source-bound-records)에서
찾는다. Proposed ADR과 오래된 receipt는 현재 구현·제품 qualification이 아니다.
