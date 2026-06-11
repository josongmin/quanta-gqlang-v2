//! E12A — the read-only zero-query starter-scaffold core.
//!
//! Moved out of the LSP handler into the ir catalog plane so every shell
//! (LSP zero-query, CLI `zero-query` verb) consumes one source of starter
//! suggestions. The scaffold is built entirely from the typed capability
//! catalog + example registry — no host/protocol dependency — and every
//! suggested query is classified through the canonical parser so a scaffold
//! can never drift from the grammar or smuggle a mutating root.

use crate::catalog::capability_catalog::{
    capability_catalog_authority_v1, capability_catalog_integrity_v1, capability_row_by_id_v1,
};
use crate::catalog::example_registry::{
    example_registry_authority_v1, example_registry_rows_v1,
    read_only_example_registry_integrity_v1, ExampleCategoryV1,
};
use crate::frontend::{parse_document_v1, AstRootV1};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ZeroQueryModeV1 {
    ReadOnly,
}

impl ZeroQueryModeV1 {
    #[must_use]
    pub const fn label_v1(self) -> &'static str {
        match self {
            Self::ReadOnly => "read-only",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ZeroQueryProvenanceV1 {
    TypedCatalogExampleMetadata,
}

impl ZeroQueryProvenanceV1 {
    #[must_use]
    pub const fn label_v1(self) -> &'static str {
        match self {
            Self::TypedCatalogExampleMetadata => "typed-catalog-example-metadata",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum StarterGroupKindV1 {
    Catalog,
    Analysis,
    Retrieval,
}

impl StarterGroupKindV1 {
    #[must_use]
    pub const fn id_v1(self) -> &'static str {
        match self {
            Self::Catalog => "catalog",
            Self::Analysis => "analysis",
            Self::Retrieval => "retrieval",
        }
    }

    #[must_use]
    pub const fn title_v1(self) -> &'static str {
        match self {
            Self::Catalog => "Catalog",
            Self::Analysis => "Analysis",
            Self::Retrieval => "Retrieval",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum StarterRootKindV1 {
    Show,
    Describe,
    Select,
    Find,
    Check,
    Explain,
    Sample,
    Compare,
    Export,
}

impl StarterRootKindV1 {
    #[must_use]
    pub const fn id_v1(self) -> &'static str {
        match self {
            Self::Show => "show",
            Self::Describe => "describe",
            Self::Select => "select",
            Self::Find => "find",
            Self::Check => "check",
            Self::Explain => "explain",
            Self::Sample => "sample",
            Self::Compare => "compare",
            Self::Export => "export",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum StarterBehaviorV1 {
    ReadOnlyScaffold,
}

impl StarterBehaviorV1 {
    #[must_use]
    pub const fn label_v1(self) -> &'static str {
        match self {
            Self::ReadOnlyScaffold => "read-only-scaffold",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum StarterReasonCodeV1 {
    CatalogOverview,
    TypedTargetDetail,
    QueryShapePreview,
    RetrievalShapePreview,
}

impl StarterReasonCodeV1 {
    #[must_use]
    pub const fn label_v1(self) -> &'static str {
        match self {
            Self::CatalogOverview => "catalog_overview",
            Self::TypedTargetDetail => "typed_target_detail",
            Self::QueryShapePreview => "query_shape_preview",
            Self::RetrievalShapePreview => "retrieval_shape_preview",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StarterReasonV1 {
    pub code: StarterReasonCodeV1,
    pub detail: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StarterSuggestionV1 {
    pub label: &'static str,
    pub root_kind: StarterRootKindV1,
    pub query_scaffold: &'static str,
    pub behavior: StarterBehaviorV1,
    pub authority: &'static str,
    pub provenance_marker: &'static str,
    pub reason: StarterReasonV1,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StarterGroupV1 {
    pub kind: StarterGroupKindV1,
    pub suggestions: Vec<StarterSuggestionV1>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StarterScaffoldReportV1 {
    pub mode: ZeroQueryModeV1,
    pub provenance: ZeroQueryProvenanceV1,
    pub authority: &'static str,
    pub groups: Vec<StarterGroupV1>,
}

impl StarterScaffoldReportV1 {
    #[must_use]
    pub fn render_v1(&self) -> String {
        let mut lines = vec![
            format!("starter.mode={}", self.mode.label_v1()),
            format!("starter.provenance={}", self.provenance.label_v1()),
            format!("starter.authority={}", self.authority),
        ];

        for (group_index, group) in self.groups.iter().enumerate() {
            lines.push(format!(
                "starter.group[{group_index}].id={}",
                group.kind.id_v1()
            ));
            lines.push(format!(
                "starter.group[{group_index}].title={}",
                group.kind.title_v1()
            ));
            for (suggestion_index, suggestion) in group.suggestions.iter().enumerate() {
                lines.push(format!(
                    "starter.group[{group_index}].suggestion[{suggestion_index}].label={}",
                    suggestion.label
                ));
                lines.push(format!(
                    "starter.group[{group_index}].suggestion[{suggestion_index}].root_kind={}",
                    suggestion.root_kind.id_v1()
                ));
                lines.push(format!(
                    "starter.group[{group_index}].suggestion[{suggestion_index}].query={}",
                    suggestion.query_scaffold
                ));
                lines.push(format!(
                    "starter.group[{group_index}].suggestion[{suggestion_index}].behavior={}",
                    suggestion.behavior.label_v1()
                ));
                lines.push(format!(
                    "starter.group[{group_index}].suggestion[{suggestion_index}].authority={}",
                    suggestion.authority
                ));
                lines.push(format!(
                    "starter.group[{group_index}].suggestion[{suggestion_index}].provenance_marker={}",
                    suggestion.provenance_marker
                ));
                lines.push(format!(
                    "starter.group[{group_index}].suggestion[{suggestion_index}].reason.code={}",
                    suggestion.reason.code.label_v1()
                ));
                lines.push(format!(
                    "starter.group[{group_index}].suggestion[{suggestion_index}].reason.detail={}",
                    suggestion.reason.detail
                ));
            }
        }

        lines.join("\n")
    }
}

#[derive(Copy, Clone)]
struct CatalogPairSpecV1 {
    show_capability_id: &'static str,
    show_query: &'static str,
    describe_capability_id: &'static str,
    describe_query: &'static str,
}

const CATALOG_PAIR_SPECS_V1: &[CatalogPairSpecV1] = &[
    CatalogPairSpecV1 {
        show_capability_id: "show.capabilities",
        show_query: "SHOW capabilities",
        describe_capability_id: "describe.capability",
        describe_query: "DESCRIBE capability::code.search",
    },
    CatalogPairSpecV1 {
        show_capability_id: "show.models",
        show_query: "SHOW models",
        describe_capability_id: "describe.model",
        describe_query: "DESCRIBE model::providers.embedding",
    },
    CatalogPairSpecV1 {
        show_capability_id: "show.views",
        show_query: "SHOW views",
        describe_capability_id: "describe.view",
        describe_query: "DESCRIBE view::catalog.users",
    },
    CatalogPairSpecV1 {
        show_capability_id: "show.rules",
        show_query: "SHOW rules",
        describe_capability_id: "describe.rule",
        describe_query: "DESCRIBE rule::owasp.sql-injection",
    },
    CatalogPairSpecV1 {
        show_capability_id: "show.packs",
        show_query: "SHOW packs",
        describe_capability_id: "describe.pack",
        describe_query: "DESCRIBE pack::security.audit",
    },
    CatalogPairSpecV1 {
        show_capability_id: "show.profiles",
        show_query: "SHOW profiles",
        describe_capability_id: "describe.profile",
        describe_query: "DESCRIBE profile::strict.review",
    },
    CatalogPairSpecV1 {
        show_capability_id: "show.gates",
        show_query: "SHOW gates",
        describe_capability_id: "describe.gate",
        describe_query: "DESCRIBE gate::release.blockers",
    },
];

pub fn build_starter_scaffold_v1() -> Result<StarterScaffoldReportV1, String> {
    capability_catalog_integrity_v1().map_err(str::to_string)?;
    read_only_example_registry_integrity_v1().map_err(str::to_string)?;

    Ok(StarterScaffoldReportV1 {
        mode: ZeroQueryModeV1::ReadOnly,
        provenance: ZeroQueryProvenanceV1::TypedCatalogExampleMetadata,
        authority: example_registry_authority_v1(),
        groups: vec![
            build_catalog_group_v1()?,
            build_example_group_v1(
                StarterGroupKindV1::Analysis,
                ExampleCategoryV1::QueryStarter,
                |stable_id| stable_id.starts_with("example."),
            )?,
            build_example_group_v1(
                StarterGroupKindV1::Retrieval,
                ExampleCategoryV1::InteractiveStarter,
                |stable_id| {
                    matches!(
                        stable_id,
                        "example.sample.repo_frontend"
                            | "example.compare.semantic"
                            | "example.export.sarif"
                    )
                },
            )?,
        ],
    })
}

fn build_catalog_group_v1() -> Result<StarterGroupV1, String> {
    let mut suggestions = Vec::with_capacity(CATALOG_PAIR_SPECS_V1.len() * 2);
    for spec in CATALOG_PAIR_SPECS_V1 {
        let show_row = capability_row_by_id_v1(spec.show_capability_id)
            .ok_or_else(|| format!("missing capability row `{}`", spec.show_capability_id))?;
        suggestions.push(StarterSuggestionV1 {
            label: spec.show_query,
            root_kind: StarterRootKindV1::Show,
            query_scaffold: spec.show_query,
            behavior: StarterBehaviorV1::ReadOnlyScaffold,
            authority: capability_catalog_authority_v1(),
            provenance_marker: show_row.provenance.source_marker,
            reason: StarterReasonV1 {
                code: StarterReasonCodeV1::CatalogOverview,
                detail: show_row.reason,
            },
        });

        let describe_row = capability_row_by_id_v1(spec.describe_capability_id)
            .ok_or_else(|| format!("missing capability row `{}`", spec.describe_capability_id))?;
        suggestions.push(StarterSuggestionV1 {
            label: spec.describe_query,
            root_kind: StarterRootKindV1::Describe,
            query_scaffold: spec.describe_query,
            behavior: StarterBehaviorV1::ReadOnlyScaffold,
            authority: capability_catalog_authority_v1(),
            provenance_marker: describe_row.provenance.source_marker,
            reason: StarterReasonV1 {
                code: StarterReasonCodeV1::TypedTargetDetail,
                detail: describe_row.reason,
            },
        });
    }

    Ok(StarterGroupV1 {
        kind: StarterGroupKindV1::Catalog,
        suggestions,
    })
}

fn build_example_group_v1(
    kind: StarterGroupKindV1,
    category: ExampleCategoryV1,
    include: impl Fn(&str) -> bool,
) -> Result<StarterGroupV1, String> {
    let reason_code = match category {
        ExampleCategoryV1::QueryStarter => StarterReasonCodeV1::QueryShapePreview,
        ExampleCategoryV1::InteractiveStarter => StarterReasonCodeV1::RetrievalShapePreview,
        ExampleCategoryV1::DeclarationStarter | ExampleCategoryV1::MetaStarter => {
            return Err("unsupported zero-query starter category".to_string())
        }
    };

    let suggestions = example_registry_rows_v1()
        .iter()
        .filter(|row| row.category == category && include(row.stable_id))
        .map(|row| {
            Ok(StarterSuggestionV1 {
                label: row.title,
                root_kind: root_kind_v1(row.query_text)?,
                query_scaffold: row.query_text,
                behavior: StarterBehaviorV1::ReadOnlyScaffold,
                authority: example_registry_authority_v1(),
                provenance_marker: row.provenance.source_marker,
                reason: StarterReasonV1 {
                    code: reason_code,
                    detail: row.reason,
                },
            })
        })
        .collect::<Result<Vec<_>, String>>()?;

    Ok(StarterGroupV1 { kind, suggestions })
}

fn root_kind_v1(query_text: &str) -> Result<StarterRootKindV1, String> {
    // Classify by the canonical parser rather than a hand-rolled first-token
    // keyword whitelist: this defers to the single source of grammar truth (the
    // same `parse_document_v1` the example-registry integrity gate validates
    // against), so the starter root kind can never drift from it on case or
    // vocabulary. Declaration/meta roots are rejected to keep scaffolds query-only.
    let document = parse_document_v1(query_text).map_err(|page| {
        let reject = page
            .diagnostics
            .first()
            .map_or("unknown_reject", |diagnostic| diagnostic.code);
        format!("starter scaffold did not parse as a query root ({reject}): {query_text}")
    })?;
    let root = document
        .roots
        .first()
        .ok_or_else(|| format!("starter scaffold parsed to an empty document: {query_text}"))?;
    match root {
        AstRootV1::Show(_) => Ok(StarterRootKindV1::Show),
        AstRootV1::Describe(_) => Ok(StarterRootKindV1::Describe),
        AstRootV1::Select(_) => Ok(StarterRootKindV1::Select),
        AstRootV1::Find(_) => Ok(StarterRootKindV1::Find),
        AstRootV1::Check(_) => Ok(StarterRootKindV1::Check),
        AstRootV1::Explain(_) => Ok(StarterRootKindV1::Explain),
        AstRootV1::Sample(_) => Ok(StarterRootKindV1::Sample),
        AstRootV1::Compare(_) => Ok(StarterRootKindV1::Compare),
        AstRootV1::Export(_) => Ok(StarterRootKindV1::Export),
        AstRootV1::View(_)
        | AstRootV1::Rule(_)
        | AstRootV1::Model(_)
        | AstRootV1::Pack(_)
        | AstRootV1::Profile(_)
        | AstRootV1::Gate(_)
        | AstRootV1::Meta(_)
        | AstRootV1::Composition(_) => {
            Err("zero-query scaffolds must stay query-only and non-mutating".to_string())
        }
    }
}
