//! Checked relational algebra lowered exclusively from the sealed semantic
//! document (ticket 05).
//!
//! This crate never reads HIR, Binder frames, source spelling, or caller
//! supplied aggregate/catalog options. [`lower_relation_item`] is the single
//! public source ingress: it retains the semantic output proof verbatim and
//! rejects an item that is not a sealed relation. Algebra nodes are opaque;
//! downstream planning can inspect only read-only contracts and cannot forge a
//! node with mismatched output, source identity, or correlation metadata.

mod diagnostic;
mod node;
mod normalize;

pub use diagnostic::{AlgebraDiagnosticV2, AlgebraFailureKindV2};
pub use node::{AlgebraFamilyV2, CheckedAlgebraNodeV2, CorrelationSetV2, LoweredRelationItemV2};
pub use normalize::lower_relation_item;
