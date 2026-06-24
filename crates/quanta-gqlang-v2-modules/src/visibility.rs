//! Module visibility / export rules (deliverable 4).
//!
//! A module's symbols are private by default; only [`VisibilityV1::Public`]
//! symbols may be imported by other modules, and only public symbols may be
//! re-exported. This is the closed, deterministic visibility lattice the import
//! graph consults — there is no implicit "export everything" behavior.

/// Visibility of a declared module symbol.
///
/// Defaults to [`VisibilityV1::Private`]: a symbol is module-private unless
/// explicitly made public, so there is no implicit "export everything".
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum VisibilityV1 {
    /// Visible only inside the declaring module. Not importable or re-exportable.
    #[default]
    Private,
    /// Visible to other modules: importable and re-exportable.
    Public,
}

impl VisibilityV1 {
    /// Whether a symbol with this visibility may be imported by another module.
    #[must_use]
    pub const fn is_importable(self) -> bool {
        matches!(self, Self::Public)
    }
}
