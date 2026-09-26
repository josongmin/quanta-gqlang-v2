//! External-consumer positive program for the pinned facade (lane W4-H).
//!
//! This crate is NOT a member of the repository workspace, and its only
//! dependency is `quanta-gqlang-v2-frontend-contract`. Everything below is
//! therefore exactly what a sibling repository can see. If a curated root
//! disappears, is renamed, or loses a public accessor or signature, this
//! program stops compiling — that is the point of the gate.
//!
//! The positive rail constructs a closed authority through facade-only DTOs and
//! calls the closed compiler. Nothing here reaches into an owner crate.

#![forbid(unsafe_code)]
#![deny(unused_imports)]

use std::path::{Path, PathBuf};

use quanta_gqlang_v2_frontend_contract::{
    analyze_source_document_v2, classify_single_source_root_v3, compile_source_document_v3,
    format_source_document_v2,
    AlgebraFamilyV2, AnalysisDocumentV2, AnalysisItemV2, AnalysisStatusV2, CheckedLocatedSpanV2,
    ClosedFrontendAuthorityBuilderV2, ClosedFrontendAuthorityV2, CompiledItemV2,
    CompiledRuleProgramV2, CompiledSourceDocumentV3, ExecutionAuthorityV3, ExecutionBindingV3,
    ExecutionCallTargetV3, ExecutionCardinalityV3, ExecutionCompositionBindingV3,
    ExecutionCompositionBodyV3, ExecutionCompositionProducerV3, ExecutionCompositionV3,
    ExecutionCorrelationV3, ExecutionDeterminismV3, ExecutionDigestV3, ExecutionDocumentV3,
    ExecutionEffectV3, ExecutionEvidenceV3, ExecutionExistsPolarityV3, ExecutionExpressionShapeV3,
    ExecutionExpressionV3, ExecutionFamilyV3, ExecutionFieldV3, ExecutionFiniteFloatV3,
    ExecutionGraphCapabilityV3, ExecutionGraphDedupIdentityV3, ExecutionGraphDirectionV3,
    ExecutionGraphKindV3, ExecutionGraphQueryV3, ExecutionGraphReferenceKindV3,
    ExecutionGraphReferenceRoleV3, ExecutionGraphReferenceV3, ExecutionGraphRelationShapeV3,
    ExecutionGraphTargetV3, ExecutionGraphWitnessTieBreakV3, ExecutionItemV3,
    ExecutionJoinModeV3, ExecutionLiteralV3, ExecutionLogicalQuantifierV3,
    ExecutionNullOrderV3, ExecutionOperatorV3, ExecutionOrderKeyV3, ExecutionPlanV3,
    ExecutionPolicyFamilyV3, ExecutionRecordFieldV3, ExecutionRelationAuthorityV3,
    ExecutionRelationV3, ExecutionRelationalOperatorV3, ExecutionSampleModeV3,
    ExecutionScalarTypeV3, ExecutionSchemaV3,
    ExecutionSortDirectionV3, ExecutionSourceKindV3, ExecutionSourceV3, ExecutionUnionVariantV3,
    ExecutionValueTypeV3, FrontendAuthorityErrorV2,
    FrontendAuthorityLocationV2, FrontendAuthoritySourceV2, FrontendBudgetV2,
    FrontendCallableParameterInputV2, FrontendCatalogAssetInputV2, FrontendCatalogPayloadV2,
    FrontendDiagnosticV2, FrontendFailureV2, FrontendGraphReferenceInputV2,
    FrontendLanguageVersionV2, FrontendLockfileInputV2, FrontendPackInputV2,
    FrontendRelationAuthorityV2, FrontendRelationCardinalityV2, FrontendRelationFieldInputV2,
    FrontendRelationInputV2, FrontendScalarTypeV2, FrontendSeverityV2,
    FrontendSourceRootClassificationErrorV3, FrontendSourceRootFamilyV3, FrontendStageV2,
    FrontendTransactionErrorV2, GraphReferenceKindV2, LocatedSpanV2, NoPlanReasonV2,
    OpenAnalysisAuthorityV2, PlannedItemV2, RuleProgramMemberV2, SemanticItemIdV2, SemanticPlanV2,
    SourceIdV2, SourceQualifiedNodeIdV2, SpanV1, UnresolvedAnalysisFactV2,
    ANALYSIS_DIAGNOSTIC_LIMIT_CODE, OPEN_AUTHORITY_INCOMPLETE_CODE,
};

/// Document identity of the probe source.
const DOCUMENT_SOURCE: SourceIdV2 = SourceIdV2::from_raw(9);

/// A source document that needs no relation, module, or graph authority.
const DOCUMENT_TEXT: &str = "let answer = 1; rule local_rule : x if x = 1; show capabilities;";

const PACK_DOCUMENT_TEXT: &str = "use pack acme.pack | activation=session | lockfile=acme.lock;";

/// A composition whose generated producer is also the exact relational body input.
const COMPOSITION_DOCUMENT_TEXT: &str =
    "let sampled = sample query=\"auth\" | mode=lexical | limit=2 from sampled s | project s.path as path with sampled";

const CHANGED_COMPOSITION_DOCUMENT_TEXT: &str =
    "let sampled = sample query=\"auth\" | mode=lexical | limit=3 from sampled s | project s.path as path with sampled";

const LITERAL_COMPOSITION_DOCUMENT_TEXT: &str =
    "let sampled = sample query=\"auth\" | mode=lexical | limit=2 from sampled s | project 42 as answer with sampled";

const MULTI_BINDING_DOCUMENT_TEXT: &str =
    "let first = sample query=\"auth\" | mode=lexical | limit=2 \
     let second = from first f | project f.path as path \
     from second s | project s.path as path with second";

const SCOPED_SAMPLE_DOCUMENT_TEXT: &str =
    "let first = sample query=\"first\" | mode=lexical | limit=1 \
     let second = sample query=\"second\" | mode=semantic | limit=1 | scope=first \
     from second s | project s.path as path with second";

const GRAPH_COMPOSITION_DOCUMENT_TEXT: &str =
    "let paths = find path a -> b | via=mid | direction=backward | depth=2 \
     find reachability paths -> b | direction=both | depth=3 with paths";

const CHECK_COMPOSITION_DOCUMENT_TEXT: &str =
    "ruledef rule1 | check-ref=check1; \
     let first = sample query=\"first\" | mode=lexical | limit=1 \
     let second = sample query=\"second\" | mode=semantic | limit=1 | scope=first \
     check rule1 | scope=second | pattern=pattern1 | confirm=skip | mode=pattern with second";

const EXPLAIN_COMPOSITION_DOCUMENT_TEXT: &str =
    "let paths = find path a -> b | depth=1 \
     let reachable = find reachability paths -> b | depth=2 \
     explain reachable | evidence=auto | truth=inspect with reachable";

/// Deliberately collides with the host `records` relation below. The local
/// binding must win; host fallback would expose no `path` field and fail.
const LOCAL_BINDING_DOCUMENT_TEXT: &str =
    "let records = sample query=\"auth\" | mode=semantic | limit=3 from records r | project r.path as path with records";

const PACK_DIGEST: &str = "0000000000000000000000000000000000000000000000000000000000000000";

/// Code every unresolved fact carries while the open authority is partial.
///
/// The facade exports `UnresolvedAnalysisFactV2` but not the pipeline constant
/// that names this code, so an external consumer pins the literal.
const OPEN_AUTHORITY_INCOMPLETE_CODE_V2: &str = "pipeline.analysis.open_authority_incomplete";

/// The three authority stages an open editor session is missing here.
const MISSING_AUTHORITY_CODES_V2: [&str; 3] = [
    "pipeline.analysis.missing_module_authority",
    "pipeline.analysis.missing_relation_authority",
    "pipeline.analysis.missing_graph_authority",
];

/// Name a facade item in a monomorphic position without invoking it.
///
/// Roots whose values only an owner crate (or a not-yet-public builder) can
/// mint are still proved *reachable*: the item must exist, and its exact
/// signature must hold.
fn pin<T>(_item: T) {}

/// Exhaustive read of the closed compiled-item vocabulary.
fn compiled_item_label(item: &CompiledItemV2) -> &'static str {
    match item {
        CompiledItemV2::Planned(planned) => match planned.as_ref() {
            PlannedItemV2::Relation { .. } => "planned-relation",
            PlannedItemV2::Graph { .. } => "planned-graph",
            PlannedItemV2::Check { .. } => "planned-check",
            PlannedItemV2::Presentation { .. } => "planned-presentation",
            PlannedItemV2::Composition { .. } => "planned-composition",
            PlannedItemV2::MetaRequireCertified { .. } => "planned-meta-require-certified",
            PlannedItemV2::RuleMember(_) => "planned-rule-member",
        },
        CompiledItemV2::NoPlan { .. } => "no-plan",
    }
}

/// A compiled item carries a plan or a non-plan reason, never both and never
/// neither, and its identity is stable across reads.
fn compiled_item_invariants_hold(item: &CompiledItemV2) -> bool {
    item.plan().is_some() == item.no_plan_reason().is_none() && item.item_id() == item.item_id()
}

/// Every compiled-source accessor the facade exposes to a sibling repository.
fn compiled_document_probe(
    document: &CompiledSourceDocumentV3,
) -> (SourceIdV2, usize, usize, bool) {
    (
        document.source_id(),
        document.diagnostics().len(),
        document.items().len(),
        document.rule_program().is_some(),
    )
}

/// Every accessor `CompiledRuleProgramV2` exposes to a sibling repository.
fn compiled_rule_program_probe(program: &CompiledRuleProgramV2) -> (usize, bool) {
    let members = program.members();
    let stratified = std::mem::size_of_val(program.stratification()) > 0;
    (members.len(), stratified)
}

/// Every accessor `RuleProgramMemberV2` exposes to a sibling repository.
fn rule_member_probe(member: &RuleProgramMemberV2) -> bool {
    member.program_id() == member.program_id() && member.member_item_id() == member.member_item_id()
}

/// Every accessor `PlannedItemV2` exposes to a sibling repository.
fn planned_item_probe(item: &PlannedItemV2) -> (bool, bool) {
    let has_plan = item.plan().is_some();
    let has_rule_member = item.rule_member().is_some();
    let stable_identity = item.item_id() == item.item_id();
    (has_plan || has_rule_member, stable_identity)
}

/// Every accessor `SemanticPlanV2` exposes to a sibling repository.
fn semantic_plan_probe(plan: &SemanticPlanV2) -> usize {
    let lane = plan.authority_lane();
    let determinism = plan.evaluation_determinism();
    let evidence = plan.evidence();
    let effect = plan.effect();
    let result = plan.result();
    let cost = plan.semantic_cost();
    let explain = plan.explain();
    let policy = plan.policy();
    std::mem::size_of_val(&lane)
        + std::mem::size_of_val(&determinism)
        + std::mem::size_of_val(&evidence)
        + std::mem::size_of_val(&effect)
        + std::mem::size_of_val(result)
        + std::mem::size_of_val(cost)
        + std::mem::size_of_val(explain)
        + std::mem::size_of_val(policy)
        + plan.capabilities().len()
}

/// An unresolved open-analysis fact reports its code and exact location.
fn unresolved_fact_probe(fact: &UnresolvedAnalysisFactV2) -> (&'static str, LocatedSpanV2) {
    (fact.code(), fact.location())
}

/// The checked-location closure type reads back the location it was minted for.
fn checked_span_probe(span: &CheckedLocatedSpanV2) -> (LocatedSpanV2, usize) {
    let identity = span.registry_identity();
    (span.location(), std::mem::size_of_val(identity))
}

/// A sealed authority reports the exact budget that sealed it.
fn closed_authority_probe(authority: &ClosedFrontendAuthorityV2) -> &FrontendBudgetV2 {
    authority.budget()
}

/// Shallow declarations may form a dependency chain as long as the item budget.
/// Compilation must complete on a bounded host thread in either authored order.
fn assert_declaration_dependency_stack(authority: &ClosedFrontendAuthorityV2) {
    const COUNT: usize = 4096;
    assert_eq!(authority.budget().max_document_items(), 4096);
    std::thread::scope(|scope| {
        std::thread::Builder::new()
            .stack_size(2 * 1024 * 1024)
            .spawn_scoped(scope, || {
                let mut declarations: Vec<_> = (0..COUNT)
                    .map(|index| {
                        if index + 1 == COUNT {
                            format!("let x{index} = 1")
                        } else {
                            format!("let x{index} = x{}", index + 1)
                        }
                    })
                    .collect();
                for _ in 0..2 {
                    let source = declarations.join("; ");
                    let compiled = compile_source_document_v3(DOCUMENT_SOURCE, &source, authority)
                        .expect("item-budget declaration chain compiles without stack overflow");
                    assert_eq!(compiled.source_id(), DOCUMENT_SOURCE);
                    assert!(compiled.diagnostics().is_empty());
                    assert_eq!(compiled.items().len(), COUNT);
                    assert!(compiled.items().iter().all(|item| matches!(
                        item,
                        CompiledItemV2::NoPlan {
                            reason: NoPlanReasonV2::ValueDeclaration,
                            ..
                        }
                    )));
                    assert!(compiled.execution().items().is_empty());
                    declarations.reverse();
                }
            })
            .expect("bounded host thread")
            .join()
            .expect("declaration dependencies do not abort the host");
    });
}

/// Exhaustive read of the closed algebra-family vocabulary.
fn algebra_family_label(family: AlgebraFamilyV2) -> &'static str {
    match family {
        AlgebraFamilyV2::Scan => "scan",
        AlgebraFamilyV2::Filter => "filter",
        AlgebraFamilyV2::Project => "project",
        AlgebraFamilyV2::Join => "join",
        AlgebraFamilyV2::Group => "group",
        AlgebraFamilyV2::Aggregate => "aggregate",
        AlgebraFamilyV2::Sort => "sort",
        AlgebraFamilyV2::SemiJoin => "semi-join",
        AlgebraFamilyV2::AntiJoin => "anti-join",
    }
}

/// Exhaustive read of the closed non-plan vocabulary.
fn no_plan_reason_label(reason: NoPlanReasonV2) -> &'static str {
    match reason {
        NoPlanReasonV2::CatalogDeclaration { .. } => "catalog-declaration",
        NoPlanReasonV2::ValueDeclaration => "value-declaration",
        NoPlanReasonV2::MetaActivation => "meta-activation",
        NoPlanReasonV2::EmptyRuleProgram => "empty-rule-program",
    }
}

/// Read the V3 composition handoff through facade-only accessors.
///
/// Semantica must consume this immutable projection rather than restoring V1
/// HIR or recompiling a name-based producer/body relation.
fn assert_exact_composition_handoff(
    document: &CompiledSourceDocumentV3,
    binding_name: &str,
    expected_mode: ExecutionSampleModeV3,
    expected_limit: u32,
) {
    let execution = document.execution();
    assert_eq!(execution.source_id(), document.source_id());
    assert_eq!(execution.items().len(), 1);
    let item = execution.items().first().expect("one composition item");
    assert_eq!(item.family(), ExecutionFamilyV3::Composition);
    assert!(
        item.relation().is_none(),
        "composition is not a top-level relation"
    );
    let composition = item.composition().expect("composition execution payload");
    assert_eq!(composition.bindings().len(), 1);
    let binding = composition.bindings().first().expect("one ordered binding");
    assert_eq!(binding.binding().ordinal(), 0);
    assert_eq!(binding.binding().name(), binding_name);
    assert_eq!(composition.with_binding(), binding.binding());
    assert_ne!(binding.producer_item_id(), composition.body_item_id());
    assert_eq!(binding.producer_family(), ExecutionFamilyV3::Presentation);
    let binding_schema = binding
        .schema()
        .expect("SAMPLE producer retains its output schema");
    let [binding_field] = binding_schema.fields() else {
        panic!("SAMPLE schema retains exactly its path field");
    };
    assert_eq!(binding_field.name(), "path");
    assert_eq!(
        binding_field.value_type(),
        &ExecutionValueTypeV3::Scalar(ExecutionScalarTypeV3::RepoPath)
    );
    assert_eq!(binding_schema.cardinality(), ExecutionCardinalityV3::Set);
    assert!(
        binding.relation().is_none(),
        "SAMPLE is not a relation producer"
    );
    assert_eq!(composition.body_family(), ExecutionFamilyV3::Relation);
    assert_eq!(composition.body(), &ExecutionCompositionBodyV3::Relation);
    let ExecutionCompositionProducerV3::Sample {
        query_text,
        mode,
        limit,
        scope,
    } = binding.producer()
    else {
        panic!("SAMPLE binding keeps its exact producer payload");
    };
    assert_eq!(query_text, "auth");
    assert_eq!(*mode, expected_mode);
    assert_eq!(*limit, expected_limit);
    assert!(scope.is_none());

    let body = composition
        .body_relation()
        .expect("a relational body retains its sealed relation tree");
    let scan = body
        .root()
        .children()
        .first()
        .expect("project body has one scan child");
    let source = scan.source().expect("body scan carries source authority");
    assert_eq!(
        source.authority(),
        ExecutionRelationAuthorityV3::CompositionBinding
    );
    assert_eq!(source.name(), binding_name);
}

/// Replay only the two shared SAMPLE vectors needed at the external boundary.
/// The pipeline owner owns the TSV and its complete corpus; this fixture never
/// implements a digest encoder or maintains a second expected-value file.
fn assert_shared_composition_vectors() {
    let vector_path = std::env::var_os("QGLANG_V3_VECTOR_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            Path::new(env!("CARGO_MANIFEST_DIR")).join(
                "../../crates/quanta-gqlang-v2-pipeline/tests/fixtures/execution-v3-vectors.tsv",
            )
        });
    let vectors = std::fs::read_to_string(vector_path).expect("shared V3 vectors are readable");
    let location = FrontendAuthorityLocationV2 {
        source_id: SourceIdV2::from_raw(1),
        start: 0,
        end: 1,
    };
    let authority = ClosedFrontendAuthorityBuilderV2::try_new(
        FrontendBudgetV2::standard_v2().expect("standard budget"),
        vec![FrontendAuthoritySourceV2 {
            source_id: SourceIdV2::from_raw(1),
            source_len: 64,
        }],
        "app".to_owned(),
        location,
        FrontendLanguageVersionV2 {
            major: 1,
            minor: 0,
            patch: 0,
        },
    )
    .expect("minimal vector authority starts")
    .finish()
    .expect("minimal vector authority seals");

    let mut selected = Vec::new();
    for (case_name, expected_source, expected_limit) in [
        ("composition_limit_2", COMPOSITION_DOCUMENT_TEXT, 2),
        ("composition_limit_3", CHANGED_COMPOSITION_DOCUMENT_TEXT, 3),
    ] {
        let row = vectors
            .lines()
            .skip(1)
            .find(|row| row.split('\t').next() == Some(case_name))
            .unwrap_or_else(|| panic!("missing shared vector: {case_name}"));
        let fields = row.split('\t').collect::<Vec<_>>();
        let [case, profile, raw_source_id, source, source_hex, contract_hex, qualification_hex] =
            fields.as_slice()
        else {
            panic!("malformed shared vector: {case_name}");
        };
        assert_eq!(*case, case_name);
        assert_eq!(*profile, "minimal");
        assert_eq!(*raw_source_id, "9");
        assert_eq!(*source, expected_source);
        let compiled = compile_source_document_v3(
            SourceIdV2::from_raw(raw_source_id.parse().expect("u32 source ID")),
            source,
            &authority,
        )
        .expect("shared SAMPLE vector compiles through facade");
        assert_exact_composition_handoff(
            &compiled,
            "sampled",
            ExecutionSampleModeV3::Lexical,
            expected_limit,
        );
        assert_eq!(compiled.execution().source_digest().to_hex(), *source_hex);
        assert_eq!(
            compiled.execution().contract_digest().to_hex(),
            *contract_hex
        );
        assert_eq!(
            compiled.execution().qualification_digest().to_hex(),
            *qualification_hex
        );
        selected.push(compiled);
    }
    let [first, second] = selected.as_slice() else {
        panic!("exactly two shared composition vectors");
    };
    let reused_item = second.execution().items()[0].item_id();
    assert_eq!(first.execution().items()[0].item_id(), reused_item);
    assert_ne!(
        first.execution().qualification_digest(),
        second.execution().qualification_digest()
    );
    assert!(
        first.execution().item(reused_item).is_some(),
        "raw item ID lookup is not cross-document provenance admission"
    );
}

/// Compile-time semantic API receipt for the facade's immutable execution DTOs.
///
/// The root allowlist gate only sees re-exported names. These typed reads make
/// rustc reject an accessor removal or return-type drift in the owner types.
/// The functions are intentionally never fed synthetic values: the real
/// closed-compiler transactions below exercise the same methods at runtime.
fn execution_document_api(document: &ExecutionDocumentV3) -> usize {
    let _: SourceIdV2 = document.source_id();
    let _: &[ExecutionItemV3] = document.items();
    let _: Option<&ExecutionItemV3> = document.item(SemanticItemIdV2::new(
        document.source_id(),
        document.items()[0].item_id().hir_item_id(),
    ));
    let _: ExecutionDigestV3 = document.source_digest();
    let _: u32 = document.source_digest_schema_version();
    let _: ExecutionDigestV3 = document.contract_digest();
    let _: u32 = document.contract_digest_schema_version();
    let _: ExecutionDigestV3 = document.qualification_digest();
    let _: u32 = document.qualification_digest_schema_version();
    document.items().len()
}

fn execution_item_api(item: &ExecutionItemV3) -> usize {
    let _: SemanticItemIdV2 = item.item_id();
    let _: ExecutionFamilyV3 = item.family();
    let _: Option<&ExecutionRelationV3> = item.relation();
    let _: Option<&ExecutionCompositionV3> = item.composition();
    let _: &ExecutionPlanV3 = item.plan();
    std::mem::size_of_val(item.plan())
}

fn execution_plan_api(plan: &ExecutionPlanV3) -> usize {
    let _: ExecutionAuthorityV3 = plan.authority();
    let _: ExecutionDeterminismV3 = plan.determinism();
    let _: ExecutionEvidenceV3 = plan.evidence();
    let _: ExecutionEffectV3 = plan.effect();
    let _: ExecutionPolicyFamilyV3 = plan.policy_family();
    std::mem::size_of_val(plan)
}

fn execution_relation_api(relation: &ExecutionRelationV3) -> usize {
    let _: SemanticItemIdV2 = relation.item_id();
    let _: &ExecutionRelationalOperatorV3 = relation.root();
    let _: &[ExecutionExpressionV3] = relation.expressions();
    relation.expressions().len()
}

fn execution_operator_node_api(node: &ExecutionRelationalOperatorV3) -> usize {
    let _: AlgebraFamilyV2 = node.family();
    let _: Option<ExecutionJoinModeV3> = node.join_mode();
    let _: SourceQualifiedNodeIdV2 = node.stage();
    let _: Option<&ExecutionSourceV3> = node.source();
    let _: &ExecutionSchemaV3 = node.schema();
    let _: &[SourceQualifiedNodeIdV2] = node.predicates();
    let _: &[SourceQualifiedNodeIdV2] = node.expressions();
    let _: &[ExecutionCorrelationV3] = node.correlations();
    let _: &[ExecutionBindingV3] = node.partition_input();
    let _: &[ExecutionRelationalOperatorV3] = node.children();
    node.children().len()
}

fn execution_schema_api(schema: &ExecutionSchemaV3) -> usize {
    let _: &[ExecutionFieldV3] = schema.fields();
    let _: ExecutionCardinalityV3 = schema.cardinality();
    let _: &[SourceQualifiedNodeIdV2] = schema.partition_keys();
    let _: &[ExecutionOrderKeyV3] = schema.order();
    schema.fields().len()
}

fn execution_field_api(field: &ExecutionFieldV3) -> usize {
    let _: &str = field.name();
    let _: &ExecutionValueTypeV3 = field.value_type();
    field.name().len()
}

fn execution_order_key_api(key: &ExecutionOrderKeyV3) -> usize {
    let _: SourceQualifiedNodeIdV2 = key.expression();
    let _: ExecutionSortDirectionV3 = key.direction();
    let _: ExecutionNullOrderV3 = key.null_order();
    let _: &ExecutionValueTypeV3 = key.value_type();
    std::mem::size_of_val(key.value_type())
}

fn execution_expression_api(expression: &ExecutionExpressionV3) -> usize {
    let _: SourceQualifiedNodeIdV2 = expression.id();
    let _: &ExecutionValueTypeV3 = expression.value_type();
    let _: &ExecutionExpressionShapeV3 = expression.shape();
    std::mem::size_of_val(expression.shape())
}

fn execution_source_api(source: &ExecutionSourceV3) -> usize {
    let _: ExecutionRelationAuthorityV3 = source.authority();
    let _: ExecutionSourceKindV3 = source.kind();
    let _: Option<&str> = source.namespace();
    let _: &str = source.name();
    source.name().len()
}

fn execution_binding_api(binding: &ExecutionBindingV3) -> usize {
    let _: SourceIdV2 = binding.source_id();
    let _: u32 = binding.owner_node();
    let _: usize = binding.ordinal();
    let _: &str = binding.name();
    binding.ordinal()
}

fn execution_correlation_api(correlation: &ExecutionCorrelationV3) -> usize {
    let _: &ExecutionBindingV3 = correlation.binding();
    let _: SourceQualifiedNodeIdV2 = correlation.reference();
    std::mem::size_of_val(correlation.binding())
}

fn execution_composition_api(composition: &ExecutionCompositionV3) -> usize {
    let _: &[ExecutionCompositionBindingV3] = composition.bindings();
    let _: &ExecutionBindingV3 = composition.with_binding();
    let _: SemanticItemIdV2 = composition.body_item_id();
    let _: ExecutionFamilyV3 = composition.body_family();
    let _: Option<&ExecutionRelationV3> = composition.body_relation();
    let _: &ExecutionCompositionBodyV3 = composition.body();
    composition.bindings().len()
}

fn execution_composition_binding_api(binding: &ExecutionCompositionBindingV3) -> usize {
    let _: &ExecutionBindingV3 = binding.binding();
    let _: SemanticItemIdV2 = binding.producer_item_id();
    let _: ExecutionFamilyV3 = binding.producer_family();
    let _: Option<&ExecutionSchemaV3> = binding.schema();
    let _: Option<&ExecutionRelationV3> = binding.relation();
    let _: &ExecutionCompositionProducerV3 = binding.producer();
    std::mem::size_of_val(binding.producer())
}

fn execution_leaf_api(
    digest: ExecutionDigestV3,
    record: &ExecutionRecordFieldV3,
    union: &ExecutionUnionVariantV3,
    float: ExecutionFiniteFloatV3,
    operator: &ExecutionOperatorV3,
) -> usize {
    let _: &[u8; 32] = digest.as_bytes();
    let _: String = digest.to_hex();
    let _: &str = record.name();
    let _: &ExecutionValueTypeV3 = record.value_type();
    let _: &str = union.tag();
    let _: &ExecutionValueTypeV3 = union.value_type();
    let _: u64 = float.bits();
    let _: f64 = float.get();
    let _: &str = operator.symbol();
    let _: u32 = operator.overload_id();
    record.name().len() + union.tag().len() + operator.symbol().len()
}

macro_rules! exhaustive_unit_enum {
    ($value:expr; $($pattern:pat_param),+ $(,)?) => {
        match $value {
            $($pattern => ()),+
        }
    };
}

/// Exhaustive matches make variant addition and removal compile failures. This
/// is stronger than a debug-string or discriminant snapshot and stays scoped
/// to facade-exported closed vocabularies.
fn execution_closed_vocabulary_api(
    family: ExecutionFamilyV3,
    authority: ExecutionAuthorityV3,
    determinism: ExecutionDeterminismV3,
    evidence: ExecutionEvidenceV3,
    effect: ExecutionEffectV3,
    policy: ExecutionPolicyFamilyV3,
) {
    exhaustive_unit_enum!(family;
        ExecutionFamilyV3::Relation, ExecutionFamilyV3::Graph, ExecutionFamilyV3::Check,
        ExecutionFamilyV3::Presentation, ExecutionFamilyV3::Composition,
        ExecutionFamilyV3::MetaRequireCertified, ExecutionFamilyV3::RuleProgram);
    exhaustive_unit_enum!(authority;
        ExecutionAuthorityV3::FrontendChecked,
        ExecutionAuthorityV3::CertifiedRequirementValidated);
    exhaustive_unit_enum!(determinism;
        ExecutionDeterminismV3::Deterministic, ExecutionDeterminismV3::NonDeterministic);
    exhaustive_unit_enum!(evidence;
        ExecutionEvidenceV3::None, ExecutionEvidenceV3::SourceLocation,
        ExecutionEvidenceV3::WitnessPayload, ExecutionEvidenceV3::ProofTree);
    exhaustive_unit_enum!(effect;
        ExecutionEffectV3::ReadOnly, ExecutionEffectV3::ProofEvidenceOnly,
        ExecutionEffectV3::ExportOnly, ExecutionEffectV3::MutationForbidden);
    exhaustive_unit_enum!(policy;
        ExecutionPolicyFamilyV3::Relational, ExecutionPolicyFamilyV3::Graph,
        ExecutionPolicyFamilyV3::Check, ExecutionPolicyFamilyV3::Presentation,
        ExecutionPolicyFamilyV3::RuleProgram, ExecutionPolicyFamilyV3::MetaRequireCertified,
        ExecutionPolicyFamilyV3::Composition);
}

fn execution_closed_type_vocabulary_api(
    scalar: ExecutionScalarTypeV3,
    cardinality: ExecutionCardinalityV3,
    direction: ExecutionSortDirectionV3,
    null_order: ExecutionNullOrderV3,
    join: ExecutionJoinModeV3,
    source_kind: ExecutionSourceKindV3,
) {
    exhaustive_unit_enum!(scalar;
        ExecutionScalarTypeV3::Bool, ExecutionScalarTypeV3::Int, ExecutionScalarTypeV3::Float,
        ExecutionScalarTypeV3::String, ExecutionScalarTypeV3::Duration,
        ExecutionScalarTypeV3::Location, ExecutionScalarTypeV3::Path,
        ExecutionScalarTypeV3::RepoPath, ExecutionScalarTypeV3::EntityRef,
        ExecutionScalarTypeV3::SymbolRef, ExecutionScalarTypeV3::RuleRef,
        ExecutionScalarTypeV3::SourceScope, ExecutionScalarTypeV3::ProofTree);
    exhaustive_unit_enum!(cardinality;
        ExecutionCardinalityV3::Set, ExecutionCardinalityV3::Bag,
        ExecutionCardinalityV3::Sequence);
    exhaustive_unit_enum!(direction; ExecutionSortDirectionV3::Asc, ExecutionSortDirectionV3::Desc);
    exhaustive_unit_enum!(null_order; ExecutionNullOrderV3::First, ExecutionNullOrderV3::Last);
    exhaustive_unit_enum!(join; ExecutionJoinModeV3::Inner, ExecutionJoinModeV3::Cross);
    exhaustive_unit_enum!(source_kind;
        ExecutionSourceKindV3::Relation, ExecutionSourceKindV3::View,
        ExecutionSourceKindV3::Composition);
}

fn execution_closed_logic_vocabulary_api(
    quantifier: ExecutionLogicalQuantifierV3,
    exists: ExecutionExistsPolarityV3,
    relation_authority: ExecutionRelationAuthorityV3,
    sample: ExecutionSampleModeV3,
    graph: ExecutionGraphKindV3,
) {
    exhaustive_unit_enum!(quantifier;
        ExecutionLogicalQuantifierV3::Exists, ExecutionLogicalQuantifierV3::Forall);
    exhaustive_unit_enum!(exists;
        ExecutionExistsPolarityV3::Exists, ExecutionExistsPolarityV3::NotExists);
    exhaustive_unit_enum!(relation_authority;
        ExecutionRelationAuthorityV3::Host,
        ExecutionRelationAuthorityV3::CanonicalWorkspaceFiles,
        ExecutionRelationAuthorityV3::CatalogView,
        ExecutionRelationAuthorityV3::CompositionBinding);
    exhaustive_unit_enum!(sample;
        ExecutionSampleModeV3::Lexical, ExecutionSampleModeV3::Semantic,
        ExecutionSampleModeV3::Hybrid);
    exhaustive_unit_enum!(graph;
        ExecutionGraphKindV3::Reachability, ExecutionGraphKindV3::Flow,
        ExecutionGraphKindV3::Slice, ExecutionGraphKindV3::Taint,
        ExecutionGraphKindV3::Path, ExecutionGraphKindV3::PointsTo,
        ExecutionGraphKindV3::Summary);
}

fn execution_graph_vocabulary_api(
    direction: ExecutionGraphDirectionV3,
    role: ExecutionGraphReferenceRoleV3,
    kind: ExecutionGraphReferenceKindV3,
    capability: ExecutionGraphCapabilityV3,
) {
    exhaustive_unit_enum!(direction;
        ExecutionGraphDirectionV3::Forward, ExecutionGraphDirectionV3::Backward,
        ExecutionGraphDirectionV3::Both, ExecutionGraphDirectionV3::Undirected);
    exhaustive_unit_enum!(role;
        ExecutionGraphReferenceRoleV3::Source, ExecutionGraphReferenceRoleV3::Sink,
        ExecutionGraphReferenceRoleV3::Anchor, ExecutionGraphReferenceRoleV3::Reference,
        ExecutionGraphReferenceRoleV3::Pointee, ExecutionGraphReferenceRoleV3::Callable,
        ExecutionGraphReferenceRoleV3::Via, ExecutionGraphReferenceRoleV3::Through,
        ExecutionGraphReferenceRoleV3::Sanitizer, ExecutionGraphReferenceRoleV3::Scope,
        ExecutionGraphReferenceRoleV3::Window, ExecutionGraphReferenceRoleV3::Frontier);
    exhaustive_unit_enum!(kind;
        ExecutionGraphReferenceKindV3::Entity, ExecutionGraphReferenceKindV3::Callable,
        ExecutionGraphReferenceKindV3::Scope, ExecutionGraphReferenceKindV3::Window,
        ExecutionGraphReferenceKindV3::Frontier, ExecutionGraphReferenceKindV3::Sanitizer,
        ExecutionGraphReferenceKindV3::Pattern, ExecutionGraphReferenceKindV3::Snapshot,
        ExecutionGraphReferenceKindV3::ExportSource);
    exhaustive_unit_enum!(capability;
        ExecutionGraphCapabilityV3::Entity, ExecutionGraphCapabilityV3::Scope);
}

fn execution_value_type_api(value: &ExecutionValueTypeV3) {
    match value {
        ExecutionValueTypeV3::Scalar(value) => pin(*value),
        ExecutionValueTypeV3::Null | ExecutionValueTypeV3::Unknown => {}
        ExecutionValueTypeV3::Tuple(values) => pin(values.as_ref()),
        ExecutionValueTypeV3::Record(fields) => pin(fields.as_ref()),
        ExecutionValueTypeV3::Optional(value) => pin(value.as_ref()),
        ExecutionValueTypeV3::Collection {
            element,
            cardinality,
        } => {
            pin(element.as_ref());
            pin(*cardinality);
        }
        ExecutionValueTypeV3::Union(variants) => pin(variants.as_ref()),
    }
}

fn execution_call_target_api(target: &ExecutionCallTargetV3) {
    match target {
        ExecutionCallTargetV3::Builtin { name } | ExecutionCallTargetV3::Predicate { name } => {
            pin(name.as_str())
        }
        ExecutionCallTargetV3::HostPredicate { fingerprint } => pin(fingerprint.as_str()),
    }
}

fn execution_expression_shape_api(shape: &ExecutionExpressionShapeV3) {
    match shape {
        ExecutionExpressionShapeV3::Literal(value) => pin(value),
        ExecutionExpressionShapeV3::Binding(value) => pin(value),
        ExecutionExpressionShapeV3::Path { base, segments } => {
            pin(*base);
            pin(segments.as_ref());
        }
        ExecutionExpressionShapeV3::Unary { operator, operand }
        | ExecutionExpressionShapeV3::Not { operator, operand } => {
            pin(operator);
            pin(*operand);
        }
        ExecutionExpressionShapeV3::Binary { operator, lhs, rhs }
        | ExecutionExpressionShapeV3::Compare { operator, lhs, rhs }
        | ExecutionExpressionShapeV3::And { operator, lhs, rhs }
        | ExecutionExpressionShapeV3::Or { operator, lhs, rhs } => {
            pin(operator);
            pin(*lhs);
            pin(*rhs);
        }
        ExecutionExpressionShapeV3::Call { target, arguments } => {
            pin(target);
            pin(arguments.as_ref());
        }
        ExecutionExpressionShapeV3::Paren(value) | ExecutionExpressionShapeV3::BoolValue(value) => {
            pin(*value)
        }
        ExecutionExpressionShapeV3::Record(fields) => pin(fields.as_ref()),
        ExecutionExpressionShapeV3::List(values) => pin(values.as_ref()),
        ExecutionExpressionShapeV3::LogicalQuantifier {
            kind,
            binders,
            body,
        } => {
            pin(*kind);
            pin(binders.as_ref());
            pin(*body);
        }
        ExecutionExpressionShapeV3::RelationExists {
            polarity,
            relation_root,
            correlations,
        } => {
            pin(*polarity);
            pin(*relation_root);
            pin(correlations.as_ref());
        }
    }
}

fn execution_literal_api(literal: &ExecutionLiteralV3) {
    match literal {
        ExecutionLiteralV3::Int(value) => pin(*value),
        ExecutionLiteralV3::Float(value) => pin(*value),
        ExecutionLiteralV3::Bool(value) => pin(*value),
        ExecutionLiteralV3::String(value) => pin(value.as_str()),
        ExecutionLiteralV3::Null => {}
    }
}

fn execution_composition_body_api(body: &ExecutionCompositionBodyV3) {
    match body {
        ExecutionCompositionBodyV3::Relation
        | ExecutionCompositionBodyV3::Check
        | ExecutionCompositionBodyV3::Explain => {}
        ExecutionCompositionBodyV3::Graph(query) => execution_graph_query_api(query),
    }
}

fn execution_composition_producer_api(producer: &ExecutionCompositionProducerV3) {
    match producer {
        ExecutionCompositionProducerV3::Relation => {}
        ExecutionCompositionProducerV3::Sample {
            query_text,
            mode,
            limit,
            scope,
        } => {
            pin(query_text.as_str());
            pin(*mode);
            pin(*limit);
            pin(scope.as_ref());
        }
        ExecutionCompositionProducerV3::Graph { query } => execution_graph_query_api(query),
    }
}

fn execution_graph_query_api(query: &ExecutionGraphQueryV3) {
    pin(query.kind());
    pin(query.references());
    pin(query.depth());
    pin(query.direction());
    pin(query.summary_facets());
    pin(query.relation_shape());
    pin(query.dedup_identity());
    pin(query.max_witnesses());
    pin(query.max_path_length());
    pin(query.witness_tie_break());
}

fn execution_graph_witness_enum_api() {
    for shape in [
        ExecutionGraphRelationShapeV3::Acyclic,
        ExecutionGraphRelationShapeV3::Cyclic,
    ] {
        pin(shape);
    }
    for identity in [
        ExecutionGraphDedupIdentityV3::Endpoints,
        ExecutionGraphDedupIdentityV3::NodeSequence,
        ExecutionGraphDedupIdentityV3::NodeSet,
    ] {
        pin(identity);
    }
    for tie_break in [
        ExecutionGraphWitnessTieBreakV3::ShortestThenLexical,
        ExecutionGraphWitnessTieBreakV3::Lexical,
    ] {
        pin(tie_break);
    }
}

fn execution_graph_reference_api(reference: &ExecutionGraphReferenceV3) {
    pin(reference.role());
    execution_graph_target_api(reference.target());
}

fn execution_graph_target_api(target: &ExecutionGraphTargetV3) {
    match target {
        ExecutionGraphTargetV3::Host {
            kind,
            spelling,
            entity_subtype,
        } => {
            pin(*kind);
            pin(spelling.as_str());
            pin(entity_subtype.as_deref());
        }
        ExecutionGraphTargetV3::Composition {
            binding,
            capability,
        } => {
            pin(binding);
            pin(*capability);
        }
    }
}

fn facade_authority_vocabulary_api(
    graph: GraphReferenceKindV2,
    scalar: FrontendScalarTypeV2,
    cardinality: FrontendRelationCardinalityV2,
    authority: FrontendRelationAuthorityV2,
) {
    exhaustive_unit_enum!(graph;
        GraphReferenceKindV2::Entity, GraphReferenceKindV2::Callable,
        GraphReferenceKindV2::Scope, GraphReferenceKindV2::Window,
        GraphReferenceKindV2::Frontier, GraphReferenceKindV2::Sanitizer,
        GraphReferenceKindV2::Pattern, GraphReferenceKindV2::Snapshot,
        GraphReferenceKindV2::ExportSource);
    exhaustive_unit_enum!(scalar;
        FrontendScalarTypeV2::Bool, FrontendScalarTypeV2::Int,
        FrontendScalarTypeV2::Float, FrontendScalarTypeV2::String,
        FrontendScalarTypeV2::Duration, FrontendScalarTypeV2::Location,
        FrontendScalarTypeV2::Path, FrontendScalarTypeV2::RepoPath,
        FrontendScalarTypeV2::EntityRef, FrontendScalarTypeV2::SymbolRef,
        FrontendScalarTypeV2::RuleRef, FrontendScalarTypeV2::SourceScope,
        FrontendScalarTypeV2::ProofTree);
    exhaustive_unit_enum!(cardinality;
        FrontendRelationCardinalityV2::Set, FrontendRelationCardinalityV2::Bag,
        FrontendRelationCardinalityV2::Sequence);
    exhaustive_unit_enum!(authority;
        FrontendRelationAuthorityV2::Host,
        FrontendRelationAuthorityV2::CanonicalWorkspaceFiles);
}

fn facade_status_vocabulary_api(
    status: AnalysisStatusV2,
    stage: FrontendStageV2,
    severity: FrontendSeverityV2,
    payload: &FrontendCatalogPayloadV2,
) {
    let _: &'static str = status.keyword();
    let _: &'static str = stage.keyword();
    let _: &'static str = severity.keyword();
    exhaustive_unit_enum!(status;
        AnalysisStatusV2::Complete, AnalysisStatusV2::Incomplete, AnalysisStatusV2::Error);
    exhaustive_unit_enum!(stage;
        FrontendStageV2::Integrity, FrontendStageV2::Declaration,
        FrontendStageV2::Resolution, FrontendStageV2::Binding,
        FrontendStageV2::Typing, FrontendStageV2::Validation,
        FrontendStageV2::Resource);
    exhaustive_unit_enum!(severity;
        FrontendSeverityV2::Error, FrontendSeverityV2::Warning, FrontendSeverityV2::Note);
    match payload {
        FrontendCatalogPayloadV2::Query
        | FrontendCatalogPayloadV2::Check
        | FrontendCatalogPayloadV2::Capability
        | FrontendCatalogPayloadV2::ModelSource
        | FrontendCatalogPayloadV2::PackSource => {}
        FrontendCatalogPayloadV2::Rule { parameters, result } => {
            let _: &Vec<FrontendCallableParameterInputV2> = parameters;
            let _: &FrontendScalarTypeV2 = result;
        }
    }
}

fn facade_authority_input_api(
    field: &FrontendRelationFieldInputV2,
    relation: &FrontendRelationInputV2,
    graph: &FrontendGraphReferenceInputV2,
) -> usize {
    let _: &String = &field.name;
    let _: FrontendScalarTypeV2 = field.scalar_type;
    let _: bool = field.optional;
    let _: FrontendAuthorityLocationV2 = field.location;
    let _: FrontendRelationAuthorityV2 = relation.authority;
    let _: &String = &relation.provenance_namespace;
    let _: &String = &relation.qualified_name;
    let _: &String = &relation.lookup_name;
    let _: &Vec<FrontendRelationFieldInputV2> = &relation.fields;
    let _: FrontendRelationCardinalityV2 = relation.cardinality;
    let _: &String = &relation.provenance;
    let _: FrontendAuthorityLocationV2 = relation.location;
    let _: &String = &graph.spelling;
    let _: GraphReferenceKindV2 = graph.kind;
    let _: &Option<String> = &graph.entity_subtype;
    let _: FrontendAuthorityLocationV2 = graph.location;
    field.name.len() + relation.lookup_name.len() + graph.spelling.len()
}

/// Pin every curated root and accessor used by sibling repositories.
fn pin_curated_roots() {
    pin(compiled_item_label as fn(&CompiledItemV2) -> &'static str);
    pin(compiled_item_invariants_hold as fn(&CompiledItemV2) -> bool);
    pin(compiled_document_probe
        as fn(&CompiledSourceDocumentV3) -> (SourceIdV2, usize, usize, bool));
    pin(compiled_rule_program_probe as fn(&CompiledRuleProgramV2) -> (usize, bool));
    pin(rule_member_probe as fn(&RuleProgramMemberV2) -> bool);
    pin(planned_item_probe as fn(&PlannedItemV2) -> (bool, bool));
    pin(semantic_plan_probe as fn(&SemanticPlanV2) -> usize);
    pin(checked_span_probe as fn(&CheckedLocatedSpanV2) -> (LocatedSpanV2, usize));
    pin(closed_authority_probe as fn(&ClosedFrontendAuthorityV2) -> &FrontendBudgetV2);
    pin(unresolved_fact_probe as fn(&UnresolvedAnalysisFactV2) -> (&'static str, LocatedSpanV2));
    pin(execution_document_api as fn(&ExecutionDocumentV3) -> usize);
    pin(execution_item_api as fn(&ExecutionItemV3) -> usize);
    pin(execution_plan_api as fn(&ExecutionPlanV3) -> usize);
    pin(execution_relation_api as fn(&ExecutionRelationV3) -> usize);
    pin(execution_operator_node_api as fn(&ExecutionRelationalOperatorV3) -> usize);
    pin(execution_schema_api as fn(&ExecutionSchemaV3) -> usize);
    pin(execution_field_api as fn(&ExecutionFieldV3) -> usize);
    pin(execution_order_key_api as fn(&ExecutionOrderKeyV3) -> usize);
    pin(execution_expression_api as fn(&ExecutionExpressionV3) -> usize);
    pin(execution_source_api as fn(&ExecutionSourceV3) -> usize);
    pin(execution_binding_api as fn(&ExecutionBindingV3) -> usize);
    pin(execution_correlation_api as fn(&ExecutionCorrelationV3) -> usize);
    pin(execution_composition_api as fn(&ExecutionCompositionV3) -> usize);
    pin(execution_composition_binding_api as fn(&ExecutionCompositionBindingV3) -> usize);
    pin(execution_leaf_api
        as fn(
            ExecutionDigestV3,
            &ExecutionRecordFieldV3,
            &ExecutionUnionVariantV3,
            ExecutionFiniteFloatV3,
            &ExecutionOperatorV3,
        ) -> usize);
    pin(execution_closed_vocabulary_api
        as fn(
            ExecutionFamilyV3,
            ExecutionAuthorityV3,
            ExecutionDeterminismV3,
            ExecutionEvidenceV3,
            ExecutionEffectV3,
            ExecutionPolicyFamilyV3,
        ));
    pin(execution_closed_type_vocabulary_api
        as fn(
            ExecutionScalarTypeV3,
            ExecutionCardinalityV3,
            ExecutionSortDirectionV3,
            ExecutionNullOrderV3,
            ExecutionJoinModeV3,
            ExecutionSourceKindV3,
        ));
    pin(execution_closed_logic_vocabulary_api
        as fn(
            ExecutionLogicalQuantifierV3,
            ExecutionExistsPolarityV3,
            ExecutionRelationAuthorityV3,
            ExecutionSampleModeV3,
            ExecutionGraphKindV3,
        ));
    pin(execution_graph_vocabulary_api
        as fn(
            ExecutionGraphDirectionV3,
            ExecutionGraphReferenceRoleV3,
            ExecutionGraphReferenceKindV3,
            ExecutionGraphCapabilityV3,
        ));
    pin(execution_value_type_api as fn(&ExecutionValueTypeV3));
    pin(execution_call_target_api as fn(&ExecutionCallTargetV3));
    pin(execution_expression_shape_api as fn(&ExecutionExpressionShapeV3));
    pin(execution_literal_api as fn(&ExecutionLiteralV3));
    pin(execution_composition_body_api as fn(&ExecutionCompositionBodyV3));
    pin(execution_composition_producer_api as fn(&ExecutionCompositionProducerV3));
    pin(execution_graph_query_api as fn(&ExecutionGraphQueryV3));
    pin(execution_graph_witness_enum_api as fn());
    pin(execution_graph_reference_api as fn(&ExecutionGraphReferenceV3));
    pin(execution_graph_target_api as fn(&ExecutionGraphTargetV3));
    pin(facade_authority_vocabulary_api
        as fn(
            GraphReferenceKindV2,
            FrontendScalarTypeV2,
            FrontendRelationCardinalityV2,
            FrontendRelationAuthorityV2,
        ));
    pin(facade_authority_input_api
        as fn(
            &FrontendRelationFieldInputV2,
            &FrontendRelationInputV2,
            &FrontendGraphReferenceInputV2,
        ) -> usize);
    pin(facade_status_vocabulary_api
        as fn(AnalysisStatusV2, FrontendStageV2, FrontendSeverityV2, &FrontendCatalogPayloadV2));
    pin(OPEN_AUTHORITY_INCOMPLETE_CODE);
    pin(ANALYSIS_DIAGNOSTIC_LIMIT_CODE);
}

fn main() {
    assert_eq!(
        ANALYSIS_DIAGNOSTIC_LIMIT_CODE,
        "pipeline.analysis.diagnostic_limit_exceeded"
    );
    let budget: FrontendBudgetV2 = FrontendBudgetV2::standard_v2().expect("standard budget");
    assert!(
        budget.max_source_bytes() > 0,
        "the canonical budget is non-degenerate"
    );

    // The public parse-only route classifier has no closed authority input.
    // These roots may be refused by a downstream runtime, but classification
    // must preserve their exact family without resolving any external names.
    // Runtime support and typed admission failures remain consumer-owned.
    for (source, expected) in [
        ("let x = 1", FrontendSourceRootFamilyV3::Declaration),
        ("rule r : x if a", FrontendSourceRootFamilyV3::Rule),
        ("view v | query-ref=r.x", FrontendSourceRootFamilyV3::Catalog),
    ] {
        assert_eq!(
            classify_single_source_root_v3(DOCUMENT_SOURCE, source, &budget)
                .expect("parse-only family classification needs no closed authority"),
            expected,
            "{source}",
        );
    }
    assert_eq!(
        classify_single_source_root_v3(
            DOCUMENT_SOURCE,
            "find path src.marker -> src.marker | depth=8",
            &budget,
        )
        .expect("one graph root"),
        FrontendSourceRootFamilyV3::Graph,
    );
    assert_eq!(
        classify_single_source_root_v3(DOCUMENT_SOURCE, COMPOSITION_DOCUMENT_TEXT, &budget)
            .expect("one composition root"),
        FrontendSourceRootFamilyV3::Composition,
    );
    assert!(matches!(
        classify_single_source_root_v3(DOCUMENT_SOURCE, "", &budget),
        Err(FrontendSourceRootClassificationErrorV3::NoItem),
    ));
    assert!(matches!(
        classify_single_source_root_v3(
            DOCUMENT_SOURCE,
            "show capabilities; show models",
            &budget,
        ),
        Err(FrontendSourceRootClassificationErrorV3::MultipleItems { observed: 2 }),
    ));

    // ---- facade-only closed authority and compilation --------------------
    let lock_text = format!("acme.pack 1.0.0 {PACK_DIGEST}\n");
    let mut closed_builder = ClosedFrontendAuthorityBuilderV2::try_new(
        budget.clone(),
        vec![
            FrontendAuthoritySourceV2 {
                source_id: SourceIdV2::from_raw(1),
                source_len: 8,
            },
            FrontendAuthoritySourceV2 {
                source_id: SourceIdV2::from_raw(2),
                source_len: lock_text.len(),
            },
        ],
        "app".to_owned(),
        FrontendAuthorityLocationV2 {
            source_id: SourceIdV2::from_raw(1),
            start: 0,
            end: 1,
        },
        FrontendLanguageVersionV2 {
            major: 1,
            minor: 0,
            patch: 0,
        },
    )
    .expect("facade-only authority builder starts");
    closed_builder.add_pack(FrontendPackInputV2 {
        pack_id: "acme.pack".to_owned(),
        version: FrontendLanguageVersionV2 {
            major: 1,
            minor: 0,
            patch: 0,
        },
        content_digest: PACK_DIGEST.to_owned(),
        language_lower: FrontendLanguageVersionV2 {
            major: 1,
            minor: 0,
            patch: 0,
        },
        language_upper: FrontendLanguageVersionV2 {
            major: 2,
            minor: 0,
            patch: 0,
        },
        location: FrontendAuthorityLocationV2 {
            source_id: SourceIdV2::from_raw(1),
            start: 1,
            end: 2,
        },
    });
    closed_builder.add_catalog_asset(FrontendCatalogAssetInputV2 {
        module_name: "app".to_owned(),
        local_name: "queries.probe".to_owned(),
        payload: FrontendCatalogPayloadV2::Query,
        provenance: "host".to_owned(),
        location: FrontendAuthorityLocationV2 {
            source_id: SourceIdV2::from_raw(1),
            start: 3,
            end: 4,
        },
    });
    closed_builder.add_catalog_asset(FrontendCatalogAssetInputV2 {
        module_name: "app".to_owned(),
        local_name: "check1".to_owned(),
        payload: FrontendCatalogPayloadV2::Check,
        provenance: "host".to_owned(),
        location: FrontendAuthorityLocationV2 {
            source_id: SourceIdV2::from_raw(1),
            start: 3,
            end: 4,
        },
    });
    pin(std::mem::size_of::<FrontendCallableParameterInputV2>());
    closed_builder.add_lockfile(FrontendLockfileInputV2 {
        path: "acme.lock".to_owned(),
        source_id: SourceIdV2::from_raw(2),
        text: lock_text,
        location: FrontendAuthorityLocationV2 {
            source_id: SourceIdV2::from_raw(1),
            start: 2,
            end: 3,
        },
    });
    let host_relation_location = FrontendAuthorityLocationV2 {
        source_id: SourceIdV2::from_raw(1),
        start: 4,
        end: 5,
    };
    closed_builder.add_relation(FrontendRelationInputV2 {
        authority: FrontendRelationAuthorityV2::Host,
        provenance_namespace: "host".to_owned(),
        qualified_name: "app::records".to_owned(),
        lookup_name: "records".to_owned(),
        fields: vec![FrontendRelationFieldInputV2 {
            name: "host_only".to_owned(),
            scalar_type: FrontendScalarTypeV2::String,
            optional: false,
            location: host_relation_location,
        }],
        cardinality: FrontendRelationCardinalityV2::Set,
        provenance: "host".to_owned(),
        location: host_relation_location,
    });
    for (spelling, kind, start) in [
        ("a", GraphReferenceKindV2::Entity, 5),
        ("b", GraphReferenceKindV2::Entity, 6),
        ("mid", GraphReferenceKindV2::Entity, 7),
        ("pattern1", GraphReferenceKindV2::Pattern, 5),
    ] {
        closed_builder.add_graph_reference(FrontendGraphReferenceInputV2 {
            spelling: spelling.to_owned(),
            kind,
            entity_subtype: None,
            location: FrontendAuthorityLocationV2 {
                source_id: SourceIdV2::from_raw(1),
                start,
                end: start + 1,
            },
        });
    }
    let closed = closed_builder
        .finish()
        .expect("facade-only authority builder seals");
    assert_declaration_dependency_stack(&closed);
    let compiled = compile_source_document_v3(DOCUMENT_SOURCE, DOCUMENT_TEXT, &closed)
        .expect("facade-only closed authority compiles the probe source");
    assert_eq!(compiled.source_id(), DOCUMENT_SOURCE);
    assert_eq!(compiled.items().len(), 3);
    let pack_compiled =
        compile_source_document_v3(SourceIdV2::from_raw(10), PACK_DOCUMENT_TEXT, &closed)
            .expect("facade-only pack authority compiles USE PACK");
    assert_eq!(pack_compiled.items().len(), 1);
    assert!(matches!(
        pack_compiled.items().first(),
        Some(CompiledItemV2::NoPlan {
            reason: NoPlanReasonV2::MetaActivation,
            ..
        })
    ));

    // ---- V3 execution/composition handoff --------------------------------
    let composition =
        compile_source_document_v3(SourceIdV2::from_raw(11), COMPOSITION_DOCUMENT_TEXT, &closed)
            .expect("facade-only SAMPLE composition compiles");
    let repeated_composition =
        compile_source_document_v3(SourceIdV2::from_raw(11), COMPOSITION_DOCUMENT_TEXT, &closed)
            .expect("same V3 composition compiles deterministically");
    assert_eq!(composition.execution(), repeated_composition.execution());
    assert_exact_composition_handoff(&composition, "sampled", ExecutionSampleModeV3::Lexical, 2);
    let selected_item = composition.execution().items()[0].item_id();
    assert_eq!(
        composition.execution().item(selected_item),
        composition.execution().items().first()
    );

    let foreign_source_identity =
        compile_source_document_v3(SourceIdV2::from_raw(12), COMPOSITION_DOCUMENT_TEXT, &closed)
            .expect("the same source under another source identity compiles");
    assert_eq!(
        composition.execution().source_digest(),
        foreign_source_identity.execution().source_digest(),
        "source bytes retain their stable digest"
    );
    assert_ne!(
        composition.execution().contract_digest(),
        foreign_source_identity.execution().contract_digest(),
        "a foreign source identity cannot reuse the contract digest"
    );
    assert_ne!(
        composition.execution().qualification_digest(),
        foreign_source_identity.execution().qualification_digest(),
        "a foreign source identity cannot reuse the qualification digest"
    );
    let foreign_item = foreign_source_identity.execution().items()[0].item_id();
    assert!(composition.execution().item(foreign_item).is_none());

    assert_shared_composition_vectors();

    let local_binding = compile_source_document_v3(
        SourceIdV2::from_raw(13),
        LOCAL_BINDING_DOCUMENT_TEXT,
        &closed,
    )
    .expect("a local composition binding wins over the same-named host relation");
    assert_exact_composition_handoff(
        &local_binding,
        "records",
        ExecutionSampleModeV3::Semantic,
        3,
    );

    let multi_binding = compile_source_document_v3(
        SourceIdV2::from_raw(15),
        MULTI_BINDING_DOCUMENT_TEXT,
        &closed,
    )
    .expect("sequential multi-binding composition compiles through the facade");
    let multi = multi_binding.execution().items()[0]
        .composition()
        .expect("multi-binding composition payload");
    assert_eq!(multi.bindings().len(), 2);
    assert_eq!(multi.bindings()[0].binding().name(), "first");
    assert_eq!(multi.bindings()[0].binding().ordinal(), 0);
    assert_eq!(multi.bindings()[1].binding().name(), "second");
    assert_eq!(multi.bindings()[1].binding().ordinal(), 1);
    assert_eq!(multi.with_binding(), multi.bindings()[1].binding());
    assert!(matches!(
        multi.bindings()[0].producer(),
        ExecutionCompositionProducerV3::Sample { scope: None, .. }
    ));
    assert_eq!(
        multi.bindings()[1]
            .relation()
            .expect("second producer retains relation")
            .root()
            .children()[0]
            .source()
            .expect("second producer scans first binding")
            .name(),
        "first"
    );

    let scoped_sample = compile_source_document_v3(
        SourceIdV2::from_raw(16),
        SCOPED_SAMPLE_DOCUMENT_TEXT,
        &closed,
    )
    .expect("a SAMPLE scope can target a prior local binding");
    let scoped = scoped_sample.execution().items()[0]
        .composition()
        .expect("scoped composition payload");
    let ExecutionCompositionProducerV3::Sample {
        scope: Some(ExecutionGraphTargetV3::Composition {
            binding,
            capability,
        }),
        ..
    } = scoped.bindings()[1].producer()
    else {
        panic!("second SAMPLE retains its exact local scope")
    };
    assert_eq!(binding, scoped.bindings()[0].binding());
    assert_eq!(*capability, ExecutionGraphCapabilityV3::Scope);

    let graph_composition = compile_source_document_v3(
        SourceIdV2::from_raw(17),
        GRAPH_COMPOSITION_DOCUMENT_TEXT,
        &closed,
    )
    .expect("graph producer and graph body compile through the facade");
    let graph = graph_composition.execution().items()[0]
        .composition()
        .expect("graph composition payload");
    let ExecutionCompositionProducerV3::Graph { query: producer } =
        graph.bindings()[0].producer()
    else {
        panic!("graph producer payload")
    };
    assert_eq!(producer.kind(), ExecutionGraphKindV3::Path);
    assert_eq!(producer.depth(), 2);
    assert_eq!(
        producer.direction(),
        Some(ExecutionGraphDirectionV3::Backward)
    );
    assert_eq!(
        producer
            .references()
            .iter()
            .map(ExecutionGraphReferenceV3::role)
            .collect::<Vec<_>>(),
        [
            ExecutionGraphReferenceRoleV3::Source,
            ExecutionGraphReferenceRoleV3::Sink,
            ExecutionGraphReferenceRoleV3::Via,
        ]
    );
    let ExecutionCompositionBodyV3::Graph(body) = graph.body() else {
        panic!("graph body payload")
    };
    assert_eq!(body.kind(), ExecutionGraphKindV3::Reachability);
    assert_eq!(body.depth(), 3);
    assert_eq!(body.direction(), Some(ExecutionGraphDirectionV3::Both));
    assert!(matches!(
        body.references()[0].target(),
        ExecutionGraphTargetV3::Composition {
            capability: ExecutionGraphCapabilityV3::Entity,
            ..
        }
    ));

    let check_composition = compile_source_document_v3(
        SourceIdV2::from_raw(18),
        CHECK_COMPOSITION_DOCUMENT_TEXT,
        &closed,
    )
    .expect("mixed multi-binding CHECK composition compiles through the facade");
    let check = check_composition.execution().items()[0]
        .composition()
        .expect("CHECK composition payload");
    assert_eq!(check.bindings().len(), 2);
    assert_eq!(check.with_binding(), check.bindings()[1].binding());
    assert_eq!(check.body(), &ExecutionCompositionBodyV3::Check);

    let explain_composition = compile_source_document_v3(
        SourceIdV2::from_raw(19),
        EXPLAIN_COMPOSITION_DOCUMENT_TEXT,
        &closed,
    )
    .expect("mixed multi-binding EXPLAIN composition compiles through the facade");
    let explain = explain_composition.execution().items()[0]
        .composition()
        .expect("EXPLAIN composition payload");
    assert_eq!(explain.bindings().len(), 2);
    assert_eq!(explain.with_binding(), explain.bindings()[1].binding());
    assert_eq!(explain.body(), &ExecutionCompositionBodyV3::Explain);

    let typed_literal = compile_source_document_v3(
        SourceIdV2::from_raw(14),
        LITERAL_COMPOSITION_DOCUMENT_TEXT,
        &closed,
    )
    .expect("facade-only typed composition body compiles");
    let body = typed_literal.execution().items()[0]
        .composition()
        .expect("composition")
        .body_relation()
        .expect("relation body");
    assert!(body.expressions().iter().any(|expression| matches!(
        expression.shape(),
        ExecutionExpressionShapeV3::Literal(ExecutionLiteralV3::Int(42))
    )));

    // ---- format_source_document_v2: twice, deterministic, idempotent -------
    let first: Result<String, FrontendFailureV2> =
        format_source_document_v2(DOCUMENT_SOURCE, DOCUMENT_TEXT, &budget);
    let first = first.expect("the probe source formats under the canonical budget");
    let second: Result<String, FrontendFailureV2> =
        format_source_document_v2(DOCUMENT_SOURCE, DOCUMENT_TEXT, &budget);
    let second = second.expect("formatting is repeatable");
    assert_eq!(first, second, "one input formats identically twice");
    assert!(!first.is_empty(), "the canonical form is not empty");
    let canonical = first.trim_end_matches('\n');
    let third = format_source_document_v2(DOCUMENT_SOURCE, canonical, &budget)
        .expect("the canonical form re-parses");
    assert_eq!(first, third, "fmt(fmt(x)) == fmt(x) through the facade");

    // ---- analyze_source_document_v2 under the open (partial) authority -----
    let authority: Result<OpenAnalysisAuthorityV2, FrontendAuthorityErrorV2> =
        OpenAnalysisAuthorityV2::try_standard_v2();
    let authority = authority.expect("the standard open authority seals");
    let analyzed: Result<AnalysisDocumentV2, FrontendTransactionErrorV2> =
        analyze_source_document_v2(DOCUMENT_SOURCE, DOCUMENT_TEXT, &authority);
    let analyzed = analyzed.expect("a partial authority is not an error");
    assert_eq!(
        analyzed.source_id(),
        DOCUMENT_SOURCE,
        "analysis keeps the source identity"
    );
    assert_eq!(analyzed.status(), AnalysisStatusV2::Incomplete);
    assert_eq!(
        analyzed.order().len(),
        3,
        "the probe source authors three items"
    );
    assert_eq!(
        analyzed.items().len(),
        analyzed.order().len(),
        "order names every item"
    );
    assert!(
        analyzed.rule_program().is_none(),
        "a partial authority seals no rule program"
    );

    let first_id = analyzed.order()[0];
    assert!(
        analyzed.item(first_id).is_some(),
        "an authored identity resolves to its status"
    );

    assert_eq!(
        analyzed.unresolved().len(),
        3,
        "every blocked item carries one unresolved fact"
    );
    for fact in analyzed.unresolved() {
        assert_eq!(fact.code(), OPEN_AUTHORITY_INCOMPLETE_CODE_V2);
        let location: LocatedSpanV2 = fact.location();
        assert_eq!(
            location.source_id, DOCUMENT_SOURCE,
            "unresolved facts are source-anchored"
        );
        assert!(
            location.span.start <= location.span.end,
            "spans are well formed"
        );
    }

    let mut blocked = 0_usize;
    for item in analyzed.items() {
        match item {
            AnalysisItemV2::BlockedByDiagnostics(diagnostics) => {
                assert!(
                    !diagnostics.is_empty(),
                    "a blocked item names at least one diagnostic"
                );
                blocked += 1;
            }
            AnalysisItemV2::Complete(_) | AnalysisItemV2::NotApplicable(_) => {}
        }
    }
    assert_eq!(blocked, 3, "a partial authority blocks every authored item");
    let AnalysisItemV2::BlockedByDiagnostics(first_diagnostics) = &analyzed.items()[0] else {
        panic!("the first authored item is blocked under a partial authority");
    };
    let first_codes: Vec<&'static str> = first_diagnostics
        .iter()
        .map(FrontendDiagnosticV2::code)
        .collect();
    for expected in MISSING_AUTHORITY_CODES_V2 {
        assert!(
            first_codes.contains(&expected),
            "the blocked item must name {expected}: {first_codes:?}"
        );
    }

    // ---- diagnostic / failure / transaction vocabulary ---------------------
    let diagnostic = FrontendDiagnosticV2::new(
        FrontendStageV2::Resource,
        "external.consumer.probe",
        FrontendSeverityV2::Warning,
        LocatedSpanV2 {
            source_id: DOCUMENT_SOURCE,
            span: SpanV1::new(0, 1),
        },
        vec![LocatedSpanV2 {
            source_id: DOCUMENT_SOURCE,
            span: SpanV1::new(1, 2),
        }],
        "external consumer diagnostic probe",
    );
    assert_eq!(diagnostic.stage(), FrontendStageV2::Resource);
    assert_eq!(diagnostic.code(), "external.consumer.probe");
    assert_eq!(diagnostic.severity(), FrontendSeverityV2::Warning);
    assert_eq!(diagnostic.primary().span, SpanV1::new(0, 1));
    assert_eq!(diagnostic.primary().source_id.raw(), DOCUMENT_SOURCE.raw());
    assert_eq!(diagnostic.secondary().len(), 1);
    assert_eq!(diagnostic.message(), "external consumer diagnostic probe");

    let failure: FrontendFailureV2 = FrontendFailureV2::one(diagnostic.clone());
    assert_eq!(
        failure.diagnostics().len(),
        1,
        "one diagnostic in, one diagnostic out"
    );
    assert!(
        FrontendFailureV2::try_new(Vec::new()).is_err(),
        "a failure without a diagnostic is not constructible"
    );
    assert!(FrontendFailureV2::try_new(vec![diagnostic.clone()]).is_ok());

    let transaction_error: FrontendTransactionErrorV2 =
        FrontendTransactionErrorV2::Document(failure);
    match &transaction_error {
        FrontendTransactionErrorV2::Authority(error) => {
            assert!(
                !error.code().is_empty(),
                "an authority failure names its code"
            );
        }
        FrontendTransactionErrorV2::Document(failure) => {
            assert_eq!(failure.diagnostics().len(), 1);
        }
    }

    // ---- plan / no-plan vocabulary ----------------------------------------
    let reason: NoPlanReasonV2 = NoPlanReasonV2::ValueDeclaration;
    assert_eq!(no_plan_reason_label(reason), "value-declaration");
    assert_eq!(
        no_plan_reason_label(NoPlanReasonV2::MetaActivation),
        "meta-activation"
    );
    assert_eq!(
        no_plan_reason_label(NoPlanReasonV2::EmptyRuleProgram),
        "empty-rule-program"
    );
    let not_applicable: AnalysisItemV2 =
        AnalysisItemV2::NotApplicable(NoPlanReasonV2::ValueDeclaration);
    let blocked_item: AnalysisItemV2 =
        AnalysisItemV2::BlockedByDiagnostics(vec![diagnostic].into_boxed_slice());
    for item in [&not_applicable, &blocked_item] {
        let label = match item {
            AnalysisItemV2::Complete(_) => "complete",
            AnalysisItemV2::NotApplicable(_) => "not-applicable",
            AnalysisItemV2::BlockedByDiagnostics(diagnostics) => {
                assert!(!diagnostics.is_empty());
                "blocked"
            }
        };
        assert!(!label.is_empty());
    }

    // ---- closed algebra vocabulary ----------------------------------------
    assert_eq!(algebra_family_label(AlgebraFamilyV2::Scan), "scan");
    assert_eq!(algebra_family_label(AlgebraFamilyV2::AntiJoin), "anti-join");

    // ---- curated roots remain type-reachable ------------------------------
    pin_curated_roots();

    println!("external-consumer positive: ok");
}
