//! Semantic versions and inclusive-lower / exclusive-upper dependency ranges.
//!
//! Ticket 11 needs reproducible, host-agnostic version comparison for pack
//! identity, dependency ranges, and language/feature compatibility windows
//! (deliverable 3). This is a deliberately small, self-contained `major.minor.patch`
//! model: no pre-release / build metadata grammar, because pack resolution here
//! is offline and deterministic (`D28`) rather than a general-purpose registry.

use std::fmt;

/// A `major.minor.patch` semantic version.
///
/// Ordering is the usual lexicographic ordering on the three numeric fields, so
/// `1.2.0 < 1.10.0` and range membership is total and deterministic.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VersionV1 {
    /// Major version component.
    pub major: u64,
    /// Minor version component.
    pub minor: u64,
    /// Patch version component.
    pub patch: u64,
}

impl VersionV1 {
    /// Construct a version from its three components.
    #[must_use]
    pub const fn new(major: u64, minor: u64, patch: u64) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    /// Parse a `major.minor.patch` string. Returns `None` for any other shape
    /// (missing component, non-numeric component, trailing data).
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        let mut parts = text.split('.');
        let major = parts.next()?.parse().ok()?;
        let minor = parts.next()?.parse().ok()?;
        let patch = parts.next()?.parse().ok()?;
        if parts.next().is_some() {
            return None;
        }
        Some(Self::new(major, minor, patch))
    }
}

impl fmt::Display for VersionV1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// A half-open dependency / compatibility range `[lower, upper)`.
///
/// Inclusive lower bound, exclusive upper bound. This is the only range shape
/// the manifest exposes so that membership is unambiguous and order-free: a
/// version is in range iff `lower <= v < upper`. An empty range (`upper <=
/// lower`) is representable but never satisfiable, which is itself a useful
/// fail-closed signal.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct VersionRangeV1 {
    /// Inclusive lower bound.
    pub lower: VersionV1,
    /// Exclusive upper bound.
    pub upper: VersionV1,
}

impl VersionRangeV1 {
    /// Construct a half-open range `[lower, upper)`.
    #[must_use]
    pub const fn new(lower: VersionV1, upper: VersionV1) -> Self {
        Self { lower, upper }
    }

    /// Whether `version` satisfies this range: `lower <= version < upper`.
    #[must_use]
    pub fn contains(&self, version: VersionV1) -> bool {
        self.lower <= version && version < self.upper
    }

    /// Whether the range admits at least one version (`lower < upper`).
    #[must_use]
    pub fn is_satisfiable(&self) -> bool {
        self.lower < self.upper
    }
}

impl fmt::Display for VersionRangeV1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, ">={}, <{}", self.lower, self.upper)
    }
}
