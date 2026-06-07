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
    pub groups: Vec<StarterGroupV1>,
}

impl StarterScaffoldReportV1 {
    #[must_use]
    pub fn render_v1(&self) -> String {
        let mut lines = vec![
            format!("starter.mode={}", self.mode.label_v1()),
            format!("starter.provenance={}", self.provenance.label_v1()),
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
struct CatalogStarterAssetV1 {
    show_query_scaffold: &'static str,
    describe_query_scaffold: &'static str,
}

#[derive(Copy, Clone)]
struct ExampleStarterAssetV1 {
    group: StarterGroupKindV1,
    label: &'static str,
    root_kind: StarterRootKindV1,
    query_scaffold: &'static str,
    reason_code: StarterReasonCodeV1,
    reason_detail: &'static str,
}

const CATALOG_SHOW_REASON_V1: &str =
    "Lists a typed catalog surface without runtime execution or search.";
const CATALOG_DESCRIBE_REASON_V1: &str =
    "Pins a concrete typed catalog target for deterministic detail preview.";
const QUERY_SHAPE_REASON_V1: &str =
    "Shows a typed query scaffold that stays in read-only syntax/sema/IR lanes.";
const RETRIEVAL_SHAPE_REASON_V1: &str =
    "Shows retrieval or interactive shape without materializing runtime results.";

const CATALOG_ASSETS_V1: &[CatalogStarterAssetV1] = &[
    CatalogStarterAssetV1 {
        show_query_scaffold: "SHOW capabilities",
        describe_query_scaffold: "DESCRIBE capability::code.search",
    },
    CatalogStarterAssetV1 {
        show_query_scaffold: "SHOW models",
        describe_query_scaffold: "DESCRIBE model::providers.embedding",
    },
    CatalogStarterAssetV1 {
        show_query_scaffold: "SHOW views",
        describe_query_scaffold: "DESCRIBE view::catalog.users",
    },
    CatalogStarterAssetV1 {
        show_query_scaffold: "SHOW rules",
        describe_query_scaffold: "DESCRIBE rule::owasp.sql-injection",
    },
    CatalogStarterAssetV1 {
        show_query_scaffold: "SHOW packs",
        describe_query_scaffold: "DESCRIBE pack::security.audit",
    },
    CatalogStarterAssetV1 {
        show_query_scaffold: "SHOW profiles",
        describe_query_scaffold: "DESCRIBE profile::strict.review",
    },
    CatalogStarterAssetV1 {
        show_query_scaffold: "SHOW gates",
        describe_query_scaffold: "DESCRIBE gate::release.blockers",
    },
];

const EXAMPLE_ASSETS_V1: &[ExampleStarterAssetV1] = &[
    ExampleStarterAssetV1 {
        group: StarterGroupKindV1::Analysis,
        label: "SELECT high severity files",
        root_kind: StarterRootKindV1::Select,
        query_scaffold: "SELECT id, path FROM catalog.files WHERE severity = 'high' LIMIT 10",
        reason_code: StarterReasonCodeV1::QueryShapePreview,
        reason_detail: QUERY_SHAPE_REASON_V1,
    },
    ExampleStarterAssetV1 {
        group: StarterGroupKindV1::Analysis,
        label: "FIND reachability from entrypoint to sink",
        root_kind: StarterRootKindV1::Find,
        query_scaffold: "FIND reachability api.entry -> db.write | via=auth | depth=3",
        reason_code: StarterReasonCodeV1::QueryShapePreview,
        reason_detail: QUERY_SHAPE_REASON_V1,
    },
    ExampleStarterAssetV1 {
        group: StarterGroupKindV1::Analysis,
        label: "CHECK owasp.sql with confirm lane",
        root_kind: StarterRootKindV1::Check,
        query_scaffold:
            "CHECK owasp.sql scope=repo://runtime ;; pattern=call = \"std::process::Command\" ;; confirm=require",
        reason_code: StarterReasonCodeV1::QueryShapePreview,
        reason_detail: QUERY_SHAPE_REASON_V1,
    },
    ExampleStarterAssetV1 {
        group: StarterGroupKindV1::Analysis,
        label: "EXPLAIN authz rule with counterexample",
        root_kind: StarterRootKindV1::Explain,
        query_scaffold:
            "EXPLAIN target=view::user_authz ;; evidence=counterexample ;; truth=refute",
        reason_code: StarterReasonCodeV1::QueryShapePreview,
        reason_detail: QUERY_SHAPE_REASON_V1,
    },
    ExampleStarterAssetV1 {
        group: StarterGroupKindV1::Retrieval,
        label: "SAMPLE callgraph entrypoints",
        root_kind: StarterRootKindV1::Sample,
        query_scaffold:
            "SAMPLE query=callgraph entrypoints ;; scope=repo::frontend ;; mode=hybrid ;; limit=5",
        reason_code: StarterReasonCodeV1::RetrievalShapePreview,
        reason_detail: RETRIEVAL_SHAPE_REASON_V1,
    },
    ExampleStarterAssetV1 {
        group: StarterGroupKindV1::Retrieval,
        label: "COMPARE two hotspot result sets",
        root_kind: StarterRootKindV1::Compare,
        query_scaffold:
            "COMPARE baseline=result::hotspots/main ;; candidate=result::hotspots/pr ;; metric=semantic",
        reason_code: StarterReasonCodeV1::RetrievalShapePreview,
        reason_detail: RETRIEVAL_SHAPE_REASON_V1,
    },
    ExampleStarterAssetV1 {
        group: StarterGroupKindV1::Retrieval,
        label: "EXPORT hotspot result to sarif",
        root_kind: StarterRootKindV1::Export,
        query_scaffold:
            "EXPORT source=result::hotspots/latest ;; kind=sarif ;; profile=security-audit",
        reason_code: StarterReasonCodeV1::RetrievalShapePreview,
        reason_detail: RETRIEVAL_SHAPE_REASON_V1,
    },
];

#[must_use]
pub fn build_starter_scaffold_v1() -> StarterScaffoldReportV1 {
    StarterScaffoldReportV1 {
        mode: ZeroQueryModeV1::ReadOnly,
        provenance: ZeroQueryProvenanceV1::TypedCatalogExampleMetadata,
        groups: vec![
            build_catalog_group_v1(),
            build_example_group_v1(StarterGroupKindV1::Analysis),
            build_example_group_v1(StarterGroupKindV1::Retrieval),
        ],
    }
}

fn build_catalog_group_v1() -> StarterGroupV1 {
    let suggestions = CATALOG_ASSETS_V1
        .iter()
        .flat_map(|asset| {
            [
                StarterSuggestionV1 {
                    label: asset.show_query_scaffold,
                    root_kind: StarterRootKindV1::Show,
                    query_scaffold: asset.show_query_scaffold,
                    behavior: StarterBehaviorV1::ReadOnlyScaffold,
                    reason: StarterReasonV1 {
                        code: StarterReasonCodeV1::CatalogOverview,
                        detail: CATALOG_SHOW_REASON_V1,
                    },
                },
                StarterSuggestionV1 {
                    label: asset.describe_query_scaffold,
                    root_kind: StarterRootKindV1::Describe,
                    query_scaffold: asset.describe_query_scaffold,
                    behavior: StarterBehaviorV1::ReadOnlyScaffold,
                    reason: StarterReasonV1 {
                        code: StarterReasonCodeV1::TypedTargetDetail,
                        detail: CATALOG_DESCRIBE_REASON_V1,
                    },
                },
            ]
        })
        .collect();

    StarterGroupV1 {
        kind: StarterGroupKindV1::Catalog,
        suggestions,
    }
}

fn build_example_group_v1(kind: StarterGroupKindV1) -> StarterGroupV1 {
    let suggestions = EXAMPLE_ASSETS_V1
        .iter()
        .filter(|asset| asset.group == kind)
        .map(|asset| StarterSuggestionV1 {
            label: asset.label,
            root_kind: asset.root_kind,
            query_scaffold: asset.query_scaffold,
            behavior: StarterBehaviorV1::ReadOnlyScaffold,
            reason: StarterReasonV1 {
                code: asset.reason_code,
                detail: asset.reason_detail,
            },
        })
        .collect();

    StarterGroupV1 { kind, suggestions }
}
