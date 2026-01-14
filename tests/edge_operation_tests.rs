use temporal_graph::TemporalGraph;

#[test]
fn test_remove_single_timestamp_from_edge() {
    let mut graph = TemporalGraph::new();
    graph.add_edge(0, 1, 5);
    graph.add_edge(0, 1, 10);
    
    // Remove one timestamp
    assert!(graph.remove_edge_timestamp(0, 1, 5));
    
    assert!(!graph.has_edge_at_time(0, 1, 5));
    assert!(graph.has_edge_at_time(0, 1, 10));
}

#[test]
fn test_remove_all_timestamps_removes_edge() {
    let mut graph = TemporalGraph::new();
    graph.add_edge(0, 1, 5);
    graph.add_edge(0, 1, 10);
    
    // Remove edge entirely
    assert!(graph.remove_edge(0, 1));
    
    assert_eq!(graph.edge_count(), 0);
    assert!(graph.edge_times(0, 1).is_none());
}

#[test]
fn test_multiple_edges_independent() {
    let mut graph = TemporalGraph::new();
    graph.add_edge(0, 1, 5);
    graph.add_edge(1, 2, 5);
    graph.add_edge(2, 3, 5);
    
    // Remove one edge
    graph.remove_edge(0, 1);
    
    // Others should still exist
    assert_eq!(graph.edge_count(), 2);
    assert!(graph.has_edge_at_time(1, 2, 5));
    assert!(graph.has_edge_at_time(2, 3, 5));
}

#[test]
fn test_duplicate_timestamp_handling() {
    let mut graph = TemporalGraph::new();
    graph.add_edge(0, 1, 5);
    graph.add_edge(0, 1, 5); // Add same timestamp again
    
    let times = graph.edge_times(0, 1).unwrap();
    // Should only have one instance of timestamp 5
    assert_eq!(times.len(), 1);
    assert!(times.contains(&5));
}
