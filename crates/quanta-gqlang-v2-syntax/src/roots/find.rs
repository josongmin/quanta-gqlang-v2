use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FindSubkindV1 {
    Reachability,
    Flow,
    Slice,
    Taint,
    Path,
    PointsTo,
    Summary,
}

impl FindSubkindV1 {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Reachability => "reachability",
            Self::Flow => "flow",
            Self::Slice => "slice",
            Self::Taint => "taint",
            Self::Path => "path",
            Self::PointsTo => "points_to",
            Self::Summary => "summary",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FindRootAstV1 {
    pub subkind: FindSubkindV1,
    pub seed: String,
}

impl FindRootAstV1 {
    pub fn parse_seed(&self) -> Result<ParsedFindRootSeedV1, FindSeedSyntaxErrorV1> {
        parse_find_seed_v1(&self.subkind, &self.seed)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FindSeedSyntaxErrorV1 {
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParsedFindRootSeedV1 {
    Reachability(FindReachabilitySeedV1),
    Flow(FindFlowSeedV1),
    Slice(FindSliceSeedV1),
    Taint(FindTaintSeedV1),
    Path(FindPathSeedV1),
    PointsTo(FindPointsToSeedV1),
    Summary(FindSummarySeedV1),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FindReachabilitySeedV1 {
    pub source: String,
    pub sink: String,
    pub via: Vec<String>,
    pub depth: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FindFlowSeedV1 {
    pub source: String,
    pub sink: String,
    pub through: Vec<String>,
    pub window: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FindSliceSeedV1 {
    pub anchor: String,
    pub direction: Option<String>,
    pub frontier: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FindTaintSeedV1 {
    pub source: String,
    pub sink: String,
    pub sanitizers: Vec<String>,
    pub depth: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FindPathSeedV1 {
    pub source: String,
    pub sink: String,
    pub through: Vec<String>,
    pub depth: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FindPointsToSeedV1 {
    pub reference: String,
    pub pointee: String,
    pub scope: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FindSummarySeedV1 {
    pub callable: String,
    pub include: Vec<String>,
    pub depth: Option<String>,
}

pub fn parse_find_seed_v1(
    subkind: &FindSubkindV1,
    seed: &str,
) -> Result<ParsedFindRootSeedV1, FindSeedSyntaxErrorV1> {
    let (head, directives) = split_seed_segments(seed)?;
    match subkind {
        FindSubkindV1::Reachability => {
            reject_unknown_directives(subkind, &directives, &["via", "depth"])?;
            let (source, sink) = split_relation(&head, "->", subkind)?;
            Ok(ParsedFindRootSeedV1::Reachability(FindReachabilitySeedV1 {
                source,
                sink,
                via: parse_csv_directive(&directives, "via")?,
                depth: read_scalar_directive(&directives, "depth"),
            }))
        }
        FindSubkindV1::Flow => {
            reject_unknown_directives(subkind, &directives, &["through", "window"])?;
            let (source, sink) = split_relation(&head, "->", subkind)?;
            Ok(ParsedFindRootSeedV1::Flow(FindFlowSeedV1 {
                source,
                sink,
                through: parse_csv_directive(&directives, "through")?,
                window: read_scalar_directive(&directives, "window"),
            }))
        }
        FindSubkindV1::Slice => {
            reject_unknown_directives(subkind, &directives, &["direction", "frontier"])?;
            Ok(ParsedFindRootSeedV1::Slice(FindSliceSeedV1 {
                anchor: validate_single_anchor(&head, subkind)?,
                direction: read_scalar_directive(&directives, "direction"),
                frontier: read_scalar_directive(&directives, "frontier"),
            }))
        }
        FindSubkindV1::Taint => {
            reject_unknown_directives(subkind, &directives, &["sanitizers", "depth"])?;
            let (source, sink) = split_relation(&head, "->", subkind)?;
            Ok(ParsedFindRootSeedV1::Taint(FindTaintSeedV1 {
                source,
                sink,
                sanitizers: parse_csv_directive(&directives, "sanitizers")?,
                depth: read_scalar_directive(&directives, "depth"),
            }))
        }
        FindSubkindV1::Path => {
            reject_unknown_directives(subkind, &directives, &["through", "depth"])?;
            let (source, sink) = split_relation(&head, "->", subkind)?;
            Ok(ParsedFindRootSeedV1::Path(FindPathSeedV1 {
                source,
                sink,
                through: parse_csv_directive(&directives, "through")?,
                depth: read_scalar_directive(&directives, "depth"),
            }))
        }
        FindSubkindV1::PointsTo => {
            reject_unknown_directives(subkind, &directives, &["scope"])?;
            let (reference, pointee) = split_relation(&head, "=>", subkind)?;
            Ok(ParsedFindRootSeedV1::PointsTo(FindPointsToSeedV1 {
                reference,
                pointee,
                scope: read_scalar_directive(&directives, "scope"),
            }))
        }
        FindSubkindV1::Summary => {
            reject_unknown_directives(subkind, &directives, &["include", "depth"])?;
            Ok(ParsedFindRootSeedV1::Summary(FindSummarySeedV1 {
                callable: validate_single_anchor(&head, subkind)?,
                include: parse_csv_directive(&directives, "include")?,
                depth: read_scalar_directive(&directives, "depth"),
            }))
        }
    }
}

fn split_seed_segments(
    seed: &str,
) -> Result<(String, BTreeMap<String, String>), FindSeedSyntaxErrorV1> {
    let trimmed = seed.trim();
    if trimmed.is_empty() {
        return Err(FindSeedSyntaxErrorV1 {
            message: "find seed must not be empty".to_string(),
        });
    }

    let mut segments = trimmed.split('|').map(str::trim);
    let head = segments
        .next()
        .ok_or_else(|| FindSeedSyntaxErrorV1 {
            message: "find seed must contain a head segment".to_string(),
        })?
        .to_string();
    if head.is_empty() {
        return Err(FindSeedSyntaxErrorV1 {
            message: "find seed head must not be empty".to_string(),
        });
    }

    let mut directives = BTreeMap::new();
    for segment in segments {
        if segment.is_empty() {
            return Err(FindSeedSyntaxErrorV1 {
                message: "find seed contains an empty directive segment".to_string(),
            });
        }
        let (raw_key, raw_value) =
            segment
                .split_once('=')
                .ok_or_else(|| FindSeedSyntaxErrorV1 {
                    message: format!("directive `{segment}` must use key=value form"),
                })?;
        let key = raw_key.trim();
        let value = raw_value.trim();
        if key.is_empty() || value.is_empty() {
            return Err(FindSeedSyntaxErrorV1 {
                message: format!("directive `{segment}` must have non-empty key and value"),
            });
        }
        if directives
            .insert(key.to_string(), value.to_string())
            .is_some()
        {
            return Err(FindSeedSyntaxErrorV1 {
                message: format!("directive `{key}` is duplicated"),
            });
        }
    }

    Ok((head, directives))
}

fn reject_unknown_directives(
    subkind: &FindSubkindV1,
    directives: &BTreeMap<String, String>,
    allowed: &[&str],
) -> Result<(), FindSeedSyntaxErrorV1> {
    for key in directives.keys() {
        if !allowed.iter().any(|candidate| candidate == &key.as_str()) {
            return Err(FindSeedSyntaxErrorV1 {
                message: format!(
                    "directive `{key}` is not allowed for `{}`",
                    subkind.as_str()
                ),
            });
        }
    }
    Ok(())
}

fn split_relation(
    head: &str,
    separator: &str,
    subkind: &FindSubkindV1,
) -> Result<(String, String), FindSeedSyntaxErrorV1> {
    let (lhs, rhs) = head
        .split_once(separator)
        .ok_or_else(|| FindSeedSyntaxErrorV1 {
            message: format!(
                "`{}` seeds must use `{separator}` between endpoints",
                subkind.as_str()
            ),
        })?;
    if rhs.contains(separator) {
        return Err(FindSeedSyntaxErrorV1 {
            message: format!(
                "`{}` seeds may contain exactly one `{separator}` relation",
                subkind.as_str()
            ),
        });
    }

    let left = lhs.trim();
    let right = rhs.trim();
    if left.is_empty() || right.is_empty() {
        return Err(FindSeedSyntaxErrorV1 {
            message: format!(
                "`{}` seeds require non-empty endpoints around `{separator}`",
                subkind.as_str()
            ),
        });
    }

    Ok((left.to_string(), right.to_string()))
}

fn validate_single_anchor(
    head: &str,
    subkind: &FindSubkindV1,
) -> Result<String, FindSeedSyntaxErrorV1> {
    if head.contains("->") || head.contains("=>") {
        return Err(FindSeedSyntaxErrorV1 {
            message: format!(
                "`{}` seeds accept a single anchor, not a relation",
                subkind.as_str()
            ),
        });
    }

    Ok(head.trim().to_string())
}

fn read_scalar_directive(directives: &BTreeMap<String, String>, key: &str) -> Option<String> {
    directives.get(key).cloned()
}

fn parse_csv_directive(
    directives: &BTreeMap<String, String>,
    key: &str,
) -> Result<Vec<String>, FindSeedSyntaxErrorV1> {
    let Some(raw_value) = directives.get(key) else {
        return Ok(Vec::new());
    };

    raw_value
        .split(',')
        .map(str::trim)
        .map(|item| {
            if item.is_empty() {
                Err(FindSeedSyntaxErrorV1 {
                    message: format!("directive `{key}` contains an empty list item"),
                })
            } else {
                Ok(item.to_string())
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::expect_used,
        reason = "FIND syntax tests assert seed decoding success for validated fixtures"
    )]

    use super::{parse_find_seed_v1, FindRootAstV1, FindSubkindV1, ParsedFindRootSeedV1};

    #[test]
    fn parses_find_subkind_matrix() {
        let reachability = parse_find_seed_v1(
            &FindSubkindV1::Reachability,
            "api.entry -> db.write | via=auth,repo | depth=3",
        )
        .expect("reachability");
        assert_eq!(
            reachability,
            ParsedFindRootSeedV1::Reachability(super::FindReachabilitySeedV1 {
                source: "api.entry".to_string(),
                sink: "db.write".to_string(),
                via: vec!["auth".to_string(), "repo".to_string()],
                depth: Some("3".to_string()),
            })
        );

        let flow = parse_find_seed_v1(
            &FindSubkindV1::Flow,
            "ingress -> egress | through=ssa | window=block",
        )
        .expect("flow");
        assert!(matches!(flow, ParsedFindRootSeedV1::Flow(_)));

        let slice = FindRootAstV1 {
            subkind: FindSubkindV1::Slice,
            seed: "callsite.foo | direction=backward | frontier=cfg".to_string(),
        }
        .parse_seed()
        .expect("slice");
        assert!(matches!(slice, ParsedFindRootSeedV1::Slice(_)));

        let taint = parse_find_seed_v1(
            &FindSubkindV1::Taint,
            "user.input -> shell.exec | sanitizers=escape,quote | depth=4",
        )
        .expect("taint");
        assert!(matches!(taint, ParsedFindRootSeedV1::Taint(_)));

        let path = parse_find_seed_v1(
            &FindSubkindV1::Path,
            "handler -> sink | through=router,middleware | depth=6",
        )
        .expect("path");
        assert!(matches!(path, ParsedFindRootSeedV1::Path(_)));

        let points_to =
            parse_find_seed_v1(&FindSubkindV1::PointsTo, "ptr => heap.obj | scope=fn:main")
                .expect("points_to");
        assert!(matches!(points_to, ParsedFindRootSeedV1::PointsTo(_)));

        let summary = parse_find_seed_v1(
            &FindSubkindV1::Summary,
            "pkg::open | include=params,returns",
        )
        .expect("summary");
        assert!(matches!(summary, ParsedFindRootSeedV1::Summary(_)));
    }

    #[test]
    fn rejects_unknown_directives() {
        let error = parse_find_seed_v1(&FindSubkindV1::Reachability, "api -> db | include=params")
            .expect_err("unknown directive must fail");
        assert_eq!(
            error.message,
            "directive `include` is not allowed for `reachability`"
        );
    }
}
