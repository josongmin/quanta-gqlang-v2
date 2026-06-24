//! Hellgate (fail-closed) rails for the DECLARATION + META catalog seam of
//! ticket 11: VIEW/MODEL/PACK/PROFILE/GATE/RULE registration + SHOW-style
//! enumeration, USE PACK / REQUIRE CERTIFIED meta semantics.
//!
//! FRONTEND-CONTRACT parity: declarations PARSE (upstream) -> BIND into a
//! catalog of stable `<kind>::<name>` assets -> resolve fail-closed under the v2
//! admissibility rules. Declarations produce NO plan. These rails pin the happy
//! path (register + enumerate) and every fail-closed reject, asserting the
//! ACTUAL typed diagnostic code + Display text. They fail red before the
//! implementation exists and must stay red on any silent-accept / last-wins
//! regression.

#![allow(clippy::unwrap_used, reason = "tests may unwrap")]
#![allow(clippy::indexing_slicing, reason = "tests may index")]
#![allow(clippy::panic, reason = "tests assert via panic on wrong variant")]
#![allow(
    clippy::wildcard_enum_match_arm,
    reason = "tests match the expected diagnostic variant and panic on any other"
)]

use quanta_gqlang_v2_hir::{
    ActivationV1, AdmissionV1, CatalogDeclHirV1, CatalogPayloadV1, GateBindingV1, GraphRefV1,
    HirNodeIdV1, MetaHirV1, MetaKindV1, QueryHirV1, RelationExprHirV1, RelationExprKindV1,
    RelationSourceV1, SpanV1, ViewBodyV1,
};
use quanta_gqlang_v2_modules::{
    CatalogKindV1, CatalogV1, ModuleDiagnosticV1, PackGraphV1, PackIdV1, PackManifestV1,
    VersionRangeV1, VersionV1,
};

// --- hir construction helpers (tests build minimal nodes directly) ----------

fn span() -> SpanV1 {
    SpanV1::point(0)
}

fn gref(name: &str) -> GraphRefV1 {
    GraphRefV1 {
        name: name.to_string(),
        span: span(),
    }
}

fn decl(id: u32, name: &str, payload: CatalogPayloadV1) -> CatalogDeclHirV1 {
    CatalogDeclHirV1::new(HirNodeIdV1::from_raw(id), span(), name, span(), payload)
}

/// An inline-VIEW body whose SELECT reads from the single relation source
/// `source_name` (used to wire VIEW -> VIEW reference edges).
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

fn host_packs() -> PackGraphV1 {
    let compat = VersionRangeV1::new(VersionV1::new(3, 0, 0), VersionV1::new(4, 0, 0));
    PackGraphV1::new().with_manifest(PackManifestV1::new(
        PackIdV1::new("security-pack", VersionV1::new(1, 0, 0)),
        compat,
    ))
}

// Happy path: register VIEW + RULE + PACK + PROFILE + GATE and SHOW-style
// enumerate. Asset ids are the stable `<kind>::<name>`.
#[test]
fn register_and_enumerate_catalog() {
    let mut catalog = CatalogV1::new();
    catalog
        .register(decl(
            1,
            "active_users",
            CatalogPayloadV1::View(ViewBodyV1::QueryRef(gref("users"))),
        ))
        .unwrap();
    catalog
        .register(decl(
            2,
            "sqli",
            CatalogPayloadV1::RuleBinding {
                check_ref: gref("check::sqli"),
            },
        ))
        .unwrap();
    catalog
        .register(decl(
            3,
            "security-pack",
            CatalogPayloadV1::Pack {
                source: gref("registry://security"),
                activation: ActivationV1::Manual,
            },
        ))
        .unwrap();
    catalog
        .register(decl(
            4,
            "strict",
            CatalogPayloadV1::Profile {
                capabilities: vec![gref("witness")],
                admission: AdmissionV1::Allowlist,
            },
        ))
        .unwrap();
    catalog
        .register(decl(
            5,
            "release",
            CatalogPayloadV1::Gate {
                checks: vec![gref("check::sqli")],
                binding: GateBindingV1::All,
            },
        ))
        .unwrap();

    assert_eq!(catalog.len(), 5);
    let resolved = catalog.resolve().unwrap();
    // SHOW-style: every asset id, deterministically sorted.
    assert_eq!(
        resolved.asset_ids(),
        &[
            "gate::release".to_string(),
            "pack::security-pack".to_string(),
            "profile::strict".to_string(),
            "rule::sqli".to_string(),
            "view::active_users".to_string(),
        ]
    );
    // SHOW <category>: projection of one kind.
    assert_eq!(
        catalog.enumerate(CatalogKindV1::View),
        &["view::active_users".to_string()]
    );
    assert_eq!(
        catalog.enumerate(CatalogKindV1::Profile),
        &["profile::strict".to_string()]
    );
}

// Fail-closed: two declarations of the same kind + name collide; no last-wins.
#[test]
fn duplicate_catalog_name_rejects() {
    let mut catalog = CatalogV1::new();
    catalog
        .register(decl(
            1,
            "active_users",
            CatalogPayloadV1::View(ViewBodyV1::QueryRef(gref("users"))),
        ))
        .unwrap();
    let err = catalog
        .register(decl(
            2,
            "active_users",
            CatalogPayloadV1::View(ViewBodyV1::QueryRef(gref("accounts"))),
        ))
        .unwrap_err();
    assert_eq!(err.code(), "modules.duplicate_catalog_name");
    match &err {
        ModuleDiagnosticV1::DuplicateCatalogName { kind, name } => {
            assert_eq!(*kind, "view");
            assert_eq!(name, "active_users");
        }
        other => panic!("expected DuplicateCatalogName, got {other:?}"),
    }
    assert_eq!(
        err.to_string(),
        "modules.duplicate_catalog_name: catalog view `active_users` is already declared"
    );
}

// Same name across DIFFERENT kinds is NOT a collision: `view::x` and `rule::x`
// are distinct assets.
#[test]
fn same_name_distinct_kinds_coexist() {
    let mut catalog = CatalogV1::new();
    catalog
        .register(decl(
            1,
            "x",
            CatalogPayloadV1::View(ViewBodyV1::QueryRef(gref("t"))),
        ))
        .unwrap();
    catalog
        .register(decl(
            2,
            "x",
            CatalogPayloadV1::RuleBinding {
                check_ref: gref("check::x"),
            },
        ))
        .unwrap();
    assert_eq!(catalog.len(), 2);
}

// Fail-closed: a VIEW whose inline SELECT transitively references itself rejects.
#[test]
fn view_inline_select_cycle_rejects() {
    let mut catalog = CatalogV1::new();
    // view::a reads from view::b, view::b reads from view::a -> cycle.
    catalog
        .register(decl(1, "a", inline_view(10, "b")))
        .unwrap();
    catalog
        .register(decl(2, "b", inline_view(20, "a")))
        .unwrap();

    let err = catalog.resolve().unwrap_err();
    assert_eq!(err.code(), "modules.view_inline_cycle");
    match &err {
        ModuleDiagnosticV1::ViewInlineCycle { cycle } => {
            // DFS from the lexically-smallest entry `a`, closed at `a`.
            assert_eq!(
                cycle,
                &vec!["a".to_string(), "b".to_string(), "a".to_string()]
            );
        }
        other => panic!("expected ViewInlineCycle, got {other:?}"),
    }
    assert_eq!(
        err.to_string(),
        "modules.view_inline_cycle: view inline-select cycle a -> b -> a"
    );
}

// A VIEW inline-select reading a non-VIEW base table is a leaf, not a cycle.
#[test]
fn view_inline_select_over_base_table_resolves() {
    let mut catalog = CatalogV1::new();
    catalog
        .register(decl(1, "v", inline_view(10, "base_table")))
        .unwrap();
    let resolved = catalog.resolve().unwrap();
    assert_eq!(resolved.asset_ids(), &["view::v".to_string()]);
}

// Fail-closed: REQUIRE CERTIFIED + a non-certified PROFILE rejects.
#[test]
fn require_certified_non_certified_profile_rejects() {
    let mut catalog = CatalogV1::new();
    catalog
        .register(decl(
            1,
            "lax",
            CatalogPayloadV1::Profile {
                capabilities: vec![gref("witness")],
                admission: AdmissionV1::ReviewRequired,
            },
        ))
        .unwrap();
    let require = MetaHirV1::new(
        HirNodeIdV1::from_raw(2),
        span(),
        MetaKindV1::RequireCertified,
    );
    catalog.apply_meta(&require, &PackGraphV1::new()).unwrap();
    assert!(catalog.certified_required());

    let err = catalog.resolve().unwrap_err();
    assert_eq!(err.code(), "modules.certified_required_violation");
    match &err {
        ModuleDiagnosticV1::CertifiedRequiredViolation { profile, admission } => {
            assert_eq!(profile, "lax");
            assert_eq!(*admission, "review-required");
        }
        other => panic!("expected CertifiedRequiredViolation, got {other:?}"),
    }
    assert_eq!(
        err.to_string(),
        "modules.certified_required_violation: REQUIRE CERTIFIED forces certified-only but profile `lax` is review-required"
    );
}

// REQUIRE CERTIFIED + an all-certified-only profile set resolves.
#[test]
fn require_certified_all_certified_resolves() {
    let mut catalog = CatalogV1::new();
    catalog
        .register(decl(
            1,
            "strict",
            CatalogPayloadV1::Profile {
                capabilities: vec![gref("witness")],
                admission: AdmissionV1::CertifiedOnly,
            },
        ))
        .unwrap();
    let require = MetaHirV1::new(
        HirNodeIdV1::from_raw(2),
        span(),
        MetaKindV1::RequireCertified,
    );
    catalog.apply_meta(&require, &PackGraphV1::new()).unwrap();
    let resolved = catalog.resolve().unwrap();
    assert_eq!(resolved.asset_ids(), &["profile::strict".to_string()]);
}

// USE PACK resolves against a known manifest and records the activation.
#[test]
fn use_pack_known_manifest_resolves() {
    let mut catalog = CatalogV1::new();
    let use_pack = MetaHirV1::new(
        HirNodeIdV1::from_raw(1),
        span(),
        MetaKindV1::UsePack {
            pack_id: "security-pack".to_string(),
            activation: ActivationV1::Session,
            lockfile: Some("packs.lock".to_string()),
        },
    );
    catalog.apply_meta(&use_pack, &host_packs()).unwrap();
}

// Fail-closed: USE PACK with no resolvable manifest rejects.
#[test]
fn use_pack_unknown_manifest_rejects() {
    let mut catalog = CatalogV1::new();
    let use_pack = MetaHirV1::new(
        HirNodeIdV1::from_raw(1),
        span(),
        MetaKindV1::UsePack {
            pack_id: "ghost-pack".to_string(),
            activation: ActivationV1::Session,
            lockfile: None,
        },
    );
    let err = catalog.apply_meta(&use_pack, &host_packs()).unwrap_err();
    assert_eq!(err.code(), "modules.unknown_pack_manifest");
    assert_eq!(
        err.to_string(),
        "modules.unknown_pack_manifest: USE PACK `ghost-pack` has no resolvable manifest"
    );
}

// Fail-closed: USE PACK lockfile not ending in `.lock` rejects.
#[test]
fn use_pack_invalid_lockfile_path_rejects() {
    let mut catalog = CatalogV1::new();
    let use_pack = MetaHirV1::new(
        HirNodeIdV1::from_raw(1),
        span(),
        MetaKindV1::UsePack {
            pack_id: "security-pack".to_string(),
            activation: ActivationV1::Workspace,
            lockfile: Some("packs.txt".to_string()),
        },
    );
    let err = catalog.apply_meta(&use_pack, &host_packs()).unwrap_err();
    assert_eq!(err.code(), "modules.invalid_lockfile_path");
    assert_eq!(
        err.to_string(),
        "modules.invalid_lockfile_path: USE PACK `security-pack` lockfile `packs.txt` is not a *.lock file"
    );
}
