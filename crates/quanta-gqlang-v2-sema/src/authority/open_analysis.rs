//! Optional authority for source-preserving editor analysis.

use super::{
    configuration_failure, FrontendAuthorityErrorV2, FrontendBudgetV2, GraphReferenceCatalogV2,
};
use quanta_gqlang_v2_hir::SourceRegistryViewV2;
use quanta_gqlang_v2_modules::ValidatedModuleAuthorityPartsV2;
use quanta_gqlang_v2_types::{ExpressionTypeKernelV2, RelationCatalogV2};

/// Explicit inputs for open editor analysis. Missing authority stays missing.
#[derive(Debug)]
pub struct OpenAnalysisAuthorityInputV2 {
    optional_modules: Option<ValidatedModuleAuthorityPartsV2>,
    optional_relations: Option<RelationCatalogV2>,
    optional_graph_references: Option<GraphReferenceCatalogV2>,
    host_sources: SourceRegistryViewV2,
    budget: FrontendBudgetV2,
}

impl OpenAnalysisAuthorityInputV2 {
    /// Construct the explicit open-authority input carrier.
    #[must_use]
    pub fn new(
        optional_modules: Option<ValidatedModuleAuthorityPartsV2>,
        optional_relations: Option<RelationCatalogV2>,
        optional_graph_references: Option<GraphReferenceCatalogV2>,
        host_sources: SourceRegistryViewV2,
        budget: FrontendBudgetV2,
    ) -> Self {
        Self {
            optional_modules,
            optional_relations,
            optional_graph_references,
            host_sources,
            budget,
        }
    }
}

/// Checked authority for editor analysis. Absence is distinct from an empty
/// closed proof.
#[derive(Debug)]
pub struct OpenAnalysisAuthorityV2 {
    modules: Option<ValidatedModuleAuthorityPartsV2>,
    relations: Option<RelationCatalogV2>,
    graph_references: Option<GraphReferenceCatalogV2>,
    host_sources: SourceRegistryViewV2,
    budget: FrontendBudgetV2,
}

impl OpenAnalysisAuthorityV2 {
    /// Validate every supplied proof against the explicit host registry and
    /// canonical budget projection.
    pub fn try_new(input: OpenAnalysisAuthorityInputV2) -> Result<Self, FrontendAuthorityErrorV2> {
        let projected = input.budget.authority_profile()?;
        if input.optional_modules.as_ref().is_some_and(|modules| {
            modules.source_registry().identity() != input.host_sources.identity()
                || modules.budget_profile() != &projected
        }) || input.optional_relations.as_ref().is_some_and(|relations| {
            relations.source_registry_identity() != &input.host_sources.identity()
                || relations.budget_profile() != &projected
        }) || input
            .optional_graph_references
            .as_ref()
            .is_some_and(|graph| {
                graph.source_registry_identity() != &input.host_sources.identity()
                    || graph.budget_profile() != &projected
            })
        {
            return Err(configuration_failure(
                "sema.open_authority_mismatch",
                "open authority proof does not match the host registry and budget",
            ));
        }
        Ok(Self {
            modules: input.optional_modules,
            relations: input.optional_relations,
            graph_references: input.optional_graph_references,
            host_sources: input.host_sources,
            budget: input.budget,
        })
    }

    /// Construct the standard open editor authority without inventing module,
    /// relation, catalog, pack, or lock authority.
    pub fn try_standard_v2() -> Result<Self, FrontendAuthorityErrorV2> {
        Self::try_empty_v2(FrontendBudgetV2::standard_v2()?)
    }

    /// Construct an open editor authority with an explicit checked budget and
    /// no fabricated host authority.
    pub fn try_empty_v2(budget: FrontendBudgetV2) -> Result<Self, FrontendAuthorityErrorV2> {
        let _kernel = ExpressionTypeKernelV2::try_closed()
            .map_err(|error| configuration_failure(error.code(), error.to_string()))?;
        let host_sources = SourceRegistryViewV2::try_new(Vec::new())
            .map_err(|error| configuration_failure(error.code(), error.to_string()))?;
        Self::try_new(OpenAnalysisAuthorityInputV2::new(
            None,
            None,
            None,
            host_sources,
            budget,
        ))
    }

    /// Optional module authority.
    #[must_use]
    pub const fn modules(&self) -> Option<&ValidatedModuleAuthorityPartsV2> {
        self.modules.as_ref()
    }
    /// Optional relation catalog.
    #[must_use]
    pub const fn relations(&self) -> Option<&RelationCatalogV2> {
        self.relations.as_ref()
    }
    /// Optional host graph selectors.
    #[must_use]
    pub const fn graph_references(&self) -> Option<&GraphReferenceCatalogV2> {
        self.graph_references.as_ref()
    }
    /// Checked host sources.
    #[must_use]
    pub const fn host_sources(&self) -> &SourceRegistryViewV2 {
        &self.host_sources
    }
    /// Frontend budget.
    #[must_use]
    pub const fn budget(&self) -> &FrontendBudgetV2 {
        &self.budget
    }
}
