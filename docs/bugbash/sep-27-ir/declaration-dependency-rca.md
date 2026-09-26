# 선언 의존성 평가 stack overflow RCA

기준 HEAD: `a4b78121aeabd99960fc930840b35913ac28c04f`.
이 항목은 기존 F-04/F-05/F-09 재판정 이후 구조 감사에서 새로 확인한 실제 결함이다.
기존 재감사 문서 변경을 보존한 working tree에서 수정했다.

## 실제 도달 경로와 원인

공개 `compile_source_document_v3` → semantic document transaction →
`ExpressionTypeKernelV2::check_document` → `DocumentChecker::type_expr` →
`BindingFromDeclaration` → 다음 선언의 `type_expr` 재귀 호출.

다음 원문은 표준 budget 안에 있다. 4,096개 item, 75,601 bytes이며 각 선언
본문은 symbol 또는 literal 하나다.

```text
let x0 = x1; let x1 = x2; ...; let x4095 = 1
```

HIR의 syntax depth 제한은 서로 독립적인 선언 본문의 의존성 깊이를 제한하지
않는다. 기존 `visiting` 집합은 순환을 검출하지만 긴 비순환 경로의 호출 스택을
제한하지 않았다. 역순으로 작성하면 앞서 계산한 fact가 cache에 있어 통과하므로
동일한 의미의 선언 집합이 authored order에 따라 성공 또는 프로세스 abort했다.

| 공개 facade 원문 probe | 수정 전 debug | 수정 전 release | 수정 후 debug/release |
| --- | --- | --- | --- |
| 정방향 256개, 2 MiB | SIGABRT | 성공 | 성공 |
| 정방향 4,096개, 2 MiB | SIGABRT | SIGABRT | 성공 |
| 역방향 4,096개, 2 MiB | 성공 | 성공 | 성공 |
| 정방향 4,096개, 8 MiB | SIGABRT | SIGABRT | 성공 |

따라서 lint나 debug profile만의 문제가 아니다. 사용자 원문을 받는 compiler
consumer의 프로세스를 종료시키는 types owner의 평가 알고리즘 결함이다.

## 수정과 수용 기준

- 타입, overload, effect, literal fact의 authority는 기존 types kernel이다.
  `check/value.rs`는 그 owner 내부의 비공개 evaluator이며 별도 resolver가 아니다.
- syntax child와 선언 참조를 같은 명시적 `ValueFrame` 스택으로 평가한다.
  완료된 fact는 재사용하고, 진행 중인 node 재진입은 기존 순환 진단을 반환한다.
- 호출 스택 사용이 선언 의존성 길이에 비례하지 않는다. 평가용 heap frame과
  active set은 도달한 expression/edge 수로 제한된다. 별도 재시도 scan은 없다.
- continuation은 좌→우 평가, collection의 즉시 거부, call argument별 기대 타입,
  parenthesized `i128::MIN` 처리를 유지한다. 오류 순서를 바꾸는 선언 선평가는 없다.
- 원문 budget, 공개 API, IR/schema, 진단 vocabulary는 그대로다. 지원하던 유효
  입력의 process abort를 제거하며, 순환 의존성은 계속 typed rejection이다.

## 회귀 검증

`v3_declaration_dependency_stack.rs`의 독립 oracle:

1. 2 MiB worker에서 4,096개 정방향/역방향 chain 모두 Duration 타입 및
   `now()`의 impure/nondeterministic/nonmonotone effect를 모든 alias에서 확인한다.
2. 4,096개 순환 chain은 정확한 `CyclicDeclarationDependency`와 source/node/span을
   반환한다.
3. list의 앞선 Int/Bool 불일치는 뒤에 있는 순환 참조보다 먼저 거부된다.

세 테스트가 통과했다. 기존 `v3_types_check_edges` 33개도 통과했다.
첫 회귀 테스트만 수정 전 HEAD의 types/HIR을 독립 scratch workspace로 복사해
실행하면 SIGABRT로 실패한다. 현재 checkout의 코드를 되돌려 실험하지 않았다.

standalone external consumer에는 공개 facade만 사용해 표준 4,096개 item 경계의
정방향/역방향 원문을 2 MiB worker에서 컴파일하는 assertion을 추가했다.
모든 item의 `ValueDeclaration` disposition, 빈 diagnostics와 빈 execution item을
확인해 중간 선언을 생략하거나 가짜 실행 계획을 만드는 해결도 거부한다.

관찰 로그와 probe는 gitignored `target/sep27-structural-reaudit/`에 있다.

- `baseline-regression.log`: 이전 HEAD에서 신규 회귀 테스트의 실제 abort.
- `declaration-chain-results.json`, `declaration-chain-release-results.json`: 수정 전
  debug/release 공개 원문 probe.
- `declaration-chain-debug-after.json`, `declaration-chain-release-after.json`: 수정 후
  공개 원문 probe 각 5개 성공.
- `types-regression.log`, `types-initial.log`: owner 회귀 및 기존 edge suite.
- `ci.log`: 최종 `just ci` 전체 결과. 이 명령에 external-consumer gate가 포함된다.

이 기록은 해당 결함의 원인과 수정 범위다. 전체 구조 감사 완료나 downstream
runtime 검증을 뜻하지 않는다. dirty working tree의 로컬 CI와 clean committed-HEAD
qualification은 구분하며, 기존 qualification receipt를 재사용하지 않는다.
