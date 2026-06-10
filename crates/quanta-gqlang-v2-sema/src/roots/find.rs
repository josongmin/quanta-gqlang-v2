use quanta_gqlang_v2_syntax::roots::find::{
    parse_find_seed_v1, FindSeedSyntaxErrorV1, FindSubkindV1, ParsedFindRootSeedV1,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedFindRootV1 {
    pub subkind: String,
    pub seed: String,
}

impl TypedFindRootV1 {
    pub fn normalized_subkind(&self) -> Result<FindSubkindV1, FindRootSemaErrorV1> {
        parse_find_subkind_name(&self.subkind)
    }

    pub fn typed_spec(&self) -> Result<TypedFindSpecV1, FindRootSemaErrorV1> {
        let subkind = self.normalized_subkind()?;
        let parsed = parse_find_seed_v1(&subkind, &self.seed)?;
        parsed.try_into()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FindRootSemaErrorV1 {
    pub message: String,
}

impl From<FindSeedSyntaxErrorV1> for FindRootSemaErrorV1 {
    fn from(value: FindSeedSyntaxErrorV1) -> Self {
        Self {
            message: value.message,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypedFindSpecV1 {
    Reachability(TypedReachabilityFindRootV1),
    Flow(TypedFlowFindRootV1),
    Slice(TypedSliceFindRootV1),
    Taint(TypedTaintFindRootV1),
    Path(TypedPathFindRootV1),
    PointsTo(TypedPointsToFindRootV1),
    Summary(TypedSummaryFindRootV1),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedReachabilityFindRootV1 {
    pub source: String,
    pub sink: String,
    pub via: Vec<String>,
    pub max_depth: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedFlowFindRootV1 {
    pub source: String,
    pub sink: String,
    pub through: Vec<String>,
    pub window: Option<FindFlowWindowV1>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedSliceFindRootV1 {
    pub anchor: String,
    pub direction: FindSliceDirectionV1,
    pub frontier: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedTaintFindRootV1 {
    pub source: String,
    pub sink: String,
    pub sanitizers: Vec<String>,
    pub max_depth: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedPathFindRootV1 {
    pub source: String,
    pub sink: String,
    pub through: Vec<String>,
    pub max_depth: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedPointsToFindRootV1 {
    pub reference: String,
    pub pointee: String,
    pub scope: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedSummaryFindRootV1 {
    pub callable: String,
    pub include: Vec<FindSummaryFacetV1>,
    pub max_depth: Option<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FindFlowWindowV1 {
    Node,
    Block,
    Function,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FindSliceDirectionV1 {
    Forward,
    Backward,
    Bidirectional,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FindSummaryFacetV1 {
    Parameters,
    Returns,
    Reads,
    Writes,
    Effects,
}

impl TryFrom<ParsedFindRootSeedV1> for TypedFindSpecV1 {
    type Error = FindRootSemaErrorV1;

    fn try_from(value: ParsedFindRootSeedV1) -> Result<Self, Self::Error> {
        match value {
            ParsedFindRootSeedV1::Reachability(spec) => {
                Ok(Self::Reachability(TypedReachabilityFindRootV1 {
                    source: spec.source,
                    sink: spec.sink,
                    via: spec.via,
                    max_depth: parse_positive_u32(spec.depth.as_deref(), "depth")?,
                }))
            }
            ParsedFindRootSeedV1::Flow(spec) => Ok(Self::Flow(TypedFlowFindRootV1 {
                source: spec.source,
                sink: spec.sink,
                through: spec.through,
                window: parse_flow_window(spec.window.as_deref())?,
            })),
            ParsedFindRootSeedV1::Slice(spec) => Ok(Self::Slice(TypedSliceFindRootV1 {
                anchor: spec.anchor,
                direction: parse_slice_direction(spec.direction.as_deref())?,
                frontier: spec.frontier,
            })),
            ParsedFindRootSeedV1::Taint(spec) => Ok(Self::Taint(TypedTaintFindRootV1 {
                source: spec.source,
                sink: spec.sink,
                sanitizers: spec.sanitizers,
                max_depth: parse_positive_u32(spec.depth.as_deref(), "depth")?,
            })),
            ParsedFindRootSeedV1::Path(spec) => Ok(Self::Path(TypedPathFindRootV1 {
                source: spec.source,
                sink: spec.sink,
                through: spec.through,
                max_depth: parse_positive_u32(spec.depth.as_deref(), "depth")?,
            })),
            ParsedFindRootSeedV1::PointsTo(spec) => Ok(Self::PointsTo(TypedPointsToFindRootV1 {
                reference: spec.reference,
                pointee: spec.pointee,
                scope: spec.scope,
            })),
            ParsedFindRootSeedV1::Summary(spec) => Ok(Self::Summary(TypedSummaryFindRootV1 {
                callable: spec.callable,
                include: parse_summary_facets(&spec.include)?,
                max_depth: parse_positive_u32(spec.depth.as_deref(), "depth")?,
            })),
        }
    }
}

fn parse_find_subkind_name(raw: &str) -> Result<FindSubkindV1, FindRootSemaErrorV1> {
    match normalize_token(raw).as_str() {
        "reachability" => Ok(FindSubkindV1::Reachability),
        "flow" => Ok(FindSubkindV1::Flow),
        "slice" => Ok(FindSubkindV1::Slice),
        "taint" => Ok(FindSubkindV1::Taint),
        "path" => Ok(FindSubkindV1::Path),
        "pointsto" => Ok(FindSubkindV1::PointsTo),
        "summary" => Ok(FindSubkindV1::Summary),
        _ => Err(FindRootSemaErrorV1 {
            message: format!("unknown FIND subkind `{raw}`"),
        }),
    }
}

fn parse_positive_u32(
    raw_value: Option<&str>,
    field: &str,
) -> Result<Option<u32>, FindRootSemaErrorV1> {
    let Some(raw_value) = raw_value else {
        return Ok(None);
    };

    let parsed = raw_value.parse::<u32>().map_err(|err| FindRootSemaErrorV1 {
        message: format!("`{field}` must be a positive u32, got `{raw_value}`: {err}"),
    })?;
    if parsed == 0 {
        return Err(FindRootSemaErrorV1 {
            message: format!("`{field}` must be greater than zero"),
        });
    }
    Ok(Some(parsed))
}

fn parse_flow_window(
    raw_value: Option<&str>,
) -> Result<Option<FindFlowWindowV1>, FindRootSemaErrorV1> {
    let Some(raw_value) = raw_value else {
        return Ok(None);
    };

    match normalize_token(raw_value).as_str() {
        "node" => Ok(Some(FindFlowWindowV1::Node)),
        "block" => Ok(Some(FindFlowWindowV1::Block)),
        "function" => Ok(Some(FindFlowWindowV1::Function)),
        _ => Err(FindRootSemaErrorV1 {
            message: format!("unknown flow window `{raw_value}`"),
        }),
    }
}

fn parse_slice_direction(
    raw_value: Option<&str>,
) -> Result<FindSliceDirectionV1, FindRootSemaErrorV1> {
    let Some(raw_value) = raw_value else {
        return Ok(FindSliceDirectionV1::Bidirectional);
    };

    match normalize_token(raw_value).as_str() {
        "forward" => Ok(FindSliceDirectionV1::Forward),
        "backward" => Ok(FindSliceDirectionV1::Backward),
        "bidirectional" => Ok(FindSliceDirectionV1::Bidirectional),
        _ => Err(FindRootSemaErrorV1 {
            message: format!("unknown slice direction `{raw_value}`"),
        }),
    }
}

fn parse_summary_facets(
    raw_values: &[String],
) -> Result<Vec<FindSummaryFacetV1>, FindRootSemaErrorV1> {
    let mut facets = Vec::with_capacity(raw_values.len());
    for raw_value in raw_values {
        let facet = match normalize_token(raw_value).as_str() {
            "params" | "parameters" => FindSummaryFacetV1::Parameters,
            "returns" => FindSummaryFacetV1::Returns,
            "reads" => FindSummaryFacetV1::Reads,
            "writes" => FindSummaryFacetV1::Writes,
            "effects" => FindSummaryFacetV1::Effects,
            _ => {
                return Err(FindRootSemaErrorV1 {
                    message: format!("unknown summary facet `{raw_value}`"),
                });
            }
        };
        if facets.contains(&facet) {
            return Err(FindRootSemaErrorV1 {
                message: format!("summary facet `{raw_value}` is duplicated"),
            });
        }
        facets.push(facet);
    }
    Ok(facets)
}

fn normalize_token(raw: &str) -> String {
    raw.chars()
        .filter(|ch| !ch.is_whitespace() && *ch != '_' && *ch != '-')
        .flat_map(char::to_lowercase)
        .collect()
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::expect_used,
        reason = "FIND sema tests assert validated seed analysis success"
    )]

    use super::{
        FindFlowWindowV1, FindSliceDirectionV1, FindSummaryFacetV1, TypedFindRootV1,
        TypedFindSpecV1,
    };

    #[test]
    fn normalizes_find_subkind_matrix() {
        let reachability = TypedFindRootV1 {
            subkind: "Reachability".to_string(),
            seed: "api.entry -> db.write | via=auth,repo | depth=3".to_string(),
        }
        .typed_spec()
        .expect("reachability");
        assert_eq!(
            reachability,
            TypedFindSpecV1::Reachability(super::TypedReachabilityFindRootV1 {
                source: "api.entry".to_string(),
                sink: "db.write".to_string(),
                via: vec!["auth".to_string(), "repo".to_string()],
                max_depth: Some(3),
            })
        );

        let flow = TypedFindRootV1 {
            subkind: "flow".to_string(),
            seed: "ingress -> egress | through=ssa | window=block".to_string(),
        }
        .typed_spec()
        .expect("flow");
        assert_eq!(
            flow,
            TypedFindSpecV1::Flow(super::TypedFlowFindRootV1 {
                source: "ingress".to_string(),
                sink: "egress".to_string(),
                through: vec!["ssa".to_string()],
                window: Some(FindFlowWindowV1::Block),
            })
        );

        let slice = TypedFindRootV1 {
            subkind: "slice".to_string(),
            seed: "callsite.foo | direction=backward | frontier=cfg".to_string(),
        }
        .typed_spec()
        .expect("slice");
        assert_eq!(
            slice,
            TypedFindSpecV1::Slice(super::TypedSliceFindRootV1 {
                anchor: "callsite.foo".to_string(),
                direction: FindSliceDirectionV1::Backward,
                frontier: Some("cfg".to_string()),
            })
        );

        let taint = TypedFindRootV1 {
            subkind: "taint".to_string(),
            seed: "user.input -> shell.exec | sanitizers=escape | depth=4".to_string(),
        }
        .typed_spec()
        .expect("taint");
        assert!(matches!(taint, TypedFindSpecV1::Taint(_)));

        let path = TypedFindRootV1 {
            subkind: "path".to_string(),
            seed: "handler -> sink | through=router,middleware | depth=6".to_string(),
        }
        .typed_spec()
        .expect("path");
        assert!(matches!(path, TypedFindSpecV1::Path(_)));

        let points_to = TypedFindRootV1 {
            subkind: "points_to".to_string(),
            seed: "ptr => heap.obj | scope=fn:main".to_string(),
        }
        .typed_spec()
        .expect("points_to");
        assert!(matches!(points_to, TypedFindSpecV1::PointsTo(_)));

        let summary = TypedFindRootV1 {
            subkind: "summary".to_string(),
            seed: "pkg::open | include=params,returns,effects | depth=2".to_string(),
        }
        .typed_spec()
        .expect("summary");
        assert_eq!(
            summary,
            TypedFindSpecV1::Summary(super::TypedSummaryFindRootV1 {
                callable: "pkg::open".to_string(),
                include: vec![
                    FindSummaryFacetV1::Parameters,
                    FindSummaryFacetV1::Returns,
                    FindSummaryFacetV1::Effects,
                ],
                max_depth: Some(2),
            })
        );
    }

    #[test]
    fn rejects_invalid_depths() {
        let error = TypedFindRootV1 {
            subkind: "path".to_string(),
            seed: "handler -> sink | depth=0".to_string(),
        }
        .typed_spec()
        .expect_err("depth zero must fail");
        assert_eq!(error.message, "`depth` must be greater than zero");
    }
}
