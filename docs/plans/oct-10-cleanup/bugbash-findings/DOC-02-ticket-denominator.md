# DOC-02: 활성 closeout 티켓 수 19/18 불일치

**최종 triage:** [활성 수정 대상 아님](FINAL-TRIAGE.md). 현재 다른 작성자의 미커밋 문서 통합안에 교정안이 있다. 통합 SHA에서 재확인한다.

- 판정: `DOC_HEAD_DEFECT`. QGLang 고정 HEAD `25fad76cf5b4510244157c2e009b4dc37aa958ff`의 문서 집계 오류.
- 실제: `docs/README.md:5`는 잔여 closeout ID가 19개라고 한다. `docs/plans/oct-6-substrate-dedup/tickets.md:9-12`는 활성 **18개**와 해결된 QG-DIAG 및 수락된 D4/D5를 구분하고, `docs/work/remaining.md:6,587-588`도 활성 18개를 소유한다.
- 독립 기대값: 활성 분모는 18이고 해결된 결정을 더하지 않는다. 문서 진입점에서 owner index와 같은 숫자·범위를 사용한다.
- 영향: 남은 작업 범위를 한 건 부풀리고 기존 해결 결정을 다시 작업 대기열로 오해할 수 있다. QGLang Rust 동작 결함은 아니다.
- 보완 상태: 다른 작성자의 현재 미커밋 `docs/README.md`에 18개 교정안이 있다. 이 감사는 그 파일을 건드리지 않았다. 통합 커밋 이후 실제 활성 ID 목록과 README를 재대조한다.
- 상세 증거: [문서 레인](06-docs-evidence.md). CI·downstream 실행 `NOT_RUN`.
