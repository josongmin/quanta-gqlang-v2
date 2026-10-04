//! Non-relation reference document families and forged variants.

use super::{
    alias, capture_edge_registry, document_parts, field_path, reassemble_document,
    reference_authority_v2, resolve_checked_document_v2, small_steps_receipt, CallArgHirV1,
    CallArgsHirV1, CallableKindV2, CheckedLocatedSpanV2, CheckedSyntaxDocumentV2,
    ClosedFrontendAuthorityV2, CompareMetricV1, CompositionBindingV1, CompositionBodyV1,
    CompositionHirV1, CompositionProducerV1, ConfirmModeV1, CurrentModuleSelectionInputV2,
    DetectionModeV1, DownstreamBudgetReceiptV2, EmitModeV1, EvidenceModeV1, ExplainHirV1,
    ExportFormatV1, ExpressionEffectsV2, ExternalCatalogAssetInputV2,
    ExternalCatalogPayloadInputV2, FrontendBudgetV2, GraphConstraintsV1, GraphDirectionV1,
    GraphEndpointsV1, GraphQueryHirV1, GraphQueryKindV1, GraphRefV1, GraphReferenceCatalogV2,
    GraphReferenceInputV2, GraphReferenceKindV2, GraphReferenceRoleV2, HirBuilderV1, HirDocumentV1,
    HirNodeIdV1, LegalityHirV1, LocatedParameterInputV2, LocatedSpanV2, MetaHirV1, MetaKindV1,
    ModuleAuthorityBuilderV2, ModuleIdV2, ModuleNodeInputV2, PresentationHirV1, PresentationKindV1,
    ProgramItemHirV1, ProjectionItemV1, RelationCatalogV2, ResolvedCompositionV2,
    ResolvedGraphQueryV2, ResolvedMetaKindV2, ResolvedPresentationV2, ResolvedValueBindingV2,
    SampleHirV1, SampleModeV1, SemanticDocumentV2, SemanticItemIdV2, SemanticItemV2,
    ShowCategoryV1, SignatureInputV2, SourceIdV2, SourceQualifiedNodeIdV2, SourceRegistrationV2,
    SourceRegistryViewV2, SpanV1, TruthModeV1, TypedExprShapeV2, UsePackActivationV1, UsePackHirV1,
    ValueTypeV1, VersionV2, REFERENCE_SOURCE_V2, SECONDARY_REFERENCE_SOURCE_V2,
    SMALL_BUDGET_MAX_SEMANTIC_STEPS,
};

// ---- graph fixtures (W3-B verification scaffold) ---------------------------

/// Fixed source length of every reference graph fixture document.
pub const REFERENCE_GRAPH_SOURCE_LEN_V2: usize = 4_096;

/// One requested resolved FIND reference.
#[derive(Clone, Debug)]
pub struct ReferenceGraphReferenceV2 {
    /// Semantic FIND role.
    pub role: GraphReferenceRoleV2,
    /// Host selector spelling registered for the role's required kind.
    pub selector: &'static str,
    /// Authored byte span of the reference.
    pub span: (usize, usize),
}

/// One requested resolved FIND document.
#[derive(Clone, Debug)]
pub struct ReferenceGraphRequestV2 {
    /// FIND subkind.
    pub kind: GraphQueryKindV1,
    /// Ordered references.
    pub references: Vec<ReferenceGraphReferenceV2>,
    /// Optional authored depth.
    pub depth: Option<u32>,
    /// Optional direction keyword.
    pub direction: Option<&'static str>,
    /// Summary facets.
    pub summary_facets: Vec<String>,
    /// Authored span of the FIND item.
    pub item_span: (usize, usize),
}

/// A sealed graph fixture document plus its FIND item identity.
#[derive(Clone, Debug)]
pub struct ReferenceGraphDocumentV2 {
    document: SemanticDocumentV2,
    item_id: SemanticItemIdV2,
}

impl ReferenceGraphDocumentV2 {
    /// Borrow the complete semantic document.
    #[must_use]
    pub const fn document(&self) -> &SemanticDocumentV2 {
        &self.document
    }
    /// Identity of the FIND item.
    #[must_use]
    pub const fn graph_item_id(&self) -> SemanticItemIdV2 {
        self.item_id
    }
    /// The resolved FIND item.
    #[must_use]
    #[expect(
        clippy::panic,
        reason = "the fixture seals this identity, so a non-Graph item is unreachable"
    )]
    pub fn graph(&self) -> &ResolvedGraphQueryV2 {
        match self.document.item(self.item_id) {
            Some(SemanticItemV2::Graph(graph)) => graph,
            _ => panic!("graph fixture item is a resolved FIND"),
        }
    }
}

/// A sealed nested CHECK/FIND fixture document plus both item identities.
#[derive(Clone, Debug)]
pub struct ReferenceNestedCheckDocumentV2 {
    document: SemanticDocumentV2,
    check_item_id: SemanticItemIdV2,
    nested_graph_item_id: SemanticItemIdV2,
}

/// Caller-selected CHECK shape resolved through the real semantic transaction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReferenceCheckFixtureV2 {
    /// Authored detection mode.
    pub mode: DetectionModeV1,
    /// Whether the CHECK carries a resolved pattern selector.
    pub with_pattern: bool,
    /// Whether the CHECK carries an explicit nested semantic-confirm FIND.
    pub with_nested_graph: bool,
}

/// A configurable sealed CHECK fixture and its optional nested FIND identity.
#[derive(Clone, Debug)]
pub struct ReferenceCheckDocumentV2 {
    document: SemanticDocumentV2,
    check_item_id: SemanticItemIdV2,
    nested_graph_item_id: Option<SemanticItemIdV2>,
}

impl ReferenceCheckDocumentV2 {
    /// Borrow the complete semantic document.
    #[must_use]
    pub const fn document(&self) -> &SemanticDocumentV2 {
        &self.document
    }
    /// Identity of the CHECK item.
    #[must_use]
    pub const fn check_item_id(&self) -> SemanticItemIdV2 {
        self.check_item_id
    }
    /// Identity of the nested FIND when the fixture requested one.
    #[must_use]
    pub const fn nested_graph_item_id(&self) -> Option<SemanticItemIdV2> {
        self.nested_graph_item_id
    }
    /// The resolved CHECK item.
    #[must_use]
    #[expect(
        clippy::panic,
        reason = "the fixture seals this identity, so a non-Check item is unreachable"
    )]
    pub fn check(&self) -> &crate::ResolvedCheckV2 {
        match self.document.item(self.check_item_id) {
            Some(SemanticItemV2::Check(check)) => check,
            _ => panic!("check fixture item is a resolved CHECK"),
        }
    }
}

impl ReferenceNestedCheckDocumentV2 {
    /// Borrow the complete semantic document.
    #[must_use]
    pub const fn document(&self) -> &SemanticDocumentV2 {
        &self.document
    }
    /// Identity of the CHECK item.
    #[must_use]
    pub const fn check_item_id(&self) -> SemanticItemIdV2 {
        self.check_item_id
    }
    /// Identity of the nested FIND item.
    #[must_use]
    pub const fn nested_graph_item_id(&self) -> SemanticItemIdV2 {
        self.nested_graph_item_id
    }
}

/// Mint one checked location against the reference graph fixture registry.
///
/// # Errors
///
/// Returns a fixture error when the byte range is out of bounds.
pub fn reference_checked_span_v2(start: usize, end: usize) -> Result<CheckedLocatedSpanV2, String> {
    let registry = SourceRegistryViewV2::try_new(vec![
        SourceRegistrationV2 {
            source_id: SourceIdV2::from_raw(1),
            source_len: 256,
        },
        SourceRegistrationV2 {
            source_id: REFERENCE_SOURCE_V2,
            source_len: REFERENCE_GRAPH_SOURCE_LEN_V2,
        },
    ])
    .map_err(|error| error.to_string())?;
    registry
        .try_check_location(LocatedSpanV2 {
            source_id: REFERENCE_SOURCE_V2,
            span: SpanV1::new(start, end),
        })
        .map_err(|error| format!("{error:?}"))
}

fn graph_reference_kind(role: GraphReferenceRoleV2) -> GraphReferenceKindV2 {
    match role {
        GraphReferenceRoleV2::Source
        | GraphReferenceRoleV2::Sink
        | GraphReferenceRoleV2::Anchor
        | GraphReferenceRoleV2::Reference
        | GraphReferenceRoleV2::Pointee
        | GraphReferenceRoleV2::Via
        | GraphReferenceRoleV2::Through => GraphReferenceKindV2::Entity,
        GraphReferenceRoleV2::Callable => GraphReferenceKindV2::Callable,
        GraphReferenceRoleV2::Sanitizer => GraphReferenceKindV2::Sanitizer,
        GraphReferenceRoleV2::Scope => GraphReferenceKindV2::Scope,
        GraphReferenceRoleV2::Window => GraphReferenceKindV2::Window,
        GraphReferenceRoleV2::Frontier => GraphReferenceKindV2::Frontier,
    }
}

fn graph_hir_query(request: &ReferenceGraphRequestV2) -> Result<GraphQueryHirV1, String> {
    graph_hir_query_with_node_v2(request, HirNodeIdV1::from_raw(2))
}

/// Author one FIND query under a caller-chosen node identity. The reference
/// fixtures use node 2; [`reference_alternate_graph_document_v2`] uses
/// [`ALTERNATE_GRAPH_NODE_ID_V2`] so a nested-graph identity mismatch is
/// reachable end to end.
fn graph_hir_query_with_node_v2(
    request: &ReferenceGraphRequestV2,
    node: HirNodeIdV1,
) -> Result<GraphQueryHirV1, String> {
    let mut endpoints = GraphEndpointsV1::default();
    let mut constraints = GraphConstraintsV1 {
        depth: request.depth,
        direction: request.direction.map(|keyword| GraphDirectionV1 {
            keyword: keyword.to_owned(),
        }),
        summary_facets: request.summary_facets.clone(),
        ..GraphConstraintsV1::default()
    };
    for reference in &request.references {
        let graph_ref = GraphRefV1 {
            name: reference.selector.to_owned(),
            span: SpanV1::new(reference.span.0, reference.span.1),
        };
        match reference.role {
            GraphReferenceRoleV2::Source => endpoints.source = Some(graph_ref),
            GraphReferenceRoleV2::Sink => endpoints.sink = Some(graph_ref),
            GraphReferenceRoleV2::Anchor => endpoints.anchor = Some(graph_ref),
            GraphReferenceRoleV2::Reference => endpoints.reference = Some(graph_ref),
            GraphReferenceRoleV2::Pointee => endpoints.pointee = Some(graph_ref),
            GraphReferenceRoleV2::Callable => endpoints.callable = Some(graph_ref),
            GraphReferenceRoleV2::Via => constraints.via.push(graph_ref),
            GraphReferenceRoleV2::Through => constraints.through.push(graph_ref),
            GraphReferenceRoleV2::Sanitizer => constraints.sanitizers.push(graph_ref),
            GraphReferenceRoleV2::Scope => constraints.scope = Some(graph_ref),
            GraphReferenceRoleV2::Window => constraints.window = Some(graph_ref),
            GraphReferenceRoleV2::Frontier => constraints.frontier = Some(graph_ref),
        }
    }
    Ok(GraphQueryHirV1::new(
        node,
        SpanV1::new(request.item_span.0, request.item_span.1),
        request.kind,
        endpoints,
        constraints,
    ))
}

fn graph_authority_v2(
    references: &[ReferenceGraphReferenceV2],
    with_rule: bool,
) -> Result<ClosedFrontendAuthorityV2, String> {
    let budget = FrontendBudgetV2::standard_v2().map_err(|error| format!("{error:?}"))?;
    graph_authority_with_budget_v2(references, with_rule, budget)
}

fn graph_authority_with_budget_v2(
    references: &[ReferenceGraphReferenceV2],
    with_rule: bool,
    budget: FrontendBudgetV2,
) -> Result<ClosedFrontendAuthorityV2, String> {
    graph_authority_with_graph_inputs_v2(with_rule, budget, |host, _location| {
        let mut inputs = Vec::<GraphReferenceInputV2>::new();
        for (index, reference) in references.iter().enumerate() {
            let span = SpanV1::new(1 + index, 2 + index);
            let declaration = host
                .try_check_location(LocatedSpanV2 {
                    source_id: SourceIdV2::from_raw(1),
                    span,
                })
                .map_err(|error| format!("{error:?}"))?;
            inputs.push(GraphReferenceInputV2 {
                spelling: reference.selector.to_owned(),
                kind: graph_reference_kind(reference.role),
                entity_subtype: None,
                location: declaration,
            });
        }
        Ok(inputs)
    })
}

/// Seal one closed authority whose graph-selector inputs are derived from the
/// already-checked host registry.
fn graph_authority_with_graph_inputs_v2(
    with_rule: bool,
    budget: FrontendBudgetV2,
    make_inputs: impl FnOnce(
        &SourceRegistryViewV2,
        &CheckedLocatedSpanV2,
    ) -> Result<Vec<GraphReferenceInputV2>, String>,
) -> Result<ClosedFrontendAuthorityV2, String> {
    let host = SourceRegistryViewV2::try_new(vec![SourceRegistrationV2 {
        source_id: SourceIdV2::from_raw(1),
        source_len: 256,
    }])
    .map_err(|error| error.to_string())?;
    let location = host
        .try_check_location(LocatedSpanV2 {
            source_id: SourceIdV2::from_raw(1),
            span: SpanV1::new(0, 1),
        })
        .map_err(|error| format!("{error:?}"))?;
    let module =
        ModuleIdV2::try_new("app", location.clone()).map_err(|error| format!("{error:?}"))?;
    let profile = budget
        .authority_profile()
        .map_err(|error| format!("{error:?}"))?;
    let mut modules = ModuleAuthorityBuilderV2::try_new(profile.clone(), host.clone())
        .map_err(|error| format!("{error:?}"))?;
    modules
        .try_add_module(ModuleNodeInputV2 {
            module_id: module.clone(),
            location: location.clone(),
        })
        .map_err(|error| format!("{error:?}"))?;
    if with_rule {
        modules
            .try_add_external(ExternalCatalogAssetInputV2 {
                module_name: "app".to_owned(),
                local_name: "rule1".to_owned(),
                kind_token: "rule".to_owned(),
                typed_payload: ExternalCatalogPayloadInputV2::Rule(SignatureInputV2 {
                    parameters: Vec::<LocatedParameterInputV2>::new(),
                    result: ValueTypeV1::bool(),
                    callable_kind: CallableKindV2::Rule,
                    effects: ExpressionEffectsV2::pure_scalar(),
                    location: location.clone(),
                }),
                provenance: "host".to_owned(),
                location: location.clone(),
                dependencies: Vec::new(),
            })
            .map_err(|error| format!("{error:?}"))?;
    }
    let modules = modules
        .finish(
            CurrentModuleSelectionInputV2 {
                module_id: module,
                location: location.clone(),
            },
            VersionV2::new(1, 0, 0),
        )
        .map_err(|error| error.to_string())?;
    let relations = RelationCatalogV2::try_new(Vec::new(), &host, profile.clone())
        .map_err(|error| format!("{error:?}"))?;
    let inputs = make_inputs(&host, &location)?;
    let graphs = GraphReferenceCatalogV2::try_new(inputs, &host, profile)
        .map_err(|error| format!("{error:?}"))?;
    ClosedFrontendAuthorityV2::try_new(modules, relations, graphs, budget)
        .map_err(|error| format!("{error:?}"))
}

/// Resolve one reference FIND fixture through the real semantic transaction.
///
/// # Errors
///
/// Returns a fixture-construction error if the fixed fixture no longer
/// satisfies the semantic contract.
pub fn reference_graph_document_v2(
    request: ReferenceGraphRequestV2,
) -> Result<ReferenceGraphDocumentV2, String> {
    let query = graph_hir_query(&request)?;
    let item_id = SemanticItemIdV2::new(REFERENCE_SOURCE_V2, query.id);
    let authority = graph_authority_v2(&request.references, false)?;
    let document = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id: REFERENCE_SOURCE_V2,
            source_len: REFERENCE_GRAPH_SOURCE_LEN_V2,
            items: vec![ProgramItemHirV1::GraphQuery(Box::new(query))],
        })
        .map_err(|error| format!("{error:?}"))?,
        &authority,
    )
    .map_err(|error| format!("{error:?}"))?;
    Ok(ReferenceGraphDocumentV2 { document, item_id })
}

/// Resolve one reference nested CHECK/FIND fixture through the real semantic
/// transaction.
///
/// # Errors
///
/// Returns a fixture-construction error if the fixed fixture no longer
/// satisfies the semantic contract.
pub fn reference_nested_check_document_v2(
    request: ReferenceGraphRequestV2,
) -> Result<ReferenceNestedCheckDocumentV2, String> {
    let fixture = reference_check_document_v2(
        request,
        ReferenceCheckFixtureV2 {
            mode: DetectionModeV1::SemanticOnly,
            with_pattern: false,
            with_nested_graph: true,
        },
    )?;
    let nested_graph_item_id = fixture
        .nested_graph_item_id
        .ok_or("nested CHECK fixture must carry a nested graph")?;
    Ok(ReferenceNestedCheckDocumentV2 {
        document: fixture.document,
        check_item_id: fixture.check_item_id,
        nested_graph_item_id,
    })
}

/// Resolve a caller-selected CHECK mode and pattern/nested-FIND combination.
///
/// The options are intentionally not normalized: tests can author both the
/// coherent combinations and forged policy combinations while every retained
/// identity/reference still comes from the real semantic transaction.
///
/// # Errors
///
/// Returns a fixture-construction error if the selected shape cannot seal.
pub fn reference_check_document_v2(
    request: ReferenceGraphRequestV2,
    fixture: ReferenceCheckFixtureV2,
) -> Result<ReferenceCheckDocumentV2, String> {
    const PATTERN_NAME: &str = "pattern1";
    const PATTERN_SPAN: SpanV1 = SpanV1::new(106, 114);

    let query = fixture
        .with_nested_graph
        .then(|| graph_hir_query(&request))
        .transpose()?;
    let nested_graph_item_id = query
        .as_ref()
        .map(|query| SemanticItemIdV2::new(REFERENCE_SOURCE_V2, query.id));
    let pattern = fixture.with_pattern.then(|| GraphRefV1 {
        name: PATTERN_NAME.to_owned(),
        span: PATTERN_SPAN,
    });
    let mut check = LegalityHirV1::new(
        HirNodeIdV1::from_raw(3),
        SpanV1::new(request.item_span.0, request.item_span.1),
        GraphRefV1 {
            name: "rule1".to_owned(),
            span: SpanV1::new(100, 105),
        },
        None,
        pattern,
        ConfirmModeV1::Skip,
        EmitModeV1::Findings,
        fixture.mode,
    );
    if let Some(query) = query {
        check = check.with_semantic_confirm(query);
    }
    let check_item_id = SemanticItemIdV2::new(REFERENCE_SOURCE_V2, check.id);
    let budget = FrontendBudgetV2::standard_v2().map_err(|error| format!("{error:?}"))?;
    let authority = graph_authority_with_graph_inputs_v2(true, budget, |host, _location| {
        let mut inputs = Vec::new();
        for (index, reference) in request.references.iter().enumerate() {
            let declaration = host
                .try_check_location(LocatedSpanV2 {
                    source_id: SourceIdV2::from_raw(1),
                    span: SpanV1::new(1 + index, 2 + index),
                })
                .map_err(|error| format!("{error:?}"))?;
            inputs.push(GraphReferenceInputV2 {
                spelling: reference.selector.to_owned(),
                kind: graph_reference_kind(reference.role),
                entity_subtype: None,
                location: declaration,
            });
        }
        if fixture.with_pattern {
            let declaration = host
                .try_check_location(LocatedSpanV2 {
                    source_id: SourceIdV2::from_raw(1),
                    span: SpanV1::new(200, 201),
                })
                .map_err(|error| format!("{error:?}"))?;
            inputs.push(GraphReferenceInputV2 {
                spelling: PATTERN_NAME.to_owned(),
                kind: GraphReferenceKindV2::Pattern,
                entity_subtype: None,
                location: declaration,
            });
        }
        Ok(inputs)
    })?;
    let document = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id: REFERENCE_SOURCE_V2,
            source_len: REFERENCE_GRAPH_SOURCE_LEN_V2,
            items: vec![ProgramItemHirV1::Legality(Box::new(check))],
        })
        .map_err(|error| format!("{error:?}"))?,
        &authority,
    )
    .map_err(|error| format!("{error:?}"))?;
    Ok(ReferenceCheckDocumentV2 {
        document,
        check_item_id,
        nested_graph_item_id,
    })
}

// ---- rule documents --------------------------------------------------------

/// Authored body shape of one fixture rule.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RuleBodyShapeV2 {
    /// Only the anchor comparison; the body proves no call at all.
    AnchorOnly,
    /// `r(x)` — one positive source-predicate dependency.
    Positive(&'static str),
    /// `not r(x)` — one negated source-predicate dependency.
    Negated(&'static str),
    /// `count(r(x)) > 0` — a source-predicate call under an aggregate call.
    Aggregated(&'static str),
    /// `count(x) > 0` — a builtin call with no source-predicate target.
    BuiltinOnly,
}

/// Rule-document fixtures consumed by the Logic owner rails.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReferenceRuleFixtureV2 {
    /// One rule whose body calls nothing.
    SingleRule,
    /// One rule that positively depends on itself.
    PositiveSelfRecursion,
    /// One rule that negated-depends on itself.
    NegatedSelfRecursion,
    /// Invalid source rule that folds itself through an aggregate call.
    AggregatedSelfRecursion,
    /// Two rules with a mutual (r -> s, s -> r) dependency.
    MutualRecursionPair,
    /// Two rules with a single nonrecursive edge (r -> s).
    NonRecursiveDependency,
    /// Two rules with a single negated nonrecursive edge (r -> not s).
    NegatedNonRecursiveDependency,
    /// Invalid source rule with a predicate call inside an aggregate argument.
    AggregatedNonRecursiveDependency,
    /// One rule whose only call is a builtin, never a graph vertex.
    BuiltinCallOnly,
}

/// Expression wrappers forged around one otherwise real nonrecursive rule
/// call. These fixtures exercise downstream closed-tree walkers over shapes
/// that cannot all be authored around a Bool-returning rule call.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ForgedRuleCallWrapperV2 {
    /// Unary expression wrapper.
    Unary,
    /// Binary expression wrapper, with the same call on both edges.
    Binary,
    /// Comparison expression wrapper.
    Compare,
    /// Logical disjunction wrapper.
    Or,
    /// Parenthesized expression wrapper.
    Paren,
    /// Path-base expression wrapper.
    Path,
    /// Record-field expression wrapper.
    Record,
    /// List-item expression wrapper.
    List,
    /// Logical-quantifier body wrapper.
    LogicalQuantifier,
}

/// A sealed rule document plus its Rule item identities in document order.
#[derive(Clone, Debug)]
pub struct ReferenceRuleDocumentV2 {
    document: SemanticDocumentV2,
    item_ids: Vec<SemanticItemIdV2>,
}

impl ReferenceRuleDocumentV2 {
    /// The sealed semantic document.
    #[must_use]
    pub const fn document(&self) -> &SemanticDocumentV2 {
        &self.document
    }

    /// The Rule item identities in document order.
    #[must_use]
    pub fn item_ids(&self) -> &[SemanticItemIdV2] {
        &self.item_ids
    }

    /// The document's own immutable downstream budget receipt.
    #[must_use]
    pub const fn budget(&self) -> DownstreamBudgetReceiptV2 {
        self.document.budget_receipt()
    }
}

fn wrap_rule_document_v2(document: SemanticDocumentV2) -> ReferenceRuleDocumentV2 {
    let item_ids = document
        .order()
        .iter()
        .copied()
        .filter(|id| matches!(document.item(*id), Some(SemanticItemV2::Rule(_))))
        .collect();
    ReferenceRuleDocumentV2 { document, item_ids }
}

fn rule_item_v2(
    builder: &HirBuilderV1,
    name: &str,
    shape: RuleBodyShapeV2,
    start: usize,
) -> Result<ProgramItemHirV1, String> {
    let head = builder.symbol("x", SpanV1::new(start + 9, start + 10));
    let anchor = builder.compare(
        "=",
        builder.symbol("x", SpanV1::new(start + 14, start + 15)),
        builder.int("1", SpanV1::new(start + 18, start + 19)),
        SpanV1::new(start + 14, start + 19),
    );
    let source_call = |callee: &str, at: usize| -> Result<_, String> {
        Ok(builder.call(
            callee,
            SpanV1::new(at, at + 1),
            CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(
                builder.symbol("x", SpanV1::new(at + 2, at + 3)),
            )])
            .map_err(|error| format!("{error:?}"))?,
            SpanV1::new(at, at + 4),
        ))
    };
    let body = match shape {
        RuleBodyShapeV2::AnchorOnly => anchor,
        RuleBodyShapeV2::Positive(callee) => {
            let call = source_call(callee, start + 24)?;
            builder.and(
                anchor,
                builder.bool_value(call, SpanV1::new(start + 24, start + 28)),
                SpanV1::new(start + 14, start + 28),
            )
        }
        RuleBodyShapeV2::Negated(callee) => {
            let call = source_call(callee, start + 24)?;
            builder.and(
                anchor,
                builder.not(
                    builder.bool_value(call, SpanV1::new(start + 24, start + 28)),
                    SpanV1::new(start + 24, start + 28),
                ),
                SpanV1::new(start + 14, start + 28),
            )
        }
        RuleBodyShapeV2::Aggregated(callee) => {
            let inner = source_call(callee, start + 40)?;
            let count = builder.call(
                "count",
                SpanV1::new(start + 34, start + 39),
                CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(inner)])
                    .map_err(|error| format!("{error:?}"))?,
                SpanV1::new(start + 34, start + 45),
            );
            let predicate = builder.compare(
                ">",
                count,
                builder.int("0", SpanV1::new(start + 48, start + 49)),
                SpanV1::new(start + 34, start + 49),
            );
            builder.and(anchor, predicate, SpanV1::new(start + 14, start + 49))
        }
        RuleBodyShapeV2::BuiltinOnly => {
            let builtin = builder.call(
                "count",
                SpanV1::new(start + 24, start + 29),
                CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(
                    builder.int("1", SpanV1::new(start + 30, start + 31)),
                )])
                .map_err(|error| format!("{error:?}"))?,
                SpanV1::new(start + 24, start + 32),
            );
            let predicate = builder.compare(
                ">",
                builtin,
                builder.int("0", SpanV1::new(start + 34, start + 35)),
                SpanV1::new(start + 24, start + 35),
            );
            builder.and(anchor, predicate, SpanV1::new(start + 14, start + 35))
        }
    };
    Ok(ProgramItemHirV1::Rule(Box::new(builder.rule(
        name,
        SpanV1::new(start + 5, start + 6),
        head,
        body,
        SpanV1::new(start, start + 50),
    ))))
}

/// Resolve one rule-document fixture through the real transaction. Invalid
/// aggregate-wrapped source-call fixtures return the semantic rejection.
pub fn reference_rule_document_v2(
    fixture: ReferenceRuleFixtureV2,
) -> Result<ReferenceRuleDocumentV2, String> {
    let checked = rule_checked_document_v2(fixture)?;
    let document = resolve_checked_document_v2(checked, &reference_authority_v2()?)
        .map_err(|error| format!("{error:?}"))?;
    Ok(wrap_rule_document_v2(document))
}

/// Forge one wrapper around the real `r -> s` call while preserving the real
/// source-qualified call identity, target, polarity, and checked location.
pub fn forged_rule_call_wrapper_document_v2(
    wrapper: ForgedRuleCallWrapperV2,
) -> Result<ReferenceRuleDocumentV2, String> {
    let base = reference_rule_document_v2(ReferenceRuleFixtureV2::NonRecursiveDependency)?;
    let first = *base
        .item_ids
        .first()
        .ok_or("the nonrecursive rule fixture names its caller")?;
    let (root, call) = match base.document.item(first) {
        Some(SemanticItemV2::Rule(rule)) => (
            rule.body().expression().key(),
            rule.calls()
                .first()
                .ok_or("the nonrecursive rule fixture retains its call")?
                .call(),
        ),
        _ => return Err("the nonrecursive fixture caller is a rule".to_owned()),
    };
    let document = base.document.forged_expressions(move |expressions| {
        let Some(expression) = expressions.remove(&root) else {
            return;
        };
        let location = expression.location().clone();
        let TypedExprShapeV2::And { operator, .. } = expression.shape() else {
            expressions.insert(root, expression);
            return;
        };
        let operator = *operator;
        let shape = match wrapper {
            ForgedRuleCallWrapperV2::Unary => TypedExprShapeV2::Unary {
                operator,
                operand: call,
            },
            ForgedRuleCallWrapperV2::Binary => TypedExprShapeV2::Binary {
                operator,
                lhs: call,
                rhs: call,
            },
            ForgedRuleCallWrapperV2::Compare => TypedExprShapeV2::Compare {
                operator,
                lhs: call,
                rhs: call,
            },
            ForgedRuleCallWrapperV2::Or => TypedExprShapeV2::Or {
                operator,
                lhs: call,
                rhs: call,
            },
            ForgedRuleCallWrapperV2::Paren => TypedExprShapeV2::Paren { inner: call },
            ForgedRuleCallWrapperV2::Path => TypedExprShapeV2::Path {
                base: call,
                segments: vec![crate::TypedPathSegmentV2::new(
                    "field".to_owned(),
                    location.clone(),
                )],
            },
            ForgedRuleCallWrapperV2::Record => {
                let Ok(field) = crate::TypedRecordFieldV2::try_new("field", location, call) else {
                    expressions.insert(root, expression);
                    return;
                };
                TypedExprShapeV2::Record(vec![field])
            }
            ForgedRuleCallWrapperV2::List => TypedExprShapeV2::List(vec![call]),
            ForgedRuleCallWrapperV2::LogicalQuantifier => TypedExprShapeV2::LogicalQuantifier {
                kind: crate::LogicalQuantifierKindV2::Exists,
                binders: Vec::new(),
                body: call,
            },
        };
        expressions.insert(root, expression.forged_with_shape(shape));
    });
    // Fail closed when the mutation silently no-opped: every wrapper arm must
    // replace the base `And` rule body, so an unchanged body means the fixture
    // (and therefore its consumer's wrapper matrix) proved nothing.
    match document.expression(root) {
        Some(expression) if !matches!(expression.shape(), TypedExprShapeV2::And { .. }) => {}
        _ => {
            return Err(
                "the forged call wrapper must replace the nonrecursive rule body".to_owned(),
            )
        }
    }
    Ok(wrap_rule_document_v2(document))
}

/// Author the checked HIR of one rule-document fixture. Both the resolving path
/// ([`reference_rule_document_v2`]) and the small-receipt re-sealing path
/// ([`forged_small_receipt_rule_document_v2`]) must author the identical node
/// sequence, so this is the one authoring site for the fixture family.
fn rule_checked_document_v2(
    fixture: ReferenceRuleFixtureV2,
) -> Result<CheckedSyntaxDocumentV2, String> {
    use ReferenceRuleFixtureV2 as Fixture;
    let specs: &[(&str, RuleBodyShapeV2, usize)] = match fixture {
        Fixture::SingleRule => &[("r", RuleBodyShapeV2::AnchorOnly, 0)],
        Fixture::PositiveSelfRecursion => &[("r", RuleBodyShapeV2::Positive("r"), 0)],
        Fixture::NegatedSelfRecursion => &[("r", RuleBodyShapeV2::Negated("r"), 0)],
        Fixture::AggregatedSelfRecursion => &[("r", RuleBodyShapeV2::Aggregated("r"), 0)],
        Fixture::MutualRecursionPair => &[
            ("r", RuleBodyShapeV2::Positive("s"), 0),
            ("s", RuleBodyShapeV2::Positive("r"), 60),
        ],
        Fixture::NonRecursiveDependency => &[
            ("r", RuleBodyShapeV2::Positive("s"), 0),
            ("s", RuleBodyShapeV2::AnchorOnly, 60),
        ],
        Fixture::NegatedNonRecursiveDependency => &[
            ("r", RuleBodyShapeV2::Negated("s"), 0),
            ("s", RuleBodyShapeV2::AnchorOnly, 60),
        ],
        Fixture::AggregatedNonRecursiveDependency => &[
            ("r", RuleBodyShapeV2::Aggregated("s"), 0),
            ("s", RuleBodyShapeV2::AnchorOnly, 60),
        ],
        Fixture::BuiltinCallOnly => &[("r", RuleBodyShapeV2::BuiltinOnly, 0)],
    };
    let owned = specs
        .iter()
        .map(|(name, shape, start)| ((*name).to_owned(), *shape, *start))
        .collect::<Vec<_>>();
    rule_checked_document_from_specs_v2(&owned)
}

/// Author a checked rule document from ordered `(name, body shape, start)`
/// specifications.
fn rule_checked_document_from_specs_v2(
    specs: &[(String, RuleBodyShapeV2, usize)],
) -> Result<CheckedSyntaxDocumentV2, String> {
    let builder = HirBuilderV1::new();
    let mut items = Vec::new();
    for (name, shape, start) in specs {
        items.push(rule_item_v2(&builder, name, *shape, *start)?);
    }
    let source_len = specs
        .iter()
        .map(|(_, _, start)| start + 50)
        .max()
        .unwrap_or(50);
    CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
        source_id: REFERENCE_SOURCE_V2,
        source_len,
        items,
    })
    .map_err(|error| format!("{error:?}"))
}

/// Membership mutations minted from the mutual-recursion pair.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ForgedRuleMembershipMutationV2 {
    /// The second Rule item is removed while its identity stays in `order`.
    DroppedItem,
    /// The second Rule item is kept while its identity leaves `order`.
    OrphanItem,
    /// The second Rule identity is authored twice in `order`.
    DuplicateOrderId,
}

/// Seal one membership-forged rule document.
pub fn forged_rule_membership_document_v2(
    mutation: ForgedRuleMembershipMutationV2,
) -> Result<ReferenceRuleDocumentV2, String> {
    let base = reference_rule_document_v2(ReferenceRuleFixtureV2::MutualRecursionPair)?;
    let second = *base
        .item_ids
        .get(1)
        .ok_or("the mutual-recursion fixture names two rules")?;
    let document = base
        .document
        .forged_membership(move |order, items| match mutation {
            ForgedRuleMembershipMutationV2::DroppedItem => {
                items.remove(&second);
            }
            ForgedRuleMembershipMutationV2::OrphanItem => {
                order.retain(|id| *id != second);
            }
            ForgedRuleMembershipMutationV2::DuplicateOrderId => {
                order.push(second);
            }
        });
    Ok(wrap_rule_document_v2(document))
}

/// Resolved-call mutations forged through the in-crate seam.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ForgedRuleFactMutationV2 {
    /// The first call's target is a predicate no document member defines.
    NonLocalTarget,
    /// The first call's target disagrees with its own expression fact.
    TargetTransplanted,
    /// The first call's polarity disagrees with the proven body polarity.
    PolarityFlipped,
    /// The first call's node identity is absent from the expression facts.
    UnknownCallId,
    /// The first call's location comes from another registry identity.
    ForeignCallLocation,
    /// The first call's location is another span in the same registry.
    MismatchedCallLocation,
    /// The rule loses the call edge its body still proves.
    DroppedCall,
    /// The rule duplicates the call edge it already carries.
    ExtraCall,
}

/// Seal one rule-fact-forged document.
pub fn forged_rule_fact_document_v2(
    fixture: ReferenceRuleFixtureV2,
    mutation: ForgedRuleFactMutationV2,
) -> Result<ReferenceRuleDocumentV2, String> {
    let base = reference_rule_document_v2(fixture)?;
    let target_item = *base
        .item_ids
        .first()
        .ok_or("a rule-fact fixture names at least one rule")?;
    let foreign_location = foreign_call_location_v2()?;
    let document = base.document.forged_membership(move |_order, items| {
        let Some(SemanticItemV2::Rule(rule)) = items.remove(&target_item) else {
            return;
        };
        let original = rule.calls().to_vec();
        let Some(first) = original.first() else {
            items.insert(target_item, SemanticItemV2::Rule(rule));
            return;
        };
        let rewritten = match mutation {
            ForgedRuleFactMutationV2::NonLocalTarget => {
                let ghost = quanta_gqlang_v2_types::ResolvedPredicateIdV2::try_new("ghost");
                match ghost {
                    Ok(ghost) => original
                        .iter()
                        .map(|call| {
                            replace_call_v2(
                                call,
                                quanta_gqlang_v2_types::ResolvedCallTargetV2::Predicate(
                                    ghost.clone(),
                                ),
                                call.polarity(),
                                call.location().clone(),
                            )
                        })
                        .collect(),
                    Err(_mutation_error) => original.clone(),
                }
            }
            ForgedRuleFactMutationV2::TargetTransplanted => {
                let own = crate::ResolvedRuleV2::predicate_id(&rule).clone();
                vec![replace_call_v2(
                    first,
                    quanta_gqlang_v2_types::ResolvedCallTargetV2::Predicate(own),
                    first.polarity(),
                    first.location().clone(),
                )]
            }
            ForgedRuleFactMutationV2::PolarityFlipped => vec![replace_call_v2(
                first,
                first.target().clone(),
                crate::RuleCallPolarityV2::Negated,
                first.location().clone(),
            )],
            ForgedRuleFactMutationV2::UnknownCallId => vec![replace_call_v2(
                first,
                first.target().clone(),
                first.polarity(),
                first.location().clone(),
            )
            .with_call_node_v2(SourceQualifiedNodeIdV2 {
                source_id: REFERENCE_SOURCE_V2,
                node_id: HirNodeIdV1::from_raw(u32::MAX),
            })],
            ForgedRuleFactMutationV2::ForeignCallLocation => vec![replace_call_v2(
                first,
                first.target().clone(),
                first.polarity(),
                foreign_location.clone(),
            )],
            ForgedRuleFactMutationV2::MismatchedCallLocation => vec![replace_call_v2(
                first,
                first.target().clone(),
                first.polarity(),
                rule.body().expression().location().clone(),
            )],
            ForgedRuleFactMutationV2::DroppedCall => Vec::new(),
            ForgedRuleFactMutationV2::ExtraCall => vec![first.clone(), first.clone()],
        };
        items.insert(
            target_item,
            SemanticItemV2::Rule(rule.forged_with_calls(rewritten)),
        );
    });
    Ok(wrap_rule_document_v2(document))
}

/// Forge an extra call edge from a real builtin typed fact. The semantic
/// producer never includes builtin calls in a rule's source dependency list.
pub fn forged_non_source_rule_call_document_v2() -> Result<ReferenceRuleDocumentV2, String> {
    let base = reference_rule_document_v2(ReferenceRuleFixtureV2::BuiltinCallOnly)?;
    let item_id = *base
        .item_ids
        .first()
        .ok_or("the builtin rule fixture names one rule")?;
    let (key, fact) = base
        .document
        .expressions()
        .find(|(_, fact)| {
            fact.resolved_call().is_some_and(|target| {
                !matches!(
                    target,
                    quanta_gqlang_v2_types::ResolvedCallTargetV2::Predicate(_)
                )
            })
        })
        .ok_or("the builtin rule fixture retains one non-source call fact")?;
    let target = fact
        .resolved_call()
        .cloned()
        .ok_or("the selected expression retains its call target")?;
    let extra = crate::ResolvedRuleCallV2::new(
        *key,
        target,
        crate::RuleCallPolarityV2::Positive,
        fact.location().clone(),
    );
    let document = base.document.forged_membership(move |_order, items| {
        if let Some(SemanticItemV2::Rule(rule)) = items.remove(&item_id) {
            items.insert(
                item_id,
                SemanticItemV2::Rule(rule.forged_with_calls(vec![extra])),
            );
        }
    });
    Ok(wrap_rule_document_v2(document))
}

/// Forge a source call's typed shape target while leaving its retained fact
/// and rule edge untouched, so Logic must compare its derived target too.
pub fn forged_rule_call_shape_target_document_v2() -> Result<ReferenceRuleDocumentV2, String> {
    let base = reference_rule_document_v2(ReferenceRuleFixtureV2::NonRecursiveDependency)?;
    let rule = base
        .document
        .rules()
        .next()
        .ok_or("the nonrecursive fixture retains its caller")?;
    let call = rule
        .calls()
        .first()
        .ok_or("the nonrecursive caller retains its source edge")?
        .call();
    let wrong_target =
        quanta_gqlang_v2_types::ResolvedCallTargetV2::Predicate(rule.predicate_id().clone());
    let document = base.document.forged_expressions(move |expressions| {
        let Some(expression) = expressions.remove(&call) else {
            return;
        };
        let TypedExprShapeV2::Call { arguments, .. } = expression.shape() else {
            expressions.insert(call, expression);
            return;
        };
        let shape = TypedExprShapeV2::Call {
            target: wrong_target,
            arguments: arguments.clone(),
        };
        expressions.insert(call, expression.forged_with_shape(shape));
    });
    Ok(wrap_rule_document_v2(document))
}

fn replace_call_v2(
    call: &crate::ResolvedRuleCallV2,
    target: quanta_gqlang_v2_types::ResolvedCallTargetV2,
    polarity: crate::RuleCallPolarityV2,
    location: CheckedLocatedSpanV2,
) -> crate::ResolvedRuleCallV2 {
    crate::ResolvedRuleCallV2::new(call.call(), target, polarity, location)
}

trait WithCallNodeV2 {
    fn with_call_node_v2(self, node: SourceQualifiedNodeIdV2) -> Self;
}

impl WithCallNodeV2 for crate::ResolvedRuleCallV2 {
    fn with_call_node_v2(self, node: SourceQualifiedNodeIdV2) -> Self {
        let target = self.target().clone();
        let polarity = self.polarity();
        let location = self.location().clone();
        crate::ResolvedRuleCallV2::new(node, target, polarity, location)
    }
}

fn foreign_call_location_v2() -> Result<CheckedLocatedSpanV2, String> {
    capture_edge_registry()?
        .try_check_location(LocatedSpanV2 {
            source_id: REFERENCE_SOURCE_V2,
            span: SpanV1::new(0, 1),
        })
        .map_err(|error| format!("{error:?}"))
}

// ---- meta fixtures ---------------------------------------------------------

/// Canonical source text of [`ReferenceMetaFixtureV2::UsePack`].
pub const REFERENCE_USE_PACK_SOURCE_V2: &str =
    "use pack acme.pack | activation=session | lockfile=acme.lock;";

/// Pack identity authored by [`ReferenceMetaFixtureV2::UsePack`].
pub const REFERENCE_USE_PACK_ID_V2: &str = "acme.pack";

/// Lockfile authored by [`ReferenceMetaFixtureV2::UsePack`].
pub const REFERENCE_USE_PACK_LOCKFILE_V2: &str = "acme.lock";

/// Canonical source text of [`ReferenceMetaFixtureV2::RequireCertified`].
pub const REFERENCE_REQUIRE_CERTIFIED_SOURCE_V2: &str = "require certified;";

/// Payload family of one reference meta-directive document.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReferenceMetaFixtureV2 {
    /// `USE PACK acme.pack | activation=session | lockfile=acme.lock`.
    UsePack,
    /// `REQUIRE CERTIFIED`.
    RequireCertified,
}

/// A sealed meta-directive document plus its one Meta item identity.
#[derive(Clone, Debug)]
pub struct ReferenceMetaDocumentV2 {
    document: SemanticDocumentV2,
    item_id: SemanticItemIdV2,
}

impl ReferenceMetaDocumentV2 {
    /// Borrow the complete semantic document.
    #[must_use]
    pub const fn document(&self) -> &SemanticDocumentV2 {
        &self.document
    }

    /// Identity of the sole Meta item.
    #[must_use]
    pub const fn meta_item_id(&self) -> SemanticItemIdV2 {
        self.item_id
    }

    /// The resolved directive payload of that item.
    #[must_use]
    #[expect(
        clippy::panic,
        reason = "the fixture seals this identity, so a non-Meta item is unreachable"
    )]
    pub fn kind(&self) -> &ResolvedMetaKindV2 {
        match self.document.item(self.item_id) {
            Some(SemanticItemV2::Meta(meta)) => meta.kind(),
            _ => panic!("meta fixture item is a resolved meta directive"),
        }
    }
}

fn meta_hir_v2(builder: &HirBuilderV1, fixture: ReferenceMetaFixtureV2) -> (MetaHirV1, usize) {
    match fixture {
        ReferenceMetaFixtureV2::RequireCertified => (
            builder.meta(MetaKindV1::RequireCertified, SpanV1::new(0, 17)),
            18,
        ),
        ReferenceMetaFixtureV2::UsePack => {
            let source_len = REFERENCE_USE_PACK_SOURCE_V2.len();
            let meta = builder.meta(
                MetaKindV1::UsePack(UsePackHirV1 {
                    pack_id: REFERENCE_USE_PACK_ID_V2.to_owned(),
                    activation: UsePackActivationV1::Session,
                    lockfile: REFERENCE_USE_PACK_LOCKFILE_V2.to_owned(),
                }),
                SpanV1::new(0, source_len - 1),
            );
            (meta, source_len)
        }
    }
}

fn meta_checked_document_v2(
    fixture: ReferenceMetaFixtureV2,
) -> Result<(CheckedSyntaxDocumentV2, SemanticItemIdV2), String> {
    let builder = HirBuilderV1::new();
    let (meta, source_len) = meta_hir_v2(&builder, fixture);
    let item_id = SemanticItemIdV2::new(REFERENCE_SOURCE_V2, meta.id);
    let checked = CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
        source_id: REFERENCE_SOURCE_V2,
        source_len,
        items: vec![ProgramItemHirV1::Meta(Box::new(meta))],
    })
    .map_err(|error| format!("{error:?}"))?;
    Ok((checked, item_id))
}

/// Resolve one reference meta-directive fixture through the real semantic
/// transaction.
///
/// `RequireCertified` seals a document holding exactly one
/// [`ResolvedMetaKindV2::RequireCertified`] item, whose directive is also the
/// document catalog's validated certification requirement.
///
/// # Errors
///
/// Returns a fixture-construction error if the fixed fixture no longer
/// satisfies the semantic contract.
pub fn reference_meta_document_v2(
    kind: ReferenceMetaFixtureV2,
) -> Result<ReferenceMetaDocumentV2, String> {
    let (checked, item_id) = meta_checked_document_v2(kind)?;
    let document = resolve_checked_document_v2(checked, &reference_authority_v2()?)
        .map_err(|error| format!("{error:?}"))?;
    Ok(ReferenceMetaDocumentV2 { document, item_id })
}

/// Compatibility test helper for the former forged USE PACK fixture.
///
/// The producer path is now implemented, so this helper deliberately delegates
/// to the real transaction instead of bypassing Modules validation.
///
/// # Errors
///
/// Returns a fixture-construction error if the reassembled document no longer
/// satisfies the assembly contract.
pub fn forged_use_pack_meta_document_v2() -> Result<ReferenceMetaDocumentV2, String> {
    reference_meta_document_v2(ReferenceMetaFixtureV2::UsePack)
}

// ---- presentation fixtures -------------------------------------------------

/// Entity selector admitted for the reference presentation fixtures.
pub const REFERENCE_PRESENTATION_ENTITY_SELECTOR_V2: &str = "input_a";

/// DESCRIBE subtype admitted for the reference presentation fixtures.
pub const REFERENCE_PRESENTATION_ENTITY_SUBTYPE_V2: &str = "entity";

/// COMPARE baseline selector admitted for the reference presentation fixtures.
pub const REFERENCE_PRESENTATION_BASELINE_SELECTOR_V2: &str = "baseline_a";

/// COMPARE candidate selector admitted for the reference presentation fixtures.
pub const REFERENCE_PRESENTATION_CANDIDATE_SELECTOR_V2: &str = "candidate_b";

/// EXPORT source selector admitted for the reference presentation fixtures.
pub const REFERENCE_PRESENTATION_EXPORT_SELECTOR_V2: &str = "export_source";

/// SAMPLE scope selector admitted for the optional-scope fixture.
pub const REFERENCE_PRESENTATION_SCOPE_SELECTOR_V2: &str = "scope_a";

/// Source length of every reference presentation fixture document.
pub const REFERENCE_PRESENTATION_SOURCE_LEN_V2: usize = 64;

/// Positive SAMPLE limit authored by the reference SAMPLE presentation.
pub const REFERENCE_PRESENTATION_SAMPLE_LIMIT_V2: u32 = 5;

/// One reference presentation verb.
///
/// `Explain` carries the exact evidence/truth pair, so all twelve permutations
/// are reachable, including the two incoherent pairs planning refuses
/// (`Counterexample` + `Confirm`, `Citation` + `Refute`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReferencePresentationFixtureV2 {
    /// EXPLAIN one admitted entity selector.
    Explain {
        /// Authored evidence mode.
        evidence: EvidenceModeV1,
        /// Authored truth mode.
        truth: TruthModeV1,
    },
    /// SHOW capabilities.
    Show,
    /// DESCRIBE the admitted `entity` subtype of the entity selector.
    Describe,
    /// SAMPLE with a positive limit and no scope.
    Sample,
    /// SAMPLE with an authored scope reference.
    SampleWithScope,
    /// COMPARE two admitted snapshot selectors.
    Compare,
    /// EXPORT the admitted export source as SARIF.
    Export,
}

/// A sealed presentation document plus its one Presentation item identity.
#[derive(Clone, Debug)]
pub struct ReferencePresentationDocumentV2 {
    document: SemanticDocumentV2,
    item_id: SemanticItemIdV2,
}

impl ReferencePresentationDocumentV2 {
    /// Borrow the complete semantic document.
    #[must_use]
    pub const fn document(&self) -> &SemanticDocumentV2 {
        &self.document
    }

    /// Identity of the sole Presentation item.
    #[must_use]
    pub const fn presentation_item_id(&self) -> SemanticItemIdV2 {
        self.item_id
    }

    /// The resolved presentation contract.
    #[must_use]
    #[expect(
        clippy::panic,
        reason = "the fixture seals this identity, so a non-Presentation item is unreachable"
    )]
    pub fn presentation(&self) -> &ResolvedPresentationV2 {
        match self.document.item(self.item_id) {
            Some(SemanticItemV2::Presentation(presentation)) => presentation,
            _ => panic!("presentation fixture item is a resolved presentation"),
        }
    }
}

fn reference_graph_selector_v2(name: &str, start: usize) -> GraphRefV1 {
    GraphRefV1 {
        name: name.to_owned(),
        span: SpanV1::new(start, start + name.len()),
    }
}

fn reference_presentation_hir(
    builder: &HirBuilderV1,
    fixture: ReferencePresentationFixtureV2,
) -> (PresentationHirV1, usize) {
    let source_len = REFERENCE_PRESENTATION_SOURCE_LEN_V2;
    let item_span = |end: usize| SpanV1::new(0, end);
    let kind = match fixture {
        ReferencePresentationFixtureV2::Explain { evidence, truth } => {
            PresentationKindV1::Explain {
                target_ref: reference_graph_selector_v2(
                    REFERENCE_PRESENTATION_ENTITY_SELECTOR_V2,
                    8,
                ),
                evidence,
                truth,
            }
        }
        ReferencePresentationFixtureV2::Show => PresentationKindV1::Show {
            category: ShowCategoryV1::Capabilities,
        },
        ReferencePresentationFixtureV2::Describe => PresentationKindV1::Describe {
            kind: REFERENCE_PRESENTATION_ENTITY_SUBTYPE_V2.to_owned(),
            path: reference_graph_selector_v2(REFERENCE_PRESENTATION_ENTITY_SELECTOR_V2, 17),
        },
        ReferencePresentationFixtureV2::Sample => PresentationKindV1::Sample {
            query_text: "auth".to_owned(),
            mode: SampleModeV1::Lexical,
            limit: REFERENCE_PRESENTATION_SAMPLE_LIMIT_V2,
            scope: None,
        },
        ReferencePresentationFixtureV2::SampleWithScope => PresentationKindV1::Sample {
            query_text: "auth".to_owned(),
            mode: SampleModeV1::Lexical,
            limit: REFERENCE_PRESENTATION_SAMPLE_LIMIT_V2,
            scope: Some(reference_graph_selector_v2(
                REFERENCE_PRESENTATION_SCOPE_SELECTOR_V2,
                20,
            )),
        },
        ReferencePresentationFixtureV2::Compare => PresentationKindV1::Compare {
            baseline_ref: reference_graph_selector_v2(
                REFERENCE_PRESENTATION_BASELINE_SELECTOR_V2,
                17,
            ),
            candidate_ref: reference_graph_selector_v2(
                REFERENCE_PRESENTATION_CANDIDATE_SELECTOR_V2,
                30,
            ),
            metric: CompareMetricV1::Structural,
        },
        ReferencePresentationFixtureV2::Export => PresentationKindV1::Export {
            source_ref: reference_graph_selector_v2(REFERENCE_PRESENTATION_EXPORT_SELECTOR_V2, 14),
            format: ExportFormatV1::Sarif,
            profile: None,
        },
    };
    let end = match fixture {
        ReferencePresentationFixtureV2::Explain { .. } => 24,
        ReferencePresentationFixtureV2::Show => 16,
        ReferencePresentationFixtureV2::Describe => 25,
        ReferencePresentationFixtureV2::Sample
        | ReferencePresentationFixtureV2::SampleWithScope => 40,
        ReferencePresentationFixtureV2::Compare => 42,
        ReferencePresentationFixtureV2::Export => 26,
    };
    (builder.presentation(kind, item_span(end)), source_len)
}

fn presentation_authority_v2() -> Result<ClosedFrontendAuthorityV2, String> {
    let budget = FrontendBudgetV2::standard_v2().map_err(|error| format!("{error:?}"))?;
    graph_authority_with_graph_inputs_v2(false, budget, |host, _location| {
        let mut inputs = Vec::<GraphReferenceInputV2>::new();
        for (index, (spelling, kind, subtype)) in [
            (
                REFERENCE_PRESENTATION_ENTITY_SELECTOR_V2,
                GraphReferenceKindV2::Entity,
                Some(REFERENCE_PRESENTATION_ENTITY_SUBTYPE_V2),
            ),
            (
                REFERENCE_PRESENTATION_BASELINE_SELECTOR_V2,
                GraphReferenceKindV2::Snapshot,
                None,
            ),
            (
                REFERENCE_PRESENTATION_CANDIDATE_SELECTOR_V2,
                GraphReferenceKindV2::Snapshot,
                None,
            ),
            (
                REFERENCE_PRESENTATION_EXPORT_SELECTOR_V2,
                GraphReferenceKindV2::ExportSource,
                None,
            ),
            (
                REFERENCE_PRESENTATION_SCOPE_SELECTOR_V2,
                GraphReferenceKindV2::Scope,
                None,
            ),
        ]
        .into_iter()
        .enumerate()
        {
            let span = SpanV1::new(32 + index, 33 + index);
            let declaration = host
                .try_check_location(LocatedSpanV2 {
                    source_id: SourceIdV2::from_raw(1),
                    span,
                })
                .map_err(|error| format!("{error:?}"))?;
            inputs.push(GraphReferenceInputV2 {
                spelling: spelling.to_owned(),
                kind,
                entity_subtype: subtype.map(str::to_owned),
                location: declaration,
            });
        }
        Ok(inputs)
    })
}

/// Resolve one reference presentation fixture through the real semantic
/// transaction.
///
/// # Errors
///
/// Returns a fixture-construction error if the fixed fixture no longer
/// satisfies the semantic contract.
pub fn reference_presentation_document_v2(
    fixture: ReferencePresentationFixtureV2,
) -> Result<ReferencePresentationDocumentV2, String> {
    let builder = HirBuilderV1::new();
    let (presentation, source_len) = reference_presentation_hir(&builder, fixture);
    let item_id = SemanticItemIdV2::new(REFERENCE_SOURCE_V2, presentation.id);
    let checked = CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
        source_id: REFERENCE_SOURCE_V2,
        source_len,
        items: vec![ProgramItemHirV1::Presentation(Box::new(presentation))],
    })
    .map_err(|error| format!("{error:?}"))?;
    let document = resolve_checked_document_v2(checked, &presentation_authority_v2()?)
        .map_err(|error| format!("{error:?}"))?;
    Ok(ReferencePresentationDocumentV2 { document, item_id })
}

// ---- composition fixtures --------------------------------------------------

/// Source length of every reference composition fixture document.
pub const REFERENCE_COMPOSITION_SOURCE_LEN_V2: usize = 256;

/// Entity selector admitted for the reference composition FIND producers.
pub const REFERENCE_COMPOSITION_SOURCE_SELECTOR_V2: &str = "input_a";

/// Sink selector admitted for the reference composition FIND producers.
pub const REFERENCE_COMPOSITION_SINK_SELECTOR_V2: &str = "exec_b";

/// Pattern selector admitted for the reference composition CHECK body.
pub const REFERENCE_COMPOSITION_PATTERN_SELECTOR_V2: &str = "pattern_a";

/// Rule identity admitted for the reference composition CHECK body.
pub const REFERENCE_COMPOSITION_RULE_V2: &str = "rule1";

/// Closed body family of one reference composition document.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReferenceCompositionBodyV2 {
    /// A relational working-set body over the ordered LET producers.
    Relational,
    /// A relational body whose pure filter preserves its input output contract.
    RelationalFilter,
    /// Same relation shape with a different pure predicate.
    RelationalFilterFalse,
    /// A FIND body.
    Graph,
    /// A CHECK body.
    Check,
    /// An EXPLAIN body over a prior FIND producer.
    Explain,
}

/// A sealed composition document plus the facts the planner re-checks.
#[derive(Clone, Debug)]
pub struct ReferenceCompositionDocumentV2 {
    document: SemanticDocumentV2,
    composition_item_id: SemanticItemIdV2,
    body_item_id: SemanticItemIdV2,
    producer_item_ids: Vec<SemanticItemIdV2>,
    with_binding: ResolvedValueBindingV2,
}

impl ReferenceCompositionDocumentV2 {
    /// Borrow the complete semantic document.
    #[must_use]
    pub const fn document(&self) -> &SemanticDocumentV2 {
        &self.document
    }

    /// Identity of the composition root item.
    #[must_use]
    pub const fn composition_item_id(&self) -> SemanticItemIdV2 {
        self.composition_item_id
    }

    /// Identity of the sealed body item.
    #[must_use]
    pub const fn body_item_id(&self) -> SemanticItemIdV2 {
        self.body_item_id
    }

    /// Ordered producer identities, in authored LET order.
    #[must_use]
    pub fn producer_item_ids(&self) -> &[SemanticItemIdV2] {
        &self.producer_item_ids
    }

    /// The exact `WITH` binding identity, which equals exactly one producer's.
    #[must_use]
    pub const fn with_binding(&self) -> &ResolvedValueBindingV2 {
        &self.with_binding
    }

    /// The resolved composition contract itself.
    #[must_use]
    #[expect(
        clippy::panic,
        reason = "the fixture seals this identity, so a non-Composition item is unreachable"
    )]
    pub fn composition(&self) -> &ResolvedCompositionV2 {
        match self.document.item(self.composition_item_id) {
            Some(SemanticItemV2::Composition(composition)) => composition,
            _ => panic!("composition fixture item is a resolved composition"),
        }
    }
}

fn composition_sample_producer(
    builder: &HirBuilderV1,
    name: &str,
    name_start: usize,
    producer_start: usize,
    query_text: &str,
    limit: u32,
) -> CompositionBindingV1 {
    let sample = SampleHirV1 {
        id: builder.next_id(),
        span: SpanV1::new(producer_start, producer_start + 36),
        query_text: query_text.to_owned(),
        mode: SampleModeV1::Lexical,
        limit,
        scope: None,
    };
    builder.composition_binding(
        name,
        SpanV1::new(name_start, name_start + name.len()),
        CompositionProducerV1::Sample(Box::new(sample)),
    )
}

fn composition_find_query(
    builder: &HirBuilderV1,
    item_span: SpanV1,
    source_span: SpanV1,
    sink_span: SpanV1,
) -> GraphQueryHirV1 {
    let endpoints = GraphEndpointsV1 {
        source: Some(GraphRefV1 {
            name: REFERENCE_COMPOSITION_SOURCE_SELECTOR_V2.to_owned(),
            span: source_span,
        }),
        sink: Some(GraphRefV1 {
            name: REFERENCE_COMPOSITION_SINK_SELECTOR_V2.to_owned(),
            span: sink_span,
        }),
        ..GraphEndpointsV1::default()
    };
    let constraints = GraphConstraintsV1 {
        depth: Some(1),
        ..GraphConstraintsV1::default()
    };
    builder.graph_query(GraphQueryKindV1::Path, endpoints, constraints, item_span)
}

fn composition_find_producer(
    builder: &HirBuilderV1,
    name: &str,
    name_start: usize,
    item_span: SpanV1,
    source_span: SpanV1,
    sink_span: SpanV1,
) -> CompositionBindingV1 {
    let query = composition_find_query(builder, item_span, source_span, sink_span);
    builder.composition_binding(
        name,
        SpanV1::new(name_start, name_start + name.len()),
        CompositionProducerV1::GraphQuery(Box::new(query)),
    )
}

fn reference_composition_hir(
    builder: &HirBuilderV1,
    body: ReferenceCompositionBodyV2,
) -> Result<CompositionHirV1, String> {
    let composition = match body {
        ReferenceCompositionBodyV2::Relational => {
            let sampled = composition_sample_producer(builder, "sampled", 4, 14, "auth", 2);
            let sampled_again = composition_sample_producer(builder, "sampled2", 61, 71, "docs", 3);
            let scan = builder.relation_source(
                "sampled",
                SpanV1::new(125, 132),
                Some(alias("s", 133, 134)),
                SpanV1::new(125, 134),
            );
            let value = field_path(builder, "s", 152, "path", 154)?;
            let project = builder
                .try_project(
                    scan,
                    vec![ProjectionItemV1 {
                        value,
                        alias: Some(alias("path", 168, 172)),
                    }],
                    SpanV1::new(125, 172),
                )
                .map_err(|error| error.to_owned())?;
            let query = builder.query(project, None, SpanV1::new(120, 172));
            builder.composition(
                vec![sampled, sampled_again],
                "sampled",
                SpanV1::new(240, 247),
                CompositionBodyV1::Query(Box::new(query)),
                SpanV1::new(0, 247),
            )
        }
        ReferenceCompositionBodyV2::RelationalFilter
        | ReferenceCompositionBodyV2::RelationalFilterFalse => {
            let sampled = composition_sample_producer(builder, "sampled", 4, 14, "auth", 2);
            let scan = builder.relation_source(
                "sampled",
                SpanV1::new(125, 132),
                None,
                SpanV1::new(125, 132),
            );
            let truth = body == ReferenceCompositionBodyV2::RelationalFilter;
            let end = if truth { 144 } else { 145 };
            let predicate = builder.bool_value(
                builder.bool_lit(truth, SpanV1::new(140, end)),
                SpanV1::new(140, end),
            );
            let filtered = builder.relation_filter(scan, predicate, SpanV1::new(125, end));
            let query = builder.query(filtered, None, SpanV1::new(120, end));
            builder.composition(
                vec![sampled],
                "sampled",
                SpanV1::new(240, 247),
                CompositionBodyV1::Query(Box::new(query)),
                SpanV1::new(0, 247),
            )
        }
        ReferenceCompositionBodyV2::Graph => {
            let paths = composition_find_producer(
                builder,
                "paths",
                4,
                SpanV1::new(12, 70),
                SpanV1::new(24, 31),
                SpanV1::new(36, 42),
            );
            let body = composition_find_query(
                builder,
                SpanV1::new(80, 140),
                SpanV1::new(92, 99),
                SpanV1::new(104, 110),
            );
            builder.composition(
                vec![paths],
                "paths",
                SpanV1::new(240, 245),
                CompositionBodyV1::GraphQuery(Box::new(body)),
                SpanV1::new(0, 245),
            )
        }
        ReferenceCompositionBodyV2::Check => {
            let sampled = composition_sample_producer(builder, "sampled", 4, 14, "auth", 2);
            let check = builder.legality(
                GraphRefV1 {
                    name: REFERENCE_COMPOSITION_RULE_V2.to_owned(),
                    span: SpanV1::new(86, 91),
                },
                None,
                Some(GraphRefV1 {
                    name: REFERENCE_COMPOSITION_PATTERN_SELECTOR_V2.to_owned(),
                    span: SpanV1::new(110, 119),
                }),
                ConfirmModeV1::Skip,
                EmitModeV1::Findings,
                DetectionModeV1::PatternOnly,
                SpanV1::new(80, 200),
            );
            builder.composition(
                vec![sampled],
                "sampled",
                SpanV1::new(240, 247),
                CompositionBodyV1::Legality(Box::new(check)),
                SpanV1::new(0, 247),
            )
        }
        ReferenceCompositionBodyV2::Explain => {
            let paths = composition_find_producer(
                builder,
                "paths",
                4,
                SpanV1::new(12, 70),
                SpanV1::new(24, 31),
                SpanV1::new(36, 42),
            );
            let explain = ExplainHirV1 {
                id: builder.next_id(),
                span: SpanV1::new(80, 140),
                target_ref: GraphRefV1 {
                    name: "paths".to_owned(),
                    span: SpanV1::new(88, 93),
                },
                evidence: EvidenceModeV1::Auto,
                truth: TruthModeV1::Inspect,
            };
            builder.composition(
                vec![paths],
                "paths",
                SpanV1::new(240, 245),
                CompositionBodyV1::Explain(Box::new(explain)),
                SpanV1::new(0, 245),
            )
        }
    };
    Ok(composition)
}

fn composition_authority_v2(
    body: ReferenceCompositionBodyV2,
) -> Result<ClosedFrontendAuthorityV2, String> {
    let budget = FrontendBudgetV2::standard_v2().map_err(|error| format!("{error:?}"))?;
    let with_rule = body == ReferenceCompositionBodyV2::Check;
    graph_authority_with_graph_inputs_v2(with_rule, budget, |host, _location| {
        let mut admissions: Vec<(&str, GraphReferenceKindV2, Option<&str>)> = Vec::new();
        if !matches!(
            body,
            ReferenceCompositionBodyV2::Relational
                | ReferenceCompositionBodyV2::RelationalFilter
                | ReferenceCompositionBodyV2::RelationalFilterFalse
        ) {
            admissions.push((
                REFERENCE_COMPOSITION_SOURCE_SELECTOR_V2,
                GraphReferenceKindV2::Entity,
                None,
            ));
            admissions.push((
                REFERENCE_COMPOSITION_SINK_SELECTOR_V2,
                GraphReferenceKindV2::Entity,
                None,
            ));
        }
        if body == ReferenceCompositionBodyV2::Check {
            admissions.push((
                REFERENCE_COMPOSITION_PATTERN_SELECTOR_V2,
                GraphReferenceKindV2::Pattern,
                None,
            ));
        }
        let mut inputs = Vec::<GraphReferenceInputV2>::new();
        for (index, (spelling, kind, subtype)) in admissions.into_iter().enumerate() {
            let span = SpanV1::new(64 + index, 65 + index);
            let declaration = host
                .try_check_location(LocatedSpanV2 {
                    source_id: SourceIdV2::from_raw(1),
                    span,
                })
                .map_err(|error| format!("{error:?}"))?;
            inputs.push(GraphReferenceInputV2 {
                spelling: spelling.to_owned(),
                kind,
                entity_subtype: subtype.map(str::to_owned),
                location: declaration,
            });
        }
        Ok(inputs)
    })
}

/// Resolve one reference composition fixture through the real semantic
/// transaction.
///
/// The `Relational` body is the pipeline-proven working-set shape: an ordered
/// SAMPLE producer named by `WITH`, consumed by a body query over that working
/// set. Every body family also carries at least one ordered producer whose
/// sealed relation output the planner retains.
///
/// # Errors
///
/// Returns a fixture-construction error if the fixed fixture no longer
/// satisfies the semantic contract.
pub fn reference_composition_document_v2(
    body: ReferenceCompositionBodyV2,
) -> Result<ReferenceCompositionDocumentV2, String> {
    let builder = HirBuilderV1::new();
    let composition = reference_composition_hir(&builder, body)?;
    let composition_item_id = SemanticItemIdV2::new(REFERENCE_SOURCE_V2, composition.id);
    let checked = CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
        source_id: REFERENCE_SOURCE_V2,
        source_len: REFERENCE_COMPOSITION_SOURCE_LEN_V2,
        items: vec![ProgramItemHirV1::Composition(Box::new(composition))],
    })
    .map_err(|error| format!("{error:?}"))?;
    let document = resolve_checked_document_v2(checked, &composition_authority_v2(body)?)
        .map_err(|error| format!("{error:?}"))?;
    let (body_item_id, producer_item_ids, with_binding) = match document.item(composition_item_id) {
        Some(SemanticItemV2::Composition(composition)) => (
            composition.body().item_id(),
            composition
                .bindings()
                .iter()
                .map(|binding| binding.producer().item_id())
                .collect::<Vec<_>>(),
            composition.with_target().clone(),
        ),
        _ => return Err("composition fixture did not seal its composition item".to_owned()),
    };
    Ok(ReferenceCompositionDocumentV2 {
        document,
        composition_item_id,
        body_item_id,
        producer_item_ids,
        with_binding,
    })
}

/// A bounded chain of relational producers over one SAMPLE working set.
/// Each query is a single Scan; used to count ownership visits independently
/// of expression traversal and wall-clock timing.
pub fn reference_query_producer_composition_v2(
    producer_count: usize,
) -> Result<ReferenceCompositionDocumentV2, String> {
    if !(1..=16).contains(&producer_count) {
        return Err("fixture producer count must be in 1..=16".to_owned());
    }
    let builder = HirBuilderV1::new();
    let mut bindings = vec![composition_sample_producer(
        &builder, "sampled", 4, 14, "auth", 2,
    )];
    let mut previous = "sampled".to_owned();
    for index in 0..producer_count {
        let start = 100 + index * 32;
        let name = format!("q{index}");
        let scan = builder.relation_source(
            &previous,
            SpanV1::new(start + 8, start + 8 + previous.len()),
            None,
            SpanV1::new(start + 8, start + 8 + previous.len()),
        );
        let query = builder.query(scan, None, SpanV1::new(start + 4, start + 24));
        bindings.push(builder.composition_binding(
            &name,
            SpanV1::new(start, start + name.len()),
            CompositionProducerV1::Query(Box::new(query)),
        ));
        previous = name;
    }
    let start = 100 + producer_count * 32;
    let source_len = start + 64;
    let scan = builder.relation_source(
        &previous,
        SpanV1::new(start + 8, start + 8 + previous.len()),
        None,
        SpanV1::new(start + 8, start + 8 + previous.len()),
    );
    let body = builder.query(scan, None, SpanV1::new(start + 4, start + 24));
    let composition = builder.composition(
        bindings,
        &previous,
        SpanV1::new(start + 32, start + 32 + previous.len()),
        CompositionBodyV1::Query(Box::new(body)),
        SpanV1::new(0, source_len),
    );
    let composition_item_id = SemanticItemIdV2::new(REFERENCE_SOURCE_V2, composition.id);
    let checked = CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
        source_id: REFERENCE_SOURCE_V2,
        source_len,
        items: vec![ProgramItemHirV1::Composition(Box::new(composition))],
    })
    .map_err(|error| format!("{error:?}"))?;
    let document = resolve_checked_document_v2(
        checked,
        &composition_authority_v2(ReferenceCompositionBodyV2::Relational)?,
    )
    .map_err(|error| format!("{error:?}"))?;
    let Some(SemanticItemV2::Composition(composition)) = document.item(composition_item_id) else {
        return Err("composition fixture did not seal".to_owned());
    };
    let body_item_id = composition.body().item_id();
    let producer_item_ids = composition
        .bindings()
        .iter()
        .map(|binding| binding.producer().item_id())
        .collect();
    let with_binding = composition.with_target().clone();
    Ok(ReferenceCompositionDocumentV2 {
        document,
        composition_item_id,
        body_item_id,
        producer_item_ids,
        with_binding,
    })
}

// ---- small-receipt rule documents ------------------------------------------

/// Exact `max_semantic_steps` charge of a document rule program with
/// `member_count` members: one program charge plus one charge per member, so
/// [`SMALL_BUDGET_MAX_SEMANTIC_STEPS`] admits exactly seven members and refuses
/// the eighth.
#[must_use]
pub const fn reference_rule_program_charge_v2(member_count: u32) -> u32 {
    member_count.saturating_add(1)
}

/// Member count whose program charge exactly equals
/// [`SMALL_BUDGET_MAX_SEMANTIC_STEPS`].
pub const SMALL_BUDGET_RULE_PROGRAM_EXACT_MEMBERS: u32 = SMALL_BUDGET_MAX_SEMANTIC_STEPS - 1;

/// Member count whose program charge is exactly one step over
/// [`SMALL_BUDGET_MAX_SEMANTIC_STEPS`].
pub const SMALL_BUDGET_RULE_PROGRAM_OVER_MEMBERS: u32 = SMALL_BUDGET_MAX_SEMANTIC_STEPS;

/// Re-seal an already-resolved rule document under the small step receipt.
fn reseal_rule_document_with_receipt_v2(
    checked: &CheckedSyntaxDocumentV2,
    reference: &SemanticDocumentV2,
    receipt: DownstreamBudgetReceiptV2,
) -> Result<ReferenceRuleDocumentV2, String> {
    let (items, expressions, nested) = document_parts(reference);
    let document = reassemble_document(
        checked,
        &reference_authority_v2()?,
        items,
        expressions,
        nested,
        receipt,
    )?;
    Ok(wrap_rule_document_v2(document))
}

/// Seal one rule-document fixture under a receipt whose `max_semantic_steps` is
/// exactly [`SMALL_BUDGET_MAX_SEMANTIC_STEPS`] while every other dimension stays
/// standard.
///
/// The members, identities, and expression facts are the already-resolved
/// fixture's, so the rule-program walk charges exactly
/// [`reference_rule_program_charge_v2`] steps for the fixture's member count.
/// The fixed fixtures carry at most two members (three steps), so use
/// [`forged_small_receipt_rule_program_document_v2`] for the exact/over pair at
/// the sealed limit.
///
/// # Errors
///
/// Returns a fixture-construction error if the backing fixture or the
/// reassembled document no longer satisfies the assembly contract.
pub fn forged_small_receipt_rule_document_v2(
    fixture: ReferenceRuleFixtureV2,
) -> Result<ReferenceRuleDocumentV2, String> {
    let checked = rule_checked_document_v2(fixture)?;
    let reference = reference_rule_document_v2(fixture)?;
    reseal_rule_document_with_receipt_v2(&checked, reference.document(), small_steps_receipt()?)
}

/// Seal an `member_count`-member anchor-only rule program under the same small
/// step receipt.
///
/// [`SMALL_BUDGET_RULE_PROGRAM_EXACT_MEMBERS`] members charge exactly
/// [`SMALL_BUDGET_MAX_SEMANTIC_STEPS`] steps and plan;
/// [`SMALL_BUDGET_RULE_PROGRAM_OVER_MEMBERS`] members charge one step more and
/// are refused at the first uncharged member.
///
/// # Errors
///
/// Returns a fixture-construction error when `member_count` is zero or the
/// reassembled document no longer satisfies the assembly contract.
pub fn forged_small_receipt_rule_program_document_v2(
    member_count: u32,
) -> Result<ReferenceRuleDocumentV2, String> {
    if member_count == 0 {
        return Err("a rule program fixture names at least one rule".to_owned());
    }
    let specs = (0..member_count)
        .map(|index| {
            let ordinal = usize::try_from(index).map_err(|_error| "member overflow".to_owned())?;
            Ok((
                format!("r{index}"),
                RuleBodyShapeV2::AnchorOnly,
                ordinal * 60,
            ))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let checked = rule_checked_document_from_specs_v2(&specs)?;
    let document = resolve_checked_document_v2(checked.clone(), &reference_authority_v2()?)
        .map_err(|error| format!("{error:?}"))?;
    reseal_rule_document_with_receipt_v2(&checked, &document, small_steps_receipt()?)
}

// ---- narrow-witness and alternate-identity graph documents -----------------

/// `max_witnesses` of every narrow-witness graph fixture.
pub const NARROW_WITNESS_MAX_WITNESSES_V2: u32 = 1;

/// Node identity used by [`reference_alternate_graph_document_v2`]; distinct
/// from the [`HirNodeIdV1::from_raw`]`(2)` every reference FIND fixture seals.
pub const ALTERNATE_GRAPH_NODE_ID_V2: u32 = 5_001;

/// The ordered reference set one FIND subkind legally requires, mirroring the
/// consumer-side request builders so a narrow-witness document is a drop-in
/// replacement for a standard-budget one.
#[must_use]
pub fn reference_witness_graph_references_v2(
    kind: GraphQueryKindV1,
) -> Vec<ReferenceGraphReferenceV2> {
    use GraphReferenceRoleV2 as Role;
    let reference =
        |role: Role, selector: &'static str, span: (usize, usize)| ReferenceGraphReferenceV2 {
            role,
            selector,
            span,
        };
    match kind {
        GraphQueryKindV1::Reachability
        | GraphQueryKindV1::Flow
        | GraphQueryKindV1::Taint
        | GraphQueryKindV1::Path => vec![
            reference(Role::Source, "input_a", (20, 27)),
            reference(Role::Sink, "exec_b", (30, 36)),
        ],
        GraphQueryKindV1::Slice => vec![reference(Role::Anchor, "input_a", (20, 27))],
        GraphQueryKindV1::PointsTo => vec![
            reference(Role::Reference, "input_a", (20, 27)),
            reference(Role::Pointee, "exec_b", (30, 36)),
        ],
        GraphQueryKindV1::Summary => vec![reference(Role::Callable, "input_a", (20, 27))],
    }
}

fn narrow_witness_budget_v2() -> Result<FrontendBudgetV2, String> {
    let standard = FrontendBudgetV2::standard_v2().map_err(|error| format!("{error:?}"))?;
    FrontendBudgetV2::try_new(
        standard.max_source_bytes(),
        standard.max_frame_bytes(),
        standard.max_tokens(),
        standard.max_document_items(),
        standard.max_expression_depth(),
        standard.max_relation_depth(),
        standard.max_expression_nodes(),
        standard.max_collection_width(),
        standard.max_projection_width(),
        standard.max_semantic_steps(),
        standard.max_module_graph_nodes(),
        standard.max_module_graph_edges(),
        standard.max_module_steps(),
        standard.max_authority_bytes(),
        standard.max_authority_string_bytes(),
        standard.max_rule_graph_nodes(),
        standard.max_rule_graph_edges(),
        standard.max_rule_steps(),
        standard.max_witness_depth(),
        NARROW_WITNESS_MAX_WITNESSES_V2,
        standard.max_diagnostics(),
        standard.max_response_bytes(),
    )
    .map_err(|error| format!("{error:?}"))
}

/// Resolve one reference FIND fixture under a budget whose `max_witnesses` is
/// exactly [`NARROW_WITNESS_MAX_WITNESSES_V2`]; every other dimension is the
/// standard budget's. The sealed FIND retains those exact witness bounds, so a
/// consumer derives the canonical-one witness selection from the document
/// itself rather than choosing it.
///
/// # Errors
///
/// Returns a fixture-construction error if the fixture no longer satisfies the
/// semantic contract.
pub fn reference_narrow_witness_graph_document_v2(
    kind: GraphQueryKindV1,
    depth: u32,
) -> Result<ReferenceGraphDocumentV2, String> {
    let request = ReferenceGraphRequestV2 {
        kind,
        references: reference_witness_graph_references_v2(kind),
        depth: Some(depth),
        direction: None,
        summary_facets: Vec::new(),
        item_span: (0, 8),
    };
    let query = graph_hir_query(&request)?;
    let item_id = SemanticItemIdV2::new(REFERENCE_SOURCE_V2, query.id);
    let authority =
        graph_authority_with_budget_v2(&request.references, false, narrow_witness_budget_v2()?)?;
    let checked = CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
        source_id: REFERENCE_SOURCE_V2,
        source_len: REFERENCE_GRAPH_SOURCE_LEN_V2,
        items: vec![ProgramItemHirV1::GraphQuery(Box::new(query))],
    })
    .map_err(|error| format!("{error:?}"))?;
    let document =
        resolve_checked_document_v2(checked, &authority).map_err(|error| format!("{error:?}"))?;
    Ok(ReferenceGraphDocumentV2 { document, item_id })
}

/// Resolve one reference FIND fixture whose graph item identity is
/// [`ALTERNATE_GRAPH_NODE_ID_V2`] instead of the reference node `2`.
///
/// A consumer pairs this document's sealed graph contract with another
/// document's resolved CHECK to reach
/// `planning.invalid_check.nested_graph_identity_mismatch` end to end.
///
/// # Errors
///
/// Returns a fixture-construction error if the fixture no longer satisfies the
/// semantic contract.
pub fn reference_alternate_graph_document_v2(
    request: ReferenceGraphRequestV2,
) -> Result<ReferenceGraphDocumentV2, String> {
    let query =
        graph_hir_query_with_node_v2(&request, HirNodeIdV1::from_raw(ALTERNATE_GRAPH_NODE_ID_V2))?;
    let item_id = SemanticItemIdV2::new(REFERENCE_SOURCE_V2, query.id);
    let authority = graph_authority_v2(&request.references, false)?;
    let checked = CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
        source_id: REFERENCE_SOURCE_V2,
        source_len: REFERENCE_GRAPH_SOURCE_LEN_V2,
        items: vec![ProgramItemHirV1::GraphQuery(Box::new(query))],
    })
    .map_err(|error| format!("{error:?}"))?;
    let document =
        resolve_checked_document_v2(checked, &authority).map_err(|error| format!("{error:?}"))?;
    Ok(ReferenceGraphDocumentV2 { document, item_id })
}

/// Resolve the standard Taint FIND under
/// [`SECONDARY_REFERENCE_SOURCE_V2`]. Its graph shape matches the primary
/// reference fixture while every semantic identity is source-qualified by the
/// second document.
///
/// # Errors
///
/// Returns a fixture-construction error if the fixed FIND no longer seals.
pub fn second_source_graph_document_v2() -> Result<ReferenceGraphDocumentV2, String> {
    let kind = GraphQueryKindV1::Taint;
    let request = ReferenceGraphRequestV2 {
        kind,
        references: reference_witness_graph_references_v2(kind),
        depth: Some(4),
        direction: None,
        summary_facets: Vec::new(),
        item_span: (0, 8),
    };
    let query = graph_hir_query(&request)?;
    let item_id = SemanticItemIdV2::new(SECONDARY_REFERENCE_SOURCE_V2, query.id);
    let authority = graph_authority_v2(&request.references, false)?;
    let checked = CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
        source_id: SECONDARY_REFERENCE_SOURCE_V2,
        source_len: REFERENCE_GRAPH_SOURCE_LEN_V2,
        items: vec![ProgramItemHirV1::GraphQuery(Box::new(query))],
    })
    .map_err(|error| format!("{error:?}"))?;
    let document =
        resolve_checked_document_v2(checked, &authority).map_err(|error| format!("{error:?}"))?;
    Ok(ReferenceGraphDocumentV2 { document, item_id })
}
