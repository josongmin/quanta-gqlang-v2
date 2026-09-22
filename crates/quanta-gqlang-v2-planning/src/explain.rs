//! The logical explain / provenance contract over sealed semantics (ticket 07,
//! decision `D24`).
//!
//! `D24`: logical explain/provenance and semantic cost signals are FRONTEND-owned
//! host-agnostic outputs; runtime physical plans and substrate profiling stay
//! downstream. [`ExplainNoteV2`] owns the LOGICAL explain note: the checked root
//! location of the planned item, the ordered checked algebra families the result
//! is built from (root-first, left-before-right), and whether the plan is
//! recursive. It says WHAT sealed constructs produce the result, never HOW a
//! runtime would execute it.
//!
//! The V1 free `ExplainNoteV1::from_root`/`for_item` constructors and their
//! recursive `collect_families` walk are deleted: the family list is produced by
//! the single iterative budgeted traversal run by [`crate::plan_outcome`].

use quanta_gqlang_v2_algebra::AlgebraFamilyV2;
use quanta_gqlang_v2_sema::CheckedLocatedSpanV2;

/// A logical explain note: a host-agnostic provenance summary of the sealed
/// plan. Ordered top-down (root family first) so CLI/LSP/tests see a stable
/// rendering.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExplainNoteV2 {
    /// Checked location of the planned item (`D20`).
    root_location: CheckedLocatedSpanV2,
    /// The checked algebra families present, root-first in stable pre-order.
    families: Vec<AlgebraFamilyV2>,
    /// Whether the plan carries an admitted recursive closure.
    recursive: bool,
}

impl ExplainNoteV2 {
    /// Build one explain note from the traversal's ordered family list.
    pub(crate) const fn new(
        root_location: CheckedLocatedSpanV2,
        families: Vec<AlgebraFamilyV2>,
        recursive: bool,
    ) -> Self {
        Self {
            root_location,
            families,
            recursive,
        }
    }

    /// Checked location of the planned item.
    #[must_use]
    pub const fn root_location(&self) -> &CheckedLocatedSpanV2 {
        &self.root_location
    }

    /// The checked algebra families present, root-first in stable pre-order.
    #[must_use]
    pub fn families(&self) -> &[AlgebraFamilyV2] {
        &self.families
    }

    /// Whether the plan carries an admitted recursive closure.
    #[must_use]
    pub const fn recursive(&self) -> bool {
        self.recursive
    }
}
