//! Edge-path rails for version-range parse/compare, lockfile round-trip and
//! mismatch rejects, and query-metadata token vocabulary (ticket 11). These
//! drive the reject / boundary branches that the positive packs rail does not
//! reach. syntax->sema->ir path pinned: NONE.

#![allow(clippy::unwrap_used, reason = "tests may unwrap")]
#![allow(clippy::indexing_slicing, reason = "tests may index")]
#![allow(clippy::panic, reason = "tests assert via panic on wrong variant")]
#![allow(
    clippy::wildcard_enum_match_arm,
    reason = "tests match the expected diagnostic variant and panic on any other"
)]

use quanta_gqlang_v2_modules::{
    LockfileV1, ModuleDiagnosticV1, PackGraphV1, PackIdV1, PackManifestV1, QueryKindV1,
    SeverityClassV1, VersionRangeV1, VersionV1,
};

fn compat() -> VersionRangeV1 {
    VersionRangeV1::new(VersionV1::new(3, 0, 0), VersionV1::new(4, 0, 0))
}

fn single_pack_graph(name: &str, version: VersionV1) -> PackGraphV1 {
    PackGraphV1::new()
        .add_manifest(PackManifestV1::new(PackIdV1::new(name, version), compat()))
        .unwrap()
}

// ---- version.rs ----------------------------------------------------------

// `VersionV1::parse` accepts exactly `major.minor.patch` and rejects every
// other shape: too few parts, non-numeric parts, and trailing data.
#[test]
fn version_parse_accepts_canonical_and_rejects_malformed() {
    assert_eq!(VersionV1::parse("1.2.3"), Some(VersionV1::new(1, 2, 3)));
    // Large components are preserved (u64).
    assert_eq!(
        VersionV1::parse("10.200.3000"),
        Some(VersionV1::new(10, 200, 3000))
    );

    // Too few components.
    assert_eq!(VersionV1::parse("1"), None);
    assert_eq!(VersionV1::parse("1.2"), None);
    // Non-numeric component.
    assert_eq!(VersionV1::parse("1.x.3"), None);
    assert_eq!(VersionV1::parse(""), None);
    // Trailing data: a fourth dotted component must reject (line 45 branch).
    assert_eq!(VersionV1::parse("1.2.3.4"), None);
    // Empty trailing component (text ends in a dot) -> unparsable patch.
    assert_eq!(VersionV1::parse("1.2."), None);
}

// Ordering is lexicographic on the numeric triple, so 1.2.0 < 1.10.0 (NOT a
// string compare) and range membership is total.
#[test]
fn version_ordering_is_numeric_not_lexical() {
    assert!(VersionV1::new(1, 2, 0) < VersionV1::new(1, 10, 0));
    assert!(VersionV1::new(1, 9, 9) < VersionV1::new(2, 0, 0));
    assert!(VersionV1::new(0, 0, 1) < VersionV1::new(0, 1, 0));
}

// Half-open `[lower, upper)` membership and the satisfiability predicate,
// including the empty-range fail-closed signal.
#[test]
fn version_range_membership_and_satisfiability() {
    let r = VersionRangeV1::new(VersionV1::new(1, 0, 0), VersionV1::new(2, 0, 0));
    // Inclusive lower.
    assert!(r.contains(VersionV1::new(1, 0, 0)));
    assert!(r.contains(VersionV1::new(1, 9, 9)));
    // Exclusive upper.
    assert!(!r.contains(VersionV1::new(2, 0, 0)));
    // Below lower.
    assert!(!r.contains(VersionV1::new(0, 9, 9)));
    assert!(r.is_satisfiable());

    // Empty range (upper == lower): never satisfiable, contains nothing.
    let empty = VersionRangeV1::new(VersionV1::new(1, 0, 0), VersionV1::new(1, 0, 0));
    assert!(!empty.is_satisfiable());
    assert!(!empty.contains(VersionV1::new(1, 0, 0)));

    // Inverted range (upper < lower): also unsatisfiable, contains nothing.
    let inverted = VersionRangeV1::new(VersionV1::new(2, 0, 0), VersionV1::new(1, 0, 0));
    assert!(!inverted.is_satisfiable());
    assert!(!inverted.contains(VersionV1::new(1, 5, 0)));
}

// Display renderings round-trip the documented canonical shapes.
#[test]
fn version_and_range_display() {
    assert_eq!(VersionV1::new(3, 1, 0).to_string(), "3.1.0");
    let r = VersionRangeV1::new(VersionV1::new(3, 0, 0), VersionV1::new(4, 0, 0));
    assert_eq!(r.to_string(), ">=3.0.0, <4.0.0");
}

// ---- metadata.rs ---------------------------------------------------------

// Every query-kind variant has its documented stable wire token.
#[test]
fn query_kind_tokens_are_stable() {
    assert_eq!(QueryKindV1::Problem.token(), "problem");
    assert_eq!(QueryKindV1::Path.token(), "path");
    assert_eq!(QueryKindV1::Metric.token(), "metric");
}

// Every severity-class variant has its documented stable wire token, and the
// ordering Note < Warning < Error holds.
#[test]
fn severity_tokens_and_ordering() {
    assert_eq!(SeverityClassV1::Note.token(), "note");
    assert_eq!(SeverityClassV1::Warning.token(), "warning");
    assert_eq!(SeverityClassV1::Error.token(), "error");
    assert!(SeverityClassV1::Note < SeverityClassV1::Warning);
    assert!(SeverityClassV1::Warning < SeverityClassV1::Error);
}

// ---- lockfile.rs ---------------------------------------------------------

// Empty/length accessors reflect pin state.
#[test]
fn lockfile_len_and_is_empty() {
    let empty = LockfileV1::new();
    assert!(empty.is_empty());
    assert_eq!(empty.len(), 0);
    assert_eq!(empty.locked_version("absent"), None);
    assert_eq!(empty.to_text(), "");

    let one = LockfileV1::new().with_pin("a", VersionV1::new(1, 0, 0));
    assert!(!one.is_empty());
    assert_eq!(one.len(), 1);

    // with_pin replaces any existing pin (builder, last value wins per key).
    let replaced = one.with_pin("a", VersionV1::new(2, 0, 0));
    assert_eq!(replaced.len(), 1);
    assert_eq!(replaced.locked_version("a"), Some(VersionV1::new(2, 0, 0)));
}

// parse skips blank lines and still produces a canonical, round-tripping lock.
#[test]
fn lockfile_parse_skips_blank_lines() {
    let text = "\n  \napp 1.0.0\n\nlib 2.1.0\n   \n";
    let lock = LockfileV1::parse(text).unwrap();
    assert_eq!(lock.len(), 2);
    assert_eq!(lock.locked_version("app"), Some(VersionV1::new(1, 0, 0)));
    assert_eq!(lock.locked_version("lib"), Some(VersionV1::new(2, 1, 0)));
    // Re-serialization is canonical (sorted, no blanks).
    assert_eq!(lock.to_text(), "app 1.0.0\nlib 2.1.0\n");
}

// parse fails closed on a malformed line: extra fields and unparsable version.
#[test]
fn lockfile_parse_rejects_malformed_lines() {
    // Extra third field (line 98 trailing-data branch).
    assert_eq!(LockfileV1::parse("app 1.0.0 extra"), None);
    // Unparsable version.
    assert_eq!(LockfileV1::parse("app notaversion"), None);
    // Missing version field entirely.
    assert_eq!(LockfileV1::parse("app"), None);
    // Version with trailing data.
    assert_eq!(LockfileV1::parse("app 1.0.0.0"), None);
}

// verify_against accepts a lock that exactly reproduces the manifest graph.
#[test]
fn verify_against_accepts_exact_lock() {
    let graph = single_pack_graph("app", VersionV1::new(1, 0, 0));
    let lock = LockfileV1::from_graph(&graph);
    assert!(lock.verify_against(&graph).is_ok());
}

// A pin at the WRONG version fails closed with a LockMismatch carrying the
// locked-vs-manifest detail (verify_against `Some(locked)` mismatch branch).
#[test]
fn verify_against_rejects_version_drift() {
    let graph = single_pack_graph("app", VersionV1::new(1, 0, 0));
    let drifted = LockfileV1::new().with_pin("app", VersionV1::new(1, 0, 1));
    let err = drifted.verify_against(&graph).unwrap_err();
    assert_eq!(err.code(), "modules.lock_mismatch");
    match &err {
        ModuleDiagnosticV1::LockMismatch { pack, detail } => {
            assert_eq!(pack, "app");
            assert_eq!(detail, "locked 1.0.1 but manifest declares 1.0.0");
        }
        other => panic!("expected LockMismatch, got {other:?}"),
    }
    assert_eq!(
        err.to_string(),
        "modules.lock_mismatch: lock entry for pack `app` mismatches manifest: \
         locked 1.0.1 but manifest declares 1.0.0"
    );
}

// A manifest pack with NO pin in the lock fails closed (None branch).
#[test]
fn verify_against_rejects_missing_pin() {
    let graph = single_pack_graph("app", VersionV1::new(1, 0, 0));
    let empty = LockfileV1::new();
    let err = empty.verify_against(&graph).unwrap_err();
    match &err {
        ModuleDiagnosticV1::LockMismatch { pack, detail } => {
            assert_eq!(pack, "app");
            assert_eq!(detail, "manifest declares 1.0.0 but lock has no pin");
        }
        other => panic!("expected LockMismatch, got {other:?}"),
    }
}

// A lock pinning a pack ABSENT from the manifest is a stale pin and fails
// closed (the second verify_against loop).
#[test]
fn verify_against_rejects_stale_pin() {
    let graph = single_pack_graph("app", VersionV1::new(1, 0, 0));
    // Pins app correctly, plus a stale "ghost" pack not in the manifest.
    let lock = LockfileV1::new()
        .with_pin("app", VersionV1::new(1, 0, 0))
        .with_pin("ghost", VersionV1::new(9, 9, 9));
    let err = lock.verify_against(&graph).unwrap_err();
    match &err {
        ModuleDiagnosticV1::LockMismatch { pack, detail } => {
            assert_eq!(pack, "ghost");
            assert_eq!(detail, "lock pins a pack absent from the manifest graph");
        }
        other => panic!("expected LockMismatch, got {other:?}"),
    }
}
