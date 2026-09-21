//! QGLang v3 typed HIR single source of truth (ticket 01).
//!
//! This crate is the architectural ROOT of the v3 language core. It owns the
//! shared, generic HIR node taxonomy embedded by every later surface (query,
//! declaration, rule, composition, result shaping). Cites packet decisions:
//!
//! - `D2`: generic HIR is the first shared semantic authority after syntax AST;
//!   value/bool/relation nodes are shared, never root-local.
//! - `D29`: [`ResultShapeHirV1`] is the only frontend-owned nested-result-shape
//!   structure; no second shape tree may appear later.
//!
//! What this crate fixes is SHAPE plus identity: every node carries a unique
//! [`HirNodeIdV1`] and a [`SpanV1`] byte range. What it deliberately does NOT do
//! (owned by later tickets):
//!
//! - closed type legality / overload resolution (ticket 03),
//! - binding / scope resolution (ticket 02),
//! - algebra normalization / join semantics (ticket 04),
//! - recursion / least-fixpoint semantics (ticket 05),
//! - planning / truth / effects (ticket 07).
//!
//! Operators, callees, composition tags, and quantifiers are therefore OPEN
//! symbolic tags here; their legality is a typed-later attachment point.
//!
//! # Building trees
//!
//! [`HirBuilderV1`] offers ergonomic id-allocating constructors; the v3 parser
//! lowers surface text through one builder, so trees reached through a public
//! compile entry point satisfy the id-uniqueness and span invariants by
//! construction. [`check_tree`] is the fail-closed checked constructor for the
//! other case — HIR assembled **by hand** by an embedder, a downstream frontend,
//! or a test fixture. It enforces unique ids and in-bounds, well-formed spans
//! and returns typed [`HirIntegrityErrorV1`] values (or a [`CheckedHirV1`] that
//! witnesses them). The compiler does not call it on its own output, and nothing
//! downstream re-validates a hand-built tree, so skipping it is a real
//! correctness gap rather than a formality.

mod build;
mod expr;
mod id;
mod integrity;
mod item;
mod relation;
mod shape;
mod span;
mod unit;
mod visit;

pub use build::HirBuilderV1;
pub use id::{HirNodeIdGen, HirNodeIdV1};
pub use span::{
    CheckedLocatedSpanV2, ExtendedSourceRegistryViewV2, LocatedSpanErrorV2, LocatedSpanV2,
    SourceIdV2, SourceRegistrationV2, SourceRegistryDiagnosticV2, SourceRegistryIdentityV2,
    SourceRegistryViewV2, SpanV1,
};

pub use expr::{
    BinaryBoolV1, BinaryExprV1, BinaryOpV1, BoolExprHirV1, BoolExprKindV1, CallArgHirV1,
    CallArgsHirV1, CallExprV1, CalleeRefV1, CompareExprV1, CompareOpV1, ListExprV1, LiteralV1,
    LogicalBinderV1, LogicalQuantifierHirV1, LogicalQuantifierKindV1, NonEmptyBindersV1,
    NonEmptyPathSegmentsV1, PathAccessV1, PathSegmentV1, RecordExprV1, RecordFieldV1,
    RelationExistsHirV1, RelationExistsPolarityV1, SymbolRefV1, UnaryExprV1, UnaryOpV1,
    ValueExprHirV1, ValueExprKindV1,
};

pub use relation::{
    AliasV1, GroupByHirV1, JoinHirV1, NonEmptyProjectionItemsV1, NonEmptyRelationKeysV1,
    OrderByHirV1, OrderKeyHirV1, ProjectionItemV1, ProjectionV1, RelationExprHirV1,
    RelationExprKindV1, RelationFilterV1, RelationSourceV1, SortDirectionHirV1,
};

pub use shape::{ResultShapeHirV1, ResultShapeKindV1, ShapeFieldV1, ShapeListV1, ShapeRecordV1};

pub use unit::{DeclHirV1, QueryHirV1, RuleHirV1};

pub use item::{
    ActivationV1, AdmissionV1, CatalogDeclHirV1, CatalogPayloadV1, CompareMetricV1,
    CompositionBindingV1, CompositionBodyV1, CompositionHirV1, CompositionProducerV1,
    ConfirmModeV1, DetectionModeV1, EmitModeV1, EvidenceModeV1, ExplainHirV1, ExportFormatV1,
    GateBindingV1, GraphConstraintsV1, GraphDirectionV1, GraphEndpointsV1, GraphQueryHirV1,
    GraphQueryKindV1, GraphRefV1, LegalityHirV1, MetaHirV1, MetaKindV1, PresentationHirV1,
    PresentationKindV1, ProgramItemHirV1, SampleHirV1, SampleModeV1, ShowCategoryV1, TruthModeV1,
    UsePackActivationV1, UsePackHirV1, ViewBodyV1,
};

pub use visit::{
    walk_bool, walk_catalog_decl, walk_composition, walk_decl, walk_explain, walk_graph_query,
    walk_item, walk_legality, walk_meta, walk_presentation, walk_query, walk_relation, walk_rule,
    walk_sample, walk_shape, walk_value, HirVisitorV1,
};

pub use integrity::{
    check_document_v2, check_tree, CheckedHirDocumentOwnedV2, CheckedHirDocumentV2, CheckedHirV1,
    HirDocumentV1, HirIntegrityDiagnosticV2, HirIntegrityErrorV1, HirRootRefV1,
};
