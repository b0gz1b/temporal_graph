use std::collections::{HashMap, HashSet};
use std::hash::Hash;

pub type VertexId = usize;
pub type TimeStep = i64;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GraphState {
    // Sorted representation for canonical comparison
    edge_labels: Vec<((VertexId, VertexId), Vec<TimeStep>)>,
}

// Undirected edge representation with temporal information
#[derive(Debug, Clone)]
pub struct TemporalEdge {
    pub u: VertexId,
    pub v: VertexId,
    pub timestamps: HashSet<TimeStep>,
}

impl TemporalEdge {
    // Normalize edge so smaller vertex ID comes first
    fn normalize_pair(a: VertexId, b: VertexId) -> (VertexId, VertexId) {
        if a <= b { (a, b) } else { (b, a) }
    }
}

#[derive(Debug)]
pub struct TemporalGraph {
    vertices: HashSet<VertexId>,
    // Map normalized (min, max) pairs to temporal edges for undirected edges
    edges: HashMap<(VertexId, VertexId), TemporalEdge>,
    vertex_labels: HashMap<VertexId, String>,
}

impl Default for TemporalGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl TemporalGraph {
    pub fn new() -> Self {
        Self {
            vertices: HashSet::new(),
            edges: HashMap::new(),
            vertex_labels: HashMap::new(),
        }
    }

    pub fn add_vertex(&mut self, id: VertexId) -> bool {
        self.vertices.insert(id)
    }

    // Add undirected edge at specific time
    pub fn add_edge(&mut self, u: VertexId, v: VertexId, time: TimeStep) {
        self.add_vertex(u);
        self.add_vertex(v);

        // Normalize to ensure {u,v} == {v,u}
        let (u_norm, v_norm) = TemporalEdge::normalize_pair(u, v);

        self.edges
            .entry((u_norm, v_norm))
            .or_insert_with(|| TemporalEdge {
                u: u_norm,
                v: v_norm,
                timestamps: HashSet::new(),
            })
            .timestamps
            .insert(time);
    }

    // Check if edge exists at given time (order-independent)
    pub fn has_edge_at_time(&self, u: VertexId, v: VertexId, time: TimeStep) -> bool {
        let (u_norm, v_norm) = TemporalEdge::normalize_pair(u, v);
        self.edges
            .get(&(u_norm, v_norm))
            .map(|edge| edge.timestamps.contains(&time))
            .unwrap_or(false)
    }

    pub fn edge_times(&self, u: VertexId, v: VertexId) -> Option<Vec<TimeStep>> {
        let (u_norm, v_norm) = if u <= v { (u, v) } else { (v, u) };
        self.edges.get(&(u_norm, v_norm)).map(|edge| {
            let mut times: Vec<TimeStep> = edge.timestamps.iter().copied().collect();
            times.sort_unstable();
            times
        })
    }
    /// Get the number of vertices
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// Get the number of edges
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Check if a vertex exists in the graph
    pub fn has_vertex(&self, v: VertexId) -> bool {
        self.vertices.contains(&v)
    }

    /// Get all vertices
    pub fn vertices(&self) -> Vec<VertexId> {
        let mut verts: Vec<VertexId> = self.vertices.iter().copied().collect();
        verts.sort_unstable();
        verts
    }

    /// Remove a specific timestamp from an edge
    pub fn remove_edge_timestamp(&mut self, u: VertexId, v: VertexId, time: TimeStep) -> bool {
        let (u_norm, v_norm) = if u <= v { (u, v) } else { (v, u) };

        if let Some(edge) = self.edges.get_mut(&(u_norm, v_norm)) {
            let removed = edge.timestamps.remove(&time);

            // Cleanup: if edge has no more timestamps, remove it entirely
            if removed && edge.timestamps.is_empty() {
                self.edges.remove(&(u_norm, v_norm));
            }

            removed
        } else {
            false
        }
    }

    /// Remove an edge entirely
    pub fn remove_edge(&mut self, u: VertexId, v: VertexId) -> bool {
        let (u_norm, v_norm) = if u <= v { (u, v) } else { (v, u) };
        self.edges.remove(&(u_norm, v_norm)).is_some()
    }
    pub fn clone_graph(&self) -> Self {
        TemporalGraph {
            vertices: self.vertices.clone(),
            edges: self.edges.clone(),
            vertex_labels: self.vertex_labels.clone(),
        }
    }

    // Get neighbors of vertex at specific time
    pub fn neighbors_at_time(&self, vertex: VertexId, time: TimeStep) -> Vec<VertexId> {
        self.edges
            .iter()
            .filter(|(_, edge)| edge.timestamps.contains(&time))
            .filter_map(|((u, v), _)| {
                if *u == vertex {
                    Some(*v)
                } else if *v == vertex {
                    Some(*u)
                } else {
                    None
                }
            })
            .collect()
    }

    // Get all edges active at specific time
    pub fn edges_at_time(&self, time: TimeStep) -> Vec<(VertexId, VertexId)> {
        self.edges
            .iter()
            .filter(|(_, edge)| edge.timestamps.contains(&time))
            .map(|((u, v), _)| (*u, *v))
            .collect()
    }

    pub fn to_state(&self) -> GraphState {
        let mut edge_labels: Vec<((VertexId, VertexId), Vec<TimeStep>)> = self
            .edges
            .iter()
            .map(|((u, v), edge)| {
                let mut times: Vec<TimeStep> = edge.timestamps.iter().copied().collect();
                times.sort_unstable();
                ((*u, *v), times)
            })
            .collect();

        // Sort edges for canonical representation
        edge_labels.sort_by_key(|(edge, _)| *edge);

        GraphState { edge_labels }
    }

    pub fn has_seen_state(&self, seen_states: &HashSet<GraphState>) -> bool {
        seen_states.contains(&self.to_state())
    }
    pub fn print_state(&self) {
        println!("Graph State:");

        if self.edge_count() == 0 {
            println!("  (no edges)");
            return;
        }

        let mut edges: Vec<_> = self.edges.iter().collect();
        edges.sort_by_key(|(k, _)| *k);

        for ((u, v), edge) in edges {
            let mut times: Vec<TimeStep> = edge.timestamps.iter().copied().collect();
            times.sort_unstable();
            let times_str = times
                .iter()
                .map(|t| t.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            println!("    {} -- {} : [{}]", u, v, times_str);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_vertex() {
        let mut graph = TemporalGraph::new();
        assert!(graph.add_vertex(0));
        assert!(graph.add_vertex(1));
        // Adding same vertex again should return false
        assert!(!graph.add_vertex(0));
        assert_eq!(graph.vertices.len(), 2);
    }

    #[test]
    fn test_add_edge_creates_vertices() {
        let mut graph = TemporalGraph::new();
        graph.add_edge(0, 1, 5);

        // Vertices should be automatically created
        assert!(graph.vertices.contains(&0));
        assert!(graph.vertices.contains(&1));
        assert_eq!(graph.vertices.len(), 2);
    }

    #[test]
    fn test_add_edge_multiple_timestamps() {
        let mut graph = TemporalGraph::new();
        graph.add_edge(0, 1, 0);
        graph.add_edge(0, 1, 5);
        graph.add_edge(0, 1, 10);

        let times = graph.edge_times(0, 1);
        assert!(times.is_some());
        let times = times.unwrap();
        assert_eq!(times.len(), 3);
        assert!(times.contains(&0));
        assert!(times.contains(&5));
        assert!(times.contains(&10));
    }

    #[test]
    fn test_edge_undirected() {
        let mut graph = TemporalGraph::new();
        graph.add_edge(0, 1, 5);

        // Both orderings should find the same edge
        assert!(graph.edge_times(0, 1).is_some());
        assert!(graph.edge_times(1, 0).is_some());

        // Both should return the same timestamps
        assert_eq!(graph.edge_times(0, 1), graph.edge_times(1, 0));
    }

    #[test]
    fn test_edges_at_time() {
        let mut graph = TemporalGraph::new();
        graph.add_edge(0, 1, 0);
        graph.add_edge(0, 1, 5);
        graph.add_edge(1, 2, 5);
        graph.add_edge(2, 3, 10);

        let edges_at_0 = graph.edges_at_time(0);
        assert_eq!(edges_at_0.len(), 1);

        let edges_at_5 = graph.edges_at_time(5);
        assert_eq!(edges_at_5.len(), 2);

        let edges_at_10 = graph.edges_at_time(10);
        assert_eq!(edges_at_10.len(), 1);

        let edges_at_100 = graph.edges_at_time(100);
        assert_eq!(edges_at_100.len(), 0);
    }

    #[test]
    fn test_neighbors_at_time() {
        let mut graph = TemporalGraph::new();
        graph.add_edge(0, 1, 5);
        graph.add_edge(0, 2, 5);
        graph.add_edge(1, 2, 5);
        graph.add_edge(0, 3, 10);

        let neighbors_0_at_5 = graph.neighbors_at_time(0, 5);
        assert_eq!(neighbors_0_at_5.len(), 2);
        assert!(neighbors_0_at_5.contains(&1));
        assert!(neighbors_0_at_5.contains(&2));

        let neighbors_0_at_10 = graph.neighbors_at_time(0, 10);
        assert_eq!(neighbors_0_at_10.len(), 1);
        assert!(neighbors_0_at_10.contains(&3));
    }

    #[test]
    fn test_has_edge_at_time() {
        let mut graph = TemporalGraph::new();
        graph.add_edge(0, 1, 5);
        graph.add_edge(0, 1, 10);

        assert!(graph.has_edge_at_time(0, 1, 5));
        assert!(graph.has_edge_at_time(0, 1, 10));
        assert!(!graph.has_edge_at_time(0, 1, 7));

        // Test undirected property
        assert!(graph.has_edge_at_time(1, 0, 5));
        assert!(graph.has_edge_at_time(1, 0, 10));
    }

    #[test]
    fn test_edge_normalization() {
        let mut graph = TemporalGraph::new();

        // Add edge in both directions
        graph.add_edge(0, 1, 5);
        graph.add_edge(1, 0, 10);

        // Should be stored as single edge with both timestamps
        assert_eq!(graph.edges.len(), 1);

        let times = graph.edge_times(0, 1).unwrap();
        assert_eq!(times.len(), 2);
        assert!(times.contains(&5));
        assert!(times.contains(&10));
    }

    #[test]
    fn test_empty_graph() {
        let graph = TemporalGraph::new();
        assert_eq!(graph.vertices.len(), 0);
        assert_eq!(graph.edges.len(), 0);
        assert_eq!(graph.edges_at_time(0).len(), 0);
    }

    #[test]
    fn test_isolated_vertices() {
        let mut graph = TemporalGraph::new();
        graph.add_vertex(0);
        graph.add_vertex(1);
        graph.add_vertex(2);

        assert_eq!(graph.vertices.len(), 3);
        assert_eq!(graph.edges.len(), 0);
        assert_eq!(graph.neighbors_at_time(0, 0).len(), 0);
    }

    #[test]
    fn test_remove_edge_timestamp_auto_cleanup() {
        let mut graph = TemporalGraph::new();
        graph.add_edge(0, 1, 5);

        assert_eq!(graph.edge_count(), 1);

        // Remove the only timestamp
        assert!(graph.remove_edge_timestamp(0, 1, 5));

        // Edge should be automatically removed
        assert_eq!(graph.edge_count(), 0);
        assert!(graph.edge_times(0, 1).is_none());
    }

    #[test]
    fn test_remove_edge_timestamp_keeps_edge_with_remaining_timestamps() {
        let mut graph = TemporalGraph::new();
        graph.add_edge(0, 1, 5);
        graph.add_edge(0, 1, 10);

        assert_eq!(graph.edge_count(), 1);

        // Remove one timestamp
        assert!(graph.remove_edge_timestamp(0, 1, 5));

        // Edge should still exist with remaining timestamp
        assert_eq!(graph.edge_count(), 1);
        assert!(graph.has_edge_at_time(0, 1, 10));
        assert!(!graph.has_edge_at_time(0, 1, 5));
    }

    #[test]
    fn test_remove_multiple_timestamps_sequential() {
        let mut graph = TemporalGraph::new();
        graph.add_edge(0, 1, 5);
        graph.add_edge(0, 1, 10);
        graph.add_edge(0, 1, 15);

        // Remove timestamps one by one
        assert!(graph.remove_edge_timestamp(0, 1, 5));
        assert_eq!(graph.edge_count(), 1); // Still has 2 timestamps

        assert!(graph.remove_edge_timestamp(0, 1, 10));
        assert_eq!(graph.edge_count(), 1); // Still has 1 timestamp

        assert!(graph.remove_edge_timestamp(0, 1, 15));
        assert_eq!(graph.edge_count(), 0); // Now removed
        assert!(graph.edge_times(0, 1).is_none());
    }
}
pub mod minimization;
pub use minimization::{
    MinimizationConfig, MinimizationResult, MinimizationStats, TerminationReason,
};
pub mod enumeration;
pub mod visualization;
pub use enumeration::{
    generate_multigraphs_nauty, generate_temporal_graphs_from_multigraphs,
    read_temporal_graphs_from_file,
};
