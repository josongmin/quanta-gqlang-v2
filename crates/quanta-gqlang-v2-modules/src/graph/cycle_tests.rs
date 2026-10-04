use std::collections::BTreeSet;

use super::*;
use crate::budget::ModuleBudgetV2;
use quanta_gqlang_v2_hir::{SourceIdV2, SourceRegistrationV2, SourceRegistryViewV2, SpanV1};
use quanta_gqlang_v2_types::AuthorityBudgetProfileV2;

fn checked(sources: &SourceRegistryViewV2, at: usize) -> CheckedLocatedSpanV2 {
    sources
        .try_check_location(LocatedSpanV2 {
            source_id: SourceIdV2::from_raw(1),
            span: SpanV1::new(at, at + 1),
        })
        .expect("test span is registered")
}

fn budget(steps: usize) -> ModuleBudgetV2 {
    ModuleBudgetV2::from_profile(
        AuthorityBudgetProfileV2::try_new(512, 512, steps, 1_000_000, 64)
            .expect("positive test budget"),
    )
}

fn graph_from_edges(nodes: usize, edges: &[(usize, usize)], reverse: bool) -> ModuleGraphBuilderV2 {
    let sources = SourceRegistryViewV2::try_new(vec![SourceRegistrationV2 {
        source_id: SourceIdV2::from_raw(1),
        source_len: 10_000,
    }])
    .expect("test source registry");
    let known_ids = nodes.max(edges.iter().map(|(_, to)| to + 1).max().unwrap_or(nodes));
    let ids = (0..known_ids)
        .map(|node| {
            ModuleIdV2::try_new(&format!("m{node:03}"), checked(&sources, node))
                .expect("test module ID")
        })
        .collect::<Vec<_>>();
    let mut graph = ModuleGraphBuilderV2::new();
    let mut session = budget(1_000_000).begin_session_v2();
    for (node, id) in ids.iter().take(nodes).enumerate() {
        graph
            .try_add_module(
                ModuleNodeInputV2 {
                    module_id: id.clone(),
                    location: checked(&sources, node),
                },
                &mut session,
            )
            .expect("test node fits budget");
    }
    let mut order = (0..edges.len()).collect::<Vec<_>>();
    if reverse {
        order.reverse();
    }
    for index in order {
        let (from, to) = edges[index];
        graph
            .try_import(
                ModuleImportInputV2 {
                    owner_module_id: ids[from].clone(),
                    local_name: format!("edge_{index}"),
                    target: CatalogAssetRefInputV2 {
                        module_id: ids[to].clone(),
                        namespace_kind: NamespaceKindV2::Value,
                        local_name: "x".to_owned(),
                        location: checked(&sources, nodes + index * 2),
                    },
                    location: checked(&sources, nodes + index * 2 + 1),
                },
                &mut session,
            )
            .expect("test edge fits budget");
    }
    graph
}

// Bounded oracle: it uses path-copying enumeration and all-rotation
// canonicalization, with the acyclic-prefix self-loop defect corrected.
fn legacy_cycle(graph: &ModuleGraphBuilderV2) -> (Option<Vec<LocatedEdgeV2>>, usize) {
    let mut candidates = Vec::new();
    let mut steps = 0usize;
    for root in graph.modules.keys() {
        let mut stack = vec![(root.clone(), Vec::new(), BTreeSet::from([root.clone()]))];
        while let Some((node_id, path, seen)) = stack.pop() {
            let Some(node) = graph.modules.get(&node_id) else {
                continue;
            };
            let mut edges = node
                .imports
                .values()
                .chain(node.reexports.values().flatten())
                .collect::<Vec<_>>();
            edges.sort_by(|a, b| {
                a.target
                    .cmp(&b.target)
                    .then(loc_key(a.location.location()).cmp(&loc_key(b.location.location())))
            });
            for edge in edges.into_iter().rev() {
                steps += 1;
                let located = located_edge(&node_id, edge);
                if seen.contains(&edge.target.module_id) {
                    let position = path
                        .iter()
                        .position(|item: &LocatedEdgeV2| item.from == edge.target.module_id)
                        .unwrap_or(path.len());
                    let mut cycle = path[position..].to_vec();
                    cycle.push(located);
                    let canonical = (0..cycle.len())
                        .map(|offset| {
                            cycle
                                .iter()
                                .cycle()
                                .skip(offset)
                                .take(cycle.len())
                                .cloned()
                                .collect::<Vec<_>>()
                        })
                        .min()
                        .expect("non-empty cycle");
                    candidates.push(canonical);
                } else {
                    let mut next_path = path.clone();
                    next_path.push(located);
                    let mut next_seen = seen.clone();
                    next_seen.insert(edge.target.module_id.clone());
                    stack.push((edge.target.module_id.clone(), next_path, next_seen));
                }
            }
        }
    }
    candidates.sort();
    (candidates.into_iter().next(), steps)
}

#[test]
fn optimized_cycle_matches_exhaustive_oracle_on_three_node_graphs() {
    for mask in 0u16..(1 << 9) {
        let edges = (0..9)
            .filter(|bit| mask & (1 << bit) != 0)
            .map(|bit| (bit / 3, bit % 3))
            .collect::<Vec<_>>();
        for reverse in [false, true] {
            let graph = graph_from_edges(3, &edges, reverse);
            let (expected, steps) = legacy_cycle(&graph);
            let mut exact = budget(steps.max(1)).begin_session_v2();
            assert_eq!(
                graph.detect_cycle(&mut exact).expect("exact step budget"),
                expected,
                "mask {mask:09b}, reverse {reverse}"
            );
            if steps > 1 {
                let mut under = budget(steps - 1).begin_session_v2();
                let error = graph.detect_cycle(&mut under).expect_err("one under fails");
                let exhaustion = error.resource_exhaustion().expect("step exhaustion");
                assert_eq!(exhaustion.dimension(), "steps");
                assert_eq!(exhaustion.limit(), steps - 1);
                assert_eq!(exhaustion.observed(), steps);
            }
        }
    }
}

#[test]
fn self_loop_witness_excludes_acyclic_prefix() {
    let graph = graph_from_edges(2, &[(0, 1), (1, 1)], false);
    let mut session = budget(8).begin_session_v2();
    let cycle = graph
        .detect_cycle(&mut session)
        .expect("cycle fits budget")
        .expect("self-loop is a cycle");
    assert_eq!(cycle.len(), 1);
    assert_eq!(cycle[0].from.as_str(), "m001");
    assert_eq!(cycle[0].to.as_str(), "m001");
    assert_eq!(cycle[0].location.span, SpanV1::new(5, 6));
}

#[test]
fn generic_witness_compares_equal_first_labels_by_complete_cycle() {
    let graph = vec![vec![(1, 0u8)], vec![(0, 2)], vec![(3, 0)], vec![(2, 1)]];
    assert_eq!(minimal_cycle_witness(&graph), Some(vec![0, 1]));
}

#[test]
fn missing_target_is_charged_once_without_becoming_cycle_vertex() {
    // Two dangling edges make a zero charge observable: with one edge and a
    // positive budget, charging 0 and charging 1 look identical.
    let graph = graph_from_edges(2, &[(0, 2), (1, 3)], false);
    let (expected, steps) = legacy_cycle(&graph);
    assert_eq!(expected, None);
    assert_eq!(steps, 2);
    let mut exact = budget(2).begin_session_v2();
    assert_eq!(graph.detect_cycle(&mut exact).expect("two steps fit"), None);
    let mut under = budget(1).begin_session_v2();
    let error = graph.detect_cycle(&mut under).expect_err("one under fails");
    let exhaustion = error.resource_exhaustion().expect("step exhaustion");
    assert_eq!(exhaustion.dimension(), "steps");
    assert_eq!(exhaustion.limit(), 1);
    assert_eq!(exhaustion.observed(), 2);
}

#[test]
fn deep_chain_and_branching_dag_do_not_enumerate_paths() {
    let chain = (0..255).map(|node| (node, node + 1)).collect::<Vec<_>>();
    let graph = graph_from_edges(256, &chain, false);
    let exact_steps = 256 * 255 / 2;
    let mut exact = budget(exact_steps).begin_session_v2();
    assert_eq!(
        graph.detect_cycle(&mut exact).expect("exact chain budget"),
        None
    );

    let mut branch = Vec::new();
    for layer in 0..18 {
        let current = layer * 2;
        let next = (layer + 1) * 2;
        branch.extend([
            (current, next),
            (current, next + 1),
            (current + 1, next),
            (current + 1, next + 1),
        ]);
    }
    let graph = graph_from_edges(38, &branch, true);
    let mut bounded = budget(1_000_000).begin_session_v2();
    let error = graph
        .detect_cycle(&mut bounded)
        .expect_err("branch path count exceeds budget");
    assert_eq!(
        error
            .resource_exhaustion()
            .expect("step exhaustion")
            .dimension(),
        "steps"
    );
}

#[test]
fn path_count_overflow_rejects_even_maximum_step_budget() {
    let edges = (0..92)
        .flat_map(|node| [(node, node + 1), (node, node + 2)])
        .collect::<Vec<_>>();
    let graph = graph_from_edges(94, &edges, false);
    let mut session = budget(usize::MAX).begin_session_v2();
    let error = graph
        .detect_cycle(&mut session)
        .expect_err("logical path count exceeds usize maximum");
    let exhaustion = error.resource_exhaustion().expect("step exhaustion");
    assert_eq!(exhaustion.dimension(), "steps");
    assert_eq!(exhaustion.limit(), usize::MAX);
    assert_eq!(exhaustion.observed(), usize::MAX);
}
