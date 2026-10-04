//! Declarative host inputs for closed frontend authority construction.

use super::{configuration_failure, FrontendAuthorityErrorV2, GraphReferenceKindV2};
use quanta_gqlang_v2_hir::{
    CheckedLocatedSpanV2, LocatedSpanV2, SourceIdV2, SourceRegistryViewV2, SpanV1,
};
use quanta_gqlang_v2_modules::{
    CallableKindV2, ExternalCatalogPayloadInputV2, LocatedParameterInputV2, SignatureInputV2,
    VersionV2,
};
use quanta_gqlang_v2_types::{
    CardinalityV1, ExpressionEffectsV2, RelationAuthorityV2, ScalarTyV1, ValueTypeV1,
    CANONICAL_WORKSPACE_FILES_FIELD_NAME_V2, CANONICAL_WORKSPACE_FILES_LOOKUP_NAME_V2,
    CANONICAL_WORKSPACE_FILES_PROVENANCE_NAMESPACE_V2, CANONICAL_WORKSPACE_FILES_PROVENANCE_V2,
    CANONICAL_WORKSPACE_FILES_QUALIFIED_NAME_V2,
};

/// Host-supplied symbol coordinates admitted into one closed authority.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrontendSymbolTargetInputV3 {
    pub selector: String,
    pub repo_path: String,
    pub symbol_id: String,
    pub start_byte: u64,
    pub end_byte: u64,
    pub location: FrontendAuthorityLocationV2,
}

/// Raw host source registration accepted by the curated authority builder.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrontendAuthoritySourceV2 {
    /// Stable source identity.
    pub source_id: SourceIdV2,
    /// Exact byte length of the registered host source.
    pub source_len: usize,
}

/// Raw host location checked against the builder's source registry at finish.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrontendAuthorityLocationV2 {
    /// Registered host source identity.
    pub source_id: SourceIdV2,
    /// Inclusive byte start.
    pub start: usize,
    /// Exclusive byte end.
    pub end: usize,
}

impl FrontendAuthorityLocationV2 {
    pub(super) fn check(
        self,
        sources: &SourceRegistryViewV2,
    ) -> Result<CheckedLocatedSpanV2, FrontendAuthorityErrorV2> {
        sources
            .try_check_location(LocatedSpanV2 {
                source_id: self.source_id,
                span: SpanV1::new(self.start, self.end),
            })
            .map_err(|error| configuration_failure(error.code(), error.to_string()))
    }
}

/// Host-agnostic scalar field vocabulary admitted by relation authority.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrontendScalarTypeV2 {
    Bool,
    Int,
    Float,
    String,
    Duration,
    Location,
    Path,
    RepoPath,
    EntityRef,
    SymbolRef,
    RuleRef,
    SourceScope,
    ProofTree,
}

impl FrontendScalarTypeV2 {
    pub(super) const fn owner(self) -> ScalarTyV1 {
        match self {
            Self::Bool => ScalarTyV1::Bool,
            Self::Int => ScalarTyV1::Int,
            Self::Float => ScalarTyV1::Float,
            Self::String => ScalarTyV1::String,
            Self::Duration => ScalarTyV1::Duration,
            Self::Location => ScalarTyV1::Location,
            Self::Path => ScalarTyV1::Path,
            Self::RepoPath => ScalarTyV1::RepoPath,
            Self::EntityRef => ScalarTyV1::EntityRef,
            Self::SymbolRef => ScalarTyV1::SymbolRef,
            Self::RuleRef => ScalarTyV1::RuleRef,
            Self::SourceScope => ScalarTyV1::SourceScope,
            Self::ProofTree => ScalarTyV1::ProofTree,
        }
    }
}

/// Relation multiplicity admitted by the curated authority builder.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrontendRelationCardinalityV2 {
    Set,
    Bag,
    Sequence,
}

/// Closed relation provenance admitted at the public frontend boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrontendRelationAuthorityV2 {
    /// An application-hosted relation with host-defined identity and schema.
    Host,
    /// The canonical workspace-file inventory relation.
    CanonicalWorkspaceFiles,
}

impl FrontendRelationAuthorityV2 {
    pub(super) const fn owner(self) -> RelationAuthorityV2 {
        match self {
            Self::Host => RelationAuthorityV2::Host,
            Self::CanonicalWorkspaceFiles => RelationAuthorityV2::CanonicalWorkspaceFiles,
        }
    }
}

impl FrontendRelationCardinalityV2 {
    pub(super) const fn owner(self) -> CardinalityV1 {
        match self {
            Self::Set => CardinalityV1::Set,
            Self::Bag => CardinalityV1::Bag,
            Self::Sequence => CardinalityV1::Sequence,
        }
    }
}

/// One relation field supplied through the public authority boundary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrontendRelationFieldInputV2 {
    pub name: String,
    pub scalar_type: FrontendScalarTypeV2,
    pub optional: bool,
    pub location: FrontendAuthorityLocationV2,
}

/// One host relation supplied through the public authority boundary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrontendRelationInputV2 {
    pub authority: FrontendRelationAuthorityV2,
    pub provenance_namespace: String,
    pub qualified_name: String,
    pub lookup_name: String,
    pub fields: Vec<FrontendRelationFieldInputV2>,
    pub cardinality: FrontendRelationCardinalityV2,
    pub provenance: String,
    pub location: FrontendAuthorityLocationV2,
}

impl FrontendRelationInputV2 {
    /// Construct the exact canonical workspace-file authority contract without
    /// requiring hosts to duplicate or infer its identity and row schema.
    #[must_use]
    pub fn canonical_workspace_files(
        location: FrontendAuthorityLocationV2,
        field_location: FrontendAuthorityLocationV2,
    ) -> Self {
        Self {
            authority: FrontendRelationAuthorityV2::CanonicalWorkspaceFiles,
            provenance_namespace: CANONICAL_WORKSPACE_FILES_PROVENANCE_NAMESPACE_V2.to_owned(),
            qualified_name: CANONICAL_WORKSPACE_FILES_QUALIFIED_NAME_V2.to_owned(),
            lookup_name: CANONICAL_WORKSPACE_FILES_LOOKUP_NAME_V2.to_owned(),
            fields: vec![FrontendRelationFieldInputV2 {
                name: CANONICAL_WORKSPACE_FILES_FIELD_NAME_V2.to_owned(),
                scalar_type: FrontendScalarTypeV2::RepoPath,
                optional: false,
                location: field_location,
            }],
            cardinality: FrontendRelationCardinalityV2::Set,
            provenance: CANONICAL_WORKSPACE_FILES_PROVENANCE_V2.to_owned(),
            location,
        }
    }
}

/// One graph selector supplied through the public authority boundary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrontendGraphReferenceInputV2 {
    pub spelling: String,
    pub kind: GraphReferenceKindV2,
    pub entity_subtype: Option<String>,
    pub location: FrontendAuthorityLocationV2,
}

/// Simple host catalog kinds admitted through the public authority builder.
/// Payload-bearing relation/predicate/model contracts remain owner-internal;
/// this vocabulary covers the opaque reference families authored by source.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FrontendCatalogPayloadV2 {
    Query,
    Check,
    Capability,
    ModelSource,
    PackSource,
    Rule {
        parameters: Vec<FrontendCallableParameterInputV2>,
        result: FrontendScalarTypeV2,
    },
}

impl FrontendCatalogPayloadV2 {
    pub(super) const fn tag(&self) -> &'static str {
        match self {
            Self::Query => "query",
            Self::Check => "check",
            Self::Capability => "capability",
            Self::ModelSource => "model_source",
            Self::PackSource => "pack_source",
            Self::Rule { .. } => "rule",
        }
    }

    pub(super) fn owner_payload(
        self,
        sources: &SourceRegistryViewV2,
        location: &CheckedLocatedSpanV2,
    ) -> Result<ExternalCatalogPayloadInputV2, FrontendAuthorityErrorV2> {
        match self {
            Self::Query => Ok(ExternalCatalogPayloadInputV2::Query),
            Self::Check => Ok(ExternalCatalogPayloadInputV2::Check),
            Self::Capability => Ok(ExternalCatalogPayloadInputV2::Capability),
            Self::ModelSource => Ok(ExternalCatalogPayloadInputV2::ModelSource),
            Self::PackSource => Ok(ExternalCatalogPayloadInputV2::PackSource),
            Self::Rule { parameters, result } => {
                Ok(ExternalCatalogPayloadInputV2::Rule(SignatureInputV2 {
                    parameters: parameters
                        .into_iter()
                        .map(|parameter| {
                            let mut value_type = ValueTypeV1::scalar(parameter.scalar_type.owner());
                            if parameter.optional {
                                value_type =
                                    ValueTypeV1::try_optional(value_type).map_err(|error| {
                                        configuration_failure(
                                            "sema.invalid_catalog_signature",
                                            error.to_string(),
                                        )
                                    })?;
                            }
                            Ok(LocatedParameterInputV2 {
                                name: parameter.name,
                                value_type,
                                location: parameter.location.check(sources)?,
                            })
                        })
                        .collect::<Result<Vec<_>, FrontendAuthorityErrorV2>>()?,
                    result: ValueTypeV1::scalar(result.owner()),
                    callable_kind: CallableKindV2::Rule,
                    effects: ExpressionEffectsV2::pure_scalar(),
                    location: location.clone(),
                }))
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrontendCallableParameterInputV2 {
    pub name: String,
    pub scalar_type: FrontendScalarTypeV2,
    pub optional: bool,
    pub location: FrontendAuthorityLocationV2,
}

/// One opaque external catalog asset supplied through the public authority
/// transaction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrontendCatalogAssetInputV2 {
    pub module_name: String,
    pub local_name: String,
    pub payload: FrontendCatalogPayloadV2,
    pub provenance: String,
    pub location: FrontendAuthorityLocationV2,
}

/// Semantic language version selected by the host authority.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrontendLanguageVersionV2 {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
}

/// One dependency-free pack manifest supplied through the public authority
/// boundary. Dependency/export/suite authoring remains an owner-level concern;
/// this DTO is the minimal closed input required for source-level `USE PACK`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrontendPackInputV2 {
    pub pack_id: String,
    pub version: FrontendLanguageVersionV2,
    pub content_digest: String,
    pub language_lower: FrontendLanguageVersionV2,
    pub language_upper: FrontendLanguageVersionV2,
    pub location: FrontendAuthorityLocationV2,
}

/// One canonical lockfile and the checked host declaration that names it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrontendLockfileInputV2 {
    pub path: String,
    pub source_id: SourceIdV2,
    pub text: String,
    pub location: FrontendAuthorityLocationV2,
}

impl FrontendLanguageVersionV2 {
    pub(super) const fn owner(self) -> VersionV2 {
        VersionV2::new(self.major, self.minor, self.patch)
    }
}
