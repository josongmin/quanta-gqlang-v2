use super::capability_catalog::{
    capability_row_by_id_v1, CatalogProvenanceV1, CatalogScopeMarkerV1, CatalogSourceKindV1,
};
use super::example_registry::example_row_by_id_v1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AutocompleteSuggestionKindV1 {
    Root,
    InteractiveRoot,
    DeclarationRoot,
    MetaRoot,
    ShowCategory,
    DescribeKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AutocompleteSuggestionCategoryV1 {
    QueryKeyword,
    InteractiveKeyword,
    DeclarationKeyword,
    MetaKeyword,
    CatalogCategoryKeyword,
    DescribeTargetPrefix,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AutocompleteSuggestionRowV1 {
    pub label: &'static str,
    pub insert_text: &'static str,
    pub kind: AutocompleteSuggestionKindV1,
    pub category: AutocompleteSuggestionCategoryV1,
    pub detail: &'static str,
    pub reason: &'static str,
    pub scope: CatalogScopeMarkerV1,
    pub capability_id: &'static str,
    pub example_ids: &'static [&'static str],
    pub provenance: CatalogProvenanceV1,
}

const AUTOCOMPLETE_SOURCE_FILE_V1: &str =
    "crates/quanta-gqlang-v2-ir/src/catalog/autocomplete_surface.rs";

const fn provenance_v1(source_marker: &'static str) -> CatalogProvenanceV1 {
    CatalogProvenanceV1 {
        source_kind: CatalogSourceKindV1::CompletionBaseline,
        source_file: AUTOCOMPLETE_SOURCE_FILE_V1,
        source_marker,
    }
}

const fn autocomplete_row_v1(
    label: &'static str,
    kind: AutocompleteSuggestionKindV1,
    category: AutocompleteSuggestionCategoryV1,
    detail: &'static str,
    reason: &'static str,
    scope: CatalogScopeMarkerV1,
    capability_id: &'static str,
    example_ids: &'static [&'static str],
) -> AutocompleteSuggestionRowV1 {
    AutocompleteSuggestionRowV1 {
        label,
        insert_text: label,
        kind,
        category,
        detail,
        reason,
        scope,
        capability_id,
        example_ids,
        provenance: provenance_v1(label),
    }
}

pub const AUTOCOMPLETE_SUGGESTION_ROWS_V1: &[AutocompleteSuggestionRowV1] = &[
    autocomplete_row_v1(
        "SELECT",
        AutocompleteSuggestionKindV1::Root,
        AutocompleteSuggestionCategoryV1::QueryKeyword,
        "table-only query root",
        "Root keyword completion must stay frontend-deterministic.",
        CatalogScopeMarkerV1::DocumentRoot,
        "root.select",
        &["example.select.catalog_users"],
    ),
    autocomplete_row_v1(
        "FIND",
        AutocompleteSuggestionKindV1::Root,
        AutocompleteSuggestionCategoryV1::QueryKeyword,
        "graph/reachability query root",
        "Root keyword completion must stay frontend-deterministic.",
        CatalogScopeMarkerV1::DocumentRoot,
        "root.find",
        &["example.find.taint"],
    ),
    autocomplete_row_v1(
        "CHECK",
        AutocompleteSuggestionKindV1::Root,
        AutocompleteSuggestionCategoryV1::QueryKeyword,
        "dual-lane policy/query root",
        "Root keyword completion must stay frontend-deterministic.",
        CatalogScopeMarkerV1::DocumentRoot,
        "root.check",
        &["example.check.hybrid"],
    ),
    autocomplete_row_v1(
        "EXPLAIN",
        AutocompleteSuggestionKindV1::Root,
        AutocompleteSuggestionCategoryV1::QueryKeyword,
        "truth/evidence query root",
        "Root keyword completion must stay frontend-deterministic.",
        CatalogScopeMarkerV1::DocumentRoot,
        "root.explain",
        &["example.explain.rule"],
    ),
    autocomplete_row_v1(
        "SHOW",
        AutocompleteSuggestionKindV1::InteractiveRoot,
        AutocompleteSuggestionCategoryV1::InteractiveKeyword,
        "catalog surface",
        "Interactive root completion must come from typed catalog assets only.",
        CatalogScopeMarkerV1::DocumentRoot,
        "interactive.show",
        &["example.show.capabilities"],
    ),
    autocomplete_row_v1(
        "DESCRIBE",
        AutocompleteSuggestionKindV1::InteractiveRoot,
        AutocompleteSuggestionCategoryV1::InteractiveKeyword,
        "target detail surface",
        "Interactive root completion must come from typed catalog assets only.",
        CatalogScopeMarkerV1::DocumentRoot,
        "interactive.describe",
        &["example.describe.rule"],
    ),
    autocomplete_row_v1(
        "SAMPLE",
        AutocompleteSuggestionKindV1::InteractiveRoot,
        AutocompleteSuggestionCategoryV1::InteractiveKeyword,
        "retrieval starter surface",
        "Interactive root completion must come from typed catalog assets only.",
        CatalogScopeMarkerV1::DocumentRoot,
        "interactive.sample",
        &["example.sample.repo_frontend"],
    ),
    autocomplete_row_v1(
        "COMPARE",
        AutocompleteSuggestionKindV1::InteractiveRoot,
        AutocompleteSuggestionCategoryV1::InteractiveKeyword,
        "comparison surface",
        "Interactive root completion must come from typed catalog assets only.",
        CatalogScopeMarkerV1::DocumentRoot,
        "interactive.compare",
        &["example.compare.semantic"],
    ),
    autocomplete_row_v1(
        "EXPORT",
        AutocompleteSuggestionKindV1::InteractiveRoot,
        AutocompleteSuggestionCategoryV1::InteractiveKeyword,
        "export surface",
        "Interactive root completion must come from typed catalog assets only.",
        CatalogScopeMarkerV1::DocumentRoot,
        "interactive.export",
        &["example.export.sarif"],
    ),
    autocomplete_row_v1(
        "VIEW",
        AutocompleteSuggestionKindV1::DeclarationRoot,
        AutocompleteSuggestionCategoryV1::DeclarationKeyword,
        "named query declaration",
        "Declaration completion stays public and frontend-owned.",
        CatalogScopeMarkerV1::DocumentRoot,
        "declaration.view",
        &[],
    ),
    autocomplete_row_v1(
        "RULE",
        AutocompleteSuggestionKindV1::DeclarationRoot,
        AutocompleteSuggestionCategoryV1::DeclarationKeyword,
        "check declaration",
        "Declaration completion stays public and frontend-owned.",
        CatalogScopeMarkerV1::DocumentRoot,
        "declaration.rule",
        &[],
    ),
    autocomplete_row_v1(
        "MODEL",
        AutocompleteSuggestionKindV1::DeclarationRoot,
        AutocompleteSuggestionCategoryV1::DeclarationKeyword,
        "provider declaration",
        "Declaration completion stays public and frontend-owned.",
        CatalogScopeMarkerV1::DocumentRoot,
        "declaration.model",
        &[],
    ),
    autocomplete_row_v1(
        "PACK",
        AutocompleteSuggestionKindV1::DeclarationRoot,
        AutocompleteSuggestionCategoryV1::DeclarationKeyword,
        "pack declaration",
        "Declaration completion stays public and frontend-owned.",
        CatalogScopeMarkerV1::DocumentRoot,
        "declaration.pack",
        &[],
    ),
    autocomplete_row_v1(
        "PROFILE",
        AutocompleteSuggestionKindV1::DeclarationRoot,
        AutocompleteSuggestionCategoryV1::DeclarationKeyword,
        "profile declaration",
        "Declaration completion stays public and frontend-owned.",
        CatalogScopeMarkerV1::DocumentRoot,
        "declaration.profile",
        &["example.profile.strict_review"],
    ),
    autocomplete_row_v1(
        "GATE",
        AutocompleteSuggestionKindV1::DeclarationRoot,
        AutocompleteSuggestionCategoryV1::DeclarationKeyword,
        "gate declaration",
        "Declaration completion stays public and frontend-owned.",
        CatalogScopeMarkerV1::DocumentRoot,
        "declaration.gate",
        &[],
    ),
    autocomplete_row_v1(
        "USE PACK",
        AutocompleteSuggestionKindV1::MetaRoot,
        AutocompleteSuggestionCategoryV1::MetaKeyword,
        "pack activation command",
        "Meta command completion must not depend on runtime discovery.",
        CatalogScopeMarkerV1::DocumentRoot,
        "meta.use_pack",
        &["example.use_pack.secure_pack"],
    ),
    autocomplete_row_v1(
        "REQUIRE CERTIFIED",
        AutocompleteSuggestionKindV1::MetaRoot,
        AutocompleteSuggestionCategoryV1::MetaKeyword,
        "trust policy command",
        "Meta command completion must not depend on runtime discovery.",
        CatalogScopeMarkerV1::DocumentRoot,
        "meta.require_certified",
        &["example.require_certified"],
    ),
    autocomplete_row_v1(
        "capabilities",
        AutocompleteSuggestionKindV1::ShowCategory,
        AutocompleteSuggestionCategoryV1::CatalogCategoryKeyword,
        "SHOW capabilities",
        "SHOW tail completion must be served from deterministic plural catalog nouns.",
        CatalogScopeMarkerV1::ShowCategoryTail,
        "show.capabilities",
        &["example.show.capabilities"],
    ),
    autocomplete_row_v1(
        "models",
        AutocompleteSuggestionKindV1::ShowCategory,
        AutocompleteSuggestionCategoryV1::CatalogCategoryKeyword,
        "SHOW models",
        "SHOW tail completion must be served from deterministic plural catalog nouns.",
        CatalogScopeMarkerV1::ShowCategoryTail,
        "show.models",
        &[],
    ),
    autocomplete_row_v1(
        "views",
        AutocompleteSuggestionKindV1::ShowCategory,
        AutocompleteSuggestionCategoryV1::CatalogCategoryKeyword,
        "SHOW views",
        "SHOW tail completion must be served from deterministic plural catalog nouns.",
        CatalogScopeMarkerV1::ShowCategoryTail,
        "show.views",
        &[],
    ),
    autocomplete_row_v1(
        "rules",
        AutocompleteSuggestionKindV1::ShowCategory,
        AutocompleteSuggestionCategoryV1::CatalogCategoryKeyword,
        "SHOW rules",
        "SHOW tail completion must be served from deterministic plural catalog nouns.",
        CatalogScopeMarkerV1::ShowCategoryTail,
        "show.rules",
        &["example.describe.rule"],
    ),
    autocomplete_row_v1(
        "packs",
        AutocompleteSuggestionKindV1::ShowCategory,
        AutocompleteSuggestionCategoryV1::CatalogCategoryKeyword,
        "SHOW packs",
        "SHOW tail completion must be served from deterministic plural catalog nouns.",
        CatalogScopeMarkerV1::ShowCategoryTail,
        "show.packs",
        &["example.use_pack.secure_pack"],
    ),
    autocomplete_row_v1(
        "profiles",
        AutocompleteSuggestionKindV1::ShowCategory,
        AutocompleteSuggestionCategoryV1::CatalogCategoryKeyword,
        "SHOW profiles",
        "SHOW tail completion must be served from deterministic plural catalog nouns.",
        CatalogScopeMarkerV1::ShowCategoryTail,
        "show.profiles",
        &["example.profile.strict_review"],
    ),
    autocomplete_row_v1(
        "gates",
        AutocompleteSuggestionKindV1::ShowCategory,
        AutocompleteSuggestionCategoryV1::CatalogCategoryKeyword,
        "SHOW gates",
        "SHOW tail completion must be served from deterministic plural catalog nouns.",
        CatalogScopeMarkerV1::ShowCategoryTail,
        "show.gates",
        &[],
    ),
    autocomplete_row_v1(
        "capability::",
        AutocompleteSuggestionKindV1::DescribeKind,
        AutocompleteSuggestionCategoryV1::DescribeTargetPrefix,
        "DESCRIBE capability::<path>",
        "DESCRIBE tail completion must seed a typed target prefix only.",
        CatalogScopeMarkerV1::DescribeTargetTail,
        "describe.capability",
        &[],
    ),
    autocomplete_row_v1(
        "model::",
        AutocompleteSuggestionKindV1::DescribeKind,
        AutocompleteSuggestionCategoryV1::DescribeTargetPrefix,
        "DESCRIBE model::<path>",
        "DESCRIBE tail completion must seed a typed target prefix only.",
        CatalogScopeMarkerV1::DescribeTargetTail,
        "describe.model",
        &[],
    ),
    autocomplete_row_v1(
        "view::",
        AutocompleteSuggestionKindV1::DescribeKind,
        AutocompleteSuggestionCategoryV1::DescribeTargetPrefix,
        "DESCRIBE view::<path>",
        "DESCRIBE tail completion must seed a typed target prefix only.",
        CatalogScopeMarkerV1::DescribeTargetTail,
        "describe.view",
        &[],
    ),
    autocomplete_row_v1(
        "rule::",
        AutocompleteSuggestionKindV1::DescribeKind,
        AutocompleteSuggestionCategoryV1::DescribeTargetPrefix,
        "DESCRIBE rule::<path>",
        "DESCRIBE tail completion must seed a typed target prefix only.",
        CatalogScopeMarkerV1::DescribeTargetTail,
        "describe.rule",
        &["example.describe.rule"],
    ),
    autocomplete_row_v1(
        "pack::",
        AutocompleteSuggestionKindV1::DescribeKind,
        AutocompleteSuggestionCategoryV1::DescribeTargetPrefix,
        "DESCRIBE pack::<path>",
        "DESCRIBE tail completion must seed a typed target prefix only.",
        CatalogScopeMarkerV1::DescribeTargetTail,
        "describe.pack",
        &["example.use_pack.secure_pack"],
    ),
    autocomplete_row_v1(
        "profile::",
        AutocompleteSuggestionKindV1::DescribeKind,
        AutocompleteSuggestionCategoryV1::DescribeTargetPrefix,
        "DESCRIBE profile::<path>",
        "DESCRIBE tail completion must seed a typed target prefix only.",
        CatalogScopeMarkerV1::DescribeTargetTail,
        "describe.profile",
        &["example.profile.strict_review"],
    ),
    autocomplete_row_v1(
        "gate::",
        AutocompleteSuggestionKindV1::DescribeKind,
        AutocompleteSuggestionCategoryV1::DescribeTargetPrefix,
        "DESCRIBE gate::<path>",
        "DESCRIBE tail completion must seed a typed target prefix only.",
        CatalogScopeMarkerV1::DescribeTargetTail,
        "describe.gate",
        &[],
    ),
];

pub const fn autocomplete_suggestion_rows_v1() -> &'static [AutocompleteSuggestionRowV1] {
    AUTOCOMPLETE_SUGGESTION_ROWS_V1
}

pub fn autocomplete_row_by_label_v1(label: &str) -> Option<&'static AutocompleteSuggestionRowV1> {
    autocomplete_suggestion_rows_v1()
        .iter()
        .find(|row| row.label == label)
}

pub fn autocomplete_matches_v1(prefix: &str) -> Vec<&'static AutocompleteSuggestionRowV1> {
    let normalized_prefix = normalize_completion_prefix_v1(prefix);
    if normalized_prefix.is_empty() {
        return autocomplete_suggestion_rows_v1().iter().collect();
    }

    autocomplete_suggestion_rows_v1()
        .iter()
        .filter(|row| normalize_completion_prefix_v1(row.label).starts_with(&normalized_prefix))
        .collect()
}

pub fn autocomplete_surface_integrity_v1() -> Result<(), &'static str> {
    for row in autocomplete_suggestion_rows_v1() {
        if capability_row_by_id_v1(row.capability_id).is_none() {
            return Err("autocomplete row references unknown capability id");
        }
        for example_id in row.example_ids {
            if example_row_by_id_v1(example_id).is_none() {
                return Err("autocomplete row references unknown example id");
            }
        }
    }
    Ok(())
}

fn normalize_completion_prefix_v1(input: &str) -> String {
    input
        .trim()
        .to_ascii_lowercase()
        .replace(['_', '-', ' '], "")
}
