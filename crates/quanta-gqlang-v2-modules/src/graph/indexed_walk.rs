//! Exact legacy all-root edge visits on a borrowed indexed DAG.

use crate::ModuleDiagnosticV2;

/// An internal indexing disagreement; callers must not turn it into acyclicity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum InvalidIndexedGraphV1 {
    Root,
    Target,
}

impl InvalidIndexedGraphV1 {
    pub(crate) fn into_diagnostic(self) -> ModuleDiagnosticV2 {
        ModuleDiagnosticV2::without_location(
            "modules.internal_cycle_graph_mismatch",
            match self {
                Self::Root => "cycle root is outside the indexed vertex set",
                Self::Target => "cycle edge target is outside the indexed vertex set",
            },
        )
    }
}

/// Count the edge visits of the former path-enumerating walk without enumerating
/// paths. `None` targets are charged terminal edges, never synthetic vertices.
/// Roots are a multiset: an empty/subset/repeated root list retains its meaning.
/// A cycle anywhere in the graph selects the caller's existing cyclic path.
///
/// FIFO order is internal: only the commutative saturating count is returned.
/// Each vertex enters the ready buffer once. A cursor avoids shifting a wide
/// ready frontier and keeps graph traversal O(vertices + edges + roots).
/// Three usize buffers of vertex capacity cover all owned scratch storage.
#[expect(
    clippy::indexing_slicing,
    reason = "roots and targets are validated; ready indices enumerate the adjacency"
)]
pub(crate) fn acyclic_steps_v1<E>(
    adjacency: &[Vec<E>],
    roots: impl IntoIterator<Item = usize>,
    target: impl Fn(&E) -> Option<usize>,
) -> Result<Option<(usize, bool)>, InvalidIndexedGraphV1> {
    let nodes = adjacency.len();
    let mut indegree = vec![0_usize; nodes];
    for edges in adjacency {
        for edge in edges {
            if let Some(node) = target(edge) {
                *indegree
                    .get_mut(node)
                    .ok_or(InvalidIndexedGraphV1::Target)? += 1;
            }
        }
    }
    let mut overflowed = false;
    let mut ways = vec![0_usize; nodes];
    for root in roots {
        let count = ways.get_mut(root).ok_or(InvalidIndexedGraphV1::Root)?;
        *count = count.checked_add(1).unwrap_or_else(|| {
            overflowed = true;
            usize::MAX
        });
    }
    let mut ready = Vec::with_capacity(nodes);
    for (node, count) in indegree.iter().enumerate() {
        if *count == 0 {
            ready.push(node);
        }
    }
    let mut steps = 0_usize;
    let mut cursor = 0;
    while let Some(&node) = ready.get(cursor) {
        cursor += 1;
        let visits = ways[node];
        let edge_visits = visits
            .checked_mul(adjacency[node].len())
            .unwrap_or_else(|| {
                overflowed = true;
                usize::MAX
            });
        steps = steps.checked_add(edge_visits).unwrap_or_else(|| {
            overflowed = true;
            usize::MAX
        });
        for edge in &adjacency[node] {
            if let Some(next) = target(edge) {
                ways[next] = ways[next].checked_add(visits).unwrap_or_else(|| {
                    overflowed = true;
                    usize::MAX
                });
                indegree[next] -= 1;
                if indegree[next] == 0 {
                    ready.push(next);
                }
            }
        }
    }
    Ok((cursor == nodes).then_some((steps, overflowed)))
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "bounded independent path enumeration and invalid-index assertions"
)]
#[path = "indexed_walk/tests.rs"]
mod tests;
