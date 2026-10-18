//! Canonical execution wire encoding and source-bound digests.

use super::{
    AlgebraFamilyV2, ExecutionAuthorityV3, ExecutionBindingV3, ExecutionCallTargetV3,
    ExecutionCardinalityV3, ExecutionCatalogIdentityV3, ExecutionCheckV3, ExecutionCommandPhaseV3,
    ExecutionCompareMetricV3, ExecutionCompositionBodyV3, ExecutionCompositionProducerV3,
    ExecutionCompositionV3, ExecutionConfirmModeV3, ExecutionDetectionModeV3,
    ExecutionDeterminismV3, ExecutionDigestV3, ExecutionEffectV3, ExecutionEmitModeV3,
    ExecutionEvidenceModeV3, ExecutionEvidenceV3, ExecutionExistsPolarityV3,
    ExecutionExplainTargetV3, ExecutionExportFormatV3, ExecutionExpressionShapeV3,
    ExecutionFamilyV3, ExecutionGraphCapabilityV3, ExecutionGraphDedupIdentityV3,
    ExecutionGraphDirectionV3, ExecutionGraphKindV3, ExecutionGraphQueryV3,
    ExecutionGraphReferenceKindV3, ExecutionGraphReferenceRoleV3, ExecutionGraphRelationShapeV3,
    ExecutionGraphTargetV3, ExecutionGraphWitnessTieBreakV3, ExecutionItemV3, ExecutionJoinModeV3,
    ExecutionLiteralV3, ExecutionLogicalQuantifierV3, ExecutionNullOrderV3, ExecutionOperatorV3,
    ExecutionPolicyFamilyV3, ExecutionPresentationKindV3, ExecutionRelationAuthorityV3,
    ExecutionRelationV3, ExecutionRelationalOperatorV3, ExecutionRenameCommandV3,
    ExecutionSampleModeV3, ExecutionScalarTypeV3, ExecutionSchemaV3, ExecutionShowCategoryV3,
    ExecutionSortDirectionV3, ExecutionSourceKindV3, ExecutionTruthModeV3, ExecutionValueTypeV3,
    SemanticItemIdV2, SourceIdV2, SourceQualifiedNodeIdV2,
};
use sha2::{Digest as _, Sha256};

mod source_v3;

/// Canonical encoding rules (all three legs): domain-separated schema
/// version first, big-endian integers, length-prefixed strings and sequences,
/// one fixed tag byte per enum variant, every DTO field exactly once, and
/// explicitly ordered collections (source order, identity order, or
/// owner-stable order — never hash order). Derived display witnesses (operator
/// spellings) are encoded deliberately so the audit stays syntactic — one
/// `put_*` call per field, no semantic filtering; cross-version drift is gated
/// by the schema version instead.
pub(super) const SOURCE_DIGEST_SCHEMA_VERSION: u32 = 3;
// The Rust DTO is still Execution V3. Its contract/qualification encoding has
// an independent revision. Schema 8 adds Rename Command target and intent
// after the earlier relation, graph, and presentation payloads.
// The source byte stream itself did not change, so its stable schema remains 3.
pub(super) const CONTRACT_DIGEST_SCHEMA_VERSION: u32 = 8;

/// One canonical byte stream can be sent directly to the digest or, in tests,
/// captured for exact-byte comparisons. Writers below own field order and
/// framing; sinks own only byte delivery.
pub(super) trait CanonicalByteSink {
    fn write_bytes(&mut self, bytes: &[u8]);

    fn write_byte(&mut self, byte: u8) {
        self.write_bytes(&[byte]);
    }
}

impl CanonicalByteSink for Sha256 {
    fn write_bytes(&mut self, bytes: &[u8]) {
        self.update(bytes);
    }
}

#[cfg(test)]
impl CanonicalByteSink for Vec<u8> {
    fn write_bytes(&mut self, bytes: &[u8]) {
        self.extend_from_slice(bytes);
    }
}

/// Digest of the raw UTF-8 source bytes only.
pub(super) fn digest_source_bytes(source: &[u8]) -> ExecutionDigestV3 {
    let mut digest = Sha256::new();
    put_str(&mut digest, "qglang.execution.source-bytes.v3");
    put_u32(&mut digest, SOURCE_DIGEST_SCHEMA_VERSION);
    put_usize(&mut digest, source.len());
    digest.write_bytes(source);
    ExecutionDigestV3(finish_sha256(digest))
}

/// Digest of the canonical DTO encoding only.
pub(super) fn digest_contract(
    source_id: SourceIdV2,
    items: &[ExecutionItemV3],
) -> ExecutionDigestV3 {
    let mut digest = Sha256::new();
    write_contract(&mut digest, source_id, items);
    ExecutionDigestV3(finish_sha256(digest))
}

/// The sole schema-8 contract encoder, shared by production hashing and the
/// bounded test sink. No complete canonical stream is materialized here.
pub(super) fn write_contract(
    out: &mut impl CanonicalByteSink,
    source_id: SourceIdV2,
    items: &[ExecutionItemV3],
) {
    put_str(out, "qglang.execution.contract.v8");
    put_u32(out, CONTRACT_DIGEST_SCHEMA_VERSION);
    put_u32(out, source_id.raw());
    put_usize(out, items.len());
    for item in items {
        put_item_id(out, item.item_id);
        put_u8(out, family_tag(item.family));
        put_u8(out, authority_tag(item.plan.authority));
        put_u8(out, determinism_tag(item.plan.determinism));
        put_u8(out, evidence_tag(item.plan.evidence));
        put_u8(out, effect_tag(item.plan.effect));
        put_u8(out, policy_family_tag(item.plan.policy_family));
        if let Some(relation) = &item.relation {
            put_relation(out, relation);
        } else {
            put_u8(out, 0);
        }
        if let Some(composition) = &item.composition {
            put_composition(out, composition);
        } else {
            put_u8(out, 0);
        }
        match &item.graph {
            Some(graph) => {
                put_u8(out, 1);
                put_graph_query(out, graph);
            }
            None => put_u8(out, 0),
        }
        match &item.check {
            Some(check) => {
                put_u8(out, 1);
                put_check(out, check);
            }
            None => put_u8(out, 0),
        }
        match &item.presentation {
            Some(presentation) => {
                put_u8(out, 1);
                put_presentation(out, presentation);
            }
            None => put_u8(out, 0),
        }
        match &item.command {
            Some(command) => {
                put_u8(out, 1);
                put_command(out, command);
            }
            None => put_u8(out, 0),
        }
    }
}

/// Digest binding the contract domain, the schema version, and both legs.
pub(super) fn digest_qualification(
    source_id: SourceIdV2,
    source_digest: ExecutionDigestV3,
    contract_digest: ExecutionDigestV3,
) -> ExecutionDigestV3 {
    let mut digest = Sha256::new();
    put_str(&mut digest, "qglang.execution.qualification.v8");
    put_u32(&mut digest, CONTRACT_DIGEST_SCHEMA_VERSION);
    put_u32(&mut digest, source_id.raw());
    digest.write_bytes(source_digest.as_bytes());
    digest.write_bytes(contract_digest.as_bytes());
    ExecutionDigestV3(finish_sha256(digest))
}

fn put_command(out: &mut impl CanonicalByteSink, command: &ExecutionRenameCommandV3) {
    put_u8(
        out,
        match command.phase {
            ExecutionCommandPhaseV3::Preview => 0,
            ExecutionCommandPhaseV3::Apply => 1,
        },
    );
    put_str(out, command.target.selector());
    put_str(out, command.target.repo_path());
    put_str(out, command.target.symbol_id());
    out.write_bytes(&command.target.start_byte.to_be_bytes());
    out.write_bytes(&command.target.end_byte.to_be_bytes());
    put_str(out, &command.new_name);
    put_str(out, &command.expected_snapshot);
    match &command.preview_token {
        Some(token) => {
            put_u8(out, 1);
            put_str(out, token);
        }
        None => put_u8(out, 0),
    }
}

pub(super) const fn family_tag(value: ExecutionFamilyV3) -> u8 {
    match value {
        ExecutionFamilyV3::Command => 7,
        ExecutionFamilyV3::Relation => 0,
        ExecutionFamilyV3::Graph => 1,
        ExecutionFamilyV3::Check => 2,
        ExecutionFamilyV3::Presentation => 3,
        ExecutionFamilyV3::Composition => 4,
        ExecutionFamilyV3::MetaRequireCertified => 5,
        ExecutionFamilyV3::RuleProgram => 6,
    }
}

pub(super) const fn authority_tag(value: ExecutionAuthorityV3) -> u8 {
    match value {
        ExecutionAuthorityV3::FrontendChecked => 0,
        ExecutionAuthorityV3::CertifiedRequirementValidated => 1,
    }
}

pub(super) const fn determinism_tag(value: ExecutionDeterminismV3) -> u8 {
    match value {
        ExecutionDeterminismV3::Deterministic => 0,
        ExecutionDeterminismV3::NonDeterministic => 1,
    }
}

pub(super) const fn evidence_tag(value: ExecutionEvidenceV3) -> u8 {
    match value {
        ExecutionEvidenceV3::None => 0,
        ExecutionEvidenceV3::SourceLocation => 1,
        ExecutionEvidenceV3::WitnessPayload => 2,
        ExecutionEvidenceV3::ProofTree => 3,
    }
}

pub(super) const fn effect_tag(value: ExecutionEffectV3) -> u8 {
    match value {
        ExecutionEffectV3::ReadOnly => 0,
        ExecutionEffectV3::ProofEvidenceOnly => 1,
        ExecutionEffectV3::ExportOnly => 2,
        ExecutionEffectV3::MutationForbidden => 3,
        ExecutionEffectV3::MutationRequiresGrant => 4,
    }
}

pub(super) const fn policy_family_tag(value: ExecutionPolicyFamilyV3) -> u8 {
    match value {
        ExecutionPolicyFamilyV3::Relational => 0,
        ExecutionPolicyFamilyV3::Graph => 1,
        ExecutionPolicyFamilyV3::Check => 2,
        ExecutionPolicyFamilyV3::Presentation => 3,
        ExecutionPolicyFamilyV3::RuleProgram => 4,
        ExecutionPolicyFamilyV3::MetaRequireCertified => 5,
        ExecutionPolicyFamilyV3::Composition => 6,
        ExecutionPolicyFamilyV3::Command => 7,
    }
}

fn put_composition(out: &mut impl CanonicalByteSink, composition: &ExecutionCompositionV3) {
    put_u8(out, 1);
    put_usize(out, composition.bindings.len());
    for binding in &composition.bindings {
        put_binding(out, &binding.binding);
        put_item_id(out, binding.producer_item_id);
        put_str(out, binding.producer_family.keyword());
        if let Some(schema) = &binding.schema {
            put_u8(out, 1);
            put_schema(out, schema);
        } else {
            put_u8(out, 0);
        }
        if let Some(relation) = &binding.relation {
            put_relation(out, relation);
        } else {
            put_u8(out, 0);
        }
        put_composition_producer(out, &binding.producer);
    }
    put_binding(out, &composition.with_binding);
    put_item_id(out, composition.body_item_id);
    put_str(out, composition.body_family.keyword());
    if let Some(relation) = &composition.body_relation {
        put_relation(out, relation);
    } else {
        put_u8(out, 0);
    }
    put_composition_body(out, &composition.body);
}

fn put_composition_producer(
    out: &mut impl CanonicalByteSink,
    producer: &ExecutionCompositionProducerV3,
) {
    match producer {
        ExecutionCompositionProducerV3::Relation => put_u8(out, 0),
        ExecutionCompositionProducerV3::Sample {
            query_text,
            mode,
            limit,
            scope,
        } => {
            put_u8(out, 1);
            put_str(out, query_text);
            put_u8(
                out,
                match mode {
                    ExecutionSampleModeV3::Lexical => 0,
                    ExecutionSampleModeV3::Semantic => 1,
                    ExecutionSampleModeV3::Hybrid => 2,
                },
            );
            put_u32(out, *limit);
            put_optional_graph_target(out, scope.as_ref());
        }
        ExecutionCompositionProducerV3::Graph { query } => {
            put_u8(out, 2);
            put_graph_query(out, query);
        }
    }
}
fn put_composition_body(out: &mut impl CanonicalByteSink, body: &ExecutionCompositionBodyV3) {
    match body {
        ExecutionCompositionBodyV3::Relation => put_u8(out, 0),
        ExecutionCompositionBodyV3::Graph(query) => {
            put_u8(out, 1);
            put_graph_query(out, query);
        }
        ExecutionCompositionBodyV3::Check => put_u8(out, 2),
        ExecutionCompositionBodyV3::Explain => put_u8(out, 3),
    }
}

pub(super) fn put_graph_query(out: &mut impl CanonicalByteSink, query: &ExecutionGraphQueryV3) {
    put_u8(out, graph_kind_tag(query.kind));
    put_usize(out, query.references.len());
    for reference in &query.references {
        put_u8(out, graph_reference_role_tag(reference.role));
        put_graph_target(out, &reference.target);
    }
    put_u32(out, query.depth);
    match query.direction {
        Some(direction) => {
            put_u8(out, 1);
            put_u8(out, graph_direction_tag(direction));
        }
        None => put_u8(out, 0),
    }
    put_usize(out, query.summary_facets.len());
    for facet in &query.summary_facets {
        put_str(out, facet);
    }
    put_u8(out, graph_relation_shape_tag(query.relation_shape));
    put_u8(out, graph_dedup_identity_tag(query.dedup_identity));
    put_u32(out, query.max_witnesses);
    match query.max_path_length {
        Some(max_path_length) => {
            put_u8(out, 1);
            put_u32(out, max_path_length);
        }
        None => put_u8(out, 0),
    }
    put_u8(out, graph_witness_tie_break_tag(query.witness_tie_break));
}

fn put_catalog_identity(out: &mut impl CanonicalByteSink, identity: &ExecutionCatalogIdentityV3) {
    put_str(out, &identity.module);
    put_str(out, &identity.name);
}

fn put_check(out: &mut impl CanonicalByteSink, check: &ExecutionCheckV3) {
    put_catalog_identity(out, &check.rule);
    put_optional_graph_target(out, check.scope.as_ref());
    put_optional_graph_target(out, check.pattern.as_ref());
    match &check.nested_graph {
        Some(nested) => {
            put_u8(out, 1);
            put_item_id(out, nested.item_id);
            put_graph_query(out, &nested.query);
        }
        None => put_u8(out, 0),
    }
    put_u8(
        out,
        match check.confirm {
            ExecutionConfirmModeV3::Skip => 0,
            ExecutionConfirmModeV3::Prompt => 1,
            ExecutionConfirmModeV3::Require => 2,
        },
    );
    put_u8(
        out,
        match check.emit {
            ExecutionEmitModeV3::Findings => 0,
            ExecutionEmitModeV3::Sarif => 1,
        },
    );
    put_u8(
        out,
        match check.mode {
            ExecutionDetectionModeV3::PatternOnly => 0,
            ExecutionDetectionModeV3::SemanticOnly => 1,
            ExecutionDetectionModeV3::Hybrid => 2,
        },
    );
}

fn put_presentation(out: &mut impl CanonicalByteSink, presentation: &ExecutionPresentationKindV3) {
    match presentation {
        ExecutionPresentationKindV3::Explain {
            target,
            evidence,
            truth,
        } => {
            put_u8(out, 0);
            match target {
                ExecutionExplainTargetV3::Rule(rule) => {
                    put_u8(out, 0);
                    put_catalog_identity(out, rule);
                }
                ExecutionExplainTargetV3::Entity(entity) => {
                    put_u8(out, 1);
                    put_graph_target(out, entity);
                }
            }
            put_u8(
                out,
                match evidence {
                    ExecutionEvidenceModeV3::Auto => 0,
                    ExecutionEvidenceModeV3::Trace => 1,
                    ExecutionEvidenceModeV3::Citation => 2,
                    ExecutionEvidenceModeV3::Counterexample => 3,
                },
            );
            put_u8(
                out,
                match truth {
                    ExecutionTruthModeV3::Inspect => 0,
                    ExecutionTruthModeV3::Confirm => 1,
                    ExecutionTruthModeV3::Refute => 2,
                },
            );
        }
        ExecutionPresentationKindV3::Show { category } => {
            put_u8(out, 1);
            put_u8(
                out,
                match category {
                    ExecutionShowCategoryV3::Capabilities => 0,
                    ExecutionShowCategoryV3::Models => 1,
                    ExecutionShowCategoryV3::Views => 2,
                    ExecutionShowCategoryV3::Rules => 3,
                    ExecutionShowCategoryV3::Packs => 4,
                    ExecutionShowCategoryV3::Profiles => 5,
                    ExecutionShowCategoryV3::Gates => 6,
                },
            );
        }
        ExecutionPresentationKindV3::Describe { target } => {
            put_u8(out, 2);
            put_graph_target(out, target);
        }
        ExecutionPresentationKindV3::Sample {
            query_text,
            mode,
            limit,
            scope,
        } => {
            put_u8(out, 3);
            put_str(out, query_text);
            put_u8(
                out,
                match mode {
                    ExecutionSampleModeV3::Lexical => 0,
                    ExecutionSampleModeV3::Semantic => 1,
                    ExecutionSampleModeV3::Hybrid => 2,
                },
            );
            put_u32(out, *limit);
            put_optional_graph_target(out, scope.as_ref());
        }
        ExecutionPresentationKindV3::Compare {
            baseline,
            candidate,
            metric,
        } => {
            put_u8(out, 4);
            put_graph_target(out, baseline);
            put_graph_target(out, candidate);
            put_u8(
                out,
                match metric {
                    ExecutionCompareMetricV3::Exact => 0,
                    ExecutionCompareMetricV3::Structural => 1,
                    ExecutionCompareMetricV3::Semantic => 2,
                },
            );
        }
        ExecutionPresentationKindV3::Export {
            source,
            format,
            profile,
        } => {
            put_u8(out, 5);
            put_graph_target(out, source);
            put_u8(
                out,
                match format {
                    ExecutionExportFormatV3::Json => 0,
                    ExecutionExportFormatV3::Csv => 1,
                    ExecutionExportFormatV3::Sarif => 2,
                    ExecutionExportFormatV3::Markdown => 3,
                    ExecutionExportFormatV3::GraphMl => 4,
                },
            );
            match profile {
                Some(profile) => {
                    put_u8(out, 1);
                    put_catalog_identity(out, profile);
                }
                None => put_u8(out, 0),
            }
        }
    }
}

fn put_optional_graph_target(
    out: &mut impl CanonicalByteSink,
    target: Option<&ExecutionGraphTargetV3>,
) {
    match target {
        Some(target) => {
            put_u8(out, 1);
            put_graph_target(out, target);
        }
        None => put_u8(out, 0),
    }
}

fn put_graph_target(out: &mut impl CanonicalByteSink, target: &ExecutionGraphTargetV3) {
    match target {
        ExecutionGraphTargetV3::Host {
            kind,
            spelling,
            entity_subtype,
        } => {
            put_u8(out, 0);
            put_u8(out, graph_reference_kind_tag(*kind));
            put_str(out, spelling);
            match entity_subtype {
                Some(subtype) => {
                    put_u8(out, 1);
                    put_str(out, subtype);
                }
                None => put_u8(out, 0),
            }
        }
        ExecutionGraphTargetV3::Composition {
            binding,
            capability,
        } => {
            put_u8(out, 1);
            put_binding(out, binding);
            put_u8(
                out,
                match capability {
                    ExecutionGraphCapabilityV3::Entity => 0,
                    ExecutionGraphCapabilityV3::Scope => 1,
                },
            );
        }
    }
}

pub(super) const fn graph_direction_tag(direction: ExecutionGraphDirectionV3) -> u8 {
    match direction {
        ExecutionGraphDirectionV3::Forward => 0,
        ExecutionGraphDirectionV3::Backward => 1,
        ExecutionGraphDirectionV3::Both => 2,
        ExecutionGraphDirectionV3::Undirected => 3,
    }
}

pub(super) const fn graph_relation_shape_tag(shape: ExecutionGraphRelationShapeV3) -> u8 {
    match shape {
        ExecutionGraphRelationShapeV3::Acyclic => 0,
        ExecutionGraphRelationShapeV3::Cyclic => 1,
    }
}

pub(super) const fn graph_dedup_identity_tag(identity: ExecutionGraphDedupIdentityV3) -> u8 {
    match identity {
        ExecutionGraphDedupIdentityV3::Endpoints => 0,
        ExecutionGraphDedupIdentityV3::NodeSequence => 1,
        ExecutionGraphDedupIdentityV3::NodeSet => 2,
    }
}

pub(super) const fn graph_witness_tie_break_tag(tie_break: ExecutionGraphWitnessTieBreakV3) -> u8 {
    match tie_break {
        ExecutionGraphWitnessTieBreakV3::ShortestThenLexical => 0,
        ExecutionGraphWitnessTieBreakV3::Lexical => 1,
    }
}

pub(super) const fn graph_reference_role_tag(role: ExecutionGraphReferenceRoleV3) -> u8 {
    match role {
        ExecutionGraphReferenceRoleV3::Source => 0,
        ExecutionGraphReferenceRoleV3::Sink => 1,
        ExecutionGraphReferenceRoleV3::Anchor => 2,
        ExecutionGraphReferenceRoleV3::Reference => 3,
        ExecutionGraphReferenceRoleV3::Pointee => 4,
        ExecutionGraphReferenceRoleV3::Callable => 5,
        ExecutionGraphReferenceRoleV3::Via => 6,
        ExecutionGraphReferenceRoleV3::Through => 7,
        ExecutionGraphReferenceRoleV3::Sanitizer => 8,
        ExecutionGraphReferenceRoleV3::Scope => 9,
        ExecutionGraphReferenceRoleV3::Window => 10,
        ExecutionGraphReferenceRoleV3::Frontier => 11,
    }
}

pub(super) const fn graph_reference_kind_tag(kind: ExecutionGraphReferenceKindV3) -> u8 {
    match kind {
        ExecutionGraphReferenceKindV3::Entity => 0,
        ExecutionGraphReferenceKindV3::Callable => 1,
        ExecutionGraphReferenceKindV3::Scope => 2,
        ExecutionGraphReferenceKindV3::Window => 3,
        ExecutionGraphReferenceKindV3::Frontier => 4,
        ExecutionGraphReferenceKindV3::Sanitizer => 5,
        ExecutionGraphReferenceKindV3::Pattern => 6,
        ExecutionGraphReferenceKindV3::Snapshot => 7,
        ExecutionGraphReferenceKindV3::ExportSource => 8,
    }
}
pub(super) const fn graph_kind_tag(kind: ExecutionGraphKindV3) -> u8 {
    match kind {
        ExecutionGraphKindV3::Reachability => 0,
        ExecutionGraphKindV3::Flow => 1,
        ExecutionGraphKindV3::Slice => 2,
        ExecutionGraphKindV3::Taint => 3,
        ExecutionGraphKindV3::Path => 4,
        ExecutionGraphKindV3::PointsTo => 5,
        ExecutionGraphKindV3::Summary => 6,
    }
}

fn put_relation(out: &mut impl CanonicalByteSink, relation: &ExecutionRelationV3) {
    put_u8(out, 1);
    put_item_id(out, relation.item_id);
    put_node(out, &relation.root);
    put_usize(out, relation.expressions.len());
    for expr in &relation.expressions {
        put_key(out, expr.id);
        put_value_type(out, &expr.value_type);
        put_expression_shape(out, &expr.shape);
    }
}
pub(super) fn put_node(out: &mut impl CanonicalByteSink, node: &ExecutionRelationalOperatorV3) {
    put_u8(out, algebra_family_tag(node.family));
    if let Some(count) = node.limit {
        // The extension marker is disjoint from the following join-mode tag.
        // Nodes without a Limit retain their existing canonical bytes.
        put_u8(out, 255);
        out.write_bytes(&count.to_be_bytes());
    }
    match node.join_mode {
        Some(mode) => {
            put_u8(out, 1);
            put_u8(out, join_mode_tag(mode));
        }
        None => put_u8(out, 0),
    }
    put_key(out, node.stage);
    source_v3::put_source_v3(out, node.source.as_ref());
    put_schema(out, &node.schema);
    put_usize(out, node.predicates.len());
    for predicate in &node.predicates {
        put_key(out, *predicate);
    }
    put_usize(out, node.expressions.len());
    for key in &node.expressions {
        put_key(out, *key);
    }
    put_usize(out, node.correlations.len());
    for edge in &node.correlations {
        put_binding(out, &edge.binding);
        put_key(out, edge.reference);
    }
    if node.family == AlgebraFamilyV2::ExistsFilter {
        put_usize(out, node.existential_terms.len());
        for term in &node.existential_terms {
            put_key(out, term.expression);
            put_u8(out, exists_polarity_tag(term.polarity));
            put_usize(out, term.correlations.len());
            for edge in &term.correlations {
                put_binding(out, &edge.binding);
                put_key(out, edge.reference);
            }
        }
    }
    put_usize(out, node.partition_input.len());
    for binding in &node.partition_input {
        put_binding(out, binding);
    }
    put_usize(out, node.children.len());
    for child in &node.children {
        put_node(out, child);
    }
}

const fn relation_authority_tag(authority: ExecutionRelationAuthorityV3) -> u8 {
    match authority {
        ExecutionRelationAuthorityV3::Host => 0,
        ExecutionRelationAuthorityV3::CanonicalWorkspaceFiles => 1,
        ExecutionRelationAuthorityV3::CatalogView => 2,
        ExecutionRelationAuthorityV3::CompositionBinding => 3,
    }
}

fn put_expression_shape(out: &mut impl CanonicalByteSink, shape: &ExecutionExpressionShapeV3) {
    match shape {
        ExecutionExpressionShapeV3::Literal(value) => {
            put_u8(out, 0);
            put_literal(out, value);
        }
        ExecutionExpressionShapeV3::Binding(binding) => {
            put_u8(out, 1);
            put_binding(out, binding);
        }
        ExecutionExpressionShapeV3::Path { base, segments } => {
            put_u8(out, 2);
            put_key(out, *base);
            put_usize(out, segments.len());
            for segment in segments {
                put_str(out, segment);
            }
        }
        ExecutionExpressionShapeV3::Unary { operator, operand } => {
            put_u8(out, 3);
            put_operator(out, operator);
            put_key(out, *operand);
        }
        ExecutionExpressionShapeV3::Binary { operator, lhs, rhs } => {
            put_u8(out, 4);
            put_operator(out, operator);
            put_key(out, *lhs);
            put_key(out, *rhs);
        }
        ExecutionExpressionShapeV3::Call { target, arguments } => {
            put_u8(out, 5);
            put_call_target(out, target);
            put_usize(out, arguments.len());
            for argument in arguments {
                if let Some(id) = argument {
                    put_u8(out, 1);
                    put_key(out, *id);
                } else {
                    put_u8(out, 0);
                }
            }
        }
        ExecutionExpressionShapeV3::Paren(id) => {
            put_u8(out, 6);
            put_key(out, *id);
        }
        ExecutionExpressionShapeV3::Record(fields) => {
            put_u8(out, 7);
            put_usize(out, fields.len());
            for (name, id) in fields {
                put_str(out, name);
                put_key(out, *id);
            }
        }
        ExecutionExpressionShapeV3::List(items) => {
            put_u8(out, 8);
            put_usize(out, items.len());
            for id in items {
                put_key(out, *id);
            }
        }
        ExecutionExpressionShapeV3::BoolValue(id) => {
            put_u8(out, 9);
            put_key(out, *id);
        }
        ExecutionExpressionShapeV3::Compare { operator, lhs, rhs } => {
            put_u8(out, 10);
            put_operator(out, operator);
            put_key(out, *lhs);
            put_key(out, *rhs);
        }
        ExecutionExpressionShapeV3::And { operator, lhs, rhs } => {
            put_u8(out, 11);
            put_operator(out, operator);
            put_key(out, *lhs);
            put_key(out, *rhs);
        }
        ExecutionExpressionShapeV3::Or { operator, lhs, rhs } => {
            put_u8(out, 12);
            put_operator(out, operator);
            put_key(out, *lhs);
            put_key(out, *rhs);
        }
        ExecutionExpressionShapeV3::Not { operator, operand } => {
            put_u8(out, 13);
            put_operator(out, operator);
            put_key(out, *operand);
        }
        ExecutionExpressionShapeV3::LogicalQuantifier {
            kind,
            binders,
            body,
        } => {
            put_u8(out, 14);
            put_u8(out, logical_quantifier_tag(*kind));
            put_usize(out, binders.len());
            for binding in binders {
                put_binding(out, binding);
            }
            put_key(out, *body);
        }
        ExecutionExpressionShapeV3::RelationExists {
            polarity,
            relation_root,
            correlations,
        } => {
            put_u8(out, 15);
            put_u8(out, exists_polarity_tag(*polarity));
            put_key(out, *relation_root);
            put_usize(out, correlations.len());
            for edge in correlations {
                put_binding(out, &edge.binding);
                put_key(out, edge.reference);
            }
        }
    }
}
pub(super) fn put_literal(out: &mut impl CanonicalByteSink, literal: &ExecutionLiteralV3) {
    match literal {
        ExecutionLiteralV3::Int(value) => {
            put_u8(out, 0);
            out.write_bytes(&value.to_be_bytes());
        }
        ExecutionLiteralV3::Float(value) => {
            put_u8(out, 1);
            out.write_bytes(&value.bits().to_be_bytes());
        }
        ExecutionLiteralV3::Bool(value) => {
            put_u8(out, 2);
            put_u8(out, u8::from(*value));
        }
        ExecutionLiteralV3::String(value) => {
            put_u8(out, 3);
            put_str(out, value);
        }
        ExecutionLiteralV3::Null => put_u8(out, 4),
    }
}
fn put_schema(out: &mut impl CanonicalByteSink, schema: &ExecutionSchemaV3) {
    put_usize(out, schema.fields.len());
    for field in &schema.fields {
        put_str(out, &field.name);
        put_value_type(out, &field.value_type);
    }
    put_u8(out, cardinality_tag(schema.cardinality));
    put_usize(out, schema.partition_keys.len());
    for key in &schema.partition_keys {
        put_key(out, *key);
    }
    put_usize(out, schema.order.len());
    for key in &schema.order {
        put_key(out, key.expression);
        put_u8(out, sort_direction_tag(key.direction));
        put_u8(out, null_order_tag(key.null_order));
        put_value_type(out, &key.value_type);
    }
}

fn put_value_type(out: &mut impl CanonicalByteSink, value: &ExecutionValueTypeV3) {
    match value {
        ExecutionValueTypeV3::Scalar(scalar) => {
            put_u8(out, 0);
            put_u8(out, scalar_tag(*scalar));
        }
        ExecutionValueTypeV3::Null => put_u8(out, 1),
        ExecutionValueTypeV3::Tuple(items) => {
            put_u8(out, 2);
            put_usize(out, items.len());
            for item in items {
                put_value_type(out, item);
            }
        }
        ExecutionValueTypeV3::Record(fields) => {
            put_u8(out, 3);
            put_usize(out, fields.len());
            for field in fields {
                put_str(out, field.name());
                put_value_type(out, field.value_type());
            }
        }
        ExecutionValueTypeV3::Optional(inner) => {
            put_u8(out, 4);
            put_value_type(out, inner);
        }
        ExecutionValueTypeV3::Collection {
            element,
            cardinality,
        } => {
            put_u8(out, 5);
            put_value_type(out, element);
            put_u8(out, cardinality_tag(*cardinality));
        }
        ExecutionValueTypeV3::Union(variants) => {
            put_u8(out, 6);
            put_usize(out, variants.len());
            for variant in variants {
                put_str(out, variant.tag());
                put_value_type(out, variant.value_type());
            }
        }
        ExecutionValueTypeV3::Unknown => put_u8(out, 7),
    }
}

pub(super) const fn scalar_tag(value: ExecutionScalarTypeV3) -> u8 {
    match value {
        ExecutionScalarTypeV3::Bool => 0,
        ExecutionScalarTypeV3::Int => 1,
        ExecutionScalarTypeV3::Float => 2,
        ExecutionScalarTypeV3::String => 3,
        ExecutionScalarTypeV3::Duration => 4,
        ExecutionScalarTypeV3::Location => 5,
        ExecutionScalarTypeV3::Path => 6,
        ExecutionScalarTypeV3::RepoPath => 7,
        ExecutionScalarTypeV3::EntityRef => 8,
        ExecutionScalarTypeV3::SymbolRef => 9,
        ExecutionScalarTypeV3::RuleRef => 10,
        ExecutionScalarTypeV3::SourceScope => 11,
        ExecutionScalarTypeV3::ProofTree => 12,
    }
}

pub(super) const fn cardinality_tag(value: ExecutionCardinalityV3) -> u8 {
    match value {
        ExecutionCardinalityV3::Set => 0,
        ExecutionCardinalityV3::Bag => 1,
        ExecutionCardinalityV3::Sequence => 2,
    }
}

pub(super) const fn sort_direction_tag(value: ExecutionSortDirectionV3) -> u8 {
    match value {
        ExecutionSortDirectionV3::Asc => 0,
        ExecutionSortDirectionV3::Desc => 1,
    }
}

pub(super) const fn null_order_tag(value: ExecutionNullOrderV3) -> u8 {
    match value {
        ExecutionNullOrderV3::First => 0,
        ExecutionNullOrderV3::Last => 1,
    }
}

pub(super) const fn algebra_family_tag(value: AlgebraFamilyV2) -> u8 {
    match value {
        AlgebraFamilyV2::Scan => 0,
        AlgebraFamilyV2::Filter => 1,
        AlgebraFamilyV2::Project => 2,
        AlgebraFamilyV2::Join => 3,
        AlgebraFamilyV2::Group => 4,
        AlgebraFamilyV2::Aggregate => 5,
        AlgebraFamilyV2::Sort => 6,
        AlgebraFamilyV2::SemiJoin => 7,
        AlgebraFamilyV2::AntiJoin => 8,
        AlgebraFamilyV2::ExistsFilter => 9,
        AlgebraFamilyV2::Limit => 10,
    }
}

pub(super) const fn join_mode_tag(value: ExecutionJoinModeV3) -> u8 {
    match value {
        ExecutionJoinModeV3::Inner => 0,
        ExecutionJoinModeV3::Cross => 1,
    }
}

pub(super) const fn source_kind_tag(value: ExecutionSourceKindV3) -> u8 {
    match value {
        ExecutionSourceKindV3::Relation => 0,
        ExecutionSourceKindV3::View => 1,
        ExecutionSourceKindV3::Composition => 2,
    }
}

fn put_call_target(out: &mut impl CanonicalByteSink, target: &ExecutionCallTargetV3) {
    match target {
        ExecutionCallTargetV3::Builtin { name } => {
            put_u8(out, 0);
            put_str(out, name);
        }
        ExecutionCallTargetV3::Predicate { name } => {
            put_u8(out, 1);
            put_str(out, name);
        }
        ExecutionCallTargetV3::HostPredicate { fingerprint } => {
            put_u8(out, 2);
            put_str(out, fingerprint);
        }
    }
}

pub(super) const fn logical_quantifier_tag(value: ExecutionLogicalQuantifierV3) -> u8 {
    match value {
        ExecutionLogicalQuantifierV3::Exists => 0,
        ExecutionLogicalQuantifierV3::Forall => 1,
    }
}

pub(super) const fn exists_polarity_tag(value: ExecutionExistsPolarityV3) -> u8 {
    match value {
        ExecutionExistsPolarityV3::Exists => 0,
        ExecutionExistsPolarityV3::NotExists => 1,
    }
}
fn put_binding(out: &mut impl CanonicalByteSink, binding: &ExecutionBindingV3) {
    put_u32(out, binding.source_id.raw());
    put_u32(out, binding.owner_node);
    put_usize(out, binding.ordinal);
    put_str(out, &binding.name);
}
fn put_operator(out: &mut impl CanonicalByteSink, operator: &ExecutionOperatorV3) {
    put_str(out, &operator.symbol);
    put_u32(out, operator.overload_id);
}
fn put_item_id(out: &mut impl CanonicalByteSink, id: SemanticItemIdV2) {
    put_u32(out, id.source_id().raw());
    put_u32(out, id.hir_item_id().raw());
}
fn put_key(out: &mut impl CanonicalByteSink, key: SourceQualifiedNodeIdV2) {
    put_u32(out, key.source_id.raw());
    put_u32(out, key.node_id.raw());
}
fn put_str(out: &mut impl CanonicalByteSink, value: &str) {
    put_usize(out, value.len());
    out.write_bytes(value.as_bytes());
}
fn put_usize(out: &mut impl CanonicalByteSink, value: usize) {
    // Total conversion: lengths beyond `u64::MAX` saturate (unreachable on any
    // real target; the digest writer must stay total regardless of platform).
    out.write_bytes(&u64::try_from(value).unwrap_or(u64::MAX).to_be_bytes());
}
fn put_u32(out: &mut impl CanonicalByteSink, value: u32) {
    out.write_bytes(&value.to_be_bytes());
}
fn put_u8(out: &mut impl CanonicalByteSink, value: u8) {
    out.write_byte(value);
}

/// Finalize a streamed canonical digest via the audited `sha2` crate.
pub(super) fn finish_sha256(hasher: Sha256) -> [u8; 32] {
    let digest = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&digest);
    out
}

#[cfg(test)]
pub(super) fn sha256(input: &[u8]) -> [u8; 32] {
    finish_sha256(Sha256::new().chain_update(input))
}
