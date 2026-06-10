# F16 — S09B/S09C/J06 레저 재동기화 + generator alias 인지 (W3, semantica)

> STATUS: PARTIAL (jun-10) — generator alias-awareness CODE done + tests green; ledger regen deferred to post-F17
>
> **Done (this run, repo `semantica-codegraph-v2`):**
> - `tools/ci/gqlang-v2/generate_dependency_snapshot.py:272-288` — new helper
>   `dependency_attribution_key_v1(dep)`: returns the manifest-local alias
>   (cargo metadata `rename`) when present, else falls back to the real package
>   `name`. Single point of responsibility for reverse-dep attribution keying.
> - `tools/ci/gqlang-v2/generate_dependency_snapshot.py:308-311` — `package_views`
>   dep-walk now attributes each dep via that helper (was `dep.get('name')`-only),
>   so package-renamed legacy aliases (e.g. `quanta-gqlang-digest-v1 =
>   { package = "codegraph-core-ir-foundation" }`) are visible to metadata
>   reverse-dep truth. Non-aliased deps still attribute by real name (regression-tested).
> - `tests/tools/test_generate_gqlang_dependency_snapshot.py` — added 4 non-vacuous tests:
>   alias attribution + real-name absence, un-renamed-name regression (incl. `rename: None`),
>   multiple-aliases-in-one-consumer, and the helper's rename-over-name precedence.
>   Verified RED (name-only behavior fails all 3 alias-asserting tests, regression test
>   still passes) → GREEN (`4 passed, 11 deselected`).
>
> **Remaining (deferred — out of this run's scope):**
> - Full ledger regen + commit (`python3 tools/ci/gqlang-v2/generate_dependency_snapshot.py`)
>   gated on F17 (digest alias drain). The 4 pre-existing module failures
>   (`*_artifact_is_fresh`, `tracks_safe_delete_and_live_blockers`,
>   `runtime_boundary_edges_distinguish...`) are committed-ledger-vs-live drift, NOT
>   alias-logic regressions; they resolve when the ledgers are regenerated post-F17.
> - S09C orphan-doc stale reconcile, F15 regen-vs-committed gate, J06 mechanical
>   `delete_rows_zero` — all unchanged by this run.

원천: S-ledger-residue-reporting + J-caller 감사. J06 "100%" 는 readiness 문서 자체가 반증.

## Gap 목록 (라이브)

1. **S09C — committed orphan 문서 stale**
   - `generated/dependency-snapshot/current-orphan-source-readiness.md:95,110-111` 이
     `quanta-gqlang-public-entry-v1` active ref 5 주장하며 `r-arch-11-wildcard-baseline.txt:72` 인용 —
     그 라인은 현재 `quanta-gqlang-sarif-projection-v1` (라이브 카운트 4). DoD line 49
     "reconcilable with live rg output" 위반
2. **S09B — generator 가 package-rename alias 를 못 봄**
   - `tools/ci/gqlang-v2/generate_dependency_snapshot.py:278-296` dep 파싱이 `dep.get('name')`
     (실제 package 명) 만 사용 — `quanta-gqlang-digest-v1 = { package = "codegraph-core-ir-foundation" }`
     alias 의 reverse-dep 이 metadata 레그에서 0 으로 보임. digest-v1 의 17개 blocker 는 전부 grep-레그.
     DoD line 50 (reverse-dep truth from cargo metadata) 가 alias rename 에 대해 구조적 미충족
3. **J06 — delete_rows_zero 가 거짓**
   - `current-public-legacy-delete-readiness.md:147-153` digest-v1 17 blockers / safe-delete-now: no,
     `:401-409` planner-v1 2 blockers (문서는 commit 67b203b, 2026-06-10 08:01 재생성본)

## 작업

1. generator 에 rename/alias 인지 추가: dep entry 의 `rename` 필드 (cargo metadata 가 제공) 를 키로
   alias-명 reverse-dep 귀속 — digest-v1 류 alias 행이 metadata-레그에서도 진실 보고
2. F17 (digest alias drain) 완료 후 전체 레저 regen + 커밋:
   `python3 tools/ci/gqlang-v2/generate_dependency_snapshot.py`
3. orphan 문서 stale 해소 확인 (public-entry-v1 행이 라이브 rg 와 일치)
4. F15 의 regen-vs-committed 게이트 단계로 재발 방지
5. J06 closeout 의 `delete_rows_zero` 판정을 readiness 문서 기반 자동 체크로 승격 (prose 금지)

## DoD

- committed 레저 == fresh regen (diff 0)
- alias crate 의 reverse-dep 이 metadata 레그에 보고됨
- J06 % 추정이 readiness 문서 수치에서 기계적으로 도출됨
