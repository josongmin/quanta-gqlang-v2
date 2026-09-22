//! Negative case A21_pack_manifest_v1 (class A: owner symbol through the facade root).
//!
//! `PackManifestV1` is a modules owner pack manifest. It must stay unreachable from
//! `quanta_gqlang_v2_frontend_contract`: the facade is a curated allowlist, not
//! a re-export of an owner root.
//!
//! Expected: E0432, "no `PackManifestV1` in the root".
use quanta_gqlang_v2_frontend_contract::PackManifestV1;

fn main() {}
