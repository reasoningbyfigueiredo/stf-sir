//! FEAT-203-2: Graph traversal helpers.
//!
//! Pure traversal functions over `SirGraph` used by the query executor.
//! All functions return sorted, deduplicated collections for determinism.

use std::collections::BTreeSet;

use crate::sir::graph::SirGraph;

// ---------------------------------------------------------------------------
// Ancestors (transitive closure via incoming edges)
// ---------------------------------------------------------------------------

/// Return all ancestor node IDs reachable from `start_id` by following
/// incoming edges (transitive closure).
///
/// Uses iterative BFS with cycle detection. The start node itself is NOT
/// included in the result unless there is a self-loop.
///
/// Returns a sorted, deduplicated `Vec<String>`.
pub fn ancestors_of(graph: &SirGraph, start_id: &str) -> Vec<String> {
    let mut visited: BTreeSet<String> = BTreeSet::new();
    let mut queue: Vec<String> = Vec::new();

    // Seed with direct parents
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

// ---------------------------------------------------------------------------
// Descendants (transitive closure via outgoing edges)
// ---------------------------------------------------------------------------

/// Return all descendant node IDs reachable from `start_id` by following
/// outgoing edges (transitive closure).
///
/// Uses iterative BFS with cycle detection. The start node itself is NOT
/// included in the result unless there is a self-loop.
///
/// Returns a sorted, deduplicated `Vec<String>`.
pub fn descendants_of(graph: &SirGraph, start_id: &str) -> Vec<String> {
    let mut visited: BTreeSet<String> = BTreeSet::new();
    let mut queue: Vec<String> = Vec::new();

    // Seed with direct children
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

// ---------------------------------------------------------------------------
// Subgraph (BFS with optional depth limit)
// ---------------------------------------------------------------------------

/// Return all node IDs reachable from `root_id` within `max_depth` levels,
/// following outgoing edges.
///
/// If `max_depth` is `None`, returns the full transitive closure (same as
/// `descendants_of` but includes the root node).
///
/// Returns a sorted, deduplicated `Vec<String>`.
pub fn subgraph_nodes(
    graph: &SirGraph,
    root_id: &str,
    max_depth: Option<usize>,
) -> Vec<String> {
    let mut visited: BTreeSet<String> = BTreeSet::new();
    // Queue entries: (node_id, current_depth)
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

// ---------------------------------------------------------------------------
// Path between two nodes
// ---------------------------------------------------------------------------

/// Return the IDs of nodes on a shortest path from `from_id` to `to_id`,
/// following outgoing edges (BFS).
///
/// Returns `None` if no path exists. The returned `Vec` includes both
/// endpoints (sorted for determinism in the result set, but the path itself
/// is unordered since we return the set of nodes on the path).
pub fn path_between(graph: &SirGraph, from_id: &str, to_id: &str) -> Option<Vec<String>> {
    if from_id == to_id {
        return Some(vec![from_id.to_string()]);
    }

    // BFS: each entry is (current_id, path_so_far)
    let mut visited: BTreeSet<String> = BTreeSet::new();
    let mut queue: Vec<(String, Vec<String>)> =
        vec![(from_id.to_string(), vec![from_id.to_string()])];

    while let Some((current, path)) = queue.first().cloned() {
        queue.remove(0);

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

// ---------------------------------------------------------------------------
// Nodes by depth range
// ---------------------------------------------------------------------------

/// Return all node IDs whose syntactic depth falls in `[min_depth, max_depth]`
/// (inclusive), given a mapping of node_id → depth.
///
/// The caller is responsible for providing the depth map (typically derived
/// from `ZToken.syntactic.depth` via the `Artifact`).
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
