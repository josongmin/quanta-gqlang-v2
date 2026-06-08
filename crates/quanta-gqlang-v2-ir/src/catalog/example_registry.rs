use super::capability_catalog::{
    capability_row_by_id_v1, CatalogProvenanceV1, CatalogScopeMarkerV1, CatalogSourceKindV1,
};
use quanta_gqlang_v2_sema::{parse_document_v1, AstRootV1};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExampleCategoryV1 {
    QueryStarter,
    InteractiveStarter,
    DeclarationStarter,
    MetaStarter,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExampleRegistryRowV1 {
    pub stable_id: &'static str,
    pub title: &'static str,
    pub query_text: &'static str,
    pub category: ExampleCategoryV1,
    pub reason: &'static str,
    pub scope: CatalogScopeMarkerV1,
    pub capability_ids: &'static [&'static str],
    pub provenance: CatalogProvenanceV1,
}

const EXAMPLE_SOURCE_FILE_V1: &str = "crates/quanta-gqlang-v2-ir/src/catalog/example_registry.rs";
const EXTERNAL_READ_ONLY_SHELL_AUTHORITY_V1: &str = "external-read-only-shell";

const fn provenance_v1(source_marker: &'static str) -> CatalogProvenanceV1 {
    CatalogProvenanceV1 {
        source_kind: CatalogSourceKindV1::CuratedReadOnlyExample,
        source_file: EXAMPLE_SOURCE_FILE_V1,
        source_marker,
    }
}

const fn example_row_v1(
    stable_id: &'static str,
    title: &'static str,
    query_text: &'static str,
    category: ExampleCategoryV1,
    reason: &'static str,
    capability_ids: &'static [&'static str],
) -> ExampleRegistryRowV1 {
    ExampleRegistryRowV1 {
        stable_id,
        title,
        query_text,
        category,
        reason,
        scope: CatalogScopeMarkerV1::ReadOnlyShell,
        capability_ids,
        provenance: provenance_v1(stable_id),
    }
}

pub const EXAMPLE_REGISTRY_ROWS_V1: &[ExampleRegistryRowV1] = &[
    example_row_v1(
        "example.select.catalog_users",
        "SELECT active catalog users",
        "SELECT id, profile.name FROM catalog.users WHERE active = true LIMIT 7",
        ExampleCategoryV1::QueryStarter,
        "Read-only shell needs a deterministic SELECT starter with no runtime truth claims.",
        &["root.select"],
    ),
    example_row_v1(
        "example.find.taint",
        "FIND taint path",
        "FIND taint api.entry -> db.write",
        ExampleCategoryV1::QueryStarter,
        "Graph starter stays frontend-only and does not imply reachable result materialization.",
        &["root.find"],
    ),
    example_row_v1(
        "example.check.hybrid",
        "CHECK hybrid policy payload",
        "CHECK rule.hybrid check:v1|class=security|scope=repo%3A%2F%2Fruntime|pattern=call%20%3D%20%22std%3A%3Aprocess%3A%3ACommand%22|confirm=require",
        ExampleCategoryV1::QueryStarter,
        "Policy starter demonstrates typed payload encoding without host-side evaluation.",
        &["root.check"],
    ),
    example_row_v1(
        "example.explain.rule",
        "EXPLAIN rule evidence",
        "EXPLAIN rule::owasp.sql-injection ;; evidence=trace ;; truth=confirm",
        ExampleCategoryV1::QueryStarter,
        "Evidence-oriented starter remains honest about missing materialized proof.",
        &["root.explain", "describe.rule"],
    ),
    example_row_v1(
        "example.show.capabilities",
        "SHOW capabilities catalog",
        "SHOW capabilities",
        ExampleCategoryV1::InteractiveStarter,
        "Interactive catalog starter is pure typed metadata, not host discovery.",
        &["interactive.show", "show.capabilities"],
    ),
    example_row_v1(
        "example.describe.rule",
        "DESCRIBE rule target",
        "DESCRIBE rule::owasp.sql-injection",
        ExampleCategoryV1::InteractiveStarter,
        "Interactive detail starter demonstrates typed path prefixing without fake catalog truth.",
        &["interactive.describe", "describe.rule"],
    ),
    example_row_v1(
        "example.sample.repo_frontend",
        "SAMPLE repo retrieval",
        "SAMPLE query=callgraph entrypoints ;; scope=repo::frontend ;; mode=semantic ;; limit=5",
        ExampleCategoryV1::InteractiveStarter,
        "Retrieval starter stays read-only and deterministic.",
        &["interactive.sample"],
    ),
    example_row_v1(
        "example.compare.semantic",
        "COMPARE semantic candidates",
        "COMPARE baseline=run::baseline-1 ;; candidate=run::candidate-2 ;; metric=semantic",
        ExampleCategoryV1::InteractiveStarter,
        "Comparison starter is frontend syntax only and avoids backend result claims.",
        &["interactive.compare"],
    ),
    example_row_v1(
        "example.export.sarif",
        "EXPORT latest result",
        "EXPORT source=result::latest ;; kind=sarif ;; profile=strict",
        ExampleCategoryV1::InteractiveStarter,
        "Export starter captures syntax intent without leaking export host DTOs.",
        &["interactive.export"],
    ),
    example_row_v1(
        "example.profile.strict_review",
        "PROFILE strict review",
        "PROFILE strict-review ;; capabilities=graph.read,graph.trace ;; admission=certified-only",
        ExampleCategoryV1::DeclarationStarter,
        "Profile declaration gives zero-query surfaces a typed policy starter.",
        &["declaration.profile", "show.profiles", "describe.profile"],
    ),
    example_row_v1(
        "example.use_pack.secure_pack",
        "USE PACK secure-pack",
        "USE PACK secure-pack ;; activation=session ;; lockfile=packs/secure-pack.lock",
        ExampleCategoryV1::MetaStarter,
        "Pack activation starter remains deterministic and host-agnostic.",
        &["meta.use_pack"],
    ),
    example_row_v1(
        "example.require_certified",
        "REQUIRE CERTIFIED policy",
        "REQUIRE CERTIFIED",
        ExampleCategoryV1::MetaStarter,
        "Trust policy starter avoids runtime capability probing.",
        &["meta.require_certified"],
    ),
];

pub const fn example_registry_rows_v1() -> &'static [ExampleRegistryRowV1] {
    EXAMPLE_REGISTRY_ROWS_V1
}

pub const fn example_registry_authority_v1() -> &'static str {
    EXTERNAL_READ_ONLY_SHELL_AUTHORITY_V1
}

pub fn read_only_example_registry_integrity_v1() -> Result<(), &'static str> {
    let rows = example_registry_rows_v1();
    for (index, row) in rows.iter().enumerate() {
        if rows
            .iter()
            .skip(index + 1)
            .any(|other| other.stable_id == row.stable_id)
        {
            return Err("example row stable_id must be unique");
        }
    }
    for row in rows {
        if row.stable_id.trim().is_empty() {
            return Err("example row stable_id must not be empty");
        }
        if row.title.trim().is_empty() {
            return Err("example row title must not be empty");
        }
        if row.query_text.trim().is_empty() {
            return Err("example row query_text must not be empty");
        }
        if row.reason.trim().is_empty() {
            return Err("example row reason must not be empty");
        }
        if row.scope != CatalogScopeMarkerV1::ReadOnlyShell {
            return Err("example row scope must stay read-only-shell");
        }
        if row.provenance.source_kind != CatalogSourceKindV1::CuratedReadOnlyExample {
            return Err("example row provenance must stay curated read-only example");
        }
        if row.provenance.source_file != EXAMPLE_SOURCE_FILE_V1 {
            return Err("example row provenance source_file drifted");
        }
        if row.provenance.source_marker != row.stable_id {
            return Err("example row provenance marker must match stable_id");
        }
        if row.capability_ids.is_empty() {
            return Err("example row capability_ids must not be empty");
        }
        if !example_query_is_admissible_v1(row.query_text, row.category) {
            return Err("example row query_text must parse and match its declared category");
        }
        for capability_id in row.capability_ids {
            if capability_row_by_id_v1(capability_id).is_none() {
                return Err("example row references unknown capability id");
            }
        }
    }
    Ok(())
}

pub fn example_row_by_id_v1(stable_id: &str) -> Option<&'static ExampleRegistryRowV1> {
    example_registry_rows_v1()
        .iter()
        .find(|row| row.stable_id == stable_id)
}

/// Validate a curated example against the REAL grammar instead of a hand-rolled
/// keyword whitelist (which had drifted from the parser in both directions), and
/// confirm the parsed root matches the row's declared category. This is the only
/// gate on `query_text`, which is handed verbatim to users, so it must agree with
/// the parser exactly.
fn example_query_is_admissible_v1(query_text: &str, category: ExampleCategoryV1) -> bool {
    let Ok(document) = parse_document_v1(query_text) else {
        return false;
    };
    if document.roots.len() != 1 {
        return false;
    }
    match document.roots.first() {
        Some(root) => root_matches_category_v1(root, category),
        None => false,
    }
}

fn root_matches_category_v1(root: &AstRootV1, category: ExampleCategoryV1) -> bool {
    match category {
        ExampleCategoryV1::QueryStarter => matches!(
            root,
            AstRootV1::Select(_) | AstRootV1::Find(_) | AstRootV1::Check(_) | AstRootV1::Explain(_)
        ),
        ExampleCategoryV1::InteractiveStarter => matches!(
            root,
            AstRootV1::Show(_)
                | AstRootV1::Describe(_)
                | AstRootV1::Sample(_)
                | AstRootV1::Compare(_)
                | AstRootV1::Export(_)
        ),
        ExampleCategoryV1::DeclarationStarter => matches!(
            root,
            AstRootV1::View(_)
                | AstRootV1::Rule(_)
                | AstRootV1::Model(_)
                | AstRootV1::Pack(_)
                | AstRootV1::Profile(_)
                | AstRootV1::Gate(_)
        ),
        ExampleCategoryV1::MetaStarter => matches!(root, AstRootV1::Meta(_)),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        example_query_is_admissible_v1, read_only_example_registry_integrity_v1, ExampleCategoryV1,
    };

    #[test]
    fn integrity_passes_for_all_curated_rows() {
        assert_eq!(read_only_example_registry_integrity_v1(), Ok(()));
    }

    #[test]
    fn admissibility_requires_category_to_match_the_parsed_root() {
        assert!(example_query_is_admissible_v1(
            "SELECT id FROM catalog.users",
            ExampleCategoryV1::QueryStarter
        ));
        assert!(!example_query_is_admissible_v1(
            "SELECT id FROM catalog.users",
            ExampleCategoryV1::InteractiveStarter
        ));
    }

    #[test]
    fn admissibility_rejects_query_text_the_old_whitelist_wrongly_accepted() {
        // Both pass the old first-token whitelist but are rejected by the grammar.
        assert!(!example_query_is_admissible_v1(
            "SELECT",
            ExampleCategoryV1::QueryStarter
        ));
        assert!(!example_query_is_admissible_v1(
            "USE PACK secure-pack",
            ExampleCategoryV1::MetaStarter
        ));
    }

    #[test]
    fn admissibility_accepts_declaration_roots_the_old_whitelist_dropped() {
        assert!(example_query_is_admissible_v1(
            "VIEW catalog.users ;; query-ref=queries/catalog/users",
            ExampleCategoryV1::DeclarationStarter
        ));
    }
}
