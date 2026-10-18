//! Source null placement reaches the single sealed execution order contract.

use super::*;

#[test]
fn source_null_policy_is_sealed_encoded_and_preserved_by_canonical_format() {
    use quanta_gqlang_v2_pipeline::{ExecutionNullOrderV3, ExecutionSortDirectionV3};

    let authority = authority_with_records();
    let mut first_digest = None;
    for (clause, expected, explicit_first) in [
        ("desc nulls first", ExecutionNullOrderV3::First, true),
        ("desc nulls last", ExecutionNullOrderV3::Last, false),
        ("desc", ExecutionNullOrderV3::Last, false),
    ] {
        let source = format!("from records r | order by r.id {clause}");
        let compiled = compile_source_document_v3(DOC_SOURCE, &source, &authority).unwrap();
        let root = compiled.execution().items()[0].relation().unwrap().root();
        assert_eq!(root.family(), AlgebraFamilyV2::Sort);
        assert_eq!(root.schema().order()[0].null_order(), expected);
        assert_eq!(
            root.schema().order()[0].direction(),
            ExecutionSortDirectionV3::Desc
        );
        if explicit_first {
            first_digest = Some(compiled.execution().contract_digest());
        } else {
            assert_ne!(first_digest, Some(compiled.execution().contract_digest()));
        }
        let formatted = format_source_document_v2(DOC_SOURCE, &source, authority.budget()).unwrap();
        assert_eq!(formatted.contains("nulls first"), explicit_first);
        let again = compile_source_document_v3(DOC_SOURCE, &formatted, &authority).unwrap();
        assert_eq!(
            compiled.execution().contract_digest(),
            again.execution().contract_digest()
        );
    }
    let omitted = compile_source_document_v3(
        DOC_SOURCE,
        "from records r | order by r.id desc",
        &authority,
    )
    .unwrap();
    let explicit = compile_source_document_v3(
        DOC_SOURCE,
        "from records r | order by r.id desc nulls last",
        &authority,
    )
    .unwrap();
    assert_eq!(
        omitted.execution().contract_digest(),
        explicit.execution().contract_digest()
    );
}
