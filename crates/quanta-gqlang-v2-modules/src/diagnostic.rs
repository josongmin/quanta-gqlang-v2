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

const fn decimal_digits(mut value: usize) -> usize {
    let mut digits = 1;
    while value >= 10 {
        value /= 10;
        digits += 1;
    }
    digits
}

const RESOURCE_MESSAGE_BYTES: usize = "authority_string_bytes limit ".len()
    + decimal_digits(usize::MAX)
    + " exceeded by observed value ".len()
    + decimal_digits(usize::MAX);

/// Fixed carrier for the request reference selector's bounded literal errors.
/// Ambiguity rows and copied IDs/witnesses are charged from exact candidate
/// inventories by the producer.
pub(crate) fn reference_diagnostic_fixed_bytes_v3() -> usize {
    const MESSAGES: &[&str] = &[
        "catalog reference uses another registry",
        "invalid catalog reference path",
        "visible declaration location lost registry proof",
        "external declaration location lost registry proof",
        "host catalog identity has distinct declaration origins",
        "catalog name resolves to distinct assets",
        "catalog reference has wrong kind",
        "catalog reference is absent",
        "document catalog validator cannot repeat or skip a phase",
    ];
    // A budget charge can fail after the read-only preflight only if a
    // producer's charge census drifts. Keep that diagnostic admitted too:
    // each formatted number is bounded by the decimal width of usize.
    std::mem::size_of::<ModuleDiagnosticDetailsV2>()
        + MESSAGES
            .iter()
            .map(|message| message.len())
            .max()
            .unwrap_or(0)
            .max(RESOURCE_MESSAGE_BYTES)
}

/// Longest emitted topology diagnostic plus the temporary invalid-name
/// excerpt. The excerpt length is censed from the authored edge names.
pub(crate) fn topology_diagnostic_fixed_bytes_v3(excerpt_bytes: usize) -> usize {
    const MESSAGES: &[&str] = &[
        "predeclaration belongs to another document validator",
        "target reference kind differs from expected kind",
        "document dependency source must be a local View",
        "document dependency target is absent",
        "catalog target location lost registry proof",
        "document dependency target has wrong kind",
        "duplicate edge disagrees on expected kind",
        "empty document View cycle witness",
        "document View dependency cycle",
        "document cycle witness index is outside the validated edge set",
        "resolved View edge has an unknown local source",
        "View dependency count overflowed",
        "resolved View edge has an unknown local target",
        "View dependency count underflowed",
        "acyclic View graph did not produce a complete topological order",
        "location belongs to another source registry",
        "cycle node capacity overflowed",
    ];
    let invalid_name_message = "invalid catalog local name `".len() + excerpt_bytes + "`".len();
    let message = MESSAGES
        .iter()
        .map(|message| message.len())
        .max()
        .unwrap_or(0)
        .max(RESOURCE_MESSAGE_BYTES)
        .max(invalid_name_message);
    std::mem::size_of::<ModuleDiagnosticDetailsV2>() + message + excerpt_bytes
}

pub(crate) fn final_catalog_diagnostic_bytes_v3(excerpt_bytes: usize) -> usize {
    const MESSAGES: &[&str] = &[
        "predeclaration belongs to another document validator",
        "View topology proof belongs to another document validator",
        "View topology proof was dropped, reordered, or substituted",
        "document catalog validator cannot repeat or skip a phase",
        "pack authority has no lock reference",
        "activation location belongs to another document registry",
        "duplicate pack activation",
        "activation lock reference differs from sealed authority",
        "activation names an unknown pack",
        "profile admission target is not a Profile",
        "profile admission target is absent",
        "duplicate profile admission",
        "profile location missing from document catalog",
        "local Profile requires exactly one admission",
        "catalog dependency capacity overflowed",
        "family reference proof belongs to another document validator",
        "family reference source is not the matching predeclared local asset",
        "family reference target kind differs from required kind",
        "family reference target is absent from sealed document or host authority",
        "duplicate edge disagrees on expected kind",
        "reachable profile capacity overflowed",
        "location belongs to another source registry",
    ];
    let message = MESSAGES
        .iter()
        .map(|message| message.len())
        .max()
        .unwrap_or(0)
        .max(RESOURCE_MESSAGE_BYTES)
        .max("invalid lock reference `".len() + excerpt_bytes + "`".len())
        .max("invalid catalog local name `".len() + excerpt_bytes + "`".len())
        .max("profile `".len() + excerpt_bytes + "` is not certified-only".len())
        .max("external profile `".len() + excerpt_bytes + "` is not certified-only".len());
    std::mem::size_of::<ModuleDiagnosticDetailsV2>() + message + excerpt_bytes
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
                message: format!("{dimension} limit {limit} exceeded by observed value {observed}"),
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

#[cfg(test)]
mod request_storage_tests {
    use super::{
        reference_diagnostic_fixed_bytes_v3, ModuleDiagnosticDetailsV2, ModuleDiagnosticV2,
    };

    #[test]
    fn reference_diagnostic_carrier_covers_longest_resource_message() {
        let diagnostic = ModuleDiagnosticV2::resource_exhausted(
            "authority_string_bytes",
            usize::MAX,
            usize::MAX,
        );
        assert!(
            std::mem::size_of::<ModuleDiagnosticDetailsV2>() + diagnostic.details.message.len()
                <= reference_diagnostic_fixed_bytes_v3()
        );
    }
}
