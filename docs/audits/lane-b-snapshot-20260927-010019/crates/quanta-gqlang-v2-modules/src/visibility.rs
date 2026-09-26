//! Closed module visibility used by checked authority.

/// Closed semantic visibility used by checked module authority.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum VisibilityV2 {
    Private,
    Public,
}

impl VisibilityV2 {
    #[must_use]
    pub const fn is_importable(self) -> bool {
        matches!(self, Self::Public)
    }
}
