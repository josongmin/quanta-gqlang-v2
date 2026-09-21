//! Feature-gated reference transactions for downstream owner tests.
//!
//! This module is not a frontend API. It gives dependent crate tests a real
//! sealed document without making them depend directly on HIR, Modules, or the
//! authority builders. The fixtures deliberately cover only stable, legal
//! relation shapes and cannot mint semantic carriers.

use crate::{
    resolve_checked_document_v2, CheckedSyntaxDocumentV2, ClosedFrontendAuthorityV2,
    FrontendBudgetV2, GraphReferenceCatalogV2, SemanticDocumentV2, SemanticItemIdV2,
};
use quanta_gqlang_v2_hir::{
    HirBuilderV1, HirDocumentV1, HirNodeIdV1, LocatedSpanV2, ProgramItemHirV1, SourceIdV2,
    SourceRegistrationV2, SourceRegistryViewV2, SpanV1,
};
use quanta_gqlang_v2_modules::{
    CurrentModuleSelectionInputV2, ModuleAuthorityBuilderV2, ModuleIdV2, ModuleNodeInputV2,
    VersionV2,
};
use quanta_gqlang_v2_types::{
    CardinalityV1, ProvenanceNamespaceV2, RecordFieldTypeV1, RecordTypeV1, RelationCatalogV2,
    RelationSourceInputV2, ValueTypeV1,
};

/// Fixed source identity used by reference relation fixtures.
pub const REFERENCE_SOURCE_V2: SourceIdV2 = SourceIdV2::from_raw(9);

/// An item identity guaranteed to be absent from every reference fixture.
#[must_use]
pub fn reference_absent_item_id_v2() -> SemanticItemIdV2 {
    SemanticItemIdV2::new(REFERENCE_SOURCE_V2, HirNodeIdV1::from_raw(u32::MAX))
}

/// Legal source relation shapes available to downstream tests.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReferenceRelationFixtureV2 {
    /// One authoritative Scan.
    Scan,
    /// Scan followed by a Bool-proven Filter.
    Filter,
    /// Two scans combined by an explicit predicate-free Cross join.
    CrossJoin,
}

/// A sealed reference transaction plus its one relation item identity.
#[derive(Clone, Debug)]
pub struct ReferenceRelationDocumentV2 {
    document: SemanticDocumentV2,
    item_id: SemanticItemIdV2,
}

impl ReferenceRelationDocumentV2 {
    /// Borrow the complete semantic document.
    #[must_use]
    pub const fn document(&self) -> &SemanticDocumentV2 {
        &self.document
    }

    /// Identity of the sole relation item.
    #[must_use]
    pub const fn item_id(&self) -> SemanticItemIdV2 {
        self.item_id
    }
}

/// Resolve one reference relation fixture through the real semantic transaction.
///
/// # Errors
///
/// Returns a fixture-construction error if the fixed fixture no longer
/// satisfies the semantic contract.
pub fn reference_relation_document_v2(
    fixture: ReferenceRelationFixtureV2,
) -> Result<ReferenceRelationDocumentV2, String> {
    let builder = HirBuilderV1::new();
    let relation = match fixture {
        ReferenceRelationFixtureV2::Scan => {
            builder.relation_source("records", SpanV1::new(5, 12), None, SpanV1::new(5, 12))
        }
        ReferenceRelationFixtureV2::Filter => builder.relation_filter(
            builder.relation_source("records", SpanV1::new(5, 12), None, SpanV1::new(5, 12)),
            builder.bool_literal(true, SpanV1::new(19, 23)),
            SpanV1::new(5, 23),
        ),
        ReferenceRelationFixtureV2::CrossJoin => {
            let left =
                builder.relation_source("records", SpanV1::new(5, 12), None, SpanV1::new(5, 12));
            let right =
                builder.relation_source("other", SpanV1::new(19, 24), None, SpanV1::new(19, 24));
            builder.cross_join(left, right, SpanV1::new(5, 24))
        }
    };
    let query = builder.query(relation, None, SpanV1::new(0, 24));
    let item_id = SemanticItemIdV2::new(REFERENCE_SOURCE_V2, query.id);
    let document = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id: REFERENCE_SOURCE_V2,
            source_len: 32,
            items: vec![ProgramItemHirV1::Query(Box::new(query))],
        })
        .map_err(|error| format!("{error:?}"))?,
        &reference_authority_v2()?,
    )
    .map_err(|error| format!("{error:?}"))?;
    Ok(ReferenceRelationDocumentV2 { document, item_id })
}

fn reference_authority_v2() -> Result<ClosedFrontendAuthorityV2, String> {
    let host = SourceRegistryViewV2::try_new(vec![SourceRegistrationV2 {
        source_id: SourceIdV2::from_raw(1),
        source_len: 256,
    }])
    .map_err(|error| error.to_string())?;
    let location = host
        .try_check_location(LocatedSpanV2 {
            source_id: SourceIdV2::from_raw(1),
            span: SpanV1::new(0, 1),
        })
        .map_err(|error| format!("{error:?}"))?;
    let module =
        ModuleIdV2::try_new("app", location.clone()).map_err(|error| format!("{error:?}"))?;
    let budget = FrontendBudgetV2::standard_v2().map_err(|error| format!("{error:?}"))?;
    let profile = budget
        .authority_profile()
        .map_err(|error| format!("{error:?}"))?;
    let mut modules = ModuleAuthorityBuilderV2::try_new(profile.clone(), host.clone())
        .map_err(|error| format!("{error:?}"))?;
    modules
        .try_add_module(ModuleNodeInputV2 {
            module_id: module.clone(),
            location: location.clone(),
        })
        .map_err(|error| format!("{error:?}"))?;
    let modules = modules
        .finish(
            CurrentModuleSelectionInputV2 {
                module_id: module,
                location: location.clone(),
            },
            VersionV2::new(1, 0, 0),
        )
        .map_err(|error| error.to_string())?;
    let relation_inputs = ["records", "other"]
        .into_iter()
        .map(|lookup_name| {
            let provenance_namespace = ProvenanceNamespaceV2::try_new("host", location.clone())
                .map_err(|error| format!("{error:?}"))?;
            let field = RecordFieldTypeV1::try_new(
                if lookup_name == "records" {
                    "id"
                } else {
                    "other_id"
                },
                ValueTypeV1::int(),
                location.clone(),
            )
            .map_err(|error| format!("{error:?}"))?;
            let row = RecordTypeV1::try_new(vec![field]).map_err(|error| format!("{error:?}"))?;
            Ok(RelationSourceInputV2 {
                provenance_namespace,
                qualified_name: format!("app::{lookup_name}"),
                lookup_name: lookup_name.to_owned(),
                row,
                cardinality: CardinalityV1::Bag,
                provenance: "host".to_owned(),
                location: location.clone(),
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let relations = RelationCatalogV2::try_new(relation_inputs, &host, profile.clone())
        .map_err(|error| format!("{error:?}"))?;
    let graphs = GraphReferenceCatalogV2::try_new(Vec::new(), &host, profile)
        .map_err(|error| format!("{error:?}"))?;
    ClosedFrontendAuthorityV2::try_new(modules, relations, graphs, budget)
        .map_err(|error| format!("{error:?}"))
}
