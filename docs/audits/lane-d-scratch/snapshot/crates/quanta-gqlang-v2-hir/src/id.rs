//! Stable node identity for HIR trees.
//!
//! Every HIR node carries a [`HirNodeIdV1`]. Ids are unique within a single
//! tree and are minted monotonically by a [`HirNodeIdGen`]. Downstream tickets
//! (binder, typer, planner) key their side tables off these ids, so id
//! stability and uniqueness are load-bearing invariants enforced by
//! [`crate::integrity`].

use core::sync::atomic::{AtomicU32, Ordering};

/// A unique identity for a single HIR node within one tree.
///
/// The wrapped value is an opaque monotonic counter; do not assume contiguity
/// or ordering semantics beyond "minted earlier => smaller raw value". The only
/// guaranteed invariant is per-tree uniqueness.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HirNodeIdV1(u32);

impl HirNodeIdV1 {
    /// Raw counter value. Exposed for stable debug rendering and side-table
    /// keys; not a positional index into any collection.
    #[must_use]
    pub const fn raw(self) -> u32 {
        self.0
    }

    /// Construct an id from a raw value. Intended for deserialization and
    /// tests; normal construction goes through [`HirNodeIdGen::next`].
    #[must_use]
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }
}

/// Monotonic generator of [`HirNodeIdV1`] values.
///
/// One generator should mint every id in a single HIR tree so uniqueness is
/// guaranteed by construction. Cloning is intentionally not provided: a clone
/// would let two builders mint colliding ids. Sharing happens via `&` borrows
/// (it is `Send + Sync`).
#[derive(Debug, Default)]
pub struct HirNodeIdGen {
    next: AtomicU32,
}

impl HirNodeIdGen {
    /// A fresh generator starting at id `0`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            next: AtomicU32::new(0),
        }
    }

    /// Mint the next unique id.
    pub fn next(&self) -> HirNodeIdV1 {
        HirNodeIdV1(self.next.fetch_add(1, Ordering::Relaxed))
    }

    /// Number of ids minted so far. Equals the exclusive upper bound of every
    /// raw value handed out.
    #[must_use]
    pub fn minted(&self) -> u32 {
        self.next.load(Ordering::Relaxed)
    }
}
