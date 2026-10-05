# FCP 검증 입력 정리 — 2026-10-06

관측 consumer main `33d1ba8ce8ad6c7b476bf7bf8b5c982a196351db`, producer main `2987f54a6d17bf4fe9b5e657d0291cb73422882a`.
소스·실행 증거 보관 후 완료된 임시 입력을 정리했다. 새 Rust/product qualification은 아니다.

## 완료

| 대상 | 결과 |
| --- | --- |
| 기존 source checkout | **24개 퇴역**: 일반/중첩 qualification21, 앱 관리2, canonical CAS1 |
| 로컬 FCP branch | **25개 제거**: consumer23, producer2. 원래 SHA 비교 후 삭제 |
| 복구 ref | **156개 유지**: branch/reflog/source HEAD. namespace `refs/archive/fcp-retired-20261006` |
| dirty overlay | **4개 별도 보관**: original index bytes와 unqualified snapshot commit |
| 이미 없던 qualifier | registration1개만 제거. 원본 Git metadata 보관; 원래 receipt 재생성 없음 |

- b018 consumer의 modified4/untracked1, 공유 producer clone의 staged3/untracked20을 보관했다. 미검토 변경을 main에 합치지 않았다.
- b018은 실제 lease/process/queue 참조 부재, 백업 bytes, 대상1개 dry-run을 확인한 뒤 canonical GC로 제거했다. 다른 CAS 입력과 공유 external producer/index clone은 유지했다.
- 앱 archive 후 남은 source-22의 qualification user-immutable 파일은 동일 bytes와 active reference 부재를 확인하고 thaw했다. System immutable/shared inode는 거절 조건으로 유지했다. Source-24 잔존 경로도 최종 readback에서 없다.
- Taskmesh·DX·Oct4 외부 입력은 제거 목록에서 제외했다. Weekly recovery 경로는 최종 관측에서 없었으나 이 작업의 삭제 대상이 아니었다.

## 유지한 미완료/보호 대상

- `fcp-producer-main-integration-oct5`의 이미 없던 앱 첨부는 남겼다. 도구가 **`This worktree is owned by another task.`**로 거절했다. 소유권을 우회하거나 원본처럼 보이는 대체 checkout을 만들지 않았다.
- Actual SDK/PTA 및 installed Graph/CHECK는 [현재 액션 계획](/Users/songmin/Documents/code-new/quanta-gqlang-v2/docs/audits/2026-10-05-current-source-audit-action-plan.md)의 잔여다. 과거 fixture 후보/실패도 복구 ref와 raw 증거에 보관했다.
- CHECK Foundation37 및 Match23 PASS는 원래 source에 묶인다. 후속 canonical codec 변경은 해당 Foundation37로 검증된 것이 아니다. Producer tracked core/CI 입력은 `8675bcd` 이후 같으며 기존 foreign dirty는 보존했다.

## 복구와 증거

[정리 ledger](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/retirement-20261006/closeout.json) · [복구 절차](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/retirement-20261006/README.md) · [Git ref/overlay 목록](/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004/consumer-execution-closure/retirement-20261006/recoverable-git-refs.json)

독립 읽기로 **186,205개 고유 blob SHA256**을 확인했다. Source root26개/파일 entry271,932개의 path·mode·symlink·index metadata를 보관한다.
전체 [보관 manifest](</Volumes/Extreme SSD 1/fcp-retirement-recovery-20261006/manifest.json>)와 [archive hash](</Volumes/Extreme SSD 1/fcp-retirement-recovery-20261006/verified.json>)가 있다.

초기 raw collector는 workspace 상위까지 수집해 reference/benchmark 자료도 보관했다. 그 넓은 raw inventory는 FCP 테스트 수나 qualification이 아니다. 실제 삭제는 명시된24개 source 입력/25개 ref에만 적용했다. 후속 collector root는 FCP evidence로 좁혔다.
없는 과거 참조와 JSON 참조 해석 실패를 숨기지 않았고 opaque JSON bytes는 보존했다.

복구는 새 경로에서 수행한다. Git history는 canonical repo의 위 archive ref로, plain source/raw는 `restore_source.py`/`restore_raw.py`로 복구한다.
Historical target link는 퇴역 build cache를 가리킬 수 있다. 복구가 fresh source custody·release approval·설치형 qualification을 발급하지 않는다.

이번 checkpoint는 문서/보관 정리다. 과거 producer `just ci`1514, Foundation37, Native Match23의 원래 결과를 유지하며 새 CI/product PASS를 추가하지 않는다.
