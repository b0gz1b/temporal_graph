use crate::{TemporalGraph, TimeStep, VertexId};
use std::collections::{HashMap, HashSet, VecDeque};

/// Result of a temporal path search between two vertices.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemporalPathResult {
    /// Whether a time-respecting path exists.
    pub reachable: bool,
    /// The earliest arrival time at the target, if reachable.
    pub earliest_arrival: Option<TimeStep>,
}

impl TemporalGraph {
    /// Check whether there is a time-respecting path from `source` to `target`.
    ///
    /// A time-respecting path is a sequence of edges whose timestamps are
    /// monotonically increasing along the path.
    ///
    /// # Parameters
    /// - `source`: starting vertex
    /// - `target`: destination vertex
    /// - `strict`: if `true`, timestamps must be *strictly* increasing
    ///   (t1 < t2 < ...); if `false`, non-decreasing is allowed (t1 <= t2 <= ...).
    ///
    /// Uses BFS over states `(current_vertex, last_used_timestamp)` to find
    /// the earliest-arrival path.
    pub fn has_time_respecting_path(
        &self,
        source: VertexId,
        target: VertexId,
        strict: bool,
    ) -> TemporalPathResult {
        if !self.has_vertex(source) || !self.has_vertex(target) {
            return TemporalPathResult {
                reachable: false,
                earliest_arrival: None,
            };
        }

        if source == target {
            return TemporalPathResult {
                reachable: true,
                earliest_arrival: None, // trivially reachable, no edge needed
            };
        }

        // BFS state: (vertex, last_timestamp_used)
        // We track the minimum last_timestamp seen for each vertex to prune
        // dominated states (arrived later via a higher timestamp).
        // best_arrival[v] = minimum last_timestamp used to reach v
        let mut best_arrival: HashMap<VertexId, TimeStep> = HashMap::new();

        // Queue entries: (current_vertex, last_timestamp)
        // Use i64::MIN as sentinel for the source (no previous edge constraint).
        let mut queue: VecDeque<(VertexId, TimeStep)> = VecDeque::new();
        queue.push_back((source, TimeStep::MIN));
        best_arrival.insert(source, TimeStep::MIN);

        while let Some((current, last_t)) = queue.pop_front() {
            // Collect all edges incident to `current` and their timestamps
            for ((u, v), edge) in &self.edges {
                // Determine if this edge involves `current`, and find the neighbor
                let neighbor = if *u == current {
                    *v
                } else if *v == current {
                    *u
                } else {
                    continue;
                };

                // Find all valid timestamps on this edge (respecting ordering)
                let mut times: Vec<TimeStep> = edge
                    .timestamps
                    .iter()
                    .copied()
                    .filter(|&t| {
                        if last_t == TimeStep::MIN {
                            true // no constraint for the first hop
                        } else if strict {
                            t > last_t
                        } else {
                            t >= last_t
                        }
                    })
                    .collect();

                if times.is_empty() {
                    continue;
                }

                // Pick the earliest valid timestamp to minimise arrival time
                times.sort_unstable();
                let best_t = times[0];

                // Prune: only enqueue if this is a better (earlier) arrival at `neighbor`
                let dominated = best_arrival
                    .get(&neighbor)
                    .map(|&prev| prev <= best_t)
                    .unwrap_or(false);

                if !dominated {
                    best_arrival.insert(neighbor, best_t);

                    if neighbor == target {
                        // Found the target — BFS guarantees this is the earliest arrival
                        return TemporalPathResult {
                            reachable: true,
                            earliest_arrival: Some(best_t),
                        };
                    }

                    queue.push_back((neighbor, best_t));
                }
            }
        }

        TemporalPathResult {
            reachable: false,
            earliest_arrival: None,
        }
    }

    /// Check whether the graph is temporally connected.
    ///
    /// A temporal graph is temporally connected if for every **ordered** pair
    /// of distinct vertices `(s, t)` there exists a time-respecting path from
    /// `s` to `t`.
    ///
    /// Note: temporal connectivity is **not symmetric** in general — a path
    /// from `s` to `t` does not imply one from `t` to `s`.
    ///
    /// # Parameters
    /// - `strict`: forwarded to `has_time_respecting_path`.
    pub fn is_temporally_connected(&self, strict: bool) -> bool {
        let vertices = self.vertices();

        if vertices.len() <= 1 {
            return true;
        }

        for &s in &vertices {
            for &t in &vertices {
                if s != t && !self.has_time_respecting_path(s, t, strict).reachable {
                    return false;
                }
            }
        }

        true
    }

    /// Return the set of vertices reachable from `source` via time-respecting paths.
    ///
    /// # Parameters
    /// - `strict`: forwarded to the BFS (strictly vs. non-decreasing timestamps).
    pub fn reachable_from(&self, source: VertexId, strict: bool) -> HashSet<VertexId> {
        let mut reachable = HashSet::new();

        if !self.has_vertex(source) {
            return reachable;
        }

        reachable.insert(source);

        // BFS over (vertex, last_timestamp)
        let mut best_arrival: HashMap<VertexId, TimeStep> = HashMap::new();
        best_arrival.insert(source, TimeStep::MIN);

        let mut queue: VecDeque<(VertexId, TimeStep)> = VecDeque::new();
        queue.push_back((source, TimeStep::MIN));

        while let Some((current, last_t)) = queue.pop_front() {
            for ((u, v), edge) in &self.edges {
                let neighbor = if *u == current {
                    *v
                } else if *v == current {
                    *u
                } else {
                    continue;
                };

                let mut times: Vec<TimeStep> = edge
                    .timestamps
                    .iter()
                    .copied()
                    .filter(|&t| {
                        if last_t == TimeStep::MIN {
                            true
                        } else if strict {
                            t > last_t
                        } else {
                            t >= last_t
                        }
                    })
                    .collect();

                if times.is_empty() {
                    continue;
                }

                times.sort_unstable();
                let best_t = times[0];

                let dominated = best_arrival
                    .get(&neighbor)
                    .map(|&prev| prev <= best_t)
                    .unwrap_or(false);

                if !dominated {
                    best_arrival.insert(neighbor, best_t);
                    reachable.insert(neighbor);
                    queue.push_back((neighbor, best_t));
                }
            }
        }

        reachable
    }

    /// Compute the earliest arrival time from `source` to every other vertex.
    ///
    /// Returns a map from `VertexId` to the earliest `TimeStep` at which
    /// that vertex can be reached via a time-respecting path from `source`.
    /// Vertices that are not reachable are absent from the map.
    /// The source itself is not included.
    ///
    /// # Parameters
    /// - `strict`: forwarded to the BFS.
    pub fn earliest_arrival_times(
        &self,
        source: VertexId,
        strict: bool,
    ) -> HashMap<VertexId, TimeStep> {
        let mut best_arrival: HashMap<VertexId, TimeStep> = HashMap::new();

        if !self.has_vertex(source) {
            return best_arrival;
        }

        let mut queue: VecDeque<(VertexId, TimeStep)> = VecDeque::new();
        // Sentinel: source has no previous timestamp constraint
        let mut source_visited: HashMap<VertexId, TimeStep> = HashMap::new();
        source_visited.insert(source, TimeStep::MIN);
        queue.push_back((source, TimeStep::MIN));

        while let Some((current, last_t)) = queue.pop_front() {
            for ((u, v), edge) in &self.edges {
                let neighbor = if *u == current {
                    *v
                } else if *v == current {
                    *u
                } else {
                    continue;
                };

                let mut times: Vec<TimeStep> = edge
                    .timestamps
                    .iter()
                    .copied()
                    .filter(|&t| {
                        if last_t == TimeStep::MIN {
                            true
                        } else if strict {
                            t > last_t
                        } else {
                            t >= last_t
                        }
                    })
                    .collect();

                if times.is_empty() {
                    continue;
                }

                times.sort_unstable();
                let best_t = times[0];

                // Update best arrival at neighbor (minimise arrival timestamp)
                let improved = best_arrival
                    .get(&neighbor)
                    .map(|&prev| best_t < prev)
                    .unwrap_or(true);

                if improved {
                    best_arrival.insert(neighbor, best_t);
                }

                // Continue BFS if state is not dominated
                let dominated = source_visited
                    .get(&neighbor)
                    .map(|&prev| prev <= best_t)
                    .unwrap_or(false);

                if !dominated {
                    source_visited.insert(neighbor, best_t);
                    queue.push_back((neighbor, best_t));
                }
            }
        }

        best_arrival
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper: simple path 0 -[1]- 1 -[2]- 2 -[3]- 3
    // Note: edges are undirected, so from any vertex you can take a first
    // hop in any direction. This graph IS fully temporally connected because
    // each single edge can always be used as a first hop.
    fn path_graph() -> TemporalGraph {
        let mut g = TemporalGraph::new();
        g.add_edge(0, 1, 1);
        g.add_edge(1, 2, 2);
        g.add_edge(2, 3, 3);
        g
    }

    /// A graph that is NOT temporally connected in the strict sense:
    /// 0 -[1]- 1 -[3]- 2
    /// From 0: reach 1 at t=1, then reach 2 at t=3. OK.
    /// From 2: reach 1 at t=3 (first hop, no constraint). Then to reach 0
    ///   we need an edge {0,1} with t > 3, but it only has t=1. Blocked.
    fn one_way_graph() -> TemporalGraph {
        let mut g = TemporalGraph::new();
        g.add_edge(0, 1, 1);
        g.add_edge(1, 2, 3);
        g
    }

    #[test]
    fn test_trivial_self_reachable() {
        let g = path_graph();
        let r = g.has_time_respecting_path(0, 0, true);
        assert!(r.reachable);
    }

    #[test]
    fn test_direct_path_strict() {
        let g = path_graph();
        // 0 -> 3 via t=1,2,3
        assert!(g.has_time_respecting_path(0, 3, true).reachable);
    }

    #[test]
    fn test_no_path_wrong_direction() {
        // From 2 in one_way_graph: first hop 2->1 at t=3, then need
        // edge {0,1} with t > 3 — only t=1 exists. Cannot reach 0.
        let g = one_way_graph();
        assert!(!g.has_time_respecting_path(2, 0, true).reachable);
        // Forward direction 0->2 must still work
        assert!(g.has_time_respecting_path(0, 2, true).reachable);
    }

    #[test]
    fn test_strict_vs_nonstrict() {
        // Edge 0-1 at t=5, edge 1-2 at t=5
        // Strict: second hop needs t > 5, but only t=5 exists. Blocked.
        // Non-strict: t >= 5 allowed, so 0->2 is possible.
        let mut g = TemporalGraph::new();
        g.add_edge(0, 1, 5);
        g.add_edge(1, 2, 5);

        assert!(!g.has_time_respecting_path(0, 2, true).reachable);
        assert!(g.has_time_respecting_path(0, 2, false).reachable);
    }

    #[test]
    fn test_earliest_arrival() {
        // 0 -[1,2]- 1 -[3]- 2: earliest arrival at 2 is t=3
        let mut g = TemporalGraph::new();
        g.add_edge(0, 1, 1);
        g.add_edge(0, 1, 2);
        g.add_edge(1, 2, 3);

        let r = g.has_time_respecting_path(0, 2, true);
        assert!(r.reachable);
        assert_eq!(r.earliest_arrival, Some(3));
    }

    #[test]
    fn test_is_temporally_connected_true() {
        // 0 -[1,2]- 1: both 0->1 (t=1) and 1->0 (t=2 first hop) work.
        let mut g = TemporalGraph::new();
        g.add_edge(0, 1, 1);
        g.add_edge(0, 1, 2);
        assert!(g.is_temporally_connected(true));
    }

    #[test]
    fn test_is_temporally_connected_false() {
        // one_way_graph: 2 cannot reach 0 via strictly increasing timestamps.
        let g = one_way_graph();
        assert!(!g.is_temporally_connected(true));
    }

    #[test]
    fn test_reachable_from_forward() {
        // From 0 in path_graph all vertices are reachable
        let g = path_graph();
        let r = g.reachable_from(0, true);
        assert!(r.contains(&0));
        assert!(r.contains(&1));
        assert!(r.contains(&2));
        assert!(r.contains(&3));
    }

    #[test]
    fn test_reachable_from_reverse() {
        // From 2 in one_way_graph: can reach 1 (t=3 first hop) but NOT 0
        // (would need t > 3 on edge {0,1}, only t=1 available).
        let g = one_way_graph();
        let r = g.reachable_from(2, true);
        assert!(r.contains(&2));  // source itself
        assert!(r.contains(&1));  // reachable via first hop t=3
        assert!(!r.contains(&0)); // blocked: no t > 3 on {0,1}
        assert_eq!(r.len(), 2);
    }

    #[test]
    fn test_earliest_arrival_times() {
        let g = path_graph();
        let arrivals = g.earliest_arrival_times(0, true);
        assert_eq!(arrivals.get(&1), Some(&1));
        assert_eq!(arrivals.get(&2), Some(&2));
        assert_eq!(arrivals.get(&3), Some(&3));
        assert!(!arrivals.contains_key(&0)); // source excluded
    }

    #[test]
    fn test_disconnected_graph() {
        // Two separate components with no path between them
        let mut g = TemporalGraph::new();
        g.add_edge(0, 1, 1);
        g.add_edge(2, 3, 2);

        assert!(!g.has_time_respecting_path(0, 3, true).reachable);
        assert!(!g.is_temporally_connected(true));
    }

    #[test]
    fn test_unknown_vertex() {
        let g = path_graph();
        let r = g.has_time_respecting_path(0, 99, true);
        assert!(!r.reachable);
    }
}
