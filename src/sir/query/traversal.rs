//! FEAT-203-2: Graph traversal helpers.

use std::collections::BTreeSet;

use crate::sir::graph::SirGraph;

/// Return all ancestor node IDs reachable from `start_id` via incoming edges.
pub fn ancestors_of(graph: &SirGraph, start_id: &str) -> Vec<String> {
    let mut visited: BTreeSet<String> = BTreeSet::new();
    let mut queue: Vec<String> = Vec::new();

    for edge in graph.incoming(start_id) {
        if edge.source != start_id {
            queue.push(edge.source.clone());
        }
    }

    while let Some(current) = queue.pop() {
        if visited.contains(&current) {
            continue;
        }
        visited.insert(current.clone());
        for edge in graph.incoming(&current) {
            if !visited.contains(&edge.source) {
                queue.push(edge.source.clone());
            }
        }
    }

    visited.into_iter().collect()
}

/// Return all descendant node IDs reachable from `start_id` via outgoing edges.
pub fn descendants_of(graph: &SirGraph, start_id: &str) -> Vec<String> {
    let mut visited: BTreeSet<String> = BTreeSet::new();
    let mut queue: Vec<String> = Vec::new();

    for edge in graph.outgoing(start_id) {
        if edge.target != start_id {
            queue.push(edge.target.clone());
        }
    }

    while let Some(current) = queue.pop() {
        if visited.contains(&current) {
            continue;
        }
        visited.insert(current.clone());
        for edge in graph.outgoing(&current) {
            if !visited.contains(&edge.target) {
                queue.push(edge.target.clone());
            }
        }
    }

    visited.into_iter().collect()
}

/// Return all node IDs reachable from `root_id` within `max_depth` levels.
pub fn subgraph_nodes(
    graph: &SirGraph,
    root_id: &str,
    max_depth: Option<usize>,
) -> Vec<String> {
    let mut visited: BTreeSet<String> = BTreeSet::new();
    let mut queue: Vec<(String, usize)> = vec![(root_id.to_string(), 0)];

    while let Some((current, depth)) = queue.pop() {
        if visited.contains(&current) {
            continue;
        }
        visited.insert(current.clone());

        let next_depth = depth + 1;
        if max_depth.map_or(true, |max| next_depth <= max) {
            for edge in graph.outgoing(&current) {
                if !visited.contains(&edge.target) {
                    queue.push((edge.target.clone(), next_depth));
                }
            }
        }
    }

    visited.into_iter().collect()
}

/// Return the path from `from_id` to `to_id` (BFS, follows outgoing edges).
pub fn path_between(graph: &SirGraph, from_id: &str, to_id: &str) -> Option<Vec<String>> {
    if from_id == to_id {
        return Some(vec![from_id.to_string()]);
    }

    let mut visited: BTreeSet<String> = BTreeSet::new();
    let mut queue: Vec<(String, Vec<String>)> =
        vec![(from_id.to_string(), vec![from_id.to_string()])];

    while !queue.is_empty() {
        let (current, path) = queue.remove(0);

        if visited.contains(&current) {
            continue;
        }
        visited.insert(current.clone());

        for edge in graph.outgoing(&current) {
            let target = &edge.target;
            if target == to_id {
                let mut final_path = path.clone();
                final_path.push(target.clone());
                return Some(final_path);
            }
            if !visited.contains(target) {
                let mut new_path = path.clone();
                new_path.push(target.clone());
                queue.push((target.clone(), new_path));
            }
        }
    }

    None
}

/// Return all node IDs whose depth falls in `[min_depth, max_depth]`.
pub fn nodes_in_depth_range(
    depth_map: &std::collections::BTreeMap<String, usize>,
    min_depth: usize,
    max_depth: usize,
) -> Vec<String> {
    let mut result: Vec<String> = depth_map
        .iter()
        .filter(|(_, &d)| d >= min_depth && d <= max_depth)
        .map(|(id, _)| id.clone())
        .collect();
    result.sort();
    result
}
