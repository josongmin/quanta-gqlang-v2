//! Canonical versions and checked half-open dependency ranges.

use crate::ModuleDiagnosticV2;
use quanta_gqlang_v2_hir::CheckedLocatedSpanV2;
use std::fmt;

/// Canonical three-component version used by checked pack authority.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VersionV2 {
    major: u64,
    minor: u64,
    patch: u64,
}

impl VersionV2 {
    #[must_use]
    pub const fn new(major: u64, minor: u64, patch: u64) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    pub fn try_parse(
        text: &str,
        location: CheckedLocatedSpanV2,
    ) -> Result<Self, ModuleDiagnosticV2> {
        let mut components = text.split('.');
        let Some(major_text) = components.next() else {
            return Err(ModuleDiagnosticV2::new(
                "modules.invalid_version",
                "version must be canonical major.minor.patch",
                &location,
            ));
        };
        let Some(minor_text) = components.next() else {
            return Err(ModuleDiagnosticV2::new(
                "modules.invalid_version",
                "version must be canonical major.minor.patch",
                &location,
            ));
        };
        let Some(patch_text) = components.next() else {
            return Err(ModuleDiagnosticV2::new(
                "modules.invalid_version",
                "version must be canonical major.minor.patch",
                &location,
            ));
        };
        if components.next().is_some()
            || [major_text, minor_text, patch_text]
                .into_iter()
                .any(|component| !canonical_decimal(component))
        {
            return Err(ModuleDiagnosticV2::new(
                "modules.invalid_version",
                "version must be canonical major.minor.patch",
                &location,
            ));
        }
        let parse = |component: &str| component.parse::<u64>().ok();
        let Some(major) = parse(major_text) else {
            return Err(ModuleDiagnosticV2::new(
                "modules.invalid_version",
                "version component exceeds u64",
                &location,
            ));
        };
        let Some(minor) = parse(minor_text) else {
            return Err(ModuleDiagnosticV2::new(
                "modules.invalid_version",
                "version component exceeds u64",
                &location,
            ));
        };
        let Some(patch) = parse(patch_text) else {
            return Err(ModuleDiagnosticV2::new(
                "modules.invalid_version",
                "version component exceeds u64",
                &location,
            ));
        };
        Ok(Self::new(major, minor, patch))
    }

    #[must_use]
    pub const fn major(self) -> u64 {
        self.major
    }
    #[must_use]
    pub const fn minor(self) -> u64 {
        self.minor
    }
    #[must_use]
    pub const fn patch(self) -> u64 {
        self.patch
    }
}

impl fmt::Display for VersionV2 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

fn canonical_decimal(component: &str) -> bool {
    !component.is_empty()
        && component.bytes().all(|byte| byte.is_ascii_digit())
        && (component == "0" || !component.starts_with('0'))
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct VersionRangeV2 {
    lower: VersionV2,
    upper: VersionV2,
    location: CheckedLocatedSpanV2,
}

impl VersionRangeV2 {
    pub fn try_new(
        lower: VersionV2,
        upper: VersionV2,
        location: CheckedLocatedSpanV2,
    ) -> Result<Self, ModuleDiagnosticV2> {
        if lower >= upper {
            return Err(ModuleDiagnosticV2::new(
                "modules.invalid_version_range",
                "version range lower bound must be less than upper bound",
                &location,
            ));
        }
        Ok(Self {
            lower,
            upper,
            location,
        })
    }
    #[must_use]
    pub const fn lower(&self) -> VersionV2 {
        self.lower
    }
    #[must_use]
    pub const fn upper(&self) -> VersionV2 {
        self.upper
    }
    #[must_use]
    pub fn contains(&self, version: VersionV2) -> bool {
        self.lower <= version && version < self.upper
    }
    #[must_use]
    pub const fn location(&self) -> &CheckedLocatedSpanV2 {
        &self.location
    }
}
