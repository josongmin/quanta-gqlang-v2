# P1 — modules fail-open audit (final, 4 passes)

Repo: `quanta-gqlang-v2`. Scope: `crates/quanta-gqlang-v2-modules/tests/`
(`v3_document_predeclaration.rs`, `v3_modules_v2_authority.rs`) and every
`#[cfg(test)]` block in `crates/quanta-gqlang-v2-modules/src/`
(`budget.rs`, `graph.rs`, `lockfile.rs`, `manifest.rs`, `authority.rs`).
Production logic untouched; fixtures/`*.expect`/`*.tsv`/docs untouched.

Method: pass 1 static read; pass 2 empirical probing of every presence-only
oracle; pass 3 four parallel READ-ONLY subagents (authority / predeclaration /
src unit blocks / cross-cut); pass 4 final residual sweep. Every new span oracle
uses only probe-observed values, never guesses; all probes removed.

## Findings (all 수정됨 unless 보류/기각)

| file | class | severity | action | note |
|---|---|---|---|---|
| `authority` `public_signature_projection_preserves_every_callable_fact` | A | 높음 | 수정됨 | `let-else { return; }` wrong-variant silent pass → `assert!(matches!(..))` 가드 |
| `authority` `pack_activation_and_model_extension_are_observable` | D'+H | 높음 | 수정됨 | fact count만 → target/attribute/value/location + activation `pack_id()` |
| `authority` `unknown_pack_dependency_rejects_at_edge` | H | 높음 | 수정됨 | "at edge"인데 code만 → primary `(22,23)` + secondary none |
| `authority` `lock_version_drift_is_rejected_exactly` | H | 높음 | 수정됨 | "exactly"인데 code만 → primary src2 `[0,len-1]` + secondary `(13,14)` |
| `authority` `pack_lock_digest_version_and_reference_are_sealed` (3) | H | 낮음 | 수정됨 | code만 → lock_mismatch primary src2 `[0,82]`+sec src1 `(6,7)`, invalid_lock_syntax primary src2 `[0,83]` |
| `authority` `lock_missing_and_stale_pins_are_both_rejected` (2) | H | 낮음 | 수정됨 | code만 → missing primary src1 `(23,24)`, stale primary src2 `[73,145]`, 각 secondary none |
| `authority` `duplicate_dependency_rejects_even_when_ranges_match` | H | 낮음 | 수정됨 | code만 → primary `(40,41)` + secondary `(30,31)` |
| `authority` `external_signature_preserves_arity_and_parameter_order` | H | 중간 | 수정됨 | name만 → `(name, value_type)` 쌍 비교 |
| `authority` `model_duplicate_fact_is_idempotent_and_observable` | H | 중간 | 수정됨 | value/location만 → target/attribute 추가 |
| `authority` `external_catalog_accepts_exact_closed_ten_kind_payload_matrix` | H | 중간 | 수정됨 | count만 → 각 asset `payload().namespace_kind()` 10종 대조 |
| `authority` `external_catalog_validates_payload_and_dependency_kinds` | H+coverage | 낮음 | 수정됨 | count만 → `view.dependencies().len()==1`, dependency `expected_kind` mismatch 부정 케이스 추가 |
| `authority` `document_view_can_depend_on_external_base_asset` | H | 중간 | 수정됨 | assets count만 → dependency edge source/target id·module·kind·family exact |
| `authority` `document_profile_and_require_certified_are_validated_in_combined_registry` | H | 낮음 | 수정됨 | `source_id`만 → full `LocatedSpanV2` + secondary none |
| `authority` `agreeing_model_packs_merge_and_retain_sorted_provenance` | H | 낮음 | 수정됨 | `next().unwrap()`만 → `facts().count()==1` |
| `authority` `local_direct_import_does_not_suppress_downstream_reexport_ambiguity` | E/H | 낮음 | 수정됨 | candidates len만 → origins `["left","right"]` |
| `authority` `require_certified_traverses_activated_external_pack_contracts` | G | 낮음 | 수정됨 | `is_ok()`+code만 → activations count + negative primary `(3,4)` |
| `authority` `module_step_budget_accepts_exact_limit_and_reports_one_over` | G | 낮음 | 수정됨 | `run(2).is_ok()`만 → `resolved_graph().current_module()=="app"` |
| `authority` `fingerprint_mutation_matrix_covers_all_manifest_semantics` | F | 낮음 | 수정됨 | `all(..)` 앞 `mutations.len()==cases.len()` 가드 |
| `authority` `version_parse_rail!` (5 reject) | E | 낮음-중간 | 수정됨 | `is_ok()==false`만 → reject는 `invalid_version` code+primary `(0,1)`, accept는 파싱값 exact |
| `authority` `multibyte_identifier_diagnostic...` | E | 낮음 | 수정됨 | `<256`+prefix만 → 64 multibyte char truncation exact 렌더링 |
| `authority` `hex_bytes` | A(무의미) | 낮음 | 수정됨 | `String` 쓰기 항상-참 `is_ok()` → `.expect(..)` |
| `authority` 16곳 `is_some()` | H | 중간 | 수정됨 | probe 확정 exact `SpanV1` |
| `authority` 6곳 `edge_witness().len()` | H | 중간 | 수정됨 | exact `LocatedSpanV2` 배열 |
| `authority` code-only 오라클 15곳 | H | 낮음 | 수정됨 | exact primary(±secondary) |
| `authority` `absent_declaration_is_not_exported` | H | 낮음 | 수정됨 | secondary none만 → primary `(22,23)` |
| `predeclaration` 10곳 `span.start` | H | 중간 | 수정됨 | full `SpanV1` |
| `predeclaration` `each_family_rejects_wrong_kind_and_unknown_target` | H | 중간 | 수정됨 | unknown code만 → primary `(60+index,61+index)` |
| `predeclaration` `optional_catalog_probe...` | H | 낮음 | 수정됨 | wrong-kind code만 → primary `(32,33)` + secondary `(20,21)` |
| `predeclaration` `view_cycle_remains_view_only_and_foreign_registry_rejects` | H | 낮음 | 수정됨 | code만 → primary src10 `(20,21)` |
| `predeclaration` `host_predicate_proof_preserves_exact_catalog_id_signature_and_session` (3) | H | 낮음 | 수정됨 | code만 → primary src9 `(42,43)`/`(41,42)`/`(41,42)` |
| `predeclaration` `host_predicate_rejects_wrong_kind_ambiguity_and_missing_signature` | E/H | 중간 | 수정됨 | loop code만 → primary `(50,51)` + ambiguity candidates len 2 (+ missing sig primary `(51,52)`) |
| `predeclaration` `optional_rule_probe_preserves_session_on_absence_in_both_orders` | E/G | 중간 | 수정됨 | `is_some()` bool만 → Some id/kind/origin exact, wrong-kind `(30,31)`, phase primary none |
| `predeclaration` `final_family_refs_must_come_from_the_same_validator_session` | H | 중간 | 수정됨 | session mismatch code만 → primary `(40,41)`/`(41,42)` |
| `predeclaration` `imported_path_is_visible...` / `imported_predicate_alias...` | H | 낮음 | 수정됨 | kind/local name + unknown ref primary src9 `(61,62)` |
| `predeclaration` `repeated_host_identity...` else | H | 중간 | 수정됨 | Ok가 origin만 → id `checks.authz` + kind `Check` |
| `predeclaration` `predeclaration_rejects_duplicate...` | E | 낮음 | 수정됨 | 입력 echo → `assets.len()==1`+kind View+module app |
| `predeclaration` phase errors (`all_six`, `optional_rule`) | D' | 낮음 | 수정됨 | `without_location`임을 `primary().is_none()`으로 고정 |
| `src/graph.rs` `missing_target_is_charged_once_without_becoming_cycle_vertex` | E | 중간 | 수정됨 | 상한만(0 vs 1 구분 불가) → dangling 2개, exact 2 + one-under `(steps,1,2)` |
| `src/manifest.rs` `dag_fast_path_is_bounded_on_long_chain_and_branching_graph` | G/E | 낮음 | 수정됨 | `.is_err()`만 → `dimension()=="steps"` |
| `predeclaration` `topology_diamond...` 중복 `span.start` | H | 낮음 | 보류 | 인접 full `SpanV1` 비교가 이미 존재 |
| `predeclaration` `reverse_authored_views...` | — | — | 기각 | subagent의 "reverse 미실행"은 오독; `if reverse { assets.reverse(); edges.reverse(); }` 정상 |

## 부정 케이스 도달 경로 관찰 (production, 참고)

dependency `expected_kind` mismatch는 external-catalog ingress(`catalog.rs:298`)
에서 `modules.catalog_dependency_kind_conflict`로 먼저 거부된다. resolve 이후
`catalog.rs:436` (`target.kind != expected_kind`)은 target id가 reference의
`namespace_kind`로 derive되고 ingress가 `ref.namespace_kind == expected_kind`를
강제하므로 도달 불가능해 보인다. 추가한 테스트는 도달 가능한 ingress 진단
(`primary=(16,17)`, secondary none)을 고정한다. (production 수정은 범위 밖 — 오너 확인용)

## Modified files

- `crates/quanta-gqlang-v2-modules/tests/v3_modules_v2_authority.rs`
- `crates/quanta-gqlang-v2-modules/tests/v3_document_predeclaration.rs`
- `crates/quanta-gqlang-v2-modules/src/graph.rs` (`#[cfg(test)]`만)
- `crates/quanta-gqlang-v2-modules/src/manifest.rs` (`#[cfg(test)]`만)

## Swept clean

`error.code()` 단독 오라클 0건, `let-else`/빈 arm/빈 루프 0건, 잔여
`eprintln!`/probe 0건, `#[ignore]`/`todo!` 0건, 오라클 내 `unwrap_or`/`.ok()` 0건.
`.all()/.any()`는 전부 사전 exact count/witness/array assert 뒤. 남은 `span.start`
1건은 인접 full-span 비교가 있는 중복뿐.

## Baseline / final

- Baseline: lib 16 + predeclaration 22 + authority 120 = **158 passed**.
- Final: **158 passed / 0 failed / 0 ignored**.
- `cargo fmt -p quanta-gqlang-v2-modules -- --check`: clean.
- `cargo clippy -p quanta-gqlang-v2-modules --tests`: clean (background job 확인).
- `just ci` / `just qualify-local` 미실행(통합자 전용).
