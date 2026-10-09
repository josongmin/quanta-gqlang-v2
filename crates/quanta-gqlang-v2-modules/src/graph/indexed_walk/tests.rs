use super::{acyclic_steps_v1, InvalidIndexedGraphV1};

fn oracle(graph: &[Vec<Option<usize>>], roots: &[usize]) -> Option<(usize, bool)> {
    let mut reachable = [[false; 3]; 3];
    for (node, edges) in graph.iter().enumerate() {
        for target in edges.iter().flatten() {
            reachable[node][*target] = true;
        }
    }
    for via in [0, 1, 2] {
        let via_row = reachable[via];
        for row in &mut reachable {
            if row[via] {
                for (cell, through) in row.iter_mut().zip(via_row) {
                    *cell |= through;
                }
            }
        }
    }
    if reachable.iter().enumerate().any(|(node, row)| row[node]) {
        return None;
    }
    // Independent path enumeration, with roots retaining multiplicity and
    // dangling edges counted once per path reaching their source.
    let mut pending = roots.to_vec();
    let mut steps = 0;
    while let Some(node) = pending.pop() {
        for target in &graph[node] {
            steps += 1;
            if let Some(target) = target {
                pending.push(*target);
            }
        }
    }
    Some((steps, false))
}

#[test]
fn all_three_vertex_graphs_match_path_oracle_with_dangling_edges_and_root_multisets() {
    for mask in 0_u16..512 {
        for dangling in 0_u8..8 {
            let graph = (0..3)
                .map(|node| {
                    let mut edges = (0..3)
                        .filter(|target| mask & (1 << (node * 3 + target)) != 0)
                        .map(Some)
                        .collect::<Vec<_>>();
                    if dangling & (1 << node) != 0 {
                        edges.push(None);
                    }
                    edges
                })
                .collect::<Vec<_>>();
            for roots in [&[][..], &[0][..], &[0, 0][..], &[0, 1, 2][..]] {
                assert_eq!(
                    acyclic_steps_v1(&graph, roots.iter().copied(), |edge| *edge)
                        .expect("valid indices"),
                    oracle(&graph, roots),
                    "mask={mask}, dangling={dangling}, roots={roots:?}"
                );
            }
        }
    }
}

#[test]
fn invalid_indices_are_not_acyclic_success_and_dangling_edges_are_charged() {
    assert_eq!(
        acyclic_steps_v1(&[vec![Some(1)]], [0], |edge| *edge),
        Err(InvalidIndexedGraphV1::Target)
    );
    assert_eq!(
        acyclic_steps_v1(&[Vec::<usize>::new()], [1], |edge| Some(*edge)),
        Err(InvalidIndexedGraphV1::Root)
    );
    assert_eq!(
        acyclic_steps_v1(&[vec![None::<usize>]], [0, 0], |edge| *edge),
        Ok(Some((2, false)))
    );
}

#[test]
fn reachable_path_overflow_is_retained_but_unreachable_paths_do_not_overflow() {
    let graph = (0..100)
        .map(|node| {
            [node + 1, node + 2]
                .into_iter()
                .filter(|target| *target < 100)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        acyclic_steps_v1(&graph, [0], |edge| Some(*edge)),
        Ok(Some((usize::MAX, true)))
    );
    assert_eq!(
        acyclic_steps_v1(&graph, [], |edge| Some(*edge)),
        Ok(Some((0, false)))
    );
}
