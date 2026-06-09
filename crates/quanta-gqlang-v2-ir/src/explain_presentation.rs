//! Shared, presentation-neutral core for the EXPLAIN/FIND front-door views.
//!
//! Both `cli` and `lsp` render an explain/witness surface over the same
//! `FindExecutionPlanIrV1`. Previously each crate carried its own copy of the
//! subject derivation and the 7-variant witness walk; the subject logic was a
//! byte-for-byte duplicate and the witness walks classified the exact same
//! variants. This module is the single source of truth for the parts that are
//! genuinely shared:
//!
//!   * [`explain_find_subject_v1`] — the identical "subject" string both crates
//!     surfaced (`source -> sink` for the path-shaped plans, the anchor /
//!     reference / callable otherwise).
//!   * [`explain_find_witness_v1`] — the single 7-variant classification,
//!     producing a NEUTRAL [`WitnessModelV1`]: a `kind` tag plus an ordered list
//!     of [`WitnessStepV1`]s. Each step carries a bare-stem `label` and a
//!     structural [`WitnessDetailV1`]. The detail is intentionally NOT a
//!     pre-formatted string for the two fields the crates render differently
//!     (slice direction casing, summary facet count-vs-list): it carries the raw
//!     enum/list so each crate formats losslessly without re-walking the plan.
//!
//! Each crate adapts this model into its OWN final representation (the CLI into
//! its indexed `PathWitnessViewV1`, the LSP into its `label=detail` preview
//! lines), so crate-specific presentation stays in each crate while the variant
//! walk lives here exactly once. Degraded-badge label maps are NOT hoisted: those
//! strings differ per crate by design and remain local.

use crate::find_ir::{
    FindExecutionPlanIrV1, FindFlowWindowIrV1, FindSliceDirectionIrV1, FindSummaryFacetIrV1,
};

/// The structural value of a witness step, kept neutral so each crate formats it.
/// Most steps are plain [`WitnessDetailV1::Text`]; the slice direction and the
/// summary facets are carried as raw enum/list because the CLI and LSP render
/// them differently (lowercase label vs debug casing; joined labels vs count).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WitnessDetailV1 {
    Text(String),
    Direction(FindSliceDirectionIrV1),
    Facets(Vec<FindSummaryFacetIrV1>),
}

/// One neutral step in a FIND witness. `label` is the bare stem (e.g. `source`,
/// `via`, `direction`); `detail` is the structural value. Crates attach their own
/// index suffixes / formatting when rendering.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WitnessStepV1 {
    pub label: String,
    pub detail: WitnessDetailV1,
}

/// A neutral, structural witness bound. Each crate decides how (and whether) to
/// surface it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WitnessBoundV1 {
    MaxHops(u32),
    Window(FindFlowWindowIrV1),
    Scope(String),
    ExpansionDepth(u32),
}

/// The neutral witness model shared by the CLI and LSP explain presenters. `kind`
/// is the witness classification (`reachability`, `flow`, ...); `steps` is the
/// ordered, presentation-neutral step list; `bound` carries the optional
/// numeric/structural bound so a crate can render its note without re-walking the
/// plan.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WitnessModelV1 {
    pub kind: &'static str,
    pub steps: Vec<WitnessStepV1>,
    pub bound: Option<WitnessBoundV1>,
}

/// The subject string both front doors surface for a FIND plan. Byte-identical to
/// the previously-forked `find_subject_v1` (CLI) / `find_title_v1` (LSP).
#[must_use]
pub fn explain_find_subject_v1(plan: &FindExecutionPlanIrV1) -> String {
    match plan {
        FindExecutionPlanIrV1::Reachability(spec) => format!("{} -> {}", spec.source, spec.sink),
        FindExecutionPlanIrV1::Flow(spec) => format!("{} -> {}", spec.source, spec.sink),
        FindExecutionPlanIrV1::Slice(spec) => spec.anchor.clone(),
        FindExecutionPlanIrV1::Taint(spec) => format!("{} -> {}", spec.source, spec.sink),
        FindExecutionPlanIrV1::Path(spec) => format!("{} -> {}", spec.source, spec.sink),
        FindExecutionPlanIrV1::PointsTo(spec) => spec.reference.clone(),
        FindExecutionPlanIrV1::Summary(spec) => spec.callable.clone(),
    }
}

/// The single 7-variant witness walk. Produces a neutral [`WitnessModelV1`]; the
/// CLI and LSP each adapt this into their own final representation.
#[must_use]
pub fn explain_find_witness_v1(plan: &FindExecutionPlanIrV1) -> WitnessModelV1 {
    match plan {
        FindExecutionPlanIrV1::Reachability(spec) => WitnessModelV1 {
            kind: "reachability",
            steps: path_sequence_steps_v1(
                &spec.source,
                &spec.required_waypoints,
                &spec.sink,
                "via",
            ),
            bound: spec.max_hops.map(WitnessBoundV1::MaxHops),
        },
        FindExecutionPlanIrV1::Flow(spec) => WitnessModelV1 {
            kind: "flow",
            steps: path_sequence_steps_v1(&spec.source, &spec.stages, &spec.sink, "through"),
            bound: Some(WitnessBoundV1::Window(spec.window)),
        },
        FindExecutionPlanIrV1::Slice(spec) => WitnessModelV1 {
            kind: "slice",
            steps: vec![
                WitnessStepV1 {
                    label: "anchor".to_string(),
                    detail: WitnessDetailV1::Text(spec.anchor.clone()),
                },
                WitnessStepV1 {
                    label: "direction".to_string(),
                    detail: WitnessDetailV1::Direction(spec.direction),
                },
                WitnessStepV1 {
                    label: "frontier".to_string(),
                    detail: WitnessDetailV1::Text(
                        spec.frontier.clone().unwrap_or_else(|| "auto".to_string()),
                    ),
                },
            ],
            bound: None,
        },
        FindExecutionPlanIrV1::Taint(spec) => WitnessModelV1 {
            kind: "taint",
            steps: path_sequence_steps_v1(&spec.source, &spec.blocked_by, &spec.sink, "sanitizer"),
            bound: spec.max_hops.map(WitnessBoundV1::MaxHops),
        },
        FindExecutionPlanIrV1::Path(spec) => WitnessModelV1 {
            kind: "path",
            steps: path_sequence_steps_v1(
                &spec.source,
                &spec.required_waypoints,
                &spec.sink,
                "through",
            ),
            bound: spec.max_hops.map(WitnessBoundV1::MaxHops),
        },
        FindExecutionPlanIrV1::PointsTo(spec) => WitnessModelV1 {
            kind: "points_to",
            steps: vec![
                WitnessStepV1 {
                    label: "reference".to_string(),
                    detail: WitnessDetailV1::Text(spec.reference.clone()),
                },
                WitnessStepV1 {
                    label: "pointee".to_string(),
                    detail: WitnessDetailV1::Text(spec.expected_pointee.clone()),
                },
            ],
            bound: spec.scope.clone().map(WitnessBoundV1::Scope),
        },
        FindExecutionPlanIrV1::Summary(spec) => WitnessModelV1 {
            kind: "summary",
            steps: vec![
                WitnessStepV1 {
                    label: "callable".to_string(),
                    detail: WitnessDetailV1::Text(spec.callable.clone()),
                },
                WitnessStepV1 {
                    label: "facets".to_string(),
                    detail: WitnessDetailV1::Facets(spec.requested_facets.clone()),
                },
            ],
            bound: spec.expansion_depth.map(WitnessBoundV1::ExpansionDepth),
        },
    }
}

/// Canonical lowercase label for a slice direction (`forward` / `backward` /
/// `bidirectional`). Exposed so a crate that renders the lowercase form derives
/// it from this single authority rather than re-matching the enum.
#[must_use]
pub fn slice_direction_label_v1(direction: FindSliceDirectionIrV1) -> &'static str {
    match direction {
        FindSliceDirectionIrV1::Forward => "forward",
        FindSliceDirectionIrV1::Backward => "backward",
        FindSliceDirectionIrV1::Bidirectional => "bidirectional",
    }
}

/// Canonical label for a summary facet. Exposed so a crate that joins facet labels
/// derives them from this single authority rather than re-matching the enum.
#[must_use]
pub fn summary_facet_label_v1(facet: &FindSummaryFacetIrV1) -> &'static str {
    match facet {
        FindSummaryFacetIrV1::Parameters => "parameters",
        FindSummaryFacetIrV1::Returns => "returns",
        FindSummaryFacetIrV1::Reads => "reads",
        FindSummaryFacetIrV1::Writes => "writes",
        FindSummaryFacetIrV1::Effects => "effects",
    }
}

/// The shared `source -> intermediates -> sink` step decomposition. The
/// intermediate `label` carries the bare middle stem (e.g. `via`); crates that
/// want an index suffix attach it from the step position.
fn path_sequence_steps_v1(
    source: &str,
    intermediates: &[String],
    sink: &str,
    middle_label: &str,
) -> Vec<WitnessStepV1> {
    let mut steps = vec![WitnessStepV1 {
        label: "source".to_string(),
        detail: WitnessDetailV1::Text(source.to_string()),
    }];
    steps.extend(intermediates.iter().map(|item| WitnessStepV1 {
        label: middle_label.to_string(),
        detail: WitnessDetailV1::Text(item.clone()),
    }));
    steps.push(WitnessStepV1 {
        label: "sink".to_string(),
        detail: WitnessDetailV1::Text(sink.to_string()),
    });
    steps
}
