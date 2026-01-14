use temporal_graph::generate_temporal_graphs_from_multigraphs;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 3 {
        eprintln!("Usage: {} <input_multigraphs.txt> <output_temporal.txt>", args[0]);
        eprintln!("\nConverts multigraphs to temporal graphs by assigning all permutations of timestamps");
        std::process::exit(1);
    }
    
    let input_file = &args[1];
    let output_file = &args[2];
    
    match generate_temporal_graphs_from_multigraphs(input_file, output_file) {
        Ok(count) => {
            println!("\n✓ Success! Generated {} temporal graphs", count);
        }
        Err(e) => {
            eprintln!("\n✗ Error: {}", e);
            std::process::exit(1);
        }
    }
}
