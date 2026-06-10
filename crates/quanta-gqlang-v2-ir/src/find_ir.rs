use quanta_gqlang_v2_sema::roots::find::{
    FindFlowWindowV1, FindRootSemaErrorV1, FindSliceDirectionV1, FindSummaryFacetV1,
    TypedFindRootV1, TypedFindSpecV1,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FindIrV1 {
    // The FIND execution plan is parsed + validated ONCE, at lowering, and stored
    // structured here. `execution_plan()` returns it without re-parsing the seed
    // on every access (the audit's SMELL-1).
    plan: FindExecutionPlanIrV1,
}

impl FindIrV1 {
    /// Parse + validate the FIND seed ONCE (called from lowering). Fails closed
    /// on an unsupported subkind / malformed seed; the plan is then stored.
    pub fn from_typed_v1(subkind: &str, seed: &str) -> Result<Self, FindIrValidationErrorV1> {
        let typed_root = TypedFindRootV1 {
            subkind: subkind.to_string(),
            seed: seed.to_string(),
        };
        Ok(Self {
            plan: typed_root.typed_spec()?.into(),
        })
    }

    /// The validated FIND execution plan. Infallible — validated at lowering.
    #[must_use]
    pub fn execution_plan(&self) -> &FindExecutionPlanIrV1 {
        &self.plan
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FindIrValidationErrorV1 {
    pub message: String,
}

impl From<FindRootSemaErrorV1> for FindIrValidationErrorV1 {
    fn from(value: FindRootSemaErrorV1) -> Self {
        Self {
            message: value.message,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FindExecutionPlanIrV1 {
    Reachability(ReachabilityPlanIrV1),
    Flow(FlowPlanIrV1),
    Slice(SlicePlanIrV1),
    Taint(TaintPlanIrV1),
    Path(PathPlanIrV1),
    PointsTo(PointsToPlanIrV1),
    Summary(SummaryPlanIrV1),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReachabilityPlanIrV1 {
    pub source: String,
    pub sink: String,
    pub required_waypoints: Vec<String>,
    pub max_hops: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FlowPlanIrV1 {
    pub source: String,
    pub sink: String,
    pub stages: Vec<String>,
    pub window: FindFlowWindowIrV1,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SlicePlanIrV1 {
    pub anchor: String,
    pub direction: FindSliceDirectionIrV1,
    pub frontier: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaintPlanIrV1 {
    pub source: String,
    pub sink: String,
    pub blocked_by: Vec<String>,
    pub max_hops: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PathPlanIrV1 {
    pub source: String,
    pub sink: String,
    pub required_waypoints: Vec<String>,
    pub max_hops: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PointsToPlanIrV1 {
    pub reference: String,
    pub expected_pointee: String,
    pub scope: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SummaryPlanIrV1 {
    pub callable: String,
    pub requested_facets: Vec<FindSummaryFacetIrV1>,
    pub expansion_depth: Option<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FindFlowWindowIrV1 {
    Node,
    Block,
    Function,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FindSliceDirectionIrV1 {
    Forward,
    Backward,
    Bidirectional,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FindSummaryFacetIrV1 {
    Parameters,
    Returns,
    Reads,
    Writes,
    Effects,
}

impl From<TypedFindSpecV1> for FindExecutionPlanIrV1 {
    fn from(value: TypedFindSpecV1) -> Self {
        match value {
            TypedFindSpecV1::Reachability(spec) => Self::Reachability(ReachabilityPlanIrV1 {
                source: spec.source,
                sink: spec.sink,
                required_waypoints: spec.via,
                max_hops: spec.max_depth,
            }),
            TypedFindSpecV1::Flow(spec) => Self::Flow(FlowPlanIrV1 {
                source: spec.source,
                sink: spec.sink,
                stages: spec.through,
                window: spec.window.map_or(FindFlowWindowIrV1::Function, Into::into),
            }),
            TypedFindSpecV1::Slice(spec) => Self::Slice(SlicePlanIrV1 {
                anchor: spec.anchor,
                direction: spec.direction.into(),
                frontier: spec.frontier,
            }),
            TypedFindSpecV1::Taint(spec) => Self::Taint(TaintPlanIrV1 {
                source: spec.source,
                sink: spec.sink,
                blocked_by: spec.sanitizers,
                max_hops: spec.max_depth,
            }),
            TypedFindSpecV1::Path(spec) => Self::Path(PathPlanIrV1 {
                source: spec.source,
                sink: spec.sink,
                required_waypoints: spec.through,
                max_hops: spec.max_depth,
            }),
            TypedFindSpecV1::PointsTo(spec) => Self::PointsTo(PointsToPlanIrV1 {
                reference: spec.reference,
                expected_pointee: spec.pointee,
                scope: spec.scope,
            }),
            TypedFindSpecV1::Summary(spec) => Self::Summary(SummaryPlanIrV1 {
                callable: spec.callable,
                requested_facets: spec.include.into_iter().map(Into::into).collect(),
                expansion_depth: spec.max_depth,
            }),
        }
    }
}

impl From<FindFlowWindowV1> for FindFlowWindowIrV1 {
    fn from(value: FindFlowWindowV1) -> Self {
        match value {
            FindFlowWindowV1::Node => Self::Node,
            FindFlowWindowV1::Block => Self::Block,
            FindFlowWindowV1::Function => Self::Function,
        }
    }
}

impl From<FindSliceDirectionV1> for FindSliceDirectionIrV1 {
    fn from(value: FindSliceDirectionV1) -> Self {
        match value {
            FindSliceDirectionV1::Forward => Self::Forward,
            FindSliceDirectionV1::Backward => Self::Backward,
            FindSliceDirectionV1::Bidirectional => Self::Bidirectional,
        }
    }
}

impl From<FindSummaryFacetV1> for FindSummaryFacetIrV1 {
    fn from(value: FindSummaryFacetV1) -> Self {
        match value {
            FindSummaryFacetV1::Parameters => Self::Parameters,
            FindSummaryFacetV1::Returns => Self::Returns,
            FindSummaryFacetV1::Reads => Self::Reads,
            FindSummaryFacetV1::Writes => Self::Writes,
            FindSummaryFacetV1::Effects => Self::Effects,
        }
    }
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::expect_used,
        reason = "FIND IR tests assert validated plan decoding success"
    )]

    use super::{FindExecutionPlanIrV1, FindFlowWindowIrV1, FindIrV1, SummaryPlanIrV1};

    #[test]
    fn lowers_find_matrix_into_execution_plans() {
        let reachability = FindIrV1::from_typed_v1("reachability", "api.entry -> db.write | via=auth,repo | depth=3").expect("reachability").execution_plan().clone();
        assert!(matches!(
            reachability,
            FindExecutionPlanIrV1::Reachability(_)
        ));

        let flow = FindIrV1::from_typed_v1("flow", "ingress -> egress | through=ssa").expect("flow").execution_plan().clone();
        assert_eq!(
            flow,
            FindExecutionPlanIrV1::Flow(super::FlowPlanIrV1 {
                source: "ingress".to_string(),
                sink: "egress".to_string(),
                stages: vec!["ssa".to_string()],
                window: FindFlowWindowIrV1::Function,
            })
        );

        let slice = FindIrV1::from_typed_v1("slice", "callsite.foo | direction=forward | frontier=cfg").expect("slice").execution_plan().clone();
        assert!(matches!(slice, FindExecutionPlanIrV1::Slice(_)));

        let taint = FindIrV1::from_typed_v1("taint", "user.input -> shell.exec | sanitizers=escape | depth=4").expect("taint").execution_plan().clone();
        assert!(matches!(taint, FindExecutionPlanIrV1::Taint(_)));

        let path = FindIrV1::from_typed_v1("path", "handler -> sink | through=router,middleware | depth=6").expect("path").execution_plan().clone();
        assert!(matches!(path, FindExecutionPlanIrV1::Path(_)));

        let points_to = FindIrV1::from_typed_v1("points_to", "ptr => heap.obj | scope=fn:main").expect("points_to").execution_plan().clone();
        assert!(matches!(points_to, FindExecutionPlanIrV1::PointsTo(_)));

        let summary = FindIrV1::from_typed_v1("summary", "pkg::open | include=params,returns | depth=2").expect("summary").execution_plan().clone();
        assert_eq!(
            summary,
            FindExecutionPlanIrV1::Summary(SummaryPlanIrV1 {
                callable: "pkg::open".to_string(),
                requested_facets: vec![
                    super::FindSummaryFacetIrV1::Parameters,
                    super::FindSummaryFacetIrV1::Returns,
                ],
                expansion_depth: Some(2),
            })
        );
    }

    #[test]
    fn propagates_find_validation_errors() {
        // The invalid window now fails closed at construction (lowering), where
        // the seed is parsed once.
        let error = FindIrV1::from_typed_v1("flow", "ingress -> egress | window=kernel")
            .expect_err("invalid window must fail");
        assert_eq!(error.message, "unknown flow window `kernel`");
    }
}
