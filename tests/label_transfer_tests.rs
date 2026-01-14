use temporal_graph::TemporalGraph;

#[test]
fn test_transfer_labels_basic() {
    let mut graph = TemporalGraph::new();
    
    // Edge 0-1 with range [0, 20]
    graph.add_edge(0, 1, 0);
    graph.add_edge(0, 1, 20);
    
    // Neighbor 2 of vertex 0, with timestamp in range
    graph.add_edge(0, 2, 10);
    
    let transferred = graph.transfer_labels_through_edge(1, 0);
    
    assert_eq!(transferred, 1);
    
    // Edge 0-2 should no longer have timestamp 10
    assert!(!graph.has_edge_at_time(0, 2, 10));
    
    // Edge 2-1 should now have timestamp 10
    assert!(graph.has_edge_at_time(2, 1, 10));
}

#[test]
fn test_transfer_labels_multiple_neighbors() {
    let mut graph = TemporalGraph::new();
    
    // Edge 0-1 with range [0, 20]
    graph.add_edge(0, 1, 0);
    graph.add_edge(0, 1, 20);
    
    // Multiple neighbors with timestamps in range
    graph.add_edge(0, 2, 5);
    graph.add_edge(0, 3, 10);
    graph.add_edge(0, 4, 15);
    
    let transferred = graph.transfer_labels_through_edge(1, 0);
    
    assert_eq!(transferred, 3);
    
    // All labels should be transferred
    assert!(graph.has_edge_at_time(2, 1, 5));
    assert!(graph.has_edge_at_time(3, 1, 10));
    assert!(graph.has_edge_at_time(4, 1, 15));
    
    // Original edges should be removed (no timestamps left)
    assert!(graph.edge_times(0, 2).is_none());
    assert!(graph.edge_times(0, 3).is_none());
    assert!(graph.edge_times(0, 4).is_none());
}

#[test]
fn test_transfer_labels_excludes_v() {
    let mut graph = TemporalGraph::new();
    
    // Edge 0-1 with range [0, 20]
    graph.add_edge(0, 1, 0);
    graph.add_edge(0, 1, 20);
    graph.add_edge(0, 1, 10); // This timestamp is in range but edge 0-1 is excluded
    
    let transferred = graph.transfer_labels_through_edge(0, 1);
    
    assert_eq!(transferred, 0);
    
    // Edge 0-1 should still have timestamp 10
    assert!(graph.has_edge_at_time(0, 1, 10));
}

#[test]
fn test_transfer_labels_partial_transfer() {
    let mut graph = TemporalGraph::new();
    
    // Edge 0-1 with range [0, 20]
    graph.add_edge(0, 1, 0);
    graph.add_edge(0, 1, 20);
    
    // Neighbor with some timestamps in range, some not
    graph.add_edge(0, 2, 5);  // In range
    graph.add_edge(0, 2, 10); // In range
    graph.add_edge(0, 2, 25); // Out of range
    
    let transferred = graph.transfer_labels_through_edge(1, 0);
    
    assert_eq!(transferred, 2);
    
    // Only timestamps in range should be transferred
    assert!(graph.has_edge_at_time(2, 1, 5));
    assert!(graph.has_edge_at_time(2, 1, 10));
    
    // Timestamp 25 should remain on edge 0-2
    assert!(graph.has_edge_at_time(0, 2, 25));
    assert!(!graph.has_edge_at_time(2, 1, 25));
}

#[test]
fn test_transfer_labels_boundary_excluded() {
    let mut graph = TemporalGraph::new();
    
    // Edge 0-1 with range [0, 20]
    graph.add_edge(0, 1, 0);
    graph.add_edge(0, 1, 20);
    
    // Neighbors with timestamps at boundaries
    graph.add_edge(0, 2, 0);  // At tmin - should not transfer
    graph.add_edge(0, 3, 20); // At tmax - should not transfer
    
    let transferred = graph.transfer_labels_through_edge(0, 1);
    
    assert_eq!(transferred, 0);
    
    // Timestamps at boundaries should not be transferred
    assert!(graph.has_edge_at_time(0, 2, 0));
    assert!(graph.has_edge_at_time(0, 3, 20));
    assert!(!graph.has_edge_at_time(2, 1, 0));
    assert!(!graph.has_edge_at_time(3, 1, 20));
}

#[test]
fn test_transfer_labels_no_neighbors() {
    let mut graph = TemporalGraph::new();
    
    // Edge 0-1 with no other edges
    graph.add_edge(0, 1, 0);
    graph.add_edge(0, 1, 20);
    
    let transferred = graph.transfer_labels_through_edge(0, 1);
    
    assert_eq!(transferred, 0);
}

#[test]
fn test_transfer_labels_nonexistent_edge() {
    let mut graph = TemporalGraph::new();
    
    // Try to transfer on non-existent edge
    let transferred = graph.transfer_labels_through_edge(0, 1);
    
    assert_eq!(transferred, 0);
}

#[test]
fn test_get_all_neighbors() {
    let mut graph = TemporalGraph::new();
    
    graph.add_edge(0, 1, 5);
    graph.add_edge(0, 2, 10);
    graph.add_edge(0, 3, 15);
    graph.add_edge(1, 2, 20);
    
    let neighbors = graph.get_all_neighbors(0);
    assert_eq!(neighbors.len(), 3);
    assert!(neighbors.contains(&1));
    assert!(neighbors.contains(&2));
    assert!(neighbors.contains(&3));
    
    let neighbors_1 = graph.get_all_neighbors(1);
    assert_eq!(neighbors_1.len(), 2);
    assert!(neighbors_1.contains(&0));
    assert!(neighbors_1.contains(&2));
}

#[test]
fn test_transfer_creates_new_edges() {
    let mut graph = TemporalGraph::new();
    
    // Edge 0-1 with range [0, 20]
    graph.add_edge(0, 1, 0);
    graph.add_edge(0, 1, 20);
    
    // Edge 0-2 that doesn't initially connect to 1
    graph.add_edge(0, 2, 10);
    
    // Before transfer, edge 2-1 doesn't exist
    assert!(graph.edge_times(2, 1).is_none());
    
    graph.transfer_labels_through_edge(1, 0);
    
    // After transfer, edge 2-1 should exist with timestamp 10
    assert!(graph.edge_times(2, 1).is_some());
    assert!(graph.has_edge_at_time(2, 1, 10));
}

#[test]
fn test_transfer_merges_with_existing_edge() {
    let mut graph = TemporalGraph::new();
    
    // Edge 0-1 with range [0, 20]
    graph.add_edge(0, 1, 0);
    graph.add_edge(0, 1, 20);
    
    // Edge 0-2 with timestamp in range
    graph.add_edge(0, 2, 10);
    
    // Edge 2-1 already exists with different timestamp
    graph.add_edge(2, 1, 25);
    
    graph.transfer_labels_through_edge(1, 0);
    
    // Edge 2-1 should have both timestamps
    let times = graph.edge_times(2, 1).unwrap();
    assert_eq!(times.len(), 2);
    assert!(times.contains(&10));
    assert!(times.contains(&25));
}

#[test]
fn test_transfer_auto_cleanup_empty_edges() {
    let mut graph = TemporalGraph::new();
    
    // Edge 0-1 with range [0, 20]
    graph.add_edge(0, 1, 0);
    graph.add_edge(0, 1, 20);
    
    // Edge 0-2 with only one timestamp in range
    graph.add_edge(0, 2, 10);
    
    assert_eq!(graph.edge_count(), 2);
    
    graph.transfer_labels_through_edge(1, 0);
    
    // Edge 0-2 should be automatically removed (no timestamps left)
    assert_eq!(graph.edge_count(), 2); // 0-1 and 2-1 remain
    assert!(graph.edge_times(0, 2).is_none());
    assert!(graph.has_edge_at_time(2, 1, 10));
}

#[test]
fn test_transfer_preserves_edges_with_remaining_timestamps() {
    let mut graph = TemporalGraph::new();
    
    // Edge 0-1 with range [0, 20]
    graph.add_edge(0, 1, 0);
    graph.add_edge(0, 1, 20);
    
    // Edge 0-2 with timestamps both in and out of range
    graph.add_edge(0, 2, 10); // In range - will be transferred
    graph.add_edge(0, 2, 25); // Out of range - will remain
    
    graph.transfer_labels_through_edge(1, 0);
    
    // Edge 0-2 should still exist with timestamp 25
    assert!(graph.edge_times(0, 2).is_some());
    assert!(graph.has_edge_at_time(0, 2, 25));
    assert!(!graph.has_edge_at_time(0, 2, 10));
}
