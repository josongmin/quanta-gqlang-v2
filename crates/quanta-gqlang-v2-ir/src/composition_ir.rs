//! `LET` / `WITH` composition normalized IR (E03C ticket 03).
//!
//! Every `LET` binding lowers to a [`WorkingSetRefIrV1`] *descriptor*. This is
//! the first production consumer of the working-set vocabulary enums
//! ([`crate::working_set::WorkingSetKindV1`] / [`WorkingSetBasisV1`]).
//!
//! NOTE (scope, do not overstate): the full [`crate::working_set::WorkingSetV1`]
//! struct (with `items`, `validate_v1`, `WorkingSetReasonV1`) remains the
//! HOST's materialization shape and is still not produced in this read-only
//! repo. The descriptor here deliberately carries no `items` field — that is
//! exactly why a composition node can never widen into a runtime packet by
//! construction (E03C DoD), but it also means audit finding #6
//! (`WorkingSetV1`-struct production-dead) is only partially addressed: its
//! enums are now live, its struct is not.

use crate::root_ir::PublicIrRootV1;
use crate::working_set::{WorkingSetBasisV1, WorkingSetKindV1};

/// A stable, read-only reference to a working set a `LET` binding names. It
/// carries identity and basis only: enough for a host to materialize later,
/// nothing that asserts a materialized result here.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkingSetRefIrV1 {
    pub working_set_id: String,
    pub kind: WorkingSetKindV1,
    pub basis: WorkingSetBasisV1,
}

/// A lowered composition: each binding pairs its working-set descriptor with the
/// lowered producer root; `narrowed_by` indexes the binding the `WITH` selected;
/// `body` is the lowered body root. Producers and body keep their own lowered
/// IR (including their own degraded lanes) — nothing is swallowed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompositionIrV1 {
    pub bindings: Vec<(WorkingSetRefIrV1, PublicIrRootV1)>,
    pub narrowed_by: usize,
    pub body: Box<PublicIrRootV1>,
}
