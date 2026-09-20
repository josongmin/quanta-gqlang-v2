use quanta_gqlang_v2_hir::{
    walk_bool, walk_item, walk_relation, walk_shape, walk_value, BoolExprHirV1, BoolExprKindV1,
    CallArgHirV1, CatalogPayloadV1, CompositionBodyV1, CompositionProducerV1, HirVisitorV1,
    MetaKindV1, PresentationKindV1, ProgramItemHirV1, RelationExprHirV1, RelationExprKindV1,
    ResultShapeHirV1, ResultShapeKindV1, ValueExprHirV1, ValueExprKindV1, ViewBodyV1,
};

/// Independent visitor-derived shape projection. Node ids and spans are
/// deliberately omitted so parse/format/parse can compare authored structure.
pub(crate) fn structural_witness(item: &ProgramItemHirV1) -> Vec<String> {
    let mut witness = Witness::default();
    witness.visit_item(item);
    witness.parts
}

#[derive(Default)]
struct Witness {
    parts: Vec<String>,
}

impl HirVisitorV1 for Witness {
    fn visit_item(&mut self, item: &ProgramItemHirV1) {
        self.parts.push(match item {
            ProgramItemHirV1::Query(_) => "item:query".to_string(),
            ProgramItemHirV1::Decl(decl) => format!("item:decl:{}", decl.name),
            ProgramItemHirV1::Rule(rule) => format!("item:rule:{}", rule.name),
            ProgramItemHirV1::GraphQuery(graph) => format!("item:{}", graph_witness(graph)),
            ProgramItemHirV1::Legality(check) => format!("item:{}", legality_witness(check)),
            ProgramItemHirV1::Presentation(presentation) => {
                presentation_witness(&presentation.kind)
            }
            ProgramItemHirV1::CatalogDecl(decl) => {
                format!(
                    "item:catalog:{}:{}",
                    decl.name,
                    catalog_witness(&decl.payload)
                )
            }
            ProgramItemHirV1::Composition(comp) => format!(
                "item:composition:bindings={:?}:producers={:?}:with={}:body={}",
                comp.bindings
                    .iter()
                    .map(|b| b.name.as_str())
                    .collect::<Vec<_>>(),
                comp.bindings
                    .iter()
                    .map(|b| producer_witness(&b.producer))
                    .collect::<Vec<_>>(),
                comp.with_name,
                body_witness(&comp.body),
            ),
            ProgramItemHirV1::Meta(meta) => match &meta.kind {
                MetaKindV1::RequireCertified => "item:meta:require-certified".to_string(),
                MetaKindV1::UsePack(use_pack) => format!(
                    "item:meta:use-pack:{}:{:?}:{}",
                    use_pack.pack_id, use_pack.activation, use_pack.lockfile
                ),
            },
        });
        walk_item(self, item);
    }

    fn visit_value(&mut self, value: &ValueExprHirV1) {
        if matches!(value.kind, ValueExprKindV1::Paren(_)) {
            walk_value(self, value);
            return;
        }
        self.parts.push(match &value.kind {
            ValueExprKindV1::Literal(lit) => format!("value:literal:{lit:?}"),
            ValueExprKindV1::Symbol(symbol) => format!("value:symbol:{}", symbol.name),
            ValueExprKindV1::Path(path) => format!(
                "value:path:{}",
                path.segments
                    .iter()
                    .map(|s| s.name.as_str())
                    .collect::<Vec<_>>()
                    .join(".")
            ),
            ValueExprKindV1::Unary(unary) => format!("value:unary:{}", unary.op.symbol),
            ValueExprKindV1::Binary(binary) => format!("value:binary:{}", binary.op.symbol),
            ValueExprKindV1::Call(call) => format!("value:call:{}", call.callee.name),
            ValueExprKindV1::Paren(_) => unreachable!("handled above"),
            ValueExprKindV1::Record(record) => format!(
                "value:record:{}",
                record
                    .fields
                    .iter()
                    .map(|f| f.name.as_str())
                    .collect::<Vec<_>>()
                    .join(",")
            ),
            ValueExprKindV1::List(list) => format!("value:list:{}", list.elements.len()),
        });
        if let ValueExprKindV1::Call(call) = &value.kind {
            for arg in call.args().as_slice() {
                match arg {
                    CallArgHirV1::Expr(expr) => {
                        self.parts.push("call-arg:expr".to_string());
                        self.visit_value(expr);
                    }
                    CallArgHirV1::Star { .. } => self.parts.push("call-arg:star".to_string()),
                }
            }
        } else {
            walk_value(self, value);
        }
    }

    fn visit_bool(&mut self, value: &BoolExprHirV1) {
        if matches!(value.kind, BoolExprKindV1::Paren(_)) {
            walk_bool(self, value);
            return;
        }
        self.parts.push(match &value.kind {
            BoolExprKindV1::Literal(v) => format!("bool:{v}"),
            BoolExprKindV1::Value(_) => "bool:value".to_string(),
            BoolExprKindV1::Compare(compare) => format!("bool:compare:{}", compare.op.symbol),
            BoolExprKindV1::And(_) => "bool:and".to_string(),
            BoolExprKindV1::Or(_) => "bool:or".to_string(),
            BoolExprKindV1::Not(_) => "bool:not".to_string(),
            BoolExprKindV1::Paren(_) => unreachable!("handled above"),
            BoolExprKindV1::LogicalQuantifier(q) => {
                format!(
                    "bool:quant:{:?}:{:?}",
                    q.kind,
                    q.binders
                        .as_slice()
                        .iter()
                        .map(|b| b.name.as_str())
                        .collect::<Vec<_>>()
                )
            }
            BoolExprKindV1::RelationExists(e) => format!("bool:relation-exists:{:?}", e.polarity),
        });
        walk_bool(self, value);
    }

    fn visit_relation(&mut self, relation: &RelationExprHirV1) {
        self.parts.push(match &relation.kind {
            RelationExprKindV1::Source(source) => format!(
                "relation:source:{}:alias={:?}",
                source.name,
                source.alias.as_ref().map(|alias| alias.name.as_str())
            ),
            RelationExprKindV1::Filter(_) => "relation:filter".to_string(),
            RelationExprKindV1::Project(project) => {
                format!(
                    "relation:project:{:?}",
                    project
                        .items()
                        .iter()
                        .map(|item| item.alias.as_ref().map(|a| a.name.as_str()))
                        .collect::<Vec<_>>()
                )
            }
            RelationExprKindV1::Join(join) => format!(
                "relation:join:{}",
                match join {
                    quanta_gqlang_v2_hir::JoinHirV1::Inner { .. } => "inner",
                    quanta_gqlang_v2_hir::JoinHirV1::Cross { .. } => "cross",
                }
            ),
            RelationExprKindV1::GroupBy(group) => format!("relation:group:{}", group.keys().len()),
            RelationExprKindV1::OrderBy(order) => format!(
                "relation:order:{:?}",
                order
                    .keys()
                    .iter()
                    .map(|key| key.direction)
                    .collect::<Vec<_>>()
            ),
        });
        walk_relation(self, relation);
    }

    fn visit_shape(&mut self, shape: &ResultShapeHirV1) {
        self.parts.push(match &shape.kind {
            ResultShapeKindV1::Leaf(_) => "shape:leaf".to_string(),
            ResultShapeKindV1::Record(record) => format!(
                "shape:record:{:?}",
                record
                    .fields
                    .iter()
                    .map(|field| field.name.as_str())
                    .collect::<Vec<_>>()
            ),
            ResultShapeKindV1::List(_) => "shape:list".to_string(),
        });
        walk_shape(self, shape);
    }
}

fn names(refs: &[quanta_gqlang_v2_hir::GraphRefV1]) -> Vec<&str> {
    refs.iter()
        .map(|reference| reference.name.as_str())
        .collect()
}

fn graph_witness(graph: &quanta_gqlang_v2_hir::GraphQueryHirV1) -> String {
    format!(
        "find:{:?}:ep={:?}:via={:?}:through={:?}:san={:?}:scope={:?}:window={:?}:direction={:?}:frontier={:?}:depth={:?}:facets={:?}",
        graph.kind,
        [
            graph.endpoints.source.as_ref().map(|r| r.name.as_str()),
            graph.endpoints.sink.as_ref().map(|r| r.name.as_str()),
            graph.endpoints.anchor.as_ref().map(|r| r.name.as_str()),
            graph.endpoints.reference.as_ref().map(|r| r.name.as_str()),
            graph.endpoints.pointee.as_ref().map(|r| r.name.as_str()),
            graph.endpoints.callable.as_ref().map(|r| r.name.as_str()),
        ],
        names(&graph.constraints.via),
        names(&graph.constraints.through),
        names(&graph.constraints.sanitizers),
        graph.constraints.scope.as_ref().map(|r| r.name.as_str()),
        graph.constraints.window.as_ref().map(|r| r.name.as_str()),
        graph.constraints.direction.as_ref().map(|d| d.keyword.as_str()),
        graph.constraints.frontier.as_ref().map(|r| r.name.as_str()),
        graph.constraints.depth,
        graph.constraints.summary_facets,
    )
}

fn legality_witness(check: &quanta_gqlang_v2_hir::LegalityHirV1) -> String {
    format!(
        "check:{}:scope={:?}:pattern={:?}:confirm={:?}:emit={:?}:mode={:?}:semantic={:?}",
        check.rule_ref.name,
        check.scope.as_ref().map(|r| r.name.as_str()),
        check.pattern.as_ref().map(|r| r.name.as_str()),
        check.confirm,
        check.emit,
        check.mode,
        check.semantic_confirm.as_deref().map(graph_witness),
    )
}

fn presentation_witness(kind: &PresentationKindV1) -> String {
    match kind {
        PresentationKindV1::Explain {
            target_ref,
            evidence,
            truth,
        } => format!(
            "item:presentation:explain:{}:{evidence:?}:{truth:?}",
            target_ref.name
        ),
        PresentationKindV1::Show { category } => format!("item:presentation:show:{category:?}"),
        PresentationKindV1::Describe { kind, path } => {
            format!("item:presentation:describe:{kind}:{}", path.name)
        }
        PresentationKindV1::Sample {
            query_text,
            mode,
            limit,
            scope,
        } => format!(
            "item:presentation:sample:{query_text}:{mode:?}:{limit}:{:?}",
            scope.as_ref().map(|r| r.name.as_str())
        ),
        PresentationKindV1::Compare {
            baseline_ref,
            candidate_ref,
            metric,
        } => format!(
            "item:presentation:compare:{}:{}:{metric:?}",
            baseline_ref.name, candidate_ref.name
        ),
        PresentationKindV1::Export {
            source_ref,
            format,
            profile,
        } => format!(
            "item:presentation:export:{}:{format:?}:{:?}",
            source_ref.name,
            profile.as_ref().map(|r| r.name.as_str())
        ),
    }
}

fn catalog_witness(payload: &CatalogPayloadV1) -> String {
    match payload {
        CatalogPayloadV1::View(ViewBodyV1::QueryRef(reference)) => {
            format!("view-ref:{}", reference.name)
        }
        CatalogPayloadV1::View(ViewBodyV1::Inline(_)) => "view-inline".to_string(),
        CatalogPayloadV1::Model { provider, source } => format!("model:{provider}:{}", source.name),
        CatalogPayloadV1::Pack { source, activation } => {
            format!("pack:{}:{activation:?}", source.name)
        }
        CatalogPayloadV1::Profile {
            capabilities,
            admission,
        } => format!("profile:{:?}:{admission:?}", names(capabilities)),
        CatalogPayloadV1::Gate { checks, binding } => {
            format!("gate:{:?}:{binding:?}", names(checks))
        }
        CatalogPayloadV1::RuleBinding { check_ref } => format!("rule-binding:{}", check_ref.name),
    }
}

fn producer_witness(producer: &CompositionProducerV1) -> String {
    match producer {
        CompositionProducerV1::Query(_) => "query".to_string(),
        CompositionProducerV1::GraphQuery(graph) => graph_witness(graph),
        CompositionProducerV1::Sample(sample) => format!(
            "sample:{}:{:?}:{}:{:?}",
            sample.query_text,
            sample.mode,
            sample.limit,
            sample.scope.as_ref().map(|r| r.name.as_str())
        ),
    }
}

fn body_witness(body: &CompositionBodyV1) -> String {
    match body {
        CompositionBodyV1::Query(_) => "query".to_string(),
        CompositionBodyV1::GraphQuery(graph) => graph_witness(graph),
        CompositionBodyV1::Legality(check) => legality_witness(check),
        CompositionBodyV1::Explain(explain) => format!(
            "explain:{}:{:?}:{:?}",
            explain.target_ref.name, explain.evidence, explain.truth
        ),
    }
}
