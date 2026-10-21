#![expect(
    clippy::expect_used,
    reason = "authority boundary tests assert exact typed outcomes"
)]

use quanta_gqlang_v2_sema::{
    ClosedFrontendAuthorityBuilderV2, FrontendAuthorityLocationV2, FrontendAuthoritySourceV2,
    FrontendBudgetV2, FrontendGraphReferenceInputV2, FrontendLanguageVersionV2,
    FrontendLockfileInputV2, FrontendPackInputV2, FrontendRelationCardinalityV2,
    FrontendRelationFieldInputV2, FrontendRelationInputV2, FrontendScalarTypeV2,
    GraphReferenceKindV2, SourceIdV2,
};

const HOST: SourceIdV2 = SourceIdV2::from_raw(1);
const LOCK: SourceIdV2 = SourceIdV2::from_raw(2);
const DIGEST: &str = "0000000000000000000000000000000000000000000000000000000000000000";

fn location(start: usize, end: usize) -> FrontendAuthorityLocationV2 {
    FrontendAuthorityLocationV2 {
        source_id: HOST,
        start,
        end,
    }
}

fn builder() -> ClosedFrontendAuthorityBuilderV2 {
    ClosedFrontendAuthorityBuilderV2::try_new(
        FrontendBudgetV2::standard_v2().expect("standard budget"),
        vec![FrontendAuthoritySourceV2 {
            source_id: HOST,
            source_len: 64,
        }],
        "app".to_owned(),
        location(0, 1),
        FrontendLanguageVersionV2 {
            major: 1,
            minor: 0,
            patch: 0,
        },
    )
    .expect("builder starts")
}

#[test]
fn raw_rule_budget_is_independent_and_copied_to_receipt() {
    let base = FrontendBudgetV2::standard_v2().expect("standard budget");
    assert_eq!(
        base.max_rule_program_rules(),
        base.max_rule_graph_nodes() + base.max_rule_graph_edges()
    );
    let tuned = base
        .clone()
        .try_with_rule_program_rules(7)
        .expect("positive raw-rule cap");
    assert_eq!(tuned.max_rule_program_rules(), 7);
    assert_eq!(tuned.downstream_receipt().max_rule_program_rules(), 7);
    assert_eq!(tuned.max_rule_graph_nodes(), base.max_rule_graph_nodes());
    assert_eq!(tuned.max_rule_graph_edges(), base.max_rule_graph_edges());
    assert_eq!(
        base.try_with_rule_program_rules(0)
            .expect_err("zero raw-rule cap rejects")
            .code(),
        "frontend.invalid_budget"
    );
}

#[test]
fn facade_builder_seals_relation_and_graph_authority_in_one_registry() {
    let mut builder = builder();
    builder.add_relation(FrontendRelationInputV2 {
        authority: quanta_gqlang_v2_sema::FrontendRelationAuthorityV2::Host,
        provenance_namespace: "host".to_owned(),
        qualified_name: "app::records".to_owned(),
        lookup_name: "records".to_owned(),
        fields: vec![FrontendRelationFieldInputV2 {
            name: "id".to_owned(),
            scalar_type: FrontendScalarTypeV2::Int,
            sequence: false,
            optional: false,
            location: location(2, 3),
        }],
        cardinality: FrontendRelationCardinalityV2::Bag,
        provenance: "host".to_owned(),
        location: location(1, 4),
        host_binding_identity: None,
    });
    builder.add_graph_reference(FrontendGraphReferenceInputV2 {
        spelling: "entry".to_owned(),
        kind: GraphReferenceKindV2::Entity,
        entity_subtype: Some("function".to_owned()),
        location: location(4, 9),
    });
    let authority = builder.finish().expect("authority seals");
    assert!(authority.relations().get("records").is_some());
    assert_eq!(authority.graph_references().len(), 1);
}

#[test]
fn facade_builder_seals_canonical_workspace_files_without_name_inference() {
    let mut builder = builder();
    builder.add_relation(FrontendRelationInputV2::canonical_workspace_files(
        location(1, 4),
        location(2, 3),
    ));
    let authority = builder.finish().expect("workspace-files authority seals");
    let source = authority
        .relations()
        .get("files")
        .expect("canonical lookup is owner supplied");
    assert_eq!(
        source.id().authority(),
        quanta_gqlang_v2_sema::RelationAuthorityV2::CanonicalWorkspaceFiles
    );
}

#[test]
fn facade_builder_rejects_workspace_files_name_and_shape_mutations() {
    let mut wrong_name =
        FrontendRelationInputV2::canonical_workspace_files(location(1, 4), location(2, 3));
    wrong_name.lookup_name = "workspace_files".to_owned();
    let mut wrong_name_builder = builder();
    wrong_name_builder.add_relation(wrong_name);
    let error = wrong_name_builder
        .finish()
        .expect_err("name mutation rejects");
    assert_eq!(error.code(), "types.invalid_relation_authority");

    let mut wrong_shape =
        FrontendRelationInputV2::canonical_workspace_files(location(1, 4), location(2, 3));
    wrong_shape
        .fields
        .first_mut()
        .expect("canonical fixture carries exactly one field")
        .scalar_type = FrontendScalarTypeV2::String;
    let mut wrong_shape_builder = builder();
    wrong_shape_builder.add_relation(wrong_shape);
    let error = wrong_shape_builder
        .finish()
        .expect_err("shape mutation rejects");
    assert_eq!(error.code(), "types.invalid_relation_authority");
}

#[test]
fn facade_builder_rejects_empty_duplicate_and_out_of_bounds_sources() {
    let budget = FrontendBudgetV2::standard_v2().expect("standard budget");
    let empty = ClosedFrontendAuthorityBuilderV2::try_new(
        budget.clone(),
        Vec::new(),
        "app".to_owned(),
        location(0, 1),
        FrontendLanguageVersionV2 {
            major: 1,
            minor: 0,
            patch: 0,
        },
    )
    .err()
    .expect("empty authority rejects");
    assert_eq!(empty.code(), "sema.empty_authority_sources");

    let duplicate = ClosedFrontendAuthorityBuilderV2::try_new(
        budget.clone(),
        vec![
            FrontendAuthoritySourceV2 {
                source_id: HOST,
                source_len: 8,
            },
            FrontendAuthoritySourceV2 {
                source_id: HOST,
                source_len: 8,
            },
        ],
        "app".to_owned(),
        location(0, 1),
        FrontendLanguageVersionV2 {
            major: 1,
            minor: 0,
            patch: 0,
        },
    )
    .expect("duplicates reject at finish")
    .finish()
    .expect_err("duplicate sources reject");
    assert_eq!(duplicate.code(), "hir.source_registry.duplicate_source_id");

    let out_of_bounds = ClosedFrontendAuthorityBuilderV2::try_new(
        budget,
        vec![FrontendAuthoritySourceV2 {
            source_id: HOST,
            source_len: 1,
        }],
        "app".to_owned(),
        location(0, 2),
        FrontendLanguageVersionV2 {
            major: 1,
            minor: 0,
            patch: 0,
        },
    )
    .expect("location rejects at finish")
    .finish()
    .expect_err("out of bounds location rejects");
    assert_eq!(
        out_of_bounds.code(),
        "hir.source_registry.location_out_of_bounds"
    );
}

#[test]
fn facade_builder_seals_pack_and_canonical_lockfile_without_owner_proofs() {
    let lock_text = format!("acme.pack 1.0.0 {DIGEST}\n");
    let mut builder = ClosedFrontendAuthorityBuilderV2::try_new(
        FrontendBudgetV2::standard_v2().expect("standard budget"),
        vec![
            FrontendAuthoritySourceV2 {
                source_id: HOST,
                source_len: 64,
            },
            FrontendAuthoritySourceV2 {
                source_id: LOCK,
                source_len: lock_text.len(),
            },
        ],
        "app".to_owned(),
        location(0, 1),
        FrontendLanguageVersionV2 {
            major: 1,
            minor: 0,
            patch: 0,
        },
    )
    .expect("builder starts");
    builder.add_pack(FrontendPackInputV2 {
        pack_id: "acme.pack".to_owned(),
        version: FrontendLanguageVersionV2 {
            major: 1,
            minor: 0,
            patch: 0,
        },
        content_digest: DIGEST.to_owned(),
        language_lower: FrontendLanguageVersionV2 {
            major: 1,
            minor: 0,
            patch: 0,
        },
        language_upper: FrontendLanguageVersionV2 {
            major: 2,
            minor: 0,
            patch: 0,
        },
        location: location(1, 10),
    });
    builder.add_lockfile(FrontendLockfileInputV2 {
        path: "acme.lock".to_owned(),
        source_id: LOCK,
        text: lock_text.clone(),
        location: location(10, 19),
    });
    let authority = builder.finish().expect("pack authority seals");
    assert_ne!(
        authority.modules().pack_authority().graph_fingerprint(),
        &[0; 32]
    );
    assert_eq!(
        authority
            .modules()
            .pack_authority()
            .verified_lock()
            .to_text(),
        lock_text
    );
}

#[test]
fn relation_sequence_field_is_independent_of_row_cardinality() {
    use quanta_gqlang_v2_types::{ScalarTyV1, ValueTypeV1};
    for optional in [false, true] {
        let mut input = builder();
        input.add_relation(FrontendRelationInputV2 {
            authority: quanta_gqlang_v2_sema::FrontendRelationAuthorityV2::Host,
            provenance_namespace: "host".into(),
            qualified_name: "app::sequences".into(),
            lookup_name: "sequences".into(),
            fields: vec![FrontendRelationFieldInputV2 {
                name: "values".into(),
                scalar_type: FrontendScalarTypeV2::String,
                sequence: true,
                optional,
                location: location(2, 3),
            }],
            cardinality: FrontendRelationCardinalityV2::Bag,
            provenance: "test".into(),
            location: location(1, 16),
            host_binding_identity: None,
        });
        let authority = input.finish().expect("sequence authority");
        let row = authority
            .relations()
            .get("sequences")
            .expect("host relation")
            .relation_type()
            .row();
        let sequence =
            ValueTypeV1::try_list(ValueTypeV1::scalar(ScalarTyV1::String)).expect("sequence");
        let expected = if optional {
            ValueTypeV1::try_optional(sequence).expect("optional sequence")
        } else {
            sequence
        };
        assert_eq!(row.fields().len(), 1);
        let field = row.fields().first().expect("one sequence field");
        assert_eq!(field.value_type(), &expected);
    }
}

#[test]
fn facade_relation_and_rule_inputs_preserve_every_scalar_and_optional_type() {
    use quanta_gqlang_v2_modules::ExternalCatalogPayloadV2;
    use quanta_gqlang_v2_sema::{
        FrontendCallableParameterInputV2, FrontendCatalogAssetInputV2, FrontendCatalogPayloadV2,
    };
    use quanta_gqlang_v2_types::{ScalarTyV1, ValueTypeV1};
    let cases = [
        (FrontendScalarTypeV2::Bool, ScalarTyV1::Bool),
        (FrontendScalarTypeV2::Int, ScalarTyV1::Int),
        (FrontendScalarTypeV2::Float, ScalarTyV1::Float),
        (FrontendScalarTypeV2::String, ScalarTyV1::String),
        (FrontendScalarTypeV2::Duration, ScalarTyV1::Duration),
        (FrontendScalarTypeV2::Location, ScalarTyV1::Location),
        (FrontendScalarTypeV2::Path, ScalarTyV1::Path),
        (FrontendScalarTypeV2::RepoPath, ScalarTyV1::RepoPath),
        (FrontendScalarTypeV2::EntityRef, ScalarTyV1::EntityRef),
        (FrontendScalarTypeV2::SymbolRef, ScalarTyV1::SymbolRef),
        (FrontendScalarTypeV2::RuleRef, ScalarTyV1::RuleRef),
        (FrontendScalarTypeV2::SourceScope, ScalarTyV1::SourceScope),
        (FrontendScalarTypeV2::ProofTree, ScalarTyV1::ProofTree),
    ];
    for optional in [false, true] {
        let mut input = builder();
        input.add_relation(FrontendRelationInputV2 {
            authority: quanta_gqlang_v2_sema::FrontendRelationAuthorityV2::Host,
            provenance_namespace: "host".to_owned(),
            qualified_name: "app::typed".to_owned(),
            lookup_name: "typed".to_owned(),
            fields: cases
                .iter()
                .enumerate()
                .map(|(index, (scalar_type, _))| FrontendRelationFieldInputV2 {
                    name: format!("field_{index}"),
                    scalar_type: *scalar_type,
                    sequence: false,
                    optional,
                    location: location(2 + index, 3 + index),
                })
                .collect(),
            cardinality: FrontendRelationCardinalityV2::Bag,
            provenance: "test".to_owned(),
            location: location(1, 16),
            host_binding_identity: None,
        });
        input.add_catalog_asset(FrontendCatalogAssetInputV2 {
            module_name: "app".to_owned(),
            local_name: "typed_rule".to_owned(),
            payload: FrontendCatalogPayloadV2::Rule {
                parameters: cases
                    .iter()
                    .enumerate()
                    .map(
                        |(index, (scalar_type, _))| FrontendCallableParameterInputV2 {
                            name: format!("param_{index}"),
                            scalar_type: *scalar_type,
                            optional,
                            location: location(20 + index, 21 + index),
                        },
                    )
                    .collect(),
                result: FrontendScalarTypeV2::Bool,
            },
            provenance: "test".to_owned(),
            location: location(19, 34),
        });
        let authority = input.finish().expect("curated input matrix seals");
        let row = authority
            .relations()
            .get("typed")
            .expect("authored relation retained")
            .relation_type()
            .row();
        assert_eq!(row.fields().len(), cases.len());
        let signature = authority
            .modules()
            .external_catalog()
            .iter()
            .find_map(|asset| {
                if let ExternalCatalogPayloadV2::Rule(signature) = asset.payload() {
                    Some(signature)
                } else {
                    None
                }
            })
            .expect("authored rule signature retained");
        assert_eq!(signature.parameters().len(), cases.len());
        assert_eq!(signature.result(), &ValueTypeV1::bool());
        for (index, (_, owner)) in cases.iter().enumerate() {
            let scalar = ValueTypeV1::scalar(owner.clone());
            let expected = if optional {
                ValueTypeV1::try_optional(scalar).expect("single optional wrapper")
            } else {
                scalar
            };
            let field = row
                .fields()
                .get(index)
                .expect("one row field per authored scalar");
            assert_eq!(field.name(), format!("field_{index}"));
            assert_eq!(field.value_type(), &expected);
            let parameter = signature
                .parameters()
                .get(index)
                .expect("one parameter per authored scalar");
            assert_eq!(parameter.name(), format!("param_{index}"));
            assert_eq!(parameter.value_type(), &expected);
        }
    }
}
