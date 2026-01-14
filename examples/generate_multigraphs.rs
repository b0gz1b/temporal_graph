use temporal_graph::generate_multigraphs_nauty;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 5 {
        eprintln!("Usage: {} <n_vertices> <m_base_edges> <M_total_edges> <output_file>", args[0]);
        eprintln!("\nExample: {} 4 3 5 multigraphs.txt", args[0]);
        std::process::exit(1);
    }
    
    let n: usize = args[1].parse().expect("n must be a positive integer");
    let m: usize = args[2].parse().expect("m must be a positive integer");
    let big_m: usize = args[3].parse().expect("M must be a positive integer");
    let filename = &args[4];
    
    match generate_multigraphs_nauty(n, m, big_m, filename) {
        Ok(count) => {
            println!("\n✓ Success! Generated {} multigraphs", count);
        }
        Err(e) => {
            eprintln!("\n✗ Error: {}", e);
            std::process::exit(1);
        }
    }
}
