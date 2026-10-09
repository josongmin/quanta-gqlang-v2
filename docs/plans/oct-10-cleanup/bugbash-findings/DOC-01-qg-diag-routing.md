# DOC-01: 해결된 QG-DIAG의 오래된 미완료 경로

**최종 triage:** [활성 수정 대상 아님](FINAL-TRIAGE.md). 현재 다른 작성자의 미커밋 문서 통합안에 교정안이 있다. 통합 SHA에서 재확인한다.

- 판정: `DOC_HEAD_DEFECT`. QGLang 고정 HEAD `25fad76cf5b4510244157c2e009b4dc37aa958ff`의 문서 오류이며 wire 동작 오류는 확인되지 않았다.
- 실제: `docs/adr/0002-test-authority-and-adversarial-oracles.md:169-170`은 diagnostics-wire 결정을 미완료로 적고 `../work/remaining.md#qg-diag`로 보낸다. `docs/adr/0004-sealed-owner-facts-and-bounded-projection.md:115-118`도 같은 앵커를 참조한다. `docs/work/remaining.md`에는 해당 앵커가 없다.
- 독립 기대값: `docs/adr/0008-private-diagnostics-primary-location-contract.md:3,31`은 Accepted이며 QG-DIAG를 현재 V2 private wire 계약으로 해결했다고 명시한다. 두 ADR은 이 결정으로 독자를 보내야 한다.
- 영향: 이미 결론 난 계약을 새 결함으로 재발급하거나 실제 근거를 못 찾게 한다.
- 보완 상태: 조사 시 다른 작성자의 미커밋 ADR-0002/0004 수정에 교정안이 이미 있었다. 이 감사는 그 hunk를 수정·커밋하지 않았다. 해당 소유자가 통합한 새 SHA에서 링크와 상태를 재확인한다.
- 상세 증거: [문서 레인](06-docs-evidence.md). 제품·wire 실행 `NOT_RUN`.
