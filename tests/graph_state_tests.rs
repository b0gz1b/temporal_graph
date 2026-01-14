use temporal_graph::TemporalGraph;
use std::collections::HashSet;

#[test]
fn test_graph_state_equality_same_graph() {
    let mut graph1 = TemporalGraph::new();
    graph1.add_edge(0, 1, 5);
    graph1.add_edge(1, 2, 10);
    
    let mut graph2 = TemporalGraph::new();
    graph2.add_edge(0, 1, 5);
    graph2.add_edge(1, 2, 10);
    
    let state1 = graph1.to_state();
    let state2 = graph2.to_state();
    
    assert_eq!(state1, state2);
}

#[test]
fn test_graph_state_equality_different_order() {
    let mut graph1 = TemporalGraph::new();
    graph1.add_edge(0, 1, 5);
    graph1.add_edge(1, 2, 10);
    
    let mut graph2 = TemporalGraph::new();
    // Add in reverse order
    graph2.add_edge(1, 2, 10);
    graph2.add_edge(0, 1, 5);
    
    let state1 = graph1.to_state();
    let state2 = graph2.to_state();
    
    // Should be equal regardless of insertion order
    assert_eq!(state1, state2);
}

#[test]
fn test_graph_state_equality_edge_direction() {
    let mut graph1 = TemporalGraph::new();
    graph1.add_edge(0, 1, 5);
    
    let mut graph2 = TemporalGraph::new();
    // Add edge in opposite direction (undirected graph)
    graph2.add_edge(1, 0, 5);
    
    let state1 = graph1.to_state();
    let state2 = graph2.to_state();
    
    // Should be equal because graph is undirected
    assert_eq!(state1, state2);
}

#[test]
fn test_graph_state_inequality_different_timestamps() {
    let mut graph1 = TemporalGraph::new();
    graph1.add_edge(0, 1, 5);
    
    let mut graph2 = TemporalGraph::new();
    graph2.add_edge(0, 1, 10);
    
    let state1 = graph1.to_state();
    let state2 = graph2.to_state();
    
    assert_ne!(state1, state2);
}

#[test]
fn test_graph_state_inequality_different_edges() {
    let mut graph1 = TemporalGraph::new();
    graph1.add_edge(0, 1, 5);
    
    let mut graph2 = TemporalGraph::new();
    graph2.add_edge(0, 2, 5);
    
    let state1 = graph1.to_state();
    let state2 = graph2.to_state();
    
    assert_ne!(state1, state2);
}

#[test]
fn test_graph_state_hash_consistency() {
    let mut graph = TemporalGraph::new();
    graph.add_edge(0, 1, 5);
    graph.add_edge(1, 2, 10);
    
    let state1 = graph.to_state();
    let state2 = graph.to_state();
    
    // Same graph should produce equal states
    assert_eq!(state1, state2);
    
    // Hash should be consistent
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher1 = DefaultHasher::new();
    state1.hash(&mut hasher1);
    let hash1 = hasher1.finish();
    
    let mut hasher2 = DefaultHasher::new();
    state2.hash(&mut hasher2);
    let hash2 = hasher2.finish();
    
    assert_eq!(hash1, hash2);
}

#[test]
fn test_graph_state_in_hashset() {
    let mut graph1 = TemporalGraph::new();
    graph1.add_edge(0, 1, 5);
    
    let mut graph2 = TemporalGraph::new();
    graph2.add_edge(0, 1, 5);
    
    let mut graph3 = TemporalGraph::new();
    graph3.add_edge(0, 1, 10);
    
    let mut seen_states = HashSet::new();
    seen_states.insert(graph1.to_state());
    
    // Same state should be found
    assert!(seen_states.contains(&graph2.to_state()));
    
    // Different state should not be found
    assert!(!seen_states.contains(&graph3.to_state()));
}

#[test]
fn test_graph_state_multiple_timestamps_ordering() {
    let mut graph1 = TemporalGraph::new();
    graph1.add_edge(0, 1, 10);
    graph1.add_edge(0, 1, 5);
    graph1.add_edge(0, 1, 15);
    
    let mut graph2 = TemporalGraph::new();
    graph2.add_edge(0, 1, 5);
    graph2.add_edge(0, 1, 15);
    graph2.add_edge(0, 1, 10);
    
    let state1 = graph1.to_state();
    let state2 = graph2.to_state();
    
    // Should be equal regardless of timestamp insertion order
    assert_eq!(state1, state2);
}

#[test]
fn test_clone_graph() {
    let mut graph = TemporalGraph::new();
    graph.add_edge(0, 1, 5);
    graph.add_edge(1, 2, 10);
    graph.add_edge(2, 3, 15);
    
    let cloned = graph.clone_graph();
    
    // States should be equal
    assert_eq!(graph.to_state(), cloned.to_state());
    
    // Verify structure
    assert_eq!(graph.vertex_count(), cloned.vertex_count());
    assert_eq!(graph.edge_count(), cloned.edge_count());
}

#[test]
fn test_has_seen_state() {
    let mut graph = TemporalGraph::new();
    graph.add_edge(0, 1, 5);
    
    let mut seen_states = HashSet::new();
    assert!(!graph.has_seen_state(&seen_states));
    
    seen_states.insert(graph.to_state());
    assert!(graph.has_seen_state(&seen_states));
    
    // Modify graph
    graph.add_edge(1, 2, 10);
    assert!(!graph.has_seen_state(&seen_states));
}
