//! One disjoint instance extension inside the existing source byte grammar.

use super::{put_str, put_u8, relation_authority_tag, source_kind_tag, CanonicalByteSink};
use crate::ExecutionSourceV3;

pub(super) fn put_source_v3(out: &mut impl CanonicalByteSink, source: Option<&ExecutionSourceV3>) {
    if let Some(source) = source {
        put_u8(out, 1);
        if let Some(identity) = source.host_binding_identity {
            put_u8(out, 255);
            out.write_bytes(&identity);
        }
        put_u8(out, relation_authority_tag(source.authority));
        put_u8(out, source_kind_tag(source.kind));
        match &source.namespace {
            Some(namespace) => {
                put_u8(out, 1);
                put_str(out, namespace);
            }
            None => put_u8(out, 0),
        }
        put_str(out, &source.name);
    } else {
        put_u8(out, 0);
    }
}
