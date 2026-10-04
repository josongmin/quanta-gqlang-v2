//! A sibling crate cannot extract the parser-issued HIR or its retained permit.

use quanta_gqlang_v2_frontend_contract::ParsedRequestSourceV3;

fn probe(parsed: &ParsedRequestSourceV3) {
    let _ = &parsed.owner;
}

fn main() {
    let _ = probe as fn(&ParsedRequestSourceV3);
}
