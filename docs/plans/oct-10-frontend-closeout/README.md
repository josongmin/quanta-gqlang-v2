# Oct-10 QGLang 기존 작업 마감

Updated: 2026-10-10. 상태: **DEDUP4_SOURCE_DONE / FOCUSED_PASS / QUALIFICATION_RECEIPT_DERIVED / PAIRED_OPEN**.

## 목적과 범위

기존 frontend 구조 정리·중복 제거·로컬 게이트의 실제 상태를 확인하고,
검증된 로컬 main SHA와 receipt로 QGLang 범위를 마감한다.
최종 정적 감사 결과는 [audit.md](audit.md), 남은 실행과 종료 조건은
[actions.md](actions.md)에서 관리한다. 실제 체크박스는 actions.md에만 둔다.
후속 production AST 중복 감사와 수정은 [dedup-audit.md](dedup-audit.md)가 소유한다.

- QGLang: syntax/HIR/sema/normalized IR, CLI/diagnostics, host-agnostic frontend-contract.
- Semantica: runtime/provider/SDK wiring, factory guard 통합, analysis 및 product 실행 증명.
- V4/A00~A15, optimizer/Index, DTO 물리 이동, fuzz 자동 주기, 추가 hosted CI는 이번 마감 범위 밖이다.
- 사용자 지정 검증 방식은 로컬 CI다. 새 유료 hosted/remote CI 조건을 추가하지 않는다.

## 감사 결론

1. 초기 감사는 구조/설정 대조였다. 후속 AST 감사에서 Modules 중복 구현2묶음을 확인해
   기존 owner 안에서 공통화했다. 기존 Modules authority120개와 전체 local CI1514개는 통과했다.
2. 파일·테스트 분할, size/MSRV 게이트, examples 18개의 closed-authority 검사는 소스에 있다.
3. Semantica closeout의 frontend/pipeline subtree 및 golden vector pin은 현재 producer와 일치한다.
4. 별도 external-workspace bootstrap의 전체 revision은 과거 `9060e469…`다.
   현재 로컬 sibling SHA와 다르므로 **두 검증 경로의 입력 일관성 조정이 남는다**.
5. clean committed HEAD qualification은 현재 HEAD의 `target/qualification/<HEAD>/receipt.json`으로
   판정한다. receipt 없음/실패는 NOT_QUALIFIED, 유효한 same-HEAD receipt exit0만 LOCAL_QUALIFIED다.
   source 문서에 실행 결과를 재커밋해 입력을 바꾸지 않는다. downstream product는 별도 미검증이다.
6. 의미 기반 caller 대조에서 Modules Kosaraju/Logic Tarjan의 SCC 계산 책임 겹침을 확인했다.
   D03/QFC-06에서 공통 partition kernel로 두 caller를 전환하고 budget/witness 정책을 보존했다.
   후속 actual caller/first-refusal/storage owner 검증이 통과했다.
7. 추가 D04 감사에서 DAG preflight 중복과 ready Vec의 반복 이동을 확인해 수정했다.
   공통 borrowed DP와 cursor FIFO, canonical View min-heap을 사용한다. 별도 cfg(test) helper만
   확인하던 oracle에 actual production exact/one-short 검증을 연결했다.

정적 감사는 명시한 소스·호출·설정 범위의 대조다. 저장소 전체 중복0,
최신 CI 통과 또는 Semantica 제품 완료를 뜻하지 않는다.

## 실행 순서

QFC-01 입력 고정 → QFC-06 SCC common-owner 전환 완료 → QFC-03 로컬 main 커밋
→ QFC-04 `just qualify-local` → QFC-05 receipt 확인·범위별 마감.

QFC-02 paired-source/bootstrap 조정은 기존 downstream E13 owner에서 별도로 진행한다.

QGLang 단독 qualification은 Semantica runtime 완성을 기다릴 필요가 없다.
paired integration은 QFC-02와 downstream의 compatible owner proof가 있어야 종료한다.

## 기존 원장과 관계

- [remaining.md](../../work/remaining.md): 기존 Parent18 상태와 별도 V4 제안의 인덱스.
  이 패킷은 QGLang frontend 마감 절차만 보충하며 Parent18을 복제하거나 종료하지 않는다.
- [ADR-0003](../../adr/0003-frontend-downstream-integration-boundary.md): durable ownership/evidence 경계.
- [frontend-execution-v3](../../contracts/frontend-execution-v3.md): normative DTO/digest 계약.
- [Semantica Oct-10 compiled](../../../../semantica-codegraph-v2/docs/handoff/oct-10/oct-10-compiled.md):
  Semantica 실행 작업의 단일 관리 위치. E01-DUP-GUARD, E13 및 E18의 상태는 해당 owner에서 관리한다.
