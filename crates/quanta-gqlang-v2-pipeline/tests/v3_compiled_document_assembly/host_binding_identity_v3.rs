//! The existing catalog identity reaches the same sealed Source and canonical contract.

use super::*;

#[test]
fn host_instance_identity_survives_sealing_and_changes_contract_without_source_change_v3() {
    let source = "from records r | project r.id as selected";
    let mut previous = None;
    for identity in [None, Some([1; 32]), Some([2; 32])] {
        let authority =
            authority_with_records_binding_v3(ValueTypeV1::int(), "app::records", identity);
        let compiled = compile_source_document_v3(DOC_SOURCE, source, &authority)
            .expect("bound host source compiles");
        let source_node = &compiled.execution().items()[0]
            .relation()
            .expect("bound source has relation")
            .root()
            .children()[0];
        assert_eq!(
            source_node
                .source()
                .expect("scan retains host source")
                .host_binding_identity(),
            identity
        );
        assert_ne!(previous, Some(compiled.execution().contract_digest()));
        previous = Some(compiled.execution().contract_digest());
        let again = compile_source_document_v3(DOC_SOURCE, source, &authority)
            .expect("bound host source recompiles");
        assert_eq!(compiled.execution(), again.execution());
    }
}

#[test]
fn canonical_files_cannot_be_relabelled_as_a_bound_host_instance_v3() {
    let location = FrontendAuthorityLocationV2 {
        source_id: HOST_SOURCE,
        start: 0,
        end: 1,
    };
    let mut builder = quanta_gqlang_v2_pipeline::ClosedFrontendAuthorityBuilderV2::try_new(
        FrontendBudgetV2::standard_v2().expect("standard budget"),
        vec![FrontendAuthoritySourceV2 {
            source_id: HOST_SOURCE,
            source_len: 1,
        }],
        "app".into(),
        location,
        FrontendLanguageVersionV2 {
            major: 1,
            minor: 0,
            patch: 0,
        },
    )
    .expect("authority builder starts");
    let mut relation = FrontendRelationInputV2::canonical_workspace_files(location, location);
    relation.host_binding_identity = Some([7; 32]);
    builder.add_relation(relation);
    let failure = builder
        .finish()
        .expect_err("canonical files reject host binding");
    assert_eq!(failure.code(), "types.invalid_relation_authority");
}
