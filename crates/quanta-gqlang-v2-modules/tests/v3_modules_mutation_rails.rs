//! Mutation-hardening rails: pin the exact observable behavior of accessor and
//! boundary code paths that cargo-mutants found unguarded. Each test fails if
//! the named mutation is applied.

#![allow(clippy::unwrap_used, reason = "tests may unwrap")]
#![allow(clippy::indexing_slicing, reason = "tests may index")]
#![allow(
    clippy::wildcard_enum_match_arm,
    reason = "tests match the expected diagnostic variant and panic on any other"
)]
#![allow(clippy::panic, reason = "tests assert via panic on wrong variant")]

use quanta_gqlang_v2_hir::{
    CatalogDeclHirV1, CatalogPayloadV1, HirNodeIdV1, QueryHirV1, RelationExprHirV1,
    RelationExprKindV1, RelationSourceV1, SpanV1, ViewBodyV1,
};
use quanta_gqlang_v2_modules::{
    compose_models, CatalogKindV1, CatalogV1, ModelFactV1, ModelPackV1, ModelSchemaV1,
    ModuleDiagnosticV1, ModuleGraphBuilderV1, PackGraphV1, PackIdV1, PackManifestV1,
    VersionRangeV1, VersionV1, VisibilityV1,
};

// --- helpers ----------------------------------------------------------------

fn span() -> SpanV1 {
    SpanV1::point(0)
}

fn decl(id: u32, name: &str, payload: CatalogPayloadV1) -> CatalogDeclHirV1 {
    CatalogDeclHirV1::new(HirNodeIdV1::from_raw(id), span(), name, span(), payload)
}

fn inline_view(id: u32, source_name: &str) -> CatalogPayloadV1 {
    let relation = RelationExprHirV1::new(
        HirNodeIdV1::from_raw(id),
        span(),
        RelationExprKindV1::Source(RelationSourceV1 {
            name: source_name.to_string(),
            alias: None,
        }),
    );
    let query = QueryHirV1::new(HirNodeIdV1::from_raw(id + 1), span(), relation, None);
    CatalogPayloadV1::View(ViewBodyV1::Inline(Box::new(query)))
}

fn compat() -> VersionRangeV1 {
    VersionRangeV1::new(VersionV1::new(3, 0, 0), VersionV1::new(4, 0, 0))
}

// --- catalog.rs:194 certified_required -> true ------------------------------

#[test]
fn certified_required_is_false_by_default() {
    // A fresh catalog has REQUIRE CERTIFIED OFF. Mutating the accessor to return
    // `true` flips this assertion.
    let catalog = CatalogV1::new();
    assert!(!catalog.certified_required());
}

// --- catalog.rs:200 asset -> None -------------------------------------------

#[test]
fn asset_returns_the_registered_declaration() {
    let mut catalog = CatalogV1::new();
    catalog
        .register(decl(1, "Edges", inline_view(2, "raw")))
        .unwrap();

    // The registered VIEW asset is borrowable by its stable id; mutating the
    // accessor to always return `None` breaks this.
    let asset = catalog.asset("view::Edges");
    assert!(asset.is_some());
    let asset = asset.unwrap();
    assert_eq!(asset.kind, CatalogKindV1::View);
    assert_eq!(asset.name, "Edges");
    // A missing id stays None (so the Some above is meaningful).
    assert!(catalog.asset("view::Missing").is_none());
}

// --- catalog.rs:212 is_empty -> true / false --------------------------------

#[test]
fn catalog_is_empty_tracks_registration() {
    let mut catalog = CatalogV1::new();
    // Empty initially: `is_empty -> false` mutation fails here.
    assert!(catalog.is_empty());

    catalog
        .register(decl(1, "Edges", inline_view(2, "raw")))
        .unwrap();
    // Non-empty after register: `is_empty -> true` mutation fails here.
    assert!(!catalog.is_empty());
    assert_eq!(catalog.len(), 1);
}

// --- extension.rs:91 match guard allowed.contains(&fact.value) -> true ------

#[test]
fn validate_rejects_value_outside_allowed_set() {
    // schema declares attr `kind` admits only `source`. A fact asserting `sink`
    // is a declared attribute with a DISALLOWED value: the guard
    // `allowed.contains(&fact.value)` must be false so the Some(_) reject fires.
    // Mutating the guard to `true` would silently accept it.
    let schema = ModelSchemaV1::new().allow("kind", "source");
    let pack = ModelPackV1::new("p", VersionV1::new(1, 0, 0))
        .with_fact(ModelFactV1::new("t", "kind", "sink"));

    let err = pack.validate(&schema).unwrap_err();
    assert_eq!(err.code(), "modules.invalid_model_data");
    match err {
        ModuleDiagnosticV1::InvalidModelData { pack, detail } => {
            assert_eq!(pack, "p");
            assert!(detail.contains("not in the schema's allowed set"));
        }
        other => panic!("expected InvalidModelData, got {other:?}"),
    }
}

#[test]
fn validate_accepts_value_inside_allowed_set() {
    // The matching positive: an allowed value must pass (so the guard is not
    // vacuously rejecting; pins the boundary on the true side).
    let schema = ModelSchemaV1::new().allow("kind", "source");
    let pack = ModelPackV1::new("p", VersionV1::new(1, 0, 0))
        .with_fact(ModelFactV1::new("t", "kind", "source"));
    pack.validate(&schema).unwrap();
}

// --- extension.rs:179 pack-pair ordering: > vs == vs >= --------------------

#[test]
fn model_conflict_values_render_in_first_seen_order_for_same_named_packs() {
    // Two packs with the SAME name "p" assert conflicting values for the same
    // (target, attribute). The first pack (processed first) supplies "aaa", the
    // second "bbb". Because the pack names are EQUAL, the reorder guard
    //   `packs_pair.first() > packs_pair.get(1)`
    // must be FALSE (strict `>` of equal names), so values stay [aaa, bbb].
    // Mutating `>` to `==` or `>=` makes the guard TRUE for equal names, which
    // would reverse the rendering to [bbb, aaa].
    let schema = ModelSchemaV1::new().allow("a", "aaa").allow("a", "bbb");
    let pack1 =
        ModelPackV1::new("p", VersionV1::new(1, 0, 0)).with_fact(ModelFactV1::new("t", "a", "aaa"));
    let pack2 =
        ModelPackV1::new("p", VersionV1::new(1, 0, 0)).with_fact(ModelFactV1::new("t", "a", "bbb"));

    let err = compose_models(&[pack1, pack2], &schema).unwrap_err();
    match err {
        ModuleDiagnosticV1::ModelConflict {
            target,
            attribute,
            values,
            packs,
        } => {
            assert_eq!(target, "t");
            assert_eq!(attribute, "a");
            // First-seen order preserved because names are equal (guard false).
            assert_eq!(values, vec!["aaa".to_string(), "bbb".to_string()]);
            assert_eq!(packs, vec!["p".to_string(), "p".to_string()]);
        }
        other => panic!("expected ModelConflict, got {other:?}"),
    }
}

// --- extension.rs:230 ComposedModelV1::is_empty -> true / false ------------

#[test]
fn composed_model_is_empty_tracks_facts() {
    let schema = ModelSchemaV1::new().allow("a", "v");

    // No packs -> empty composition: `is_empty -> false` mutation fails here.
    let empty = compose_models(&[], &schema).unwrap();
    assert!(empty.is_empty());
    assert_eq!(empty.len(), 0);

    // One fact -> non-empty: `is_empty -> true` mutation fails here.
    let pack =
        ModelPackV1::new("p", VersionV1::new(1, 0, 0)).with_fact(ModelFactV1::new("t", "a", "v"));
    let composed = compose_models(&[pack], &schema).unwrap();
    assert!(!composed.is_empty());
    assert_eq!(composed.len(), 1);
}

// --- graph.rs:238 resolve_export_origin ambiguity: origins.len() > 1 -------

#[test]
fn reexport_with_two_distinct_origins_is_ambiguous() {
    // `facade` re-exports `g` from BOTH `a::f` and `b::f` (two distinct declaring
    // origins). `query` imports `g` from `facade`. Following the re-export yields
    // two origins, so `origins.len() > 1` must hold and AmbiguousImport fires.
    // Mutating `> 1` to `< 1` skips the reject and silently picks one origin.
    let graph = ModuleGraphBuilderV1::new()
        .declare("a", "f", VisibilityV1::Public)
        .declare("b", "f", VisibilityV1::Public)
        .reexport("facade", "g", "a", "f")
        .reexport("facade", "g", "b", "f")
        .import("query", "g", "facade", "g");

    let err = graph.resolve().unwrap_err();
    assert_eq!(err.code(), "modules.ambiguous_import");
    match err {
        ModuleDiagnosticV1::AmbiguousImport { name, candidates } => {
            assert_eq!(name, "g");
            assert_eq!(candidates, vec!["a::f".to_string(), "b::f".to_string()]);
        }
        other => panic!("expected AmbiguousImport, got {other:?}"),
    }
}

// --- graph.rs:402 module_count -> 0 / 1 ------------------------------------

#[test]
fn module_count_reflects_distinct_modules() {
    // Two distinct modules participate (`lib` declares, `query` imports). The
    // frozen graph must report exactly 2. Mutating the accessor to `0` or `1`
    // breaks this exact-count assertion.
    let resolved = ModuleGraphBuilderV1::new()
        .declare("lib", "x", VisibilityV1::Public)
        .import("query", "x", "lib", "x")
        .resolve()
        .unwrap();
    assert_eq!(resolved.module_count(), 2);
}

// --- manifest.rs:196 len -> 1 ----------------------------------------------

#[test]
fn pack_graph_len_reflects_manifest_count() {
    // Empty graph has len 0 (kills `len -> 1`).
    let empty = PackGraphV1::new();
    assert_eq!(empty.len(), 0);

    // Two manifests -> len 2 (also kills `len -> 1`).
    let graph = PackGraphV1::new()
        .add_manifest(PackManifestV1::new(
            PackIdV1::new("a", VersionV1::new(1, 0, 0)),
            compat(),
        ))
        .unwrap()
        .add_manifest(PackManifestV1::new(
            PackIdV1::new("b", VersionV1::new(1, 0, 0)),
            compat(),
        ))
        .unwrap();
    assert_eq!(graph.len(), 2);
}

// --- manifest.rs:212 is_empty -> true / false ------------------------------

#[test]
fn pack_graph_is_empty_tracks_manifests() {
    // Empty initially: `is_empty -> false` mutation fails here.
    let empty = PackGraphV1::new();
    assert!(empty.is_empty());

    // Non-empty after add: `is_empty -> true` mutation fails here.
    let graph = PackGraphV1::new()
        .add_manifest(PackManifestV1::new(
            PackIdV1::new("a", VersionV1::new(1, 0, 0)),
            compat(),
        ))
        .unwrap();
    assert!(!graph.is_empty());
    assert_eq!(graph.len(), 1);
}
