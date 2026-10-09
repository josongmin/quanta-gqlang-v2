use super::{partition_v1, scratch_bytes_v1, SccErrorV1, SccEventV1};

fn labels(graph: &[Vec<usize>]) -> Vec<usize> {
    let roots = vec![(); graph.len()];
    partition_v1(
        &roots,
        |node| graph[node].len(),
        |node, position| graph[node].get(position).map(|&target| (target, &())),
        |_, _| Ok::<_, ()>(()),
    )
    .unwrap()
    .component_of
}

#[test]
fn all_three_vertex_graphs_match_mutual_reachability_in_both_edge_orders() {
    // Independent transitive-closure oracle, including disconnected vertices,
    // self-loops, and all combinations of forward/back edges.
    for mask in 0_u16..512 {
        let mut graph = vec![Vec::new(); 3];
        let mut reachable = [[false; 3]; 3];
        for (source, bucket) in graph.iter_mut().enumerate() {
            reachable[source][source] = true;
            for (target, slot) in reachable[source].iter_mut().enumerate() {
                if mask & (1 << (source * 3 + target)) != 0 {
                    bucket.push(target);
                    *slot = true;
                }
            }
        }
        for via in 0..3 {
            for source in 0..3 {
                for target in 0..3 {
                    reachable[source][target] |= reachable[source][via] && reachable[via][target];
                }
            }
        }
        for _ in 0..2 {
            let result = labels(&graph);
            for source in 0..3 {
                for target in 0..3 {
                    assert_eq!(
                        result[source] == result[target],
                        reachable[source][target] && reachable[target][source]
                    );
                }
            }
            for bucket in &mut graph {
                bucket.reverse();
            }
        }
    }
    assert!(labels(&[]).is_empty());
    assert_eq!(
        labels(&[vec![1, 1], vec![0, 0]])[0],
        labels(&[vec![1], vec![0]])[1]
    );
}

#[test]
fn first_refusal_keeps_discovery_and_emission_context_without_later_events() {
    let roots = [10, 11, 12];
    let graph = [vec![(1, 101)], vec![(2, 102)], vec![]];
    let expected = [
        (SccEventV1::Discover, 10),
        (SccEventV1::Discover, 101),
        (SccEventV1::Discover, 102),
        (SccEventV1::Emit, 102),
        (SccEventV1::Emit, 101),
        (SccEventV1::Emit, 10),
    ];
    for limit in 0..=expected.len() {
        let mut events = Vec::new();
        let result = partition_v1(
            &roots,
            |node| graph[node].len(),
            |node, position| {
                graph[node]
                    .get(position)
                    .map(|(target, context)| (*target, context))
            },
            |event, &context| {
                if events.len() == limit {
                    return Err((event, context));
                }
                events.push((event, context));
                Ok(())
            },
        );
        assert_eq!(events, expected[..limit]);
        if limit < expected.len() {
            assert_eq!(result.unwrap_err(), SccErrorV1::Rejected(expected[limit]));
        } else {
            assert_eq!(result.unwrap().component_count, 3);
        }
    }
}

#[test]
fn malformed_adjacency_rejects_without_discovering_target() {
    for edge in [None, Some((1, &()))] {
        let mut events = Vec::new();
        let result = partition_v1(
            &[()],
            |_| 1,
            |_, _| edge,
            |event, _| {
                events.push(event);
                Ok::<_, ()>(())
            },
        );
        assert_eq!(
            result.unwrap_err(),
            SccErrorV1::InvalidEdge {
                source: 0,
                position: 0
            }
        );
        assert_eq!(events, [SccEventV1::Discover]);
    }
}

#[test]
fn capacity_bound_counts_all_six_buffers_and_rejects_overflow() {
    // Frame consists of node, edge cursor, and a borrowed context pointer.
    let per_node = 7 * std::mem::size_of::<usize>() + std::mem::size_of::<bool>();
    assert_eq!(scratch_bytes_v1(0), Some(0));
    assert_eq!(scratch_bytes_v1(50_000), Some(50_000 * per_node));
    assert_eq!(scratch_bytes_v1(usize::MAX), None);
}
