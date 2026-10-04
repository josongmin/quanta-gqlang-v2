//! Canonical frontend budgets and sealed downstream receipts.

use super::{configuration_failure, AuthorityBudgetProfileV2, FrontendAuthorityErrorV2};
use std::num::NonZeroU32;

/// Immutable configuration for every bounded frontend stage.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrontendBudgetV2 {
    max_source_bytes: NonZeroU32,
    max_frame_bytes: NonZeroU32,
    max_tokens: NonZeroU32,
    max_document_items: NonZeroU32,
    max_expression_depth: NonZeroU32,
    max_relation_depth: NonZeroU32,
    max_expression_nodes: NonZeroU32,
    max_collection_width: NonZeroU32,
    max_projection_width: NonZeroU32,
    max_semantic_steps: NonZeroU32,
    max_module_graph_nodes: NonZeroU32,
    max_module_graph_edges: NonZeroU32,
    max_module_steps: NonZeroU32,
    max_authority_bytes: NonZeroU32,
    max_authority_string_bytes: NonZeroU32,
    max_rule_graph_nodes: NonZeroU32,
    max_rule_graph_edges: NonZeroU32,
    max_rule_program_rules: NonZeroU32,
    max_rule_steps: NonZeroU32,
    max_witness_depth: NonZeroU32,
    max_witnesses: NonZeroU32,
    max_diagnostics: NonZeroU32,
    max_response_bytes: NonZeroU32,
}

/// Immutable read-only projection of one `FrontendBudgetV2` retained by the
/// sealed semantic document. Downstream stages (algebra, logic, planning, and
/// the front doors) read their traversal and output caps from the document;
/// no consumer accepts an independent budget argument, a caller-selected
/// replacement, or a default. The receipt never extends the canonical budget:
/// every dimension is copied exactly once by
/// [`FrontendBudgetV2::downstream_receipt`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DownstreamBudgetReceiptV2 {
    max_source_bytes: NonZeroU32,
    max_frame_bytes: NonZeroU32,
    max_tokens: NonZeroU32,
    max_document_items: NonZeroU32,
    max_expression_depth: NonZeroU32,
    max_relation_depth: NonZeroU32,
    max_expression_nodes: NonZeroU32,
    max_collection_width: NonZeroU32,
    max_projection_width: NonZeroU32,
    max_semantic_steps: NonZeroU32,
    max_module_graph_nodes: NonZeroU32,
    max_module_graph_edges: NonZeroU32,
    max_module_steps: NonZeroU32,
    max_authority_bytes: NonZeroU32,
    max_authority_string_bytes: NonZeroU32,
    max_rule_graph_nodes: NonZeroU32,
    max_rule_graph_edges: NonZeroU32,
    max_rule_program_rules: NonZeroU32,
    max_rule_steps: NonZeroU32,
    max_witness_depth: NonZeroU32,
    max_witnesses: NonZeroU32,
    max_diagnostics: NonZeroU32,
    max_response_bytes: NonZeroU32,
}

impl DownstreamBudgetReceiptV2 {
    /// Maximum source bytes admitted to one document transaction.
    #[must_use]
    pub const fn max_source_bytes(&self) -> u32 {
        self.max_source_bytes.get()
    }
    /// Maximum transport frame bytes for one request.
    #[must_use]
    pub const fn max_frame_bytes(&self) -> u32 {
        self.max_frame_bytes.get()
    }
    /// Maximum surface parse tokens.
    #[must_use]
    pub const fn max_tokens(&self) -> u32 {
        self.max_tokens.get()
    }
    /// Maximum authored document items.
    #[must_use]
    pub const fn max_document_items(&self) -> u32 {
        self.max_document_items.get()
    }
    /// Maximum expression nesting depth.
    #[must_use]
    pub const fn max_expression_depth(&self) -> u32 {
        self.max_expression_depth.get()
    }
    /// Maximum nested relation stages on one execution path.
    #[must_use]
    pub const fn max_relation_depth(&self) -> u32 {
        self.max_relation_depth.get()
    }
    /// Expression walk limit shared by every downstream expression traversal.
    #[must_use]
    pub const fn max_expression_nodes(&self) -> u32 {
        self.max_expression_nodes.get()
    }
    /// Maximum authored collection width.
    #[must_use]
    pub const fn max_collection_width(&self) -> u32 {
        self.max_collection_width.get()
    }
    /// Maximum authored projection width.
    #[must_use]
    pub const fn max_projection_width(&self) -> u32 {
        self.max_projection_width.get()
    }
    /// Relation walk and planning traversal step limit.
    #[must_use]
    pub const fn max_semantic_steps(&self) -> u32 {
        self.max_semantic_steps.get()
    }
    /// Module graph node limit.
    #[must_use]
    pub const fn max_module_graph_nodes(&self) -> u32 {
        self.max_module_graph_nodes.get()
    }
    /// Module graph edge limit.
    #[must_use]
    pub const fn max_module_graph_edges(&self) -> u32 {
        self.max_module_graph_edges.get()
    }
    /// Module admission step limit.
    #[must_use]
    pub const fn max_module_steps(&self) -> u32 {
        self.max_module_steps.get()
    }
    /// Authority byte session limit.
    #[must_use]
    pub const fn max_authority_bytes(&self) -> u32 {
        self.max_authority_bytes.get()
    }
    /// Authority string byte limit.
    #[must_use]
    pub const fn max_authority_string_bytes(&self) -> u32 {
        self.max_authority_string_bytes.get()
    }
    /// Logic rule graph node limit.
    #[must_use]
    pub const fn max_rule_graph_nodes(&self) -> u32 {
        self.max_rule_graph_nodes.get()
    }
    /// Logic rule graph edge limit.
    #[must_use]
    pub const fn max_rule_graph_edges(&self) -> u32 {
        self.max_rule_graph_edges.get()
    }
    /// Maximum raw rules admitted before constructing a logic graph.
    #[must_use]
    pub const fn max_rule_program_rules(&self) -> u32 {
        self.max_rule_program_rules.get()
    }
    /// Logic rule relaxation step limit.
    #[must_use]
    pub const fn max_rule_steps(&self) -> u32 {
        self.max_rule_steps.get()
    }
    /// Graph witness depth limit.
    #[must_use]
    pub const fn max_witness_depth(&self) -> u32 {
        self.max_witness_depth.get()
    }
    /// Graph witness count limit.
    #[must_use]
    pub const fn max_witnesses(&self) -> u32 {
        self.max_witnesses.get()
    }
    /// Maximum diagnostics accumulated for one frontend outcome.
    #[must_use]
    pub const fn max_diagnostics(&self) -> u32 {
        self.max_diagnostics.get()
    }
    /// Maximum serialized response bytes.
    #[must_use]
    pub const fn max_response_bytes(&self) -> u32 {
        self.max_response_bytes.get()
    }
}

impl FrontendBudgetV2 {
    /// Construct one checked frontend budget. Every limit must be positive,
    /// depths must fit the shared HIR safety caps, and the transport frame must
    /// be able to contain the admitted source.
    pub fn try_new(
        max_source_bytes: u32,
        max_frame_bytes: u32,
        max_tokens: u32,
        max_document_items: u32,
        max_expression_depth: u32,
        max_relation_depth: u32,
        max_expression_nodes: u32,
        max_collection_width: u32,
        max_projection_width: u32,
        max_semantic_steps: u32,
        max_module_graph_nodes: u32,
        max_module_graph_edges: u32,
        max_module_steps: u32,
        max_authority_bytes: u32,
        max_authority_string_bytes: u32,
        max_rule_graph_nodes: u32,
        max_rule_graph_edges: u32,
        max_rule_steps: u32,
        max_witness_depth: u32,
        max_witnesses: u32,
        max_diagnostics: u32,
        max_response_bytes: u32,
    ) -> Result<Self, FrontendAuthorityErrorV2> {
        let raw = [
            max_source_bytes,
            max_frame_bytes,
            max_tokens,
            max_document_items,
            max_expression_depth,
            max_relation_depth,
            max_expression_nodes,
            max_collection_width,
            max_projection_width,
            max_semantic_steps,
            max_module_graph_nodes,
            max_module_graph_edges,
            max_module_steps,
            max_authority_bytes,
            max_authority_string_bytes,
            max_rule_graph_nodes,
            max_rule_graph_edges,
            max_rule_steps,
            max_witness_depth,
            max_witnesses,
            max_diagnostics,
            max_response_bytes,
        ];
        if raw.contains(&0)
            || max_frame_bytes < max_source_bytes
            || usize::try_from(max_expression_depth).unwrap_or(usize::MAX)
                > quanta_gqlang_v2_hir::MAX_SAFE_EXPRESSION_DEPTH_V2
            || usize::try_from(max_relation_depth).unwrap_or(usize::MAX)
                > quanta_gqlang_v2_hir::MAX_SAFE_RELATION_DEPTH_V2
        {
            return Err(configuration_failure(
                "frontend.invalid_budget",
                "all frontend limits must be positive, frame bytes must cover source bytes, and expression/relation depths must stay within their safe maxima",
            ));
        }
        let mut values = raw.into_iter().map(NonZeroU32::new);
        let mut field = || {
            values
                .next()
                .flatten()
                .ok_or_else(|| configuration_failure("frontend.invalid_budget", "invalid budget"))
        };
        // Preserve the existing constructor's profile while adding a distinct
        // cap for caller-built programs with multiple rules per predicate.
        let max_rule_program_rules = NonZeroU32::new(
            max_rule_graph_nodes.saturating_add(max_rule_graph_edges),
        )
        .ok_or_else(|| configuration_failure("frontend.invalid_budget", "invalid rule cap"))?;
        Ok(Self {
            max_source_bytes: field()?,
            max_frame_bytes: field()?,
            max_tokens: field()?,
            max_document_items: field()?,
            max_expression_depth: field()?,
            max_relation_depth: field()?,
            max_expression_nodes: field()?,
            max_collection_width: field()?,
            max_projection_width: field()?,
            max_semantic_steps: field()?,
            max_module_graph_nodes: field()?,
            max_module_graph_edges: field()?,
            max_module_steps: field()?,
            max_authority_bytes: field()?,
            max_authority_string_bytes: field()?,
            max_rule_graph_nodes: field()?,
            max_rule_graph_edges: field()?,
            max_rule_program_rules,
            max_rule_steps: field()?,
            max_witness_depth: field()?,
            max_witnesses: field()?,
            max_diagnostics: field()?,
            max_response_bytes: field()?,
        })
    }

    /// Set the independent raw-rule admission cap before sealing an authority.
    /// This does not change the graph vertex, edge, or traversal budgets.
    pub fn try_with_rule_program_rules(
        mut self,
        limit: u32,
    ) -> Result<Self, FrontendAuthorityErrorV2> {
        self.max_rule_program_rules = NonZeroU32::new(limit).ok_or_else(|| {
            configuration_failure("frontend.invalid_budget", "raw rule cap must be positive")
        })?;
        Ok(self)
    }

    /// Pinned CLI/service profile.
    pub fn standard_v2() -> Result<Self, FrontendAuthorityErrorV2> {
        Self::try_new(
            1_048_576,
            4_194_304,
            131_072,
            4_096,
            u32::try_from(quanta_gqlang_v2_hir::MAX_SAFE_EXPRESSION_DEPTH_V2).unwrap_or(u32::MAX),
            u32::try_from(quanta_gqlang_v2_hir::MAX_SAFE_RELATION_DEPTH_V2).unwrap_or(u32::MAX),
            262_144,
            4_096,
            4_096,
            1_000_000,
            50_000,
            200_000,
            1_000_000,
            16_777_216,
            65_536,
            50_000,
            200_000,
            1_000_000,
            1_024,
            10_000,
            1_024,
            8_388_608,
        )
    }

    /// The one immutable downstream budget receipt. Every downstream traversal
    /// and output cap is read from this projection on the sealed semantic
    /// document; it is never extended, replaced, or accepted as an independent
    /// consumer argument.
    #[must_use]
    pub const fn downstream_receipt(&self) -> DownstreamBudgetReceiptV2 {
        DownstreamBudgetReceiptV2 {
            max_source_bytes: self.max_source_bytes,
            max_frame_bytes: self.max_frame_bytes,
            max_tokens: self.max_tokens,
            max_document_items: self.max_document_items,
            max_expression_depth: self.max_expression_depth,
            max_relation_depth: self.max_relation_depth,
            max_expression_nodes: self.max_expression_nodes,
            max_collection_width: self.max_collection_width,
            max_projection_width: self.max_projection_width,
            max_semantic_steps: self.max_semantic_steps,
            max_module_graph_nodes: self.max_module_graph_nodes,
            max_module_graph_edges: self.max_module_graph_edges,
            max_module_steps: self.max_module_steps,
            max_authority_bytes: self.max_authority_bytes,
            max_authority_string_bytes: self.max_authority_string_bytes,
            max_rule_graph_nodes: self.max_rule_graph_nodes,
            max_rule_graph_edges: self.max_rule_graph_edges,
            max_rule_program_rules: self.max_rule_program_rules,
            max_rule_steps: self.max_rule_steps,
            max_witness_depth: self.max_witness_depth,
            max_witnesses: self.max_witnesses,
            max_diagnostics: self.max_diagnostics,
            max_response_bytes: self.max_response_bytes,
        }
    }

    /// Canonical projection used by module, relation, and per-document catalog
    /// admission. Each admission owns its own non-resettable counter.
    pub fn authority_profile(&self) -> Result<AuthorityBudgetProfileV2, FrontendAuthorityErrorV2> {
        AuthorityBudgetProfileV2::try_new(
            usize::try_from(self.max_module_graph_nodes.get()).unwrap_or(usize::MAX),
            usize::try_from(self.max_module_graph_edges.get()).unwrap_or(usize::MAX),
            usize::try_from(self.max_module_steps.get()).unwrap_or(usize::MAX),
            usize::try_from(self.max_authority_bytes.get()).unwrap_or(usize::MAX),
            usize::try_from(self.max_authority_string_bytes.get()).unwrap_or(usize::MAX),
        )
        .map_err(|error| configuration_failure(error.code(), error.to_string()))
    }

    /// Maximum source bytes.
    #[must_use]
    pub const fn max_source_bytes(&self) -> u32 {
        self.max_source_bytes.get()
    }
    /// Maximum frame bytes.
    #[must_use]
    pub const fn max_frame_bytes(&self) -> u32 {
        self.max_frame_bytes.get()
    }
    /// Maximum tokens.
    #[must_use]
    pub const fn max_tokens(&self) -> u32 {
        self.max_tokens.get()
    }
    /// Maximum document items.
    #[must_use]
    pub const fn max_document_items(&self) -> u32 {
        self.max_document_items.get()
    }
    /// Maximum expression depth.
    #[must_use]
    pub const fn max_expression_depth(&self) -> u32 {
        self.max_expression_depth.get()
    }
    /// Maximum nested relation stages on one execution path.
    #[must_use]
    pub const fn max_relation_depth(&self) -> u32 {
        self.max_relation_depth.get()
    }
    /// Maximum expression nodes.
    #[must_use]
    pub const fn max_expression_nodes(&self) -> u32 {
        self.max_expression_nodes.get()
    }
    /// Maximum collection width.
    #[must_use]
    pub const fn max_collection_width(&self) -> u32 {
        self.max_collection_width.get()
    }
    /// Maximum projection width.
    #[must_use]
    pub const fn max_projection_width(&self) -> u32 {
        self.max_projection_width.get()
    }
    /// Maximum semantic steps.
    #[must_use]
    pub const fn max_semantic_steps(&self) -> u32 {
        self.max_semantic_steps.get()
    }
    /// Maximum module graph nodes.
    #[must_use]
    pub const fn max_module_graph_nodes(&self) -> u32 {
        self.max_module_graph_nodes.get()
    }
    /// Maximum module graph edges.
    #[must_use]
    pub const fn max_module_graph_edges(&self) -> u32 {
        self.max_module_graph_edges.get()
    }
    /// Maximum module steps.
    #[must_use]
    pub const fn max_module_steps(&self) -> u32 {
        self.max_module_steps.get()
    }
    /// Maximum authority bytes per sealed admission session.
    #[must_use]
    pub const fn max_authority_bytes(&self) -> u32 {
        self.max_authority_bytes.get()
    }
    /// Maximum bytes in one authority string.
    #[must_use]
    pub const fn max_authority_string_bytes(&self) -> u32 {
        self.max_authority_string_bytes.get()
    }
    /// Maximum rule graph nodes.
    #[must_use]
    pub const fn max_rule_graph_nodes(&self) -> u32 {
        self.max_rule_graph_nodes.get()
    }
    /// Maximum rule graph edges.
    #[must_use]
    pub const fn max_rule_graph_edges(&self) -> u32 {
        self.max_rule_graph_edges.get()
    }
    /// Maximum raw rules admitted before constructing a logic graph.
    #[must_use]
    pub const fn max_rule_program_rules(&self) -> u32 {
        self.max_rule_program_rules.get()
    }
    /// Maximum rule steps.
    #[must_use]
    pub const fn max_rule_steps(&self) -> u32 {
        self.max_rule_steps.get()
    }
    /// Maximum witness depth.
    #[must_use]
    pub const fn max_witness_depth(&self) -> u32 {
        self.max_witness_depth.get()
    }
    /// Maximum witness count.
    #[must_use]
    pub const fn max_witnesses(&self) -> u32 {
        self.max_witnesses.get()
    }
    /// Maximum diagnostics accumulated for one frontend outcome.
    #[must_use]
    pub const fn max_diagnostics(&self) -> u32 {
        self.max_diagnostics.get()
    }
    /// Maximum serialized response bytes.
    #[must_use]
    pub const fn max_response_bytes(&self) -> u32 {
        self.max_response_bytes.get()
    }
}
