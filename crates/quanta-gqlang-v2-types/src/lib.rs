//! QGLang v3 type universe and operator/function registry (ticket 03).
//!
//! Owns the closed scalar/structural/domain type universe ([`ty`]), the closed
//! operator registry ([`op`]) and closed builtin-function registry with metadata
//! ([`builtin`]), overload resolution and value-expression type checking
//! ([`check`]) against a caller-provided type environment ([`env`]), and typed
//! mismatch diagnostics ([`diag`]).
//!
//! Packet decisions cited:
//!
//! - `D4`: type checking uses a closed operator/function registry with typed
//!   errors; no backend/provider state participates in any lookup.
//! - `D10`: unsupported constructs reject explicitly via typed diagnostics; no
//!   silent downgrade.
//! - `D20`: result kinds, source locations, witness/path payloads, and
//!   explainability are semantic types here (`location`, `path`, `proof_tree`),
//!   not CLI-only presentation.
//! - `D21`/`D24`: builtin metadata (purity, determinism, monotonicity,
//!   aggregation class, cost class) is frontend-owned and consumed downstream by
//!   recursion legality (`05`) and planning/cost (`07`).
//! - `D29`: nested result-shape STRUCTURE is owned by ticket 01; this crate
//!   TYPES it through the shared record/list value nodes and introduces no
//!   second shape tree.
//!
//! # Binder seam
//!
//! To stay parallel-safe with the binder (ticket 02), the checker consumes a
//! caller-provided symbol -> type map ([`TypeEnvV1`]); it performs no scope or
//! import resolution. Ticket 02 will produce that map from resolved bindings.

mod builtin;
mod check;
mod diag;
mod env;
mod op;
mod ty;

pub use builtin::{
    AggregationClassV1, BuiltinEntryV1, BuiltinMetaV1, BuiltinRegistryV1, BuiltinSignatureV1,
    CostClassV1, DeterminismV1, MonotonicityV1, PurityV1,
};
pub use check::{scalar, TypeCheckerV1};
pub use diag::{IllegalAggregateReasonV1, TypeErrorV1};
pub use env::TypeEnvV1;
pub use op::{BinaryOverloadV1, OperatorRegistryV1, UnaryOverloadV1};
pub use ty::{CardinalityV1, RecordFieldTyV1, ScalarTyV1, TypeV1, UnionVariantTyV1};
