//! E07A — typed retrieval surface owners.
//!
//! The flat `RetrievalIrV1` carrier projects into a typed search-owner spec so
//! consumers route to the right search surface (text/semantic/hybrid/symbol)
//! instead of re-reading a stringly `mode`. Projection fails closed on an
//! unsupported search-owner flag (Appendix N.6) rather than silently passing a
//! sourcegraph-only operator the local surface cannot represent.

use crate::retrieval_ir::{RetrievalIrV1, RetrievalModeIrV1};
use crate::IrLoweringErrorV1;

/// Search dialect a text query is interpreted under.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RetrievalSyntaxIrV1 {
    Native,
    Sourcegraph,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextSearchSpecIrV1 {
    pub query_text: String,
    pub syntax: RetrievalSyntaxIrV1,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SemanticSearchSpecIrV1 {
    pub query_text: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HybridSearchSpecIrV1 {
    pub query_text: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SymbolProjectionIrV1 {
    WorkspaceSymbols,
    DocumentSymbols,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymbolSearchSpecIrV1 {
    pub query_text: String,
    pub projection: SymbolProjectionIrV1,
}

/// The closed set of typed retrieval surfaces.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RetrievalSpecIrV1 {
    Text(TextSearchSpecIrV1),
    Semantic(SemanticSearchSpecIrV1),
    Hybrid(HybridSearchSpecIrV1),
    Symbol(SymbolSearchSpecIrV1),
}

/// Sourcegraph-only operators the local typed surface cannot represent. A query
/// carrying one fails closed rather than being silently dropped.
const UNSUPPORTED_SEARCH_OWNER_FLAGS_V1: &[&str] = &["patterntype:", "count:all", "rev:"];

impl RetrievalIrV1 {
    /// Project the flat retrieval carrier into a typed search-owner spec.
    pub fn typed_spec_v1(&self) -> Result<RetrievalSpecIrV1, IrLoweringErrorV1> {
        if let Some(flag) = UNSUPPORTED_SEARCH_OWNER_FLAGS_V1
            .iter()
            .find(|flag| self.query.contains(**flag))
        {
            return Err(IrLoweringErrorV1 {
                code: "retrieval_unsupported_flag",
                message: format!(
                    "retrieval query uses unsupported search-owner flag `{flag}`; \
                     the typed retrieval surface cannot represent it"
                ),
            });
        }
        Ok(match self.mode {
            RetrievalModeIrV1::Lexical => RetrievalSpecIrV1::Text(TextSearchSpecIrV1 {
                query_text: self.query.clone(),
                syntax: RetrievalSyntaxIrV1::Native,
            }),
            RetrievalModeIrV1::Semantic => RetrievalSpecIrV1::Semantic(SemanticSearchSpecIrV1 {
                query_text: self.query.clone(),
            }),
            RetrievalModeIrV1::Hybrid => RetrievalSpecIrV1::Hybrid(HybridSearchSpecIrV1 {
                query_text: self.query.clone(),
            }),
        })
    }
}
