# Ticket 03 Deleted Module Test Disposition

Baseline: `e896a11024bd5a1e1ef03c094a70032f4e6c765f`.

All 56 authored tests from the six deleted V1 suites are listed below. A
replacement names the V2 owner test that preserves or strengthens the semantic
case. `INTENTIONAL DELETE` means the old public behavior was itself removed by
Ticket 03 and is guarded by a zero-definition/API inventory rather than by a
compatibility assertion.

| Deleted suite | Deleted test | V2 disposition |
|---|---|---|
| mutation rails | `certified_required_is_false_by_default` | `document_profile_and_require_certified_are_validated_in_combined_registry` plus the three `admission_*_tag` cases; no default authority constructor remains |
| mutation rails | `asset_returns_the_registered_declaration` | `external_catalog_validates_payload_and_dependency_kinds` |
| mutation rails | `catalog_is_empty_tracks_registration` | `external_catalog_rejects_duplicate_and_unresolved_assets`; closed catalogs replace observable mutable emptiness |
| mutation rails | `validate_rejects_value_outside_allowed_set` | `model_value_outside_schema_rejects` |
| mutation rails | `validate_accepts_value_inside_allowed_set` | `agreeing_model_packs_merge_and_retain_sorted_provenance` |
| mutation rails | `model_conflict_values_render_in_first_seen_order_for_same_named_packs` | `model_conflict_is_permutation_stable_and_retains_both_locations`; canonical order replaces insertion order |
| mutation rails | `composed_model_is_empty_tracks_facts` | `model_duplicate_fact_is_idempotent_and_observable`; closed composed result replaces mutable emptiness |
| mutation rails | `reexport_with_two_distinct_origins_is_ambiguous` | `three_origin_ambiguity_retains_all_candidates_independent_of_input_order` |
| mutation rails | `module_count_reflects_distinct_modules` | `explicit_module_current_selection_and_typed_namespaces_resolve` plus `duplicate_module_preserves_both_locations` |
| mutation rails | `pack_graph_len_reflects_manifest_count` | `nonempty_pack_graph_requires_lockfile` plus `duplicate_pack_rejects_before_graph_validation` |
| mutation rails | `pack_graph_is_empty_tracks_manifests` | `empty_pack_graph_rejects_unexpected_lockfile` |
| graph | `small_graph_resolves_deterministically` | `explicit_module_current_selection_and_typed_namespaces_resolve` |
| graph | `local_declaration_shadows_import` | `local_declaration_shadows_direct_import` |
| graph | `direct_import_precedence_over_reexport` | `direct_import_precedes_reexport` |
| graph | `binder_view_projection_is_singleton` | `explicit_module_current_selection_and_typed_namespaces_resolve`; sealed `ResolvedModuleBindingsV2` replaces the V1 binder-view projection |
| graph | `self_cycle_rejects` | `self_import_cycle_rejects_with_one_edge_witness` |
| graph | `acyclic_diamond_resolves` | `acyclic_diamond_collapses_identical_origin` |
| catalog | `register_and_enumerate_catalog` | `external_catalog_validates_payload_and_dependency_kinds` |
| catalog | `duplicate_catalog_name_rejects` | `external_catalog_rejects_duplicate_and_unresolved_assets` |
| catalog | `same_name_distinct_kinds_coexist` | `external_catalog_accepts_exact_closed_ten_kind_payload_matrix` |
| catalog | `view_inline_select_cycle_rejects` | `external_catalog_cycle_witness_excludes_acyclic_tail_and_is_permutation_stable` |
| catalog | `view_inline_select_over_base_table_resolves` | `document_view_can_depend_on_external_base_asset` |
| catalog | `require_certified_non_certified_profile_rejects` | `document_profile_and_require_certified_are_validated_in_combined_registry` |
| catalog | `require_certified_all_certified_resolves` | `require_certified_traverses_activated_external_pack_contracts` |
| catalog | `use_pack_known_manifest_resolves` | `pack_activation_and_model_extension_are_observable` |
| catalog | `use_pack_unknown_manifest_rejects` | `use_pack_rejects_unknown_pack_with_checked_activation_location` |
| catalog | `use_pack_invalid_lockfile_path_rejects` | `activation_lock_reference_must_match_sealed_reference` |
| packs | `manifest_and_lock_round_trip` | `canonical_lock_roundtrip_is_byte_exact` plus `pack_lock_digest_version_and_reference_are_sealed` |
| packs | `suite_membership_is_deterministic` | `duplicate_export_and_suite_member_are_semantically_idempotent` plus `pack_fingerprint_is_permutation_stable` |
| packs | `signature_instantiates` | `external_signature_preserves_arity_and_parameter_order` plus types `predicate_calls_are_checked_once_and_retain_opaque_identity_and_effects`; module-side instantiation was deleted |
| packs | `query_metadata_model` | `INTENTIONAL DELETE`: orphan query metadata DTOs have no production caller and zero definitions |
| packs | `agreeing_model_packs_compose` | `agreeing_model_packs_merge_and_retain_sorted_provenance` |
| version/lock | `version_parse_accepts_canonical_and_rejects_malformed` | `canonical_ids_versions_and_digests_reject_alias_spellings` plus generated `version_*_rejects` cases |
| version/lock | `version_ordering_is_numeric_not_lexical` | `version_numeric_order_and_display_are_canonical` |
| version/lock | `version_range_membership_and_satisfiability` | `version_range_is_half_open_and_requires_progress` |
| version/lock | `version_and_range_display` | `version_numeric_order_and_display_are_canonical` plus `version_range_is_half_open_and_requires_progress` |
| version/lock | `query_kind_tokens_are_stable` | `INTENTIONAL DELETE`: orphan query metadata kinds were removed; closed namespace/callable/gate tag tables are tested separately |
| version/lock | `severity_tokens_and_ordering` | `INTENTIONAL DELETE`: V1 query-metadata severity was removed; diagnostics use stable codes and structured locations |
| version/lock | `lockfile_len_and_is_empty` | `nonempty_pack_graph_requires_lockfile` plus `empty_pack_graph_rejects_unexpected_lockfile`; mutable length/emptiness is not public authority |
| version/lock | `lockfile_parse_skips_blank_lines` | `lock_blank_line_is_rejected_with_that_line_as_primary`; canonical V2 intentionally rejects rather than skips |
| version/lock | `lockfile_parse_rejects_malformed_lines` | `lock_blank_line_is_rejected_with_that_line_as_primary`, `lock_without_final_newline_rejects_canonicality`, and `lock_source_length_mismatch_has_checked_source_span` |
| version/lock | `verify_against_accepts_exact_lock` | `pack_lock_digest_version_and_reference_are_sealed` |
| version/lock | `verify_against_rejects_version_drift` | `lock_version_drift_is_rejected_exactly` |
| version/lock | `verify_against_rejects_missing_pin` | `lock_missing_and_stale_pins_are_both_rejected` |
| version/lock | `verify_against_rejects_stale_pin` | `lock_missing_and_stale_pins_are_both_rejected` |
| hellgate | `ambiguous_import_two_reexports_rejects` | `three_origin_ambiguity_retains_all_candidates_independent_of_input_order` plus `ambiguity_primary_tracks_first_origin_not_first_target_edge` |
| hellgate | `overlapping_pack_export_rejects` | `overlapping_pack_export_rejects_with_both_locations` |
| hellgate | `duplicate_pack_name_rejects` | `duplicate_pack_rejects_before_graph_validation` |
| hellgate | `import_cycle_rejects` | `disconnected_cycle_rejects_whole_graph_with_located_witness` plus `competing_cycles_choose_same_canonical_witness_under_permutation` |
| hellgate | `compat_mismatch_rejects` | `compatibility_mismatch_rejects` |
| hellgate | `lock_mismatch_rejects` | `pack_lock_digest_version_and_reference_are_sealed` plus `lock_version_drift_is_rejected_exactly` |
| hellgate | `dependency_unsatisfied_rejects` | `unsatisfied_pack_dependency_retains_target_location` |
| hellgate | `model_conflict_rejects` | `conflicting_values_across_distinct_model_packs_reject_with_both_locations` |
| hellgate | `invalid_model_data_rejects` | `model_value_outside_schema_rejects` |
| hellgate | `not_exported_rejects` | `private_declaration_is_not_importable` plus `absent_declaration_is_not_exported` |
| hellgate | `signature_arity_mismatch_rejects` | types `builtin_bad_arity_rejects_without_fabricating_a_target` and `predicate_calls_are_checked_once_and_retain_opaque_identity_and_effects`; invocation checking moved to the sole types kernel |

Count check: `11 + 6 + 10 + 5 + 13 + 11 = 56` deleted authored cases;
all 56 rows above have an explicit semantic disposition.
