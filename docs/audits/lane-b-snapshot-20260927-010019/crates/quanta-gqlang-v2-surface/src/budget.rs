//! Parse-owned resource budgets and one non-resetting document counter.

use crate::diag::ParseDiagnosticV1;
use quanta_gqlang_v2_hir::{SpanV1, MAX_SAFE_EXPRESSION_DEPTH_V2, MAX_SAFE_RELATION_DEPTH_V2};
use std::cell::Cell;

/// Stable parse-budget dimension names.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParseBudgetDimensionV2 {
    /// Lexed tokens, including EOF.
    Tokens,
    /// Top-level document items.
    DocumentItems,
    /// Value and Bool HIR nodes.
    ExpressionNodes,
    /// Recursive parser descent and the resulting expression-tree depth.
    ExpressionDepth,
    /// Nested relation stages on one execution path.
    RelationDepth,
    /// Elements in one collection.
    CollectionWidth,
    /// Items in one projection/group/order key list.
    ProjectionWidth,
}

impl ParseBudgetDimensionV2 {
    /// Stable wire/debug spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Tokens => "tokens",
            Self::DocumentItems => "document_items",
            Self::ExpressionNodes => "expression_nodes",
            Self::ExpressionDepth => "expression_depth",
            Self::RelationDepth => "relation_depth",
            Self::CollectionWidth => "collection_width",
            Self::ProjectionWidth => "projection_width",
        }
    }
}

/// Immutable positive parse budget configuration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseBudgetV2 {
    tokens: usize,
    document_items: usize,
    expression_nodes: usize,
    expression_depth: usize,
    relation_depth: usize,
    collection_width: usize,
    projection_width: usize,
}

impl ParseBudgetV2 {
    /// Construct a positive budget below the shared HIR traversal safety caps.
    /// Zero is rejected because it cannot even admit the mandatory EOF token.
    pub fn try_new(
        tokens: usize,
        document_items: usize,
        expression_nodes: usize,
        expression_depth: usize,
        relation_depth: usize,
        collection_width: usize,
        projection_width: usize,
    ) -> Result<Self, &'static str> {
        if [
            tokens,
            document_items,
            expression_nodes,
            expression_depth,
            relation_depth,
            collection_width,
            projection_width,
        ]
        .contains(&0)
            || expression_depth > MAX_SAFE_EXPRESSION_DEPTH_V2
            || relation_depth > MAX_SAFE_RELATION_DEPTH_V2
        {
            return Err("parse budget dimensions must be positive and depths must stay within their safe maxima");
        }
        Ok(Self {
            tokens,
            document_items,
            expression_nodes,
            expression_depth,
            relation_depth,
            collection_width,
            projection_width,
        })
    }

    /// Repository-standard bounded profile.
    #[must_use]
    pub const fn standard_v2() -> Self {
        Self {
            tokens: 65_536,
            document_items: 4_096,
            expression_nodes: 131_072,
            expression_depth: MAX_SAFE_EXPRESSION_DEPTH_V2,
            relation_depth: MAX_SAFE_RELATION_DEPTH_V2,
            collection_width: 4_096,
            projection_width: 1_024,
        }
    }

    pub(crate) const fn max_expression_depth(&self) -> usize {
        self.expression_depth
    }

    pub(crate) const fn max_relation_depth(&self) -> usize {
        self.relation_depth
    }
}

/// One counter per parse. Interior mutability lets recursive descent share it
/// without creating a second ownership path.
#[derive(Debug)]
pub(crate) struct ParseBudgetCounterV2 {
    budget: ParseBudgetV2,
    tokens: Cell<usize>,
    document_items: Cell<usize>,
    expression_nodes: Cell<usize>,
    expression_depth: Cell<usize>,
    relation_depth: Cell<usize>,
}

impl ParseBudgetCounterV2 {
    pub(crate) fn new(budget: &ParseBudgetV2) -> Self {
        Self {
            budget: budget.clone(),
            tokens: Cell::new(0),
            document_items: Cell::new(0),
            expression_nodes: Cell::new(0),
            expression_depth: Cell::new(0),
            relation_depth: Cell::new(0),
        }
    }

    pub(crate) fn max_expression_depth(&self) -> usize {
        self.budget.max_expression_depth()
    }

    pub(crate) fn max_relation_depth(&self) -> usize {
        self.budget.max_relation_depth()
    }

    pub(crate) fn relation_depth(&self) -> usize {
        self.relation_depth.get()
    }

    pub(crate) fn restore_relation_depth(&self, depth: usize) {
        self.relation_depth.set(depth);
    }

    pub(crate) fn enter_relation_depth(&self, span: SpanV1) -> Result<(), ParseDiagnosticV1> {
        self.cumulative(
            ParseBudgetDimensionV2::RelationDepth,
            &self.relation_depth,
            self.budget.relation_depth,
            span,
        )
    }

    fn cumulative(
        &self,
        dimension: ParseBudgetDimensionV2,
        cell: &Cell<usize>,
        limit: usize,
        span: SpanV1,
    ) -> Result<(), ParseDiagnosticV1> {
        let observed = cell.get().saturating_add(1);
        if observed > limit {
            return Err(ParseDiagnosticV1::resource_exhausted(
                dimension, limit, observed, span,
            ));
        }
        cell.set(observed);
        Ok(())
    }

    pub(crate) fn charge_token(&self, span: SpanV1) -> Result<(), ParseDiagnosticV1> {
        self.cumulative(
            ParseBudgetDimensionV2::Tokens,
            &self.tokens,
            self.budget.tokens,
            span,
        )
    }

    pub(crate) fn charge_document_item(&self, span: SpanV1) -> Result<(), ParseDiagnosticV1> {
        self.cumulative(
            ParseBudgetDimensionV2::DocumentItems,
            &self.document_items,
            self.budget.document_items,
            span,
        )
    }

    pub(crate) fn charge_expression_node(&self, span: SpanV1) -> Result<(), ParseDiagnosticV1> {
        self.cumulative(
            ParseBudgetDimensionV2::ExpressionNodes,
            &self.expression_nodes,
            self.budget.expression_nodes,
            span,
        )
    }

    pub(crate) fn enter_expression_depth(&self, span: SpanV1) -> Result<(), ParseDiagnosticV1> {
        self.cumulative(
            ParseBudgetDimensionV2::ExpressionDepth,
            &self.expression_depth,
            self.budget.expression_depth,
            span,
        )
    }

    pub(crate) fn leave_expression_depth(&self) {
        self.expression_depth
            .set(self.expression_depth.get().saturating_sub(1));
    }

    pub(crate) fn charge_collection_element(
        &self,
        observed: usize,
        span: SpanV1,
    ) -> Result<(), ParseDiagnosticV1> {
        if observed > self.budget.collection_width {
            Err(ParseDiagnosticV1::resource_exhausted(
                ParseBudgetDimensionV2::CollectionWidth,
                self.budget.collection_width,
                observed,
                span,
            ))
        } else {
            Ok(())
        }
    }

    pub(crate) fn charge_projection_element(
        &self,
        observed: usize,
        span: SpanV1,
    ) -> Result<(), ParseDiagnosticV1> {
        self.charge_collection_element(observed, span)?;
        if observed > self.budget.projection_width {
            Err(ParseDiagnosticV1::resource_exhausted(
                ParseBudgetDimensionV2::ProjectionWidth,
                self.budget.projection_width,
                observed,
                span,
            ))
        } else {
            Ok(())
        }
    }
}
