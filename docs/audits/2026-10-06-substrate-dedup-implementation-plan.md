# Substrate 중복 제거 실행 계획 — 경로 변경

Canonical 실행 문서: [docs/plans/oct-6-substrate-dedup/readme.md](../plans/oct-6-substrate-dedup/readme.md).

목적·배경·D1–D7 작업 범위·F1 선행 조건·의존 관계·checkpoint·로컬 검증·DoD·상태 ledger를 위 문서로 통합했다. 이 경로에는 별도 실행 계획을 유지하지 않는다.

- [누적 substrate 감사](2026-10-06-substrate-ownership-and-usecase-audit.md)는 판정 근거와 이력이다.
- [이전 계획 source snapshot](2026-10-06-substrate-dedup-plan-source.json)의 `plan_sha256`과 검증 결과는 commit `6932c256`의 이동 전 계획에 대한 historical receipt다. 새 계획이나 현재 코드 완료의 증거로 사용하지 않는다.
- D1–D7 구현·Rust/QBC·Python owner·제품 qualification 상태는 canonical plan의 ledger에서 관리한다.
