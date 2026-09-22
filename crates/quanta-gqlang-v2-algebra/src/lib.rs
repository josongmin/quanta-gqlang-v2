//! Checked relational algebra lowered exclusively from the sealed semantic
//! document (ticket 05).
//!
//! This crate never reads HIR, Binder frames, source spelling, or caller
//! supplied aggregate/catalog options. [`lower_relation_item`] is the single
//! top-level-item ingress and [`lower_nested_relation_v2`] lowers a sealed
//! NESTED relation (composition body/producer) through the same canonical
//! path: both retain the semantic output proof verbatim and reject a root that
//! is not a sealed relation of the document. Algebra nodes are opaque;
//! downstream planning can inspect only read-only contracts and cannot forge a
//! node with mismatched output, source identity, or correlation metadata.

mod diagnostic;
mod node;
mod normalize;

pub use diagnostic::{AlgebraDiagnosticV2, AlgebraFailureKindV2};
pub use node::{AlgebraFamilyV2, CheckedAlgebraNodeV2, CorrelationSetV2, LoweredRelationItemV2};
pub use normalize::{lower_nested_relation_v2, lower_relation_item, AlgebraSessionV2};
