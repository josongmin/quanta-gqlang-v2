//! Positive rails for packs, lockfiles, signatures, metadata, and model
//! composition (ticket 11). syntax->sema->ir path pinned: NONE.

#![allow(clippy::unwrap_used, reason = "tests may unwrap")]
#![allow(clippy::indexing_slicing, reason = "tests may index")]

use quanta_gqlang_v2_modules::{
    compose_models, DependencyReqV1, HelpDocV1, LockfileV1, ModelFactV1, ModelPackV1,
    ModelSchemaV1, PackGraphV1, PackIdV1, PackManifestV1, QueryKindV1, QueryMetadataV1,
    SeverityClassV1, SignatureParamV1, SignatureV1, SuiteV1, VersionRangeV1, VersionV1,
};

fn compat() -> VersionRangeV1 {
    VersionRangeV1::new(VersionV1::new(3, 0, 0), VersionV1::new(4, 0, 0))
}

// Hellgate 4 positive half: a manifest + lockfile round-trip and a satisfied
// compat window validate cleanly.
#[test]
fn manifest_and_lock_round_trip() {
    let host = VersionV1::new(3, 1, 0);
    let graph = PackGraphV1::new()
        .add_manifest(
            PackManifestV1::new(PackIdV1::new("app", VersionV1::new(1, 0, 0)), compat())
                .with_dependency(DependencyReqV1::new(
                    "lib",
                    VersionRangeV1::new(VersionV1::new(2, 0, 0), VersionV1::new(3, 0, 0)),
                ))
                .with_export("AppQuery"),
        )
        .unwrap()
        .add_manifest(
            PackManifestV1::new(PackIdV1::new("lib", VersionV1::new(2, 1, 0)), compat())
                .with_export("LibHelper"),
        )
        .unwrap();

    // Validation passes: compat ok, dependency in range, no overlap.
    graph.validate(host).unwrap();

    // Lock derived from the graph reproduces and verifies.
    let lock = LockfileV1::from_graph(&graph);
    lock.verify_against(&graph).unwrap();
    assert_eq!(lock.locked_version("app"), Some(VersionV1::new(1, 0, 0)));
    assert_eq!(lock.locked_version("lib"), Some(VersionV1::new(2, 1, 0)));

    // Textual round-trip is byte-stable and canonical (sorted).
    let text = lock.to_text();
    assert_eq!(text, "app 1.0.0\nlib 2.1.0\n");
    let reparsed = LockfileV1::parse(&text).unwrap();
    assert_eq!(reparsed, lock);
    reparsed.verify_against(&graph).unwrap();
}

// Suite membership is deterministic and offline-reproducible.
#[test]
fn suite_membership_is_deterministic() {
    let manifest = PackManifestV1::new(PackIdV1::new("sec", VersionV1::new(1, 0, 0)), compat())
        .with_suite(
            SuiteV1::new("default")
                .with_member("q2")
                .with_member("q1")
                .with_member("q1"),
        );
    let suite = manifest.suites.get("default").unwrap();
    // BTreeSet ordering + de-dup => deterministic membership.
    assert_eq!(
        suite.members.iter().cloned().collect::<Vec<_>>(),
        vec!["q1".to_string(), "q2".to_string()]
    );
}

// Parameterized signature instantiation binds positional args structurally.
#[test]
fn signature_instantiates() {
    let sig = SignatureV1::new("isSink", "bool")
        .with_param(SignatureParamV1::new("node", "Node"))
        .with_param(SignatureParamV1::new("cfg", "Config"));
    assert_eq!(sig.arity(), 2);

    let inst = sig
        .instantiate(&["Expr".to_string(), "DefaultConfig".to_string()])
        .unwrap();
    assert_eq!(inst.signature, "isSink");
    assert_eq!(inst.result_tag, "bool");
    assert_eq!(inst.bindings[0].0.name, "node");
    assert_eq!(inst.bindings[0].1, "Expr");
    assert_eq!(inst.bindings[1].0.name, "cfg");
    assert_eq!(inst.bindings[1].1, "DefaultConfig");
}

// Query metadata carries kind/severity/tags/help deterministically.
#[test]
fn query_metadata_model() {
    let md = QueryMetadataV1::new("sql-injection", QueryKindV1::Path, SeverityClassV1::Error)
        .with_tag("security")
        .with_tag("cwe-089")
        .with_help("Detects SQL injection", "Use parameterized queries.");
    assert_eq!(md.kind.token(), "path");
    assert_eq!(md.severity.token(), "error");
    assert_eq!(
        md.tags.iter().cloned().collect::<Vec<_>>(),
        vec!["cwe-089".to_string(), "security".to_string()]
    );
    assert_eq!(
        md.help,
        HelpDocV1 {
            summary: "Detects SQL injection".to_string(),
            description: "Use parameterized queries.".to_string(),
        }
    );
}

// Two model packs that AGREE on a fact compose idempotently (no false conflict).
#[test]
fn agreeing_model_packs_compose() {
    let schema = ModelSchemaV1::new().allow("taint_kind", "source");
    let a = ModelPackV1::new("a", VersionV1::new(1, 0, 0)).with_fact(ModelFactV1::new(
        "lib::h",
        "taint_kind",
        "source",
    ));
    let b = ModelPackV1::new("b", VersionV1::new(1, 0, 0))
        .with_fact(ModelFactV1::new("lib::h", "taint_kind", "source"))
        .with_fact(ModelFactV1::new("lib::g", "taint_kind", "source"));

    let composed = compose_models(&[b, a], &schema).unwrap();
    assert_eq!(composed.len(), 2);
    let facts: Vec<_> = composed.facts().cloned().collect();
    assert_eq!(facts[0], ModelFactV1::new("lib::g", "taint_kind", "source"));
    assert_eq!(facts[1], ModelFactV1::new("lib::h", "taint_kind", "source"));
}
