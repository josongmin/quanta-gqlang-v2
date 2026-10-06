//! Upper demand for the pinned Rust 1.81/1.92 BTree node layout.
//!
//! Both installed `alloc/collections/btree/node.rs` versions use B=6, eleven
//! key/value slots per node, twelve internal edges, and the same LeafNode and
//! InternalNode fields. Callers separately count owned key/value payloads.

/// Retained BTree nodes for at most `keys` distinct source keys plus the
/// transient split path of the next insertion. A live node contains at least
/// one key; the split path adds at most one new node per tree level while the
/// old node remains live. This bound is shared by frontend owner crates.
pub fn btree_retained_with_next_insert_peak_bytes_v3<K, V>(keys: usize) -> Option<usize> {
    let node = node_upper_bytes_v3::<K, V>()?;
    let retained = keys.checked_mul(node)?;
    let next = keys.checked_add(1)?;
    let levels = (usize::BITS - next.leading_zeros()) as usize + 1;
    let split = levels.checked_mul(2)?.checked_mul(node)?;
    retained.checked_add(split)
}

fn node_upper_bytes_v3<K, V>() -> Option<usize> {
    const KV_SLOTS: usize = 11;
    const EDGES: usize = 12;
    const FIELD_GROUPS: usize = 6;
    let alignment = std::mem::align_of::<K>()
        .max(std::mem::align_of::<V>())
        .max(std::mem::align_of::<usize>())
        .max(std::mem::align_of::<u16>());
    std::mem::size_of::<Option<std::ptr::NonNull<()>>>()
        .checked_add(2 * std::mem::size_of::<u16>())?
        .checked_add(KV_SLOTS.checked_mul(std::mem::size_of::<K>())?)?
        .checked_add(KV_SLOTS.checked_mul(std::mem::size_of::<V>())?)?
        .checked_add(EDGES.checked_mul(std::mem::size_of::<usize>())?)?
        .checked_add(FIELD_GROUPS.checked_mul(alignment.checked_sub(1)?)?)
}
