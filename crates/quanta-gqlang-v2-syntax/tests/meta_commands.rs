#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::wildcard_enum_match_arm,
    reason = "meta-command acceptance asserts validated parses/shapes, exercises fail-closed rejects, and bind-all match arms panic on unexpected variants"
)]

//! F10 proof rail (jun-10-gqlnag-v2-followup) — E03D interactive meta commands
//! and context builtins (`--test meta_commands`).
//!
//! NOTE TO OTHER AGENTS: this file pins the E03D contract. Invariants that must
//! never regress:
//!   1. Meta commands (`USE PACK`, `REQUIRE CERTIFIED`) parse at the document
//!      level into typed `MetaCommandAstV1` shapes; a malformed payload (e.g. a
//!      multi-token pack id) fails closed with a typed diagnostic.
//!   2. The context builtins `current_diff`, `changed_files`, `changed_symbols`
//!      are an EXPLICIT, CLOSED set of typed syntax nodes (`ContextBuiltinV1`).
//!      An UNKNOWN builtin — or a smuggled cross-plane command verb such as
//!      `apply` / `mutate` — is rejected at decode time, never passed through as
//!      an opaque token. This is the fail-closed validation E03D requires.
//!
//! Each surface has acceptance AND reject coverage. Do not weaken into
//! positive-only checks.
//!
//! DEFERRED (noted in the F10 ticket): routing `CONTEXT <builtin>` as its own
//! document-level root variant would require extending `MetaCommandAstV1` /
//! `AstRootV1`, which are pattern-matched exhaustively by the sema/ir/cli/lsp
//! crates this ticket may not touch. The typed node + fail-closed decoder live
//! here in the syntax layer; the cross-crate root wiring is a separate ticket.

use quanta_gqlang_v2_syntax::ast::AstRootV1;
use quanta_gqlang_v2_syntax::interactive::meta::{
    ContextBuiltinV1, MetaCommandAstV1, MetaCommandKindV1, MetaCommandShapeV1,
};
use quanta_gqlang_v2_syntax::parse_document_v1;

fn single_meta_command_v1(input: &str) -> MetaCommandAstV1 {
    let doc = parse_document_v1(input).expect("parse");
    assert_eq!(doc.roots.len(), 1, "expected one root: {input}");
    match doc.roots.into_iter().next().expect("root") {
        AstRootV1::Meta(command) => command,
        other => panic!("expected meta root, got {other:?}"),
    }
}

fn reject_codes_v1(input: &str) -> Vec<&'static str> {
    let page = parse_document_v1(input).expect_err("expected fail-closed rejection");
    page.diagnostics.iter().map(|d| d.code).collect()
}

#[test]
fn use_pack_meta_command_parses_typed_shape_v1() {
    let command = single_meta_command_v1(
        "USE PACK secure-pack ;; activation=workspace ;; lockfile=packs/secure-pack.lock",
    );
    assert_eq!(command.kind(), MetaCommandKindV1::UsePack);
    let MetaCommandShapeV1::UsePack(spec) = command.shape().expect("use-pack shape") else {
        panic!("expected use-pack shape");
    };
    assert_eq!(spec.pack_id, "secure-pack");
    assert_eq!(spec.lockfile_path, "packs/secure-pack.lock");
}

#[test]
fn use_pack_rejects_malformed_pack_id_fail_closed_v1() {
    // Multi-token pack id must fail closed at parse, not be silently truncated.
    let codes =
        reject_codes_v1("USE PACK secure pack ;; activation=session ;; lockfile=packs/x.lock");
    assert!(
        codes.iter().any(|code| code.starts_with("use_pack_")),
        "expected a use_pack_* reject, got {codes:?}"
    );
}

#[test]
fn require_certified_meta_command_parses_v1() {
    let command = single_meta_command_v1("REQUIRE CERTIFIED");
    assert_eq!(command.kind(), MetaCommandKindV1::RequireCertified);
    assert!(command.require_certified_shape().is_some());
}

#[test]
fn require_rejects_unknown_target_fail_closed_v1() {
    let codes = reject_codes_v1("REQUIRE UNCERTIFIED");
    assert!(
        codes.contains(&"unsupported_require_meta"),
        "expected unsupported_require_meta, got {codes:?}"
    );
}

#[test]
fn context_builtins_decode_the_frozen_set_v1() {
    // The three E03D builtins each decode to their typed node and round-trip
    // their canonical surface token.
    let frozen = [
        ("current_diff", ContextBuiltinV1::CurrentDiff),
        ("changed_files", ContextBuiltinV1::ChangedFiles),
        ("changed_symbols", ContextBuiltinV1::ChangedSymbols),
    ];
    for (token, expected) in frozen {
        let decoded = ContextBuiltinV1::from_token_v1(token).expect("known builtin decodes");
        assert_eq!(decoded, expected);
        assert_eq!(decoded.canonical_name_v1(), token);
    }
    // Case-insensitive surface, canonical lower-snake output.
    assert_eq!(
        ContextBuiltinV1::from_token_v1("CURRENT_DIFF").expect("uppercase decodes"),
        ContextBuiltinV1::CurrentDiff
    );
}

#[test]
fn context_builtin_rejects_unknown_token_fail_closed_v1() {
    let err = ContextBuiltinV1::from_token_v1("staged_diff")
        .expect_err("unknown builtin must fail closed");
    assert_eq!(err.code, "unsupported_context_builtin");
    assert!(
        err.message.contains("staged_diff"),
        "reject must name the unknown builtin, got: {}",
        err.message
    );
}

#[test]
fn context_builtin_rejects_cross_plane_command_verb_fail_closed_v1() {
    // A command-plane verb must not be smuggled into the query plane as a
    // context builtin: it is an unknown builtin and is rejected.
    for verb in ["apply", "mutate", "commit"] {
        let err = ContextBuiltinV1::from_token_v1(verb)
            .expect_err("command verb must not pass as a context builtin");
        assert_eq!(err.code, "unsupported_context_builtin");
    }
}

#[test]
fn context_builtin_rejects_blank_and_multi_token_fail_closed_v1() {
    let blank = ContextBuiltinV1::from_token_v1("   ").expect_err("blank must fail closed");
    assert_eq!(blank.code, "context_builtin_missing");

    let multi =
        ContextBuiltinV1::from_token_v1("current diff").expect_err("multi-token must fail closed");
    assert_eq!(multi.code, "context_builtin_invalid_shape");
}
