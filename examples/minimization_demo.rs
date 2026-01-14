use temporal_graph::{TemporalGraph, MinimizationConfig};

fn main() {
    let mut graph = TemporalGraph::new();
    
    // Build example graph with potential redundancy
    graph.add_edge(0, 1, 7);
    graph.add_edge(1, 2, 1);
    graph.add_edge(1, 2, 4);
    graph.add_edge(2, 3, 2);
    graph.add_edge(2, 3, 5);
    graph.add_edge(0, 3, 3);
    graph.add_edge(0, 3, 6);
    
    println!("Testing label minimization...\n");
    
    let config = MinimizationConfig::new()
        .with_max_iterations(1000)
        .with_statistics()
        .verbose();
    
    let result = graph.is_label_minimal_with_config(config);
    
    println!("FINAL RESULT");
    println!("Is minimal: {}", result.is_minimal);
    println!("Termination reason: {:?}", result.termination_reason);
    
    if let Some(stats) = result.stats {
        println!("\nStatistics:");
        println!("  Iterations: {}", stats.iterations);
        println!("  States visited: {}", stats.states_visited);
        println!("  Transfers attempted: {}", stats.transfers_attempted);
        println!("  Transfers successful: {}", stats.transfers_successful);
        println!("  Useless labels found: {}", stats.useless_labels_found);
    }
}
