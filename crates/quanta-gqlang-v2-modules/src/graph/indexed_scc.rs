//! Workspace-internal SCC partition primitive. Domain witnesses and budgets
//! remain with the caller; this module is not a downstream frontend contract.

/// Charged traversal boundaries, in ascending root and supplied edge order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SccEventV1 {
    /// Before a vertex is discovered, using its incoming edge (or root) context.
    Discover,
    /// Before a component is materialized, using its root's discovery context.
    Emit,
}

/// A malformed indexed graph or a caller's first refusal.
#[derive(Debug, PartialEq, Eq)]
pub enum SccErrorV1<E> {
    /// An advertised edge was missing or targeted a vertex outside `roots`.
    InvalidEdge { source: usize, position: usize },
    /// The caller refused work before discovery or emission.
    Rejected(E),
}

/// Dense component labels. Only label equality and count are meaningful;
/// callers own the canonical order of rendered components or witnesses.
#[derive(Debug)]
pub struct SccPartitionV1 {
    pub component_of: Vec<usize>,
    pub component_count: usize,
}

struct Frame<'a, C> {
    node: usize,
    next_edge: usize,
    context: &'a C,
}

/// Upper bound on live Vec payload capacity, including returned labels.
/// Contexts and adjacency are borrowed; their storage belongs to the caller.
pub fn scratch_bytes_v1(nodes: usize) -> Option<usize> {
    let per_node = std::mem::size_of::<usize>()
        .checked_mul(4)? // index, lowlink, pending members, output labels
        .checked_add(std::mem::size_of::<bool>())?
        .checked_add(std::mem::size_of::<Frame<'_, ()>>())?;
    nodes.checked_mul(per_node)
}

/// Iterative Tarjan over borrowed indexed adjacency. Roots define the complete
/// vertex set; each edge context must outlive traversal. No graph is copied.
///
/// The callback runs once per discovery and component emission, before state
/// mutation for that event. Its first error stops traversal immediately. Edge
/// shape is checked before indexing; malformed input cannot yield a partition.
/// Work is O(vertices + edges), scratch O(vertices), with no native recursion.
#[expect(
    clippy::indexing_slicing,
    reason = "roots enumerate vertices and every edge target is range-checked before indexing"
)]
pub fn partition_v1<'a, C, E>(
    roots: &'a [C],
    edge_count: impl Fn(usize) -> usize,
    edge: impl Fn(usize, usize) -> Option<(usize, &'a C)>,
    mut before: impl FnMut(SccEventV1, &C) -> Result<(), E>,
) -> Result<SccPartitionV1, SccErrorV1<E>> {
    let nodes = roots.len();
    let mut index = vec![usize::MAX; nodes];
    let mut lowlink = vec![0; nodes];
    let mut on_stack = vec![false; nodes];
    let mut pending = Vec::with_capacity(nodes);
    let mut frames: Vec<Frame<'_, C>> = Vec::with_capacity(nodes);
    let mut component_of = vec![usize::MAX; nodes];
    let mut next_index = 0;
    let mut component_count = 0;
    for (root, context) in roots.iter().enumerate() {
        if index[root] != usize::MAX {
            continue;
        }
        before(SccEventV1::Discover, context).map_err(SccErrorV1::Rejected)?;
        index[root] = next_index;
        lowlink[root] = next_index;
        next_index += 1;
        on_stack[root] = true;
        pending.push(root);
        frames.push(Frame {
            node: root,
            next_edge: 0,
            context,
        });
        while let Some(frame) = frames.last_mut() {
            let node = frame.node;
            let position = frame.next_edge;
            if position < edge_count(node) {
                frame.next_edge += 1;
                let (target, context) = edge(node, position)
                    .filter(|(target, _)| *target < nodes)
                    .ok_or(SccErrorV1::InvalidEdge {
                        source: node,
                        position,
                    })?;
                if index[target] == usize::MAX {
                    before(SccEventV1::Discover, context).map_err(SccErrorV1::Rejected)?;
                    index[target] = next_index;
                    lowlink[target] = next_index;
                    next_index += 1;
                    on_stack[target] = true;
                    pending.push(target);
                    frames.push(Frame {
                        node: target,
                        next_edge: 0,
                        context,
                    });
                } else if on_stack[target] {
                    lowlink[node] = lowlink[node].min(index[target]);
                }
            } else {
                let context = frame.context;
                frames.pop();
                if let Some(parent) = frames.last() {
                    lowlink[parent.node] = lowlink[parent.node].min(lowlink[node]);
                }
                if lowlink[node] == index[node] {
                    before(SccEventV1::Emit, context).map_err(SccErrorV1::Rejected)?;
                    while let Some(member) = pending.pop() {
                        on_stack[member] = false;
                        component_of[member] = component_count;
                        if member == node {
                            break;
                        }
                    }
                    component_count += 1;
                }
            }
        }
    }
    Ok(SccPartitionV1 {
        component_of,
        component_count,
    })
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "bounded independent graph oracle and first-refusal assertions"
)]
#[path = "indexed_scc/tests.rs"]
mod tests;
