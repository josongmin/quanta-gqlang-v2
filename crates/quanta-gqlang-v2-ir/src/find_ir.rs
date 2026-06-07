use quanta_gqlang_v2_sema::roots::find::{
    FindFlowWindowV1, FindRootSemaErrorV1, FindSliceDirectionV1, FindSummaryFacetV1,
    TypedFindRootV1, TypedFindSpecV1,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FindIrV1 {
    pub subkind: String,
    pub seed: String,
}

impl FindIrV1 {
    pub fn execution_plan(&self) -> Result<FindExecutionPlanIrV1, FindIrValidationErrorV1> {
        let typed_root = TypedFindRootV1 {
            subkind: self.subkind.clone(),
            seed: self.seed.clone(),
        };
        let typed_spec = typed_root.typed_spec()?;
        Ok(typed_spec.into())
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
    use super::{FindExecutionPlanIrV1, FindFlowWindowIrV1, FindIrV1, SummaryPlanIrV1};

    #[test]
    fn lowers_find_matrix_into_execution_plans() {
        let reachability = FindIrV1 {
            subkind: "reachability".to_string(),
            seed: "api.entry -> db.write | via=auth,repo | depth=3".to_string(),
        }
        .execution_plan()
        .expect("reachability");
        assert!(matches!(
            reachability,
            FindExecutionPlanIrV1::Reachability(_)
        ));

        let flow = FindIrV1 {
            subkind: "flow".to_string(),
            seed: "ingress -> egress | through=ssa".to_string(),
        }
        .execution_plan()
        .expect("flow");
        assert_eq!(
            flow,
            FindExecutionPlanIrV1::Flow(super::FlowPlanIrV1 {
                source: "ingress".to_string(),
                sink: "egress".to_string(),
                stages: vec!["ssa".to_string()],
                window: FindFlowWindowIrV1::Function,
            })
        );

        let slice = FindIrV1 {
            subkind: "slice".to_string(),
            seed: "callsite.foo | direction=forward | frontier=cfg".to_string(),
        }
        .execution_plan()
        .expect("slice");
        assert!(matches!(slice, FindExecutionPlanIrV1::Slice(_)));

        let taint = FindIrV1 {
            subkind: "taint".to_string(),
            seed: "user.input -> shell.exec | sanitizers=escape | depth=4".to_string(),
        }
        .execution_plan()
        .expect("taint");
        assert!(matches!(taint, FindExecutionPlanIrV1::Taint(_)));

        let path = FindIrV1 {
            subkind: "path".to_string(),
            seed: "handler -> sink | through=router,middleware | depth=6".to_string(),
        }
        .execution_plan()
        .expect("path");
        assert!(matches!(path, FindExecutionPlanIrV1::Path(_)));

        let points_to = FindIrV1 {
            subkind: "points_to".to_string(),
            seed: "ptr => heap.obj | scope=fn:main".to_string(),
        }
        .execution_plan()
        .expect("points_to");
        assert!(matches!(points_to, FindExecutionPlanIrV1::PointsTo(_)));

        let summary = FindIrV1 {
            subkind: "summary".to_string(),
            seed: "pkg::open | include=params,returns | depth=2".to_string(),
        }
        .execution_plan()
        .expect("summary");
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
        let error = FindIrV1 {
            subkind: "flow".to_string(),
            seed: "ingress -> egress | window=kernel".to_string(),
        }
        .execution_plan()
        .expect_err("invalid window must fail");
        assert_eq!(error.message, "unknown flow window `kernel`");
    }
}
