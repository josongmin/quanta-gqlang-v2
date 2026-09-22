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
            optional: false,
            location: location(2, 3),
        }],
        cardinality: FrontendRelationCardinalityV2::Bag,
        provenance: "host".to_owned(),
        location: location(1, 4),
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
    wrong_shape.fields[0].scalar_type = FrontendScalarTypeV2::String;
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
