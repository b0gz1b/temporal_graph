use temporal_graph::TemporalGraph;

fn main() -> std::io::Result<()> {
    let mut graph = TemporalGraph::new();
    
    // Create temporal graph
    graph.add_edge(0, 1, 0);
    graph.add_edge(0, 1, 1);
    graph.add_edge(0, 1, 2);
    graph.add_edge(1, 2, 1);
    graph.add_edge(1, 2, 2);
    graph.add_edge(2, 3, 2);
    graph.add_edge(2, 3, 3);
    graph.add_edge(0, 3, 3);
    
    // Generate single visualization with edge labels showing timestamps
    graph.save_with_labels("temporal_graph")?;
    
    // Optionally, still generate snapshots
    graph.save_snapshot(2, "snapshot_t2")?;
    
    Ok(())
}
