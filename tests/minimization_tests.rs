use temporal_graph::TemporalGraph;

#[test]
fn test_find_wrappable_edge_exists() {
    let mut graph = TemporalGraph::new();

    // Edge 0-1 with timestamps [0, 10]
    graph.add_edge(0, 1, 0);
    graph.add_edge(0, 1, 10);

    // Incident edge 1-2 with timestamp 5 (between 0 and 10)
    graph.add_edge(1, 2, 5);

    let result = graph.find_wrappable_edge();
    assert!(result.is_some());

    let (u, v) = result.unwrap();
    // Should find edge 0-1 (order might be swapped due to normalization)
    assert!((u == 0 && v == 1) || (u == 1 && v == 0));
}

#[test]
fn test_find_wrappable_edge_none_exists() {
    let mut graph = TemporalGraph::new();

    // Edge 0-1 with timestamps [0, 10]
    graph.add_edge(0, 1, 0);
    graph.add_edge(0, 1, 10);

    // Incident edge 1-2 with timestamp 15 (not between 0 and 10)
    graph.add_edge(1, 2, 15);

    let result = graph.find_wrappable_edge();
    assert!(result.is_none());
}

#[test]
fn test_find_wrappable_edge_single_label() {
    let mut graph = TemporalGraph::new();

    // Edge with only one label
    graph.add_edge(0, 1, 5);

    // Incident edge
    graph.add_edge(1, 2, 3);

    let result = graph.find_wrappable_edge();
    // Should return None because edge 0-1 has only one label
    assert!(result.is_none());
}

#[test]
fn test_find_wrappable_edge_boundary_not_included() {
    let mut graph = TemporalGraph::new();

    // Edge 0-1 with timestamps [0, 10]
    graph.add_edge(0, 1, 0);
    graph.add_edge(0, 1, 10);

    // Incident edges with timestamps at boundaries (should not count)
    graph.add_edge(1, 2, 0);
    graph.add_edge(1, 3, 10);

    let result = graph.find_wrappable_edge();
    // Should return None because 0 and 10 are not strictly between tmin and tmax
    assert!(result.is_none());
}

#[test]
fn test_find_wrappable_edge_multiple_incident() {
    let mut graph = TemporalGraph::new();

    // Edge 0-1 with timestamps [0, 20]
    graph.add_edge(0, 1, 0);
    graph.add_edge(0, 1, 20);

    // Multiple incident edges, one has timestamp in range
    graph.add_edge(0, 2, 25); // Not in range
    graph.add_edge(1, 2, 10); // In range (0 < 10 < 20)
    graph.add_edge(1, 3, 30); // Not in range

    let result = graph.find_wrappable_edge();
    assert!(result.is_some());
}
#[test]
fn test_find_min_incident_in_range_basic() {
    let mut graph = TemporalGraph::new();

    // Edge 0-1 with timestamps [0, 10]
    graph.add_edge(0, 1, 0);
    graph.add_edge(0, 1, 10);

    // Incident edges with various timestamps
    graph.add_edge(1, 2, 5); // In range, min
    graph.add_edge(0, 3, 7); // In range
    graph.add_edge(1, 4, 12); // Out of range (too large)

    let result = graph.find_min_incident_in_range(0, 1);
    assert!(result.is_some());

    let (u, v, t) = result.unwrap();
    // Should find edge 1-2 with timestamp 5 (the minimum in range)
    assert_eq!(t, 5);
    assert!((u == 1 && v == 2) || (u == 2 && v == 1));
}

#[test]
fn test_find_min_incident_in_range_no_incident() {
    let mut graph = TemporalGraph::new();

    // Edge 0-1 with timestamps [0, 10]
    graph.add_edge(0, 1, 0);
    graph.add_edge(0, 1, 10);

    // Incident edges but no timestamps in range
    graph.add_edge(1, 2, 15);
    graph.add_edge(0, 3, 20);

    let result = graph.find_min_incident_in_range(0, 1);
    assert!(result.is_none());
}

#[test]
fn test_find_min_incident_in_range_single_timestamp() {
    let mut graph = TemporalGraph::new();

    // Edge with only one timestamp
    graph.add_edge(0, 1, 5);

    let result = graph.find_min_incident_in_range(0, 1);
    // Should return None because edge needs at least 2 timestamps
    assert!(result.is_none());
}

#[test]
fn test_find_min_incident_in_range_boundary_excluded() {
    let mut graph = TemporalGraph::new();

    // Edge 0-1 with timestamps [0, 10]
    graph.add_edge(0, 1, 0);
    graph.add_edge(0, 1, 10);

    // Incident edges at boundaries
    graph.add_edge(1, 2, 0); // At tmin, should be excluded
    graph.add_edge(0, 3, 10); // At tmax, should be excluded

    let result = graph.find_min_incident_in_range(0, 1);
    assert!(result.is_none());
}

#[test]
fn test_find_min_incident_in_range_multiple_choices() {
    let mut graph = TemporalGraph::new();

    // Edge 0-1 with timestamps [0, 20]
    graph.add_edge(0, 1, 0);
    graph.add_edge(0, 1, 20);

    // Multiple incident edges in range
    graph.add_edge(1, 2, 15); // In range but not min
    graph.add_edge(0, 3, 10); // In range but not min
    graph.add_edge(1, 4, 5); // In range, this is min
    graph.add_edge(0, 5, 8); // In range but not min

    let result = graph.find_min_incident_in_range(0, 1);
    assert!(result.is_some());

    let (_, _, t) = result.unwrap();
    assert_eq!(t, 5); // Should return minimum timestamp
}

#[test]
fn test_find_min_incident_in_range_same_edge_multiple_timestamps() {
    let mut graph = TemporalGraph::new();

    // Edge 0-1 with timestamps [0, 20]
    graph.add_edge(0, 1, 0);
    graph.add_edge(0, 1, 20);

    // Incident edge with multiple timestamps, some in range
    graph.add_edge(1, 2, 5);
    graph.add_edge(1, 2, 15);
    graph.add_edge(1, 2, 25); // Out of range

    let result = graph.find_min_incident_in_range(0, 1);
    assert!(result.is_some());

    let (u, v, t) = result.unwrap();
    assert_eq!(t, 5); // Should return minimum timestamp in range
    assert!((u == 1 && v == 2) || (u == 2 && v == 1));
}

#[test]
fn test_find_min_incident_both_vertices() {
    let mut graph = TemporalGraph::new();

    // Edge 0-1 with timestamps [0, 20]
    graph.add_edge(0, 1, 0);
    graph.add_edge(0, 1, 20);

    // Incident to vertex 0
    graph.add_edge(0, 2, 10);

    // Incident to vertex 1
    graph.add_edge(1, 3, 5); // This is minimum

    let result = graph.find_min_incident_in_range(0, 1);
    assert!(result.is_some());

    let (_, _, t) = result.unwrap();
    assert_eq!(t, 5);
}

#[test]
fn test_get_edge_time_range() {
    let mut graph = TemporalGraph::new();

    graph.add_edge(0, 1, 5);
    graph.add_edge(0, 1, 10);
    graph.add_edge(0, 1, 15);

    let range = graph.get_edge_time_range(0, 1);
    assert!(range.is_some());

    let (tmin, tmax) = range.unwrap();
    assert_eq!(tmin, 5);
    assert_eq!(tmax, 15);
}

#[test]
fn test_get_edge_time_range_nonexistent() {
    let graph = TemporalGraph::new();

    let range = graph.get_edge_time_range(0, 1);
    assert!(range.is_none());
}

#[test]
fn test_get_edge_time_range_undirected() {
    let mut graph = TemporalGraph::new();

    graph.add_edge(0, 1, 5);
    graph.add_edge(0, 1, 15);

    // Both directions should work
    let range1 = graph.get_edge_time_range(0, 1);
    let range2 = graph.get_edge_time_range(1, 0);

    assert_eq!(range1, range2);
    assert_eq!(range1.unwrap(), (5, 15));
}
#[test]
fn test_find_min_incident_in_range_returns_correct_order() {
    let mut graph = TemporalGraph::new();
    
    // Edge 0-1 with timestamps [0, 10]
    graph.add_edge(0, 1, 0);
    graph.add_edge(0, 1, 10);
    
    // Incident edge: 1-2 with timestamp 5
    // Common vertex is 1, neighbor is 2
    graph.add_edge(1, 2, 5);
    
    let result = graph.find_min_incident_in_range(0, 1);
    assert!(result.is_some());
    
    let (neighbor, common, t) = result.unwrap();
    assert_eq!(neighbor, 2, "First element should be neighbor");
    assert_eq!(common, 1, "Second element should be common vertex");
    assert_eq!(t, 5);
}

#[test]
fn test_find_min_incident_both_endpoints() {
    let mut graph = TemporalGraph::new();
    
    // Edge 0-1 with timestamps [0, 20]
    graph.add_edge(0, 1, 0);
    graph.add_edge(0, 1, 20);
    
    // Incident to vertex 0: edge 0-2 at time 10
    graph.add_edge(0, 2, 10);
    
    // Incident to vertex 1: edge 1-3 at time 5
    graph.add_edge(1, 3, 5);
    
    let result = graph.find_min_incident_in_range(0, 1);
    assert!(result.is_some());
    
    let (neighbor, common, t) = result.unwrap();
    // Should return the one with minimum time (edge 1-3 at time 5)
    assert_eq!(t, 5);
    assert_eq!(neighbor, 3); // 3 is the neighbor
    assert_eq!(common, 1);   // 1 is the common vertex with edge 0-1
}

#[test]
fn test_find_min_incident_order_regardless_of_edge_direction() {
    let mut graph = TemporalGraph::new();
    
    // Edge 0-1 with timestamps [0, 20]
    graph.add_edge(0, 1, 0);
    graph.add_edge(0, 1, 20);
    
    // Add incident edge in "reverse" order (neighbor first)
    graph.add_edge(2, 1, 10);
    
    let result = graph.find_min_incident_in_range(0, 1);
    assert!(result.is_some());
    
    let (neighbor, common, t) = result.unwrap();
    assert_eq!(neighbor, 2); // 2 is the neighbor (not in {0,1})
    assert_eq!(common, 1);   // 1 is the common vertex (in {0,1})
    assert_eq!(t, 10);
}
