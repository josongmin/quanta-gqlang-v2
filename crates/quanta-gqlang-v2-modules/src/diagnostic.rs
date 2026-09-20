//! Source-rich module authority diagnostics with stable codes.

use std::fmt;

use crate::graph::AmbiguousCandidateV2;
use quanta_gqlang_v2_hir::{CheckedLocatedSpanV2, LocatedSpanV2};

/// Source-rich failure produced by checked authority construction.
///
/// Consumers use stable codes and structured location/witness accessors rather
/// than matching rendered strings.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModuleDiagnosticV2 {
    code: &'static str,
    details: Box<ModuleDiagnosticDetailsV2>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ModuleDiagnosticDetailsV2 {
    message: String,
    resource_exhaustion: Option<ResourceExhaustionV2>,
    primary: Option<LocatedSpanV2>,
    secondary: Option<LocatedSpanV2>,
    edge_witness: Vec<LocatedSpanV2>,
    ambiguous_candidates: Vec<AmbiguousCandidateV2>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceExhaustionV2 {
    dimension: &'static str,
    limit: usize,
    observed: usize,
}

impl ResourceExhaustionV2 {
    #[must_use]
    pub const fn dimension(&self) -> &'static str {
        self.dimension
    }

    #[must_use]
    pub const fn limit(&self) -> usize {
        self.limit
    }

    #[must_use]
    pub const fn observed(&self) -> usize {
        self.observed
    }
}

impl ModuleDiagnosticV2 {
    pub(crate) fn new(
        code: &'static str,
        message: impl Into<String>,
        primary: &CheckedLocatedSpanV2,
    ) -> Self {
        Self {
            code,
            details: Box::new(ModuleDiagnosticDetailsV2 {
                message: bounded(message.into(), 1_024),
                resource_exhaustion: None,
                primary: Some(primary.location()),
                secondary: None,
                edge_witness: Vec::new(),
                ambiguous_candidates: Vec::new(),
            }),
        }
    }

    pub(crate) fn conflict(
        code: &'static str,
        message: impl Into<String>,
        primary: &CheckedLocatedSpanV2,
        secondary: &CheckedLocatedSpanV2,
    ) -> Self {
        Self {
            code,
            details: Box::new(ModuleDiagnosticDetailsV2 {
                message: bounded(message.into(), 1_024),
                resource_exhaustion: None,
                primary: Some(primary.location()),
                secondary: Some(secondary.location()),
                edge_witness: Vec::new(),
                ambiguous_candidates: Vec::new(),
            }),
        }
    }

    pub(crate) fn with_witness(mut self, witness: Vec<LocatedSpanV2>) -> Self {
        self.details.edge_witness = witness;
        self
    }

    pub(crate) fn with_ambiguous_candidates(
        mut self,
        candidates: Vec<AmbiguousCandidateV2>,
    ) -> Self {
        self.details.ambiguous_candidates = candidates;
        self
    }

    pub(crate) fn without_location(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            details: Box::new(ModuleDiagnosticDetailsV2 {
                message: bounded(message.into(), 1_024),
                resource_exhaustion: None,
                primary: None,
                secondary: None,
                edge_witness: Vec::new(),
                ambiguous_candidates: Vec::new(),
            }),
        }
    }

    pub(crate) fn resource_exhausted(
        dimension: &'static str,
        limit: usize,
        observed: usize,
    ) -> Self {
        Self {
            code: "modules.resource_exhausted",
            details: Box::new(ModuleDiagnosticDetailsV2 {
                message: format!(
                    "{dimension} limit {limit} exceeded by observed value {observed}"
                ),
                resource_exhaustion: Some(ResourceExhaustionV2 {
                    dimension,
                    limit,
                    observed,
                }),
                primary: None,
                secondary: None,
                edge_witness: Vec::new(),
                ambiguous_candidates: Vec::new(),
            }),
        }
    }

    #[must_use]
    pub const fn code(&self) -> &'static str {
        self.code
    }

    #[must_use]
    pub fn primary(&self) -> Option<LocatedSpanV2> {
        self.details.primary
    }

    #[must_use]
    pub fn secondary(&self) -> Option<LocatedSpanV2> {
        self.details.secondary
    }

    #[must_use]
    pub fn edge_witness(&self) -> &[LocatedSpanV2] {
        &self.details.edge_witness
    }

    #[must_use]
    pub fn ambiguous_candidates(&self) -> &[AmbiguousCandidateV2] {
        &self.details.ambiguous_candidates
    }

    #[must_use]
    pub const fn resource_exhaustion(&self) -> Option<&ResourceExhaustionV2> {
        self.details.resource_exhaustion.as_ref()
    }
}

fn bounded(mut value: String, limit: usize) -> String {
    if value.len() <= limit {
        return value;
    }
    let mut boundary = limit;
    while !value.is_char_boundary(boundary) {
        boundary -= 1;
    }
    value.truncate(boundary);
    value
}

impl fmt::Display for ModuleDiagnosticV2 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code, self.details.message)
    }
}

impl std::error::Error for ModuleDiagnosticV2 {}
