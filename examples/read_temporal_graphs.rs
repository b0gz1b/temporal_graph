use temporal_graph::read_temporal_graphs_from_file;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 2 {
        eprintln!("Usage: {} <temporal_graphs.txt>", args[0]);
        std::process::exit(1);
    }
    
    let filename = &args[1];
    
    match read_temporal_graphs_from_file(filename) {
        Ok(graphs) => {
            println!("\n✓ Successfully read {} temporal graphs\n", graphs.len());
            
            // Display first few graphs
            for (i, graph) in graphs.iter().take(5).enumerate() {
                println!("Graph {}:", i + 1);
                graph.print_state();
                println!();
            }
            
            if graphs.len() > 5 {
                println!("... and {} more graphs", graphs.len() - 5);
            }
        }
        Err(e) => {
            eprintln!("\n✗ Error: {}", e);
            std::process::exit(1);
        }
    }
}
