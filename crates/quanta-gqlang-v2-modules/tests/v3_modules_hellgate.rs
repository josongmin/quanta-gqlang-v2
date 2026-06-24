//! Hellgate (fail-closed) rails for ticket 11: module system, packs, extensions.
//!
//! syntax->sema->ir path pinned: NONE (this is the reuse/extension layer above
//! the core). These rails pin the deterministic fail-closed rejects the binder
//! (ticket 02) and CLI/LSP depend on. Each asserts the ACTUAL typed diagnostic
//! code + Display text (`D28`, `D30`). They fail red before the implementation
//! exists and must stay red on any silent-accept / last-wins regression.

#![allow(clippy::unwrap_used, reason = "tests may unwrap")]
#![allow(clippy::indexing_slicing, reason = "tests may index")]
#![allow(clippy::panic, reason = "tests assert via panic on wrong variant")]
#![allow(
    clippy::wildcard_enum_match_arm,
    reason = "tests match the expected diagnostic variant and panic on any other"
)]

use quanta_gqlang_v2_modules::{
    compose_models, LockfileV1, ModelFactV1, ModelPackV1, ModelSchemaV1, ModuleDiagnosticV1,
    ModuleGraphBuilderV1, PackGraphV1, PackIdV1, PackManifestV1, SignatureParamV1, SignatureV1,
    VersionRangeV1, VersionV1, VisibilityV1,
};

// Hellgate 2 (fail-closed): two re-exports of the same name from distinct
// targets are ambiguous; resolution rejects with no last-wins (`D28`).
#[test]
fn ambiguous_import_two_reexports_rejects() {
    let graph = ModuleGraphBuilderV1::new()
        .declare("auth_a", "isUser", VisibilityV1::Public)
        .declare("auth_b", "isUser", VisibilityV1::Public)
        // `facade` re-exports `isUser` from BOTH sources: ambiguous.
        .reexport("facade", "isUser", "auth_a", "isUser")
        .reexport("facade", "isUser", "auth_b", "isUser")
        .module("consumer");

    let err = graph.resolve().unwrap_err();
    assert_eq!(err.code(), "modules.ambiguous_import");
    match &err {
        ModuleDiagnosticV1::AmbiguousImport { name, candidates } => {
            assert_eq!(name, "isUser");
            // Deterministic, sorted, no winner picked.
            assert_eq!(
                candidates,
                &vec!["auth_a::isUser".to_string(), "auth_b::isUser".to_string()]
            );
        }
        other => panic!("expected AmbiguousImport, got {other:?}"),
    }
    assert_eq!(
        err.to_string(),
        "modules.ambiguous_import: imported symbol `isUser` is ambiguous between auth_a::isUser, auth_b::isUser"
    );
}

// Hellgate 2 variant: two packs exporting the same public name is an
// overlapping-pack conflict (no last-wins, `D28`).
#[test]
fn overlapping_pack_export_rejects() {
    let host = VersionV1::new(3, 0, 0);
    let compat = VersionRangeV1::new(VersionV1::new(3, 0, 0), VersionV1::new(4, 0, 0));
    let graph = PackGraphV1::new()
        .add_manifest(
            PackManifestV1::new(PackIdV1::new("pack-a", VersionV1::new(1, 0, 0)), compat)
                .with_export("Sanitizer"),
        )
        .unwrap()
        .add_manifest(
            PackManifestV1::new(PackIdV1::new("pack-b", VersionV1::new(2, 0, 0)), compat)
                .with_export("Sanitizer"),
        )
        .unwrap();

    let err = graph.validate(host).unwrap_err();
    assert_eq!(err.code(), "modules.overlapping_pack_export");
    match &err {
        ModuleDiagnosticV1::OverlappingPackExport { symbol, packs } => {
            assert_eq!(symbol, "Sanitizer");
            assert_eq!(
                packs,
                &vec!["pack-a@1.0.0".to_string(), "pack-b@2.0.0".to_string()]
            );
        }
        other => panic!("expected OverlappingPackExport, got {other:?}"),
    }
    assert_eq!(
        err.to_string(),
        "modules.overlapping_pack_export: symbol `Sanitizer` is exported by overlapping packs pack-a@1.0.0, pack-b@2.0.0"
    );
}

// Hellgate 2 variant (fail-closed): adding two manifests with the same pack
// name but different versions is a hard reject — never a silent last-wins
// overwrite (`D28`). The graph rejects before mutating its state.
#[test]
fn duplicate_pack_name_rejects() {
    let compat = VersionRangeV1::new(VersionV1::new(3, 0, 0), VersionV1::new(4, 0, 0));
    let first = PackGraphV1::new()
        .add_manifest(
            PackManifestV1::new(PackIdV1::new("mypack", VersionV1::new(1, 0, 0)), compat)
                .with_export("Symbol1"),
        )
        .unwrap();

    let err = first
        .clone()
        .add_manifest(
            PackManifestV1::new(PackIdV1::new("mypack", VersionV1::new(2, 0, 0)), compat)
                .with_export("Symbol2"),
        )
        .unwrap_err();

    assert_eq!(err.code(), "modules.duplicate_pack_name");
    match &err {
        ModuleDiagnosticV1::DuplicatePackName {
            name,
            existing,
            incoming,
        } => {
            assert_eq!(name, "mypack");
            assert_eq!(*existing, VersionV1::new(1, 0, 0));
            assert_eq!(*incoming, VersionV1::new(2, 0, 0));
        }
        other => panic!("expected DuplicatePackName, got {other:?}"),
    }
    assert_eq!(
        err.to_string(),
        "modules.duplicate_pack_name: pack `mypack` is declared twice (existing 1.0.0 vs incoming 2.0.0)"
    );

    // Fail-closed before mutation: the original graph still pins 1.0.0 only and
    // is byte-identical to its pre-rejection state (no partial overwrite).
    assert_eq!(first.len(), 1);
    assert_eq!(
        first.manifest("mypack").map(|m| m.id.version),
        Some(VersionV1::new(1, 0, 0))
    );
}

// Hellgate 3 (fail-closed): an import cycle hits the explicit cycle policy
// (`D30`) with a typed diagnostic; never a silent accept.
#[test]
fn import_cycle_rejects() {
    let graph = ModuleGraphBuilderV1::new()
        .declare("m_a", "a", VisibilityV1::Public)
        .declare("m_b", "b", VisibilityV1::Public)
        .declare("m_c", "c", VisibilityV1::Public)
        .import("m_a", "b", "m_b", "b")
        .import("m_b", "c", "m_c", "c")
        .import("m_c", "a", "m_a", "a");

    let err = graph.resolve().unwrap_err();
    assert_eq!(err.code(), "modules.import_cycle");
    match &err {
        ModuleDiagnosticV1::ImportCycle { cycle } => {
            // Canonicalized: rotated to lexically smallest start, closed.
            assert_eq!(
                cycle,
                &vec![
                    "m_a".to_string(),
                    "m_b".to_string(),
                    "m_c".to_string(),
                    "m_a".to_string()
                ]
            );
        }
        other => panic!("expected ImportCycle, got {other:?}"),
    }
    assert_eq!(
        err.to_string(),
        "modules.import_cycle: module import cycle m_a -> m_b -> m_c -> m_a"
    );
}

// Hellgate 4 (fail-closed): a language/compat mismatch rejects with a typed
// diagnostic.
#[test]
fn compat_mismatch_rejects() {
    let host = VersionV1::new(2, 9, 0);
    let compat = VersionRangeV1::new(VersionV1::new(3, 0, 0), VersionV1::new(4, 0, 0));
    let graph = PackGraphV1::new()
        .add_manifest(PackManifestV1::new(
            PackIdV1::new("needs-v3", VersionV1::new(1, 0, 0)),
            compat,
        ))
        .unwrap();

    let err = graph.validate(host).unwrap_err();
    assert_eq!(err.code(), "modules.compat_mismatch");
    assert_eq!(
        err.to_string(),
        "modules.compat_mismatch: pack `needs-v3@1.0.0` requires language >=3.0.0, <4.0.0 but host is 2.9.0"
    );
}

// Hellgate 4 variant: a lockfile that disagrees with the manifest graph rejects.
#[test]
fn lock_mismatch_rejects() {
    let compat = VersionRangeV1::new(VersionV1::new(3, 0, 0), VersionV1::new(4, 0, 0));
    let graph = PackGraphV1::new()
        .add_manifest(PackManifestV1::new(
            PackIdV1::new("p", VersionV1::new(1, 2, 0)),
            compat,
        ))
        .unwrap();
    // Lock pins a different version than the manifest declares.
    let stale = LockfileV1::new().with_pin("p", VersionV1::new(1, 1, 0));

    let err = stale.verify_against(&graph).unwrap_err();
    assert_eq!(err.code(), "modules.lock_mismatch");
    assert_eq!(
        err.to_string(),
        "modules.lock_mismatch: lock entry for pack `p` mismatches manifest: locked 1.1.0 but manifest declares 1.2.0"
    );
}

// Dependency range unsatisfied is a typed reject.
#[test]
fn dependency_unsatisfied_rejects() {
    let host = VersionV1::new(3, 0, 0);
    let compat = VersionRangeV1::new(VersionV1::new(3, 0, 0), VersionV1::new(4, 0, 0));
    let graph = PackGraphV1::new()
        .add_manifest(
            PackManifestV1::new(PackIdV1::new("app", VersionV1::new(1, 0, 0)), compat)
                .with_dependency(quanta_gqlang_v2_modules::DependencyReqV1::new(
                    "lib",
                    VersionRangeV1::new(VersionV1::new(2, 0, 0), VersionV1::new(3, 0, 0)),
                )),
        )
        .unwrap()
        // lib is present but at 1.5.0, outside the [2.0.0, 3.0.0) range.
        .add_manifest(PackManifestV1::new(
            PackIdV1::new("lib", VersionV1::new(1, 5, 0)),
            compat,
        ))
        .unwrap();

    let err = graph.validate(host).unwrap_err();
    assert_eq!(err.code(), "modules.dependency_unsatisfied");
    assert_eq!(
        err.to_string(),
        "modules.dependency_unsatisfied: pack `app@1.0.0` requires `lib` >=2.0.0, <3.0.0 but 1.5.0 is offered"
    );
}

// Model packs that assert conflicting facts for the same target attribute reject
// (`D28`, no last-wins).
#[test]
fn model_conflict_rejects() {
    let schema = ModelSchemaV1::new()
        .allow("taint_kind", "source")
        .allow("taint_kind", "sink");
    let pack_a = ModelPackV1::new("pack-a", VersionV1::new(1, 0, 0)).with_fact(ModelFactV1::new(
        "lib::handle",
        "taint_kind",
        "source",
    ));
    let pack_b = ModelPackV1::new("pack-b", VersionV1::new(1, 0, 0)).with_fact(ModelFactV1::new(
        "lib::handle",
        "taint_kind",
        "sink",
    ));

    let err = compose_models(&[pack_b, pack_a], &schema).unwrap_err();
    assert_eq!(err.code(), "modules.model_conflict");
    assert_eq!(
        err.to_string(),
        "modules.model_conflict: model target `lib::handle` attribute `taint_kind` conflicts (source, sink) from packs pack-a, pack-b"
    );
}

// Invalid model data (attribute not in schema) rejects.
#[test]
fn invalid_model_data_rejects() {
    let schema = ModelSchemaV1::new().allow("taint_kind", "source");
    let pack = ModelPackV1::new("bad", VersionV1::new(1, 0, 0)).with_fact(ModelFactV1::new(
        "lib::x",
        "unknown_attr",
        "v",
    ));

    let err = pack.validate(&schema).unwrap_err();
    assert_eq!(err.code(), "modules.invalid_model_data");
    assert_eq!(
        err.to_string(),
        "modules.invalid_model_data: pack `bad` has invalid model data: attribute `unknown_attr` is not declared in the schema"
    );
}

// Importing a private (non-exported) symbol rejects.
#[test]
fn not_exported_rejects() {
    let graph = ModuleGraphBuilderV1::new()
        .declare("lib", "secret", VisibilityV1::Private)
        .import("consumer", "secret", "lib", "secret");

    let err = graph.resolve().unwrap_err();
    assert_eq!(err.code(), "modules.not_exported");
    assert_eq!(
        err.to_string(),
        "modules.not_exported: module `lib` does not export `secret`"
    );
}

// Signature instantiation arity mismatch rejects.
#[test]
fn signature_arity_mismatch_rejects() {
    let sig = SignatureV1::new("isSink", "bool")
        .with_param(SignatureParamV1::new("node", "Node"))
        .with_param(SignatureParamV1::new("config", "Config"));

    let err = sig.instantiate(&["Node".to_string()]).unwrap_err();
    assert_eq!(err.code(), "modules.signature_arity_mismatch");
    assert_eq!(
        err.to_string(),
        "modules.signature_arity_mismatch: signature `isSink` expects 2 parameter(s) but got 1"
    );
}
