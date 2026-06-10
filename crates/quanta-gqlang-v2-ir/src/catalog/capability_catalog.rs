#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CatalogSourceKindV1 {
    FrontendSpec,
    CuratedReadOnlyExample,
    CompletionBaseline,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CatalogProvenanceV1 {
    pub source_kind: CatalogSourceKindV1,
    pub source_file: &'static str,
    pub source_marker: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CatalogScopeMarkerV1 {
    DocumentRoot,
    ShowCategoryTail,
    DescribeTargetTail,
    ReadOnlyShell,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CapabilityCategoryV1 {
    QueryRoot,
    InteractiveRoot,
    DeclarationRoot,
    MetaCommand,
    ShowCategory,
    DescribeTargetKind,
    CompositionOperator,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CapabilityCatalogRowV1 {
    pub stable_id: &'static str,
    pub label: &'static str,
    pub category: CapabilityCategoryV1,
    pub summary: &'static str,
    pub reason: &'static str,
    pub scope: CatalogScopeMarkerV1,
    pub provenance: CatalogProvenanceV1,
}

const CATALOG_SOURCE_FILE_V1: &str = "crates/quanta-gqlang-v2-ir/src/catalog/capability_catalog.rs";
const EXTERNAL_FRONTEND_CATALOG_AUTHORITY_V1: &str = "external-frontend-catalog";

const fn provenance_v1(
    source_kind: CatalogSourceKindV1,
    source_marker: &'static str,
) -> CatalogProvenanceV1 {
    CatalogProvenanceV1 {
        source_kind,
        source_file: CATALOG_SOURCE_FILE_V1,
        source_marker,
    }
}

const fn capability_row_v1(
    stable_id: &'static str,
    label: &'static str,
    category: CapabilityCategoryV1,
    summary: &'static str,
    reason: &'static str,
    scope: CatalogScopeMarkerV1,
) -> CapabilityCatalogRowV1 {
    CapabilityCatalogRowV1 {
        stable_id,
        label,
        category,
        summary,
        reason,
        scope,
        provenance: provenance_v1(CatalogSourceKindV1::FrontendSpec, stable_id),
    }
}

pub const CAPABILITY_CATALOG_ROWS_V1: &[CapabilityCatalogRowV1] = &[
    capability_row_v1(
        "root.select",
        "SELECT",
        CapabilityCategoryV1::QueryRoot,
        "table-only query root",
        "Read-only query entrypoint stays discoverable without backend execution.",
        CatalogScopeMarkerV1::DocumentRoot,
    ),
    capability_row_v1(
        "root.find",
        "FIND",
        CapabilityCategoryV1::QueryRoot,
        "graph/reachability query root",
        "Graph query root must be present as frontend-owned language surface only.",
        CatalogScopeMarkerV1::DocumentRoot,
    ),
    capability_row_v1(
        "root.check",
        "CHECK",
        CapabilityCategoryV1::QueryRoot,
        "dual-lane policy/query root",
        "Policy lane must remain typed without implying runtime confirmation truth.",
        CatalogScopeMarkerV1::DocumentRoot,
    ),
    capability_row_v1(
        "root.explain",
        "EXPLAIN",
        CapabilityCategoryV1::QueryRoot,
        "truth/evidence query root",
        "Evidence-oriented query root is frontend-owned even when truth materialization is absent.",
        CatalogScopeMarkerV1::DocumentRoot,
    ),
    capability_row_v1(
        "interactive.show",
        "SHOW",
        CapabilityCategoryV1::InteractiveRoot,
        "catalog surface",
        "Interactive catalog introspection must be suggested from deterministic assets.",
        CatalogScopeMarkerV1::DocumentRoot,
    ),
    capability_row_v1(
        "interactive.describe",
        "DESCRIBE",
        CapabilityCategoryV1::InteractiveRoot,
        "target detail surface",
        "Target detail discovery belongs to the frontend catalog, not host DTOs.",
        CatalogScopeMarkerV1::DocumentRoot,
    ),
    capability_row_v1(
        "interactive.sample",
        "SAMPLE",
        CapabilityCategoryV1::InteractiveRoot,
        "retrieval starter surface",
        "Read-only retrieval starters need typed hints without fabricating result truth.",
        CatalogScopeMarkerV1::DocumentRoot,
    ),
    capability_row_v1(
        "interactive.compare",
        "COMPARE",
        CapabilityCategoryV1::InteractiveRoot,
        "comparison surface",
        "Comparison surface stays frontend-owned and host-agnostic.",
        CatalogScopeMarkerV1::DocumentRoot,
    ),
    capability_row_v1(
        "interactive.export",
        "EXPORT",
        CapabilityCategoryV1::InteractiveRoot,
        "export surface",
        "Export intent is public syntax even when execution backends are unavailable.",
        CatalogScopeMarkerV1::DocumentRoot,
    ),
    capability_row_v1(
        "declaration.view",
        "VIEW",
        CapabilityCategoryV1::DeclarationRoot,
        "named query declaration",
        "Named query declarations must be discoverable through typed frontend assets.",
        CatalogScopeMarkerV1::DocumentRoot,
    ),
    capability_row_v1(
        "declaration.rule",
        "RULE",
        CapabilityCategoryV1::DeclarationRoot,
        "check declaration",
        "Rule declarations remain part of the public frontend contract.",
        CatalogScopeMarkerV1::DocumentRoot,
    ),
    capability_row_v1(
        "declaration.model",
        "MODEL",
        CapabilityCategoryV1::DeclarationRoot,
        "provider declaration",
        "Model declarations can be surfaced without leaking runtime provider registries.",
        CatalogScopeMarkerV1::DocumentRoot,
    ),
    capability_row_v1(
        "declaration.pack",
        "PACK",
        CapabilityCategoryV1::DeclarationRoot,
        "pack declaration",
        "Pack declarations support frontend-owned activation workflows.",
        CatalogScopeMarkerV1::DocumentRoot,
    ),
    capability_row_v1(
        "declaration.profile",
        "PROFILE",
        CapabilityCategoryV1::DeclarationRoot,
        "profile declaration",
        "Profile declarations anchor admission policy hints without backend coupling.",
        CatalogScopeMarkerV1::DocumentRoot,
    ),
    capability_row_v1(
        "declaration.gate",
        "GATE",
        CapabilityCategoryV1::DeclarationRoot,
        "gate declaration",
        "Gate declarations remain catalog-visible as public language nouns.",
        CatalogScopeMarkerV1::DocumentRoot,
    ),
    capability_row_v1(
        "meta.use_pack",
        "USE PACK",
        CapabilityCategoryV1::MetaCommand,
        "pack activation command",
        "Pack activation command must stay available from pure typed assets.",
        CatalogScopeMarkerV1::DocumentRoot,
    ),
    capability_row_v1(
        "meta.require_certified",
        "REQUIRE CERTIFIED",
        CapabilityCategoryV1::MetaCommand,
        "trust policy command",
        "Trust policy command must not depend on host-side capability discovery.",
        CatalogScopeMarkerV1::DocumentRoot,
    ),
    capability_row_v1(
        "show.capabilities",
        "capabilities",
        CapabilityCategoryV1::ShowCategory,
        "SHOW capabilities",
        "Plural catalog category is a deterministic interactive tail token.",
        CatalogScopeMarkerV1::ShowCategoryTail,
    ),
    capability_row_v1(
        "show.models",
        "models",
        CapabilityCategoryV1::ShowCategory,
        "SHOW models",
        "Plural catalog category is a deterministic interactive tail token.",
        CatalogScopeMarkerV1::ShowCategoryTail,
    ),
    capability_row_v1(
        "show.views",
        "views",
        CapabilityCategoryV1::ShowCategory,
        "SHOW views",
        "Plural catalog category is a deterministic interactive tail token.",
        CatalogScopeMarkerV1::ShowCategoryTail,
    ),
    capability_row_v1(
        "show.rules",
        "rules",
        CapabilityCategoryV1::ShowCategory,
        "SHOW rules",
        "Plural catalog category is a deterministic interactive tail token.",
        CatalogScopeMarkerV1::ShowCategoryTail,
    ),
    capability_row_v1(
        "show.packs",
        "packs",
        CapabilityCategoryV1::ShowCategory,
        "SHOW packs",
        "Plural catalog category is a deterministic interactive tail token.",
        CatalogScopeMarkerV1::ShowCategoryTail,
    ),
    capability_row_v1(
        "show.profiles",
        "profiles",
        CapabilityCategoryV1::ShowCategory,
        "SHOW profiles",
        "Plural catalog category is a deterministic interactive tail token.",
        CatalogScopeMarkerV1::ShowCategoryTail,
    ),
    capability_row_v1(
        "show.gates",
        "gates",
        CapabilityCategoryV1::ShowCategory,
        "SHOW gates",
        "Plural catalog category is a deterministic interactive tail token.",
        CatalogScopeMarkerV1::ShowCategoryTail,
    ),
    capability_row_v1(
        "describe.capability",
        "capability::",
        CapabilityCategoryV1::DescribeTargetKind,
        "DESCRIBE capability::<path>",
        "Typed prefix seeds capability target lookup without host fetches.",
        CatalogScopeMarkerV1::DescribeTargetTail,
    ),
    capability_row_v1(
        "describe.model",
        "model::",
        CapabilityCategoryV1::DescribeTargetKind,
        "DESCRIBE model::<path>",
        "Typed prefix seeds model target lookup without host fetches.",
        CatalogScopeMarkerV1::DescribeTargetTail,
    ),
    capability_row_v1(
        "describe.view",
        "view::",
        CapabilityCategoryV1::DescribeTargetKind,
        "DESCRIBE view::<path>",
        "Typed prefix seeds view target lookup without host fetches.",
        CatalogScopeMarkerV1::DescribeTargetTail,
    ),
    capability_row_v1(
        "describe.rule",
        "rule::",
        CapabilityCategoryV1::DescribeTargetKind,
        "DESCRIBE rule::<path>",
        "Typed prefix seeds rule target lookup without host fetches.",
        CatalogScopeMarkerV1::DescribeTargetTail,
    ),
    capability_row_v1(
        "describe.pack",
        "pack::",
        CapabilityCategoryV1::DescribeTargetKind,
        "DESCRIBE pack::<path>",
        "Typed prefix seeds pack target lookup without host fetches.",
        CatalogScopeMarkerV1::DescribeTargetTail,
    ),
    capability_row_v1(
        "describe.profile",
        "profile::",
        CapabilityCategoryV1::DescribeTargetKind,
        "DESCRIBE profile::<path>",
        "Typed prefix seeds profile target lookup without host fetches.",
        CatalogScopeMarkerV1::DescribeTargetTail,
    ),
    capability_row_v1(
        "describe.gate",
        "gate::",
        CapabilityCategoryV1::DescribeTargetKind,
        "DESCRIBE gate::<path>",
        "Typed prefix seeds gate target lookup without host fetches.",
        CatalogScopeMarkerV1::DescribeTargetTail,
    ),
];

pub const fn capability_catalog_rows_v1() -> &'static [CapabilityCatalogRowV1] {
    CAPABILITY_CATALOG_ROWS_V1
}

pub const fn capability_catalog_authority_v1() -> &'static str {
    EXTERNAL_FRONTEND_CATALOG_AUTHORITY_V1
}

pub fn capability_catalog_integrity_v1() -> Result<(), &'static str> {
    let rows = capability_catalog_rows_v1();
    for (index, row) in rows.iter().enumerate() {
        if rows
            .iter()
            .skip(index + 1)
            .any(|other| other.stable_id == row.stable_id)
        {
            return Err("capability row stable_id must be unique");
        }
    }
    for row in rows {
        if row.stable_id.trim().is_empty() {
            return Err("capability row stable_id must not be empty");
        }
        if row.label.trim().is_empty() {
            return Err("capability row label must not be empty");
        }
        if row.summary.trim().is_empty() {
            return Err("capability row summary must not be empty");
        }
        if row.reason.trim().is_empty() {
            return Err("capability row reason must not be empty");
        }
        if row.provenance.source_kind != CatalogSourceKindV1::FrontendSpec {
            return Err("capability row provenance must stay frontend-owned");
        }
        if row.provenance.source_file != CATALOG_SOURCE_FILE_V1 {
            return Err("capability row provenance source_file drifted");
        }
        if row.provenance.source_marker != row.stable_id {
            return Err("capability row provenance marker must match stable_id");
        }
        if row.scope != expected_scope_for_category_v1(row.category) {
            return Err("capability row scope/category pairing drifted");
        }
    }
    Ok(())
}

pub fn capability_row_by_id_v1(stable_id: &str) -> Option<&'static CapabilityCatalogRowV1> {
    capability_catalog_rows_v1()
        .iter()
        .find(|row| row.stable_id == stable_id)
}

pub fn capability_row_by_label_v1(label: &str) -> Option<&'static CapabilityCatalogRowV1> {
    capability_catalog_rows_v1()
        .iter()
        .find(|row| row.label == label)
}

const fn expected_scope_for_category_v1(category: CapabilityCategoryV1) -> CatalogScopeMarkerV1 {
    match category {
        CapabilityCategoryV1::ShowCategory => CatalogScopeMarkerV1::ShowCategoryTail,
        CapabilityCategoryV1::DescribeTargetKind => CatalogScopeMarkerV1::DescribeTargetTail,
        CapabilityCategoryV1::QueryRoot
        | CapabilityCategoryV1::InteractiveRoot
        | CapabilityCategoryV1::DeclarationRoot
        | CapabilityCategoryV1::MetaCommand
        | CapabilityCategoryV1::CompositionOperator => CatalogScopeMarkerV1::DocumentRoot,
    }
}
