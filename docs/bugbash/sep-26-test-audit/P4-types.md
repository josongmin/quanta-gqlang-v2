# P4 — types fail-open audit

Scope: `crates/quanta-gqlang-v2-types/tests/` (`v3_rule_type_solver.rs`,
`v3_types_check_edges.rs`, `v3_types_hellgate.rs`, `v3_types_universe.rs`) and
every `#[cfg(test)]` block in `crates/quanta-gqlang-v2-types/src/`
(`builtin.rs`, `diag.rs`, `op.rs`). Static review plus three parallel
adversarial sweeps (one per large test file, one for universe + src modules,
one verifier that re-checked each edit against production). All work is
test-only; no production code was modified.

Result: **~45 proof gaps raised, all mechanically closed or explicitly held.**

## 1) Round 1 — committed earlier

| file:line | class | severity | action | note |
|---|---|---|---|---|
| `check_edges:1503` | E | 높음 | 수정됨 | `relation_row()`는 `Option`이라 행 0개면 `None==None`으로 "exact row reuse" 오라클 공허 통과 → 양변 `.unwrap()`. |
| `check_edges:1337` | H | 중간 | 수정됨 | 프로젝션 필드 1의 authored 이름(`"n"`) 미검증 → `assert_eq!(integer_field.name(), "n")`. |
| `check_edges:729` | E | 중간 | 수정됨 | `operator_id.is_some()` → 케이스별 exact raw(`101`/`106`). |
| `check_edges:968` | E | 낮음 | 수정됨 | `Builtin(_)` → `sum` float `raw()==4`/`name()=="sum"`. |
| `hellgate:123` | E | 높음 | 수정됨 | 이름은 "stable operator/builtin IDs"인데 `is_some()`/`Builtin(_)`만 확인 → `+` 101/symbol, `count` raw 1/name 고정. |
| `hellgate:251` | E | 낮음 | 수정됨 | star overload `Builtin(_)` → raw 2/name. |
| `hellgate:776` | H | 중간 | 수정됨 | `call_key` fact를 effects만 비교 → state + `resolved_call()==Predicate(pkg::p)`. |
| `hellgate:867` | G/E | 중간 | 수정됨 | `let _relation_id = catalog.get(..)` 폐기로 catalog 구성 전체가 smoke → lookup/authority/qualified 검증. |
| `hellgate:1251` | H | 낮음 | 수정됨 | compare/parent `operator_id.is_some()` → raw 200 / raw 7. |
| `src/diag.rs:830` | F | 중간 | 수정됨 | 43-variant 리터럴 순회에 `cases.len()==43` 가드. |
| `src/op.rs:97` | F | 중간 | 수정됨 | `count==67` 고정 + unary/binary non-empty 가드. |

## 2) Round 2 — adversarial sweep supplements

### check_edges
| file:line | class | action | note |
|---|---|---|---|
| `:160` | H | 수정됨 | alias fact가 producer의 `effects`를 실제 재사용하는지 검증. |
| `:208,:440` | E | 수정됨 | alias/target state가 둘 다 `Unknown`이어도 통과하던 것 → concrete `Known(Duration)` + full 5-axis effects. |
| `:772` | H | 수정됨 | parenthesized child의 `literal` 미검증 → `Some(Value(Int(7)))` 고정. |
| `:1342` | E | 수정됨 | `source_record` 내용 미검증(순환 oracle) → `base:int` 필드 고정. |
| `:1509` | E | 수정됨 | 행 내용 미검증 → `left_id:int` 고정. |
| `:1634` | E | 수정됨 | upstream row 내용 미검증 → `id:int` 고정. |
| `:1896` | H | 수정됨 | swapped right row `value_type` 누락 → `string` 추가. |
| `:766` | H | 수정됨 | `BadOverload` code만 → node_id/name/operands/location full equality. |

### hellgate
| file:line | class | action | note |
|---|---|---|---|
| `:78` | G/H | 수정됨 | extra-entry 케이스 부재 + 네 node-fact 진단이 code-only → `Missing/Duplicate/Foreign/ExtraNodeFact` full equality + extra 케이스 추가. |
| `:147` | D'/E | 수정됨 | `literal` 필드 전무 → operand literal `Int(1)` + sum literal `None` 고정. |
| `:445` | E | 수정됨 | `is_some()==expects_id` → `not` raw 7 / `+` `None` exact table. |
| `:519` | H | 수정됨 | parent effects 3/5축 → full struct. |
| `:552` | E | 수정됨 | `Unknown(_)` → `UnresolvedEditorCallable` exact. |
| `:643` | E | 수정됨 | host key length-only → prefix + BE lengths + kind tag + payload exact `Vec<u8>`. |
| `:914` | E | 수정됨 | catalog 3/6 getter → namespace/provenance/relation_type 추가. |
| `:1086` | E | 수정됨 | produced relation row 미검증 → `relation_row_count()==2`, source row empty, nested `Known(bool)`. |
| `:1152` | E | 수정됨 | `Unknown(_)` → `UnresolvedEditorBinding` exact. |
| `:1248` | G/E | 수정됨 | `relation_exists_captures...`가 kernel을 아예 호출 안 함 → `check_document` 실행 + exists full effects. |
| `:1311,:1341` | H | 수정됨 | compare/parent effects 1~0축 → full struct. |
| `:1430` | E/H | 수정됨 | ambiguous sum `Unknown(_)` → exact cause + full 5-axis effects. |
| `:341` | H | 수정됨 | unknown/bad operator code-only → `UnknownOperator`/`BadOverload` full equality. |
| `:364` | G | 수정됨 | unused contextual code-only → full `UnusedContextualInput`. |

### solver
| file:line | class | action | note |
|---|---|---|---|
| `:172` | H | 수정됨 | source-predicate callable effects 2/5축 → full `conservative_source_predicate` tuple. |
| `:114,:685` | E | 수정됨 | `steps>0`, permutation usage 둘 다 default(0) 통과 가능 → nodes/edges/steps>0. |
| `:249` | H | 수정됨 | `diagnostics().first()`만 → `len()==1` 고정. |
| `:381` | WN | 수정됨 | 이름의 "ordinal" 미검증(모든 fixture가 ordinal 0) → 2-binder quantifier로 ordinal 1(`int`) 실제 검증. |
| `:528` | H | 수정됨 | shadow forged binding code-only → `InvalidRuleInput{span(35,36)}` full equality. |
| `:886` | H | 수정됨 | unanchored mutual code-only → full `UnanchoredRecursiveRule`. |
| `:897` | H | 수정됨 | 6종 malformation + unknown call이 동일 code만 → 각 케이스 exact location/`UnknownRuleCallable` full equality. |
| `:1394` | H | 수정됨 | "exact defense-in-depth"인데 code-only → full `RelationExistsInRule`. |
| `:1451` | H/E | 수정됨 | depth prepass code-only + `..` wildcard → 64 `{dimension:"depth", span(15,16)}`, 63 `{span(9,10), candidates:30}` full equality. |

### universe + src modules
| file:line | class | action | note |
|---|---|---|---|
| `universe:79` | E | 수정됨 | `INFINITY` `is_err()` → exact code + `NEG_INFINITY` 추가. |
| `universe:90` | WN | 수정됨 | 이름의 "structured_id" 미검증 → same-id/different-name 케이스 추가. |
| `universe:183` | E | 수정됨 | inequality-only identity → 동일 canonical 재구성 `assert_eq` 대조 추가. |
| `universe:225` | E | 수정됨 | 항상 참인 `<=1024` → 정확한 truncation 렌더링(128바이트) 고정. |
| `universe:342` | G | 수정됨 | 수용-only → catalog retention(`iter().count()==1`, `get`). |
| `universe:368` | E | 수정됨 | `dash-name` 수용-only → field name/type 검증. |
| `universe:372` | E | 수정됨 | cap+1만 → exact-limit 수용 + tag 고정. |
| `universe:349` | E | 수정됨 | zero 1축만 → 5개 위치 각각 zero 루프. |
| `universe:507` | WN | 수정됨 | 이름의 "hash" 미검증 → `HashSet` len 1 추가. |
| `universe:589` | E | 수정됨 | canonical 수용-only → `as_str()` verbatim 고정. |
| `universe:581` | WN | 수정됨 | 이름의 "capture" 미검증 → `ResolvedRelationCaptureInputV2` round-trip(ordinal 1) + foreign/duplicate 거부 full equality. |
| `src/op.rs` | H | 수정됨 | "complete operator surface"인데 operand/result 미고정 → 67행 `(id, symbol, operand, result)` / `(id, symbol, lhs, rhs, result)` 전수 테이블 추가. |
| `src/diag.rs:830` | E | 수정됨 | count-only 가드 → distinct `.code()` set len 43으로 중복-치환 차단. |
| `src/diag.rs:863` | G | 수정됨 | `TypeAuthorityDiagnosticV2`(9 variant) code/Display 전무 → 전수 테이블 test 추가. |
| `src/diag.rs:7` | E | 수정됨 | `diagnostic_excerpt` 문자경계 backoff 미검증 → ASCII/짧은/멀티바이트 테스트 추가. |

## 3) Held items and reasons

- `check_edges:126` — `ResolvedTypeInputsV2::try_new`(check.rs:511)는
  `source_len(source_id)`만 비교하고 registry `identity()`는 비교하지 않는다. 동일 length·다른
  identity registry가 수용되는 것은 **프로덕션 동작**이므로 테스트에서 거부를 단정할 수 없다
  (프로덕션 수정 금지 범위). 보고만.
- `src/diag.rs` diagnostic payload(`node_id`/`symbol`/`text`/`dimension`/`candidates`/`operands`)는
  public accessor가 없고 `cases` 리터럴과 동일 값을 재비교하면 항진명제다. accessor 추가는
  프로덕션 변경이라 보류.
- `hellgate` `ResolvedRelationCaptureInputV2.target_ordinal`은 kernel(check.rs:816-831)이 읽지 않아
  round-trip 이상을 검증할 수 없다(DTO round-trip은 위에서 고정).
- `hellgate` `HostCallableNamespaceV2::Rule` tag byte 2는 `assert_ne!`로만 커버(선택적 강화 여지).
- 이름 over-claim(`universe:250` `_rejects_oob`, `universe:374` `..._or_nested`,
  `universe:499` `_and_capture_`(일부 해소), `solver:164` negative control 부재)은 오라클이 아니라
  이름·구성 문제라 별도 판단으로 보류.

## 4) Swept clean (no action)

- class A: 모든 `if let` assert 블록은 직전 unconditional `assert!(matches!(..))`가 variant를
  고정하거나(guarded) helper에 code/field/span assert를 둔다.
- class B: 동적 컬렉션을 `.get()`/`.find()`로 조용히 건너뛰는 루프 0건 — 전부 `.unwrap()`.
- class C: `if let Some(caller_param)` 게이트 0건(fixture `anchored`/`anchor_head`는 모든 값이
  exercised되고 부정 경로도 assert).
- class F: 동적 컬렉션 루프는 없고, 리터럴/정적 레지스트리 순회는 개수 가드(count 67, len 21,
  len 43, relation_row_count)로 봉쇄.
- `#[ignore]` 0건, `should_panic` 0건, 테스트 대상 `let _ =` 0건, `unwrap_or`/`.ok()` 오라클 0건.

## 5) Baseline / final

- Baseline: lib 11 + solver 19 + check_edges 32 + hellgate 19 + universe 16 = **97 passed**.
- Final: lib **14** + solver 19 + check_edges 32 + hellgate 19 + universe 16 = **100 passed,
  0 failed, 0 ignored** (신규 3: op semantics table, diag authority table, diag excerpt).
- `cargo clippy -p quanta-gqlang-v2-types --tests`: clean.
- `cargo fmt -p quanta-gqlang-v2-types -- --check`: clean.
- `just ci` / `just qualify-local` 미실행(통합자 전용).
