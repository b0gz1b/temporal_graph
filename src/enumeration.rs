use crate::TemporalGraph;
use itertools::Itertools;
use rayon::prelude::*;
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::process::Command;

fn parse_temporal_graph_line(line: &str) -> Result<TemporalGraph, String> {
    let parts: Vec<&str> = line.split_whitespace().collect();

    if parts.len() < 2 {
        return Err("Line too short".to_string());
    }

    let num_vertices: usize = parts[0].parse().map_err(|_| "Invalid number of vertices")?;
    let _num_edges: usize = parts[1].parse().map_err(|_| "Invalid number of edges")?;

    let mut graph = TemporalGraph::new();

    // Add all vertices
    for v in 0..num_vertices {
        graph.add_vertex(v);
    }

    // Parse edges
    let mut idx = 2;
    while idx < parts.len() {
        if idx + 2 >= parts.len() {
            return Err(format!("Incomplete edge data at position {}", idx));
        }

        let u: usize = parts[idx]
            .parse()
            .map_err(|_| format!("Invalid vertex u at position {}", idx))?;
        let v: usize = parts[idx + 1]
            .parse()
            .map_err(|_| format!("Invalid vertex v at position {}", idx + 1))?;
        let k: usize = parts[idx + 2]
            .parse()
            .map_err(|_| format!("Invalid timestamp count at position {}", idx + 2))?;

        if idx + 2 + k >= parts.len() {
            return Err(format!("Not enough timestamps for edge ({}, {})", u, v));
        }

        // Read k timestamps
        for i in 0..k {
            let timestamp: i64 = parts[idx + 3 + i]
                .parse()
                .map_err(|_| format!("Invalid timestamp at position {}", idx + 3 + i))?;
            graph.add_edge(u, v, timestamp);
        }

        idx += 3 + k;
    }

    Ok(graph)
}

pub fn read_temporal_graphs_from_file(filename: &str) -> Result<Vec<TemporalGraph>, String> {
    println!("Reading temporal graphs from: {}", filename);

    let file = File::open(filename).map_err(|e| format!("Failed to open file: {}", e))?;
    let reader = BufReader::new(file);

    let mut graphs = Vec::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line = line.map_err(|e| format!("Failed to read line {}: {}", line_num + 1, e))?;

        if line.trim().is_empty() {
            continue;
        }

        let graph = parse_temporal_graph_line(&line)
            .map_err(|e| format!("Line {}: {}", line_num + 1, e))?;

        graphs.push(graph);
    }

    println!("Successfully read {} temporal graphs", graphs.len());

    Ok(graphs)
}

#[derive(Debug, Clone)]
struct MultigraphLine {
    num_vertices: usize,
    edges: Vec<(usize, usize, usize)>, // (u, v, multiplicity)
}

impl MultigraphLine {
    fn parse(line: &str) -> Result<Self, String> {
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() < 2 {
            return Err("Line too short".to_string());
        }

        let num_vertices: usize = parts[0].parse().map_err(|_| "Invalid number of vertices")?;

        let mut edges = Vec::new();
        let mut idx = 2;

        while idx + 2 < parts.len() {
            let u: usize = parts[idx]
                .parse()
                .map_err(|_| format!("Invalid vertex u at position {}", idx))?;
            let v: usize = parts[idx + 1]
                .parse()
                .map_err(|_| format!("Invalid vertex v at position {}", idx + 1))?;
            let mult: usize = parts[idx + 2]
                .parse()
                .map_err(|_| format!("Invalid multiplicity at position {}", idx + 2))?;

            edges.push((u, v, mult));
            idx += 3;
        }

        Ok(MultigraphLine {
            num_vertices,
            edges,
        })
    }

    /// Convert to temporal graph with given timestamp permutation
    fn to_temporal_graph(&self, timestamps: &[i64]) -> String {
        let total_edges: usize = self.edges.iter().map(|(_, _, mult)| mult).sum();
        let mut result = format!("{} {}", self.num_vertices, total_edges);
        let mut timestamp_idx = 0;

        for &(u, v, mult) in &self.edges {
            // Get timestamps for this edge
            let edge_timestamps: Vec<i64> =
                timestamps[timestamp_idx..timestamp_idx + mult].to_vec();
            timestamp_idx += mult;

            // Format: u v k t1 t2 ... tk
            result.push_str(&format!(" {} {} {}", u, v, mult));
            for &t in &edge_timestamps {
                result.push_str(&format!(" {}", t));
            }
        }

        result
    }
}
pub fn generate_temporal_graphs_from_multigraphs(
    input_file: &str,
    output_file: &str,
) -> Result<usize, String> {
    println!("Generating temporal graphs from multigraphs (parallel):");
    println!("  Input: {}", input_file);
    println!("  Output: {}", output_file);

    // Read all lines from input file
    let file = File::open(input_file).map_err(|e| format!("Failed to open input file: {}", e))?;
    let reader = BufReader::new(file);

    let lines: Vec<String> = reader
        .lines()
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to read input file: {}", e))?;

    println!("Processing {} multigraphs in parallel...", lines.len());
    // Calculate total edges from first non-empty line (same for all graphs)
    let total_edges: usize = lines
        .iter()
        .find(|line| !line.trim().is_empty())
        .and_then(|line| MultigraphLine::parse(line).ok())
        .map(|mg| mg.edges.iter().map(|(_, _, mult)| mult).sum())
        .ok_or("No valid multigraphs in input file")?;

    if total_edges == 0 {
        return Err("Multigraphs have no edges".to_string());
    }

    println!("Total edges per graph: {}", total_edges);

    // Generate all permutations of [1, 2, ..., total_edges] once
    let timestamps: Vec<i64> = (1..=total_edges as i64).collect();
    // Process each line in parallel
    let results: Vec<Vec<String>> = lines
        .par_iter()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(line_num, line)| {
            // Parse multigraph
            let multigraph =
                MultigraphLine::parse(line).map_err(|e| format!("Line {}: {}", line_num + 1, e))?;

            let mut temporal_graphs = Vec::new();

            for perm in timestamps.iter().permutations(total_edges) {
                // Convert iterator of references to owned vector
                let perm_owned: Vec<i64> = perm.into_iter().copied().collect();

                // Generate temporal graph with this permutation
                let temporal_line = multigraph.to_temporal_graph(&perm_owned);
                temporal_graphs.push(temporal_line);
            }

            println!(
                "  Multigraph {} -> {} temporal graphs",
                line_num + 1,
                temporal_graphs.len()
            );

            Ok(temporal_graphs)
        })
        .collect::<Result<Vec<_>, String>>()?;

    // Write all results to output file
    let mut output =
        File::create(output_file).map_err(|e| format!("Failed to create output file: {}", e))?;

    let mut total_generated = 0;
    for temporal_graphs in results {
        for line in temporal_graphs {
            writeln!(output, "{}", line).map_err(|e| format!("Failed to write output: {}", e))?;
            total_generated += 1;
        }
    }

    println!("\nTotal temporal graphs generated: {}", total_generated);

    Ok(total_generated)
}

// Convenience methods on TemporalGraph
impl TemporalGraph {
    /// Check if the graph is connected (at any time step)
    pub fn is_connected(&self) -> bool {
        if self.vertex_count() == 0 {
            return true;
        }

        if self.edge_count() == 0 {
            return self.vertex_count() <= 1;
        }

        // BFS to check connectivity
        let vertices = self.vertices();
        if vertices.is_empty() {
            return true;
        }

        let start = vertices[0];
        let mut visited = HashSet::new();
        let mut queue = vec![start];
        visited.insert(start);

        while let Some(current) = queue.pop() {
            // Get all neighbors at any time
            let neighbors = self.get_all_neighbors(current);
            for neighbor in neighbors {
                if visited.insert(neighbor) {
                    queue.push(neighbor);
                }
            }
        }

        visited.len() == vertices.len()
    }
}
pub fn generate_multigraphs_nauty(
    n: usize,
    m: usize,
    big_m: usize,
    filename: &str,
) -> Result<usize, String> {
    // Validate parameters
    if n == 0 {
        return Err("Number of vertices must be positive".to_string());
    }

    if big_m < m {
        return Err(format!(
            "Total edges M={} must be >= base edges m={}",
            big_m, m
        ));
    }

    let max_edges = n * (n - 1) / 2; // Maximum edges in simple graph
    if m > max_edges {
        return Err(format!(
            "Base edges m={} exceeds maximum {} for {} vertices",
            m, max_edges, n
        ));
    }

    println!("Generating multigraphs with nauty:");
    println!("  Vertices: {}", n);
    println!("  Base edges: {}", m);
    println!("  Total edges: {}", big_m);
    println!("  Output file: {}", filename);

    // Step 1: Run geng to generate base graphs
    let geng_output = Command::new("geng")
        .arg("-c") // connected graphs only
        .arg(n.to_string())
        .arg(m.to_string())
        .arg("-q") // quiet mode (suppress auxiliary output)
        .output()
        .map_err(|e| format!("Failed to execute geng: {}. Is nauty installed?", e))?;

    if !geng_output.status.success() {
        return Err(format!(
            "geng failed with status: {}. stderr: {}",
            geng_output.status,
            String::from_utf8_lossy(&geng_output.stderr)
        ));
    }

    // Step 2: Pipe geng output to multig
    let multig_output = Command::new("multig")
        .arg("-T") // Use simple format for output
        .arg(format!("-e{}", big_m)) // Exact number of edges
        .arg("-q") // Quiet mode
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn multig: {}. Is nauty installed?", e))?;

    // Write geng output to multig stdin
    {
        let mut stdin = multig_output
            .stdin
            .as_ref()
            .ok_or("Failed to open multig stdin")?;
        stdin
            .write_all(&geng_output.stdout)
            .map_err(|e| format!("Failed to write to multig stdin: {}", e))?;
    }

    // Wait for multig to complete and collect output
    let multig_result = multig_output
        .wait_with_output()
        .map_err(|e| format!("Failed to wait for multig: {}", e))?;

    if !multig_result.status.success() {
        return Err(format!(
            "multig failed with status: {}. stderr: {}",
            multig_result.status,
            String::from_utf8_lossy(&multig_result.stderr)
        ));
    }

    // Step 3: Write output to file
    let mut file =
        File::create(filename).map_err(|e| format!("Failed to create output file: {}", e))?;

    file.write_all(&multig_result.stdout)
        .map_err(|e| format!("Failed to write to output file: {}", e))?;

    // Count lines (number of graphs generated)
    let reader = BufReader::new(multig_result.stdout.as_slice());
    let count = reader.lines().count();

    println!("Successfully generated {} multigraphs", count);
    println!("Output saved to: {}", filename);

    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, path::Path};

    #[test]
    fn test_is_connected_empty() {
        let graph = TemporalGraph::new();
        assert!(graph.is_connected());
    }

    #[test]
    fn test_is_connected_single_vertex() {
        let mut graph = TemporalGraph::new();
        graph.add_vertex(0);
        assert!(graph.is_connected());
    }

    #[test]
    fn test_is_connected_path() {
        let mut graph = TemporalGraph::new();
        graph.add_edge(0, 1, 0);
        graph.add_edge(1, 2, 1);
        graph.add_edge(2, 3, 2);

        assert!(graph.is_connected());
    }

    #[test]
    fn test_is_not_connected() {
        let mut graph = TemporalGraph::new();
        graph.add_edge(0, 1, 0);
        graph.add_edge(2, 3, 1);

        assert!(!graph.is_connected());
    }

    #[test]
    fn test_is_connected_disconnected_vertices() {
        let mut graph = TemporalGraph::new();
        graph.add_vertex(0);
        graph.add_vertex(1);
        graph.add_vertex(2);

        assert!(!graph.is_connected());
    }
    #[test]
    #[ignore]
    fn test_generate_multigraphs_small() {
        let result = generate_multigraphs_nauty(4, 3, 4, "test_multigraphs.txt");

        match result {
            Ok(count) => {
                assert!(count > 0, "Should generate at least one multigraph");
                assert!(Path::new("test_multigraphs.txt").exists());

                // Cleanup
                let _ = fs::remove_file("test_multigraphs.txt");
            }
            Err(e) if e.contains("Is nauty installed?") => {
                eprintln!("Skipping test: nauty not installed");
            }
            Err(e) => panic!("Unexpected error: {}", e),
        }
    }

    #[test]
    fn test_validate_parameters() {
        // n must be positive
        let result = generate_multigraphs_nauty(0, 3, 4, "test.txt");
        assert!(result.is_err());

        // M must be >= m
        let result = generate_multigraphs_nauty(4, 5, 3, "test.txt");
        assert!(result.is_err());

        // m must be <= max possible edges
        let result = generate_multigraphs_nauty(4, 10, 10, "test.txt");
        assert!(result.is_err());
    }
    #[test]
    fn test_parse_multigraph_line() {
        let line = "7 6  0 3 2 0 5 1 1 4 1 1 6 1 2 5 1 2 6 1";
        let mg = MultigraphLine::parse(line).unwrap();

        assert_eq!(mg.num_vertices, 7);
        assert_eq!(mg.edges.len(), 6);
        assert_eq!(mg.edges[0], (0, 3, 2));
        assert_eq!(mg.edges[1], (0, 5, 1));
    }

    #[test]
    fn test_to_temporal_graph() {
        let mg = MultigraphLine {
            num_vertices: 4,
            edges: vec![(0, 1, 2), (1, 2, 1)],
        };

        let timestamps = vec![1, 2, 3];
        let result = mg.to_temporal_graph(&timestamps);

        // Format: 4 3 0 1 2 1 2 1 2 1 3
        assert!(result.contains("4 3"));
        assert!(result.contains("0 1 2 1 2"));
        assert!(result.contains("1 2 1 3"));
    }

    #[test]
    fn test_generate_temporal_graphs() {
        // Create test input file
        let input = "test_multigraph_input.txt";
        let output = "test_temporal_output.txt";

        {
            let mut file = File::create(input).unwrap();
            writeln!(file, "3 2  0 1 1 1 2 1").unwrap();
        }

        let result = generate_temporal_graphs_from_multigraphs(input, output);

        match result {
            Ok(count) => {
                assert_eq!(count, 2); // 2! = 2 permutations

                // Check output
                let content = fs::read_to_string(output).unwrap();
                let lines: Vec<&str> = content.lines().collect();
                assert_eq!(lines.len(), 2);

                // Should have permutations [1,2] and [2,1]
                assert!(content.contains("0 1 1 1") || content.contains("0 1 1 2"));

                // Cleanup
                let _ = fs::remove_file(input);
                let _ = fs::remove_file(output);
            }
            Err(e) => panic!("Unexpected error: {}", e),
        }
    }
    #[test]
    fn test_parse_temporal_graph_line_simple() {
        let line = "3 2  0 1 1 5  1 2 1 10";
        let graph = parse_temporal_graph_line(line).unwrap();

        assert_eq!(graph.vertex_count(), 3);
        assert_eq!(graph.edge_count(), 2);
        assert!(graph.has_edge_at_time(0, 1, 5));
        assert!(graph.has_edge_at_time(1, 2, 10));
    }

    #[test]
    fn test_parse_temporal_graph_line_multiple_timestamps() {
        let line = "4 3  0 1 2 1 2  1 2 1 3";
        let graph = parse_temporal_graph_line(line).unwrap();

        assert_eq!(graph.vertex_count(), 4);

        // Edge 0-1 has timestamps 1 and 2
        assert!(graph.has_edge_at_time(0, 1, 1));
        assert!(graph.has_edge_at_time(0, 1, 2));

        // Edge 1-2 has timestamp 3
        assert!(graph.has_edge_at_time(1, 2, 3));

        let edge_01_times = graph.edge_times(0, 1).unwrap();
        assert_eq!(edge_01_times.len(), 2);
    }

    #[test]
    fn test_parse_temporal_graph_line_example() {
        // From your example: 7 6  0 3 2 t1 t2  0 5 1 t3  1 4 1 t4  1 6 1 t5  2 5 1 t6  2 6 1 t7
        let line = "7 6  0 3 2 1 2  0 5 1 3  1 4 1 4  1 6 1 5  2 5 1 6  2 6 1 7";
        let graph = parse_temporal_graph_line(line).unwrap();

        assert_eq!(graph.vertex_count(), 7);
        assert_eq!(graph.edge_count(), 6);

        // Edge 0-3 has 2 timestamps
        let edge_03_times = graph.edge_times(0, 3).unwrap();
        assert_eq!(edge_03_times.len(), 2);
        assert!(edge_03_times.contains(&1));
        assert!(edge_03_times.contains(&2));

        // Other edges have 1 timestamp each
        assert!(graph.has_edge_at_time(0, 5, 3));
        assert!(graph.has_edge_at_time(1, 4, 4));
    }

    #[test]
    fn test_parse_invalid_line() {
        let line = "3 2  0 1"; // Incomplete
        let result = parse_temporal_graph_line(line);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_temporal_graphs_from_file() {
        let filename = "test_temporal_read.txt";

        // Create test file
        {
            let mut file = File::create(filename).unwrap();
            writeln!(file, "3 2  0 1 1 5  1 2 1 10").unwrap();
            writeln!(file, "4 3  0 1 2 1 2  1 2 1 3").unwrap();
            writeln!(file).unwrap(); // Empty line
            writeln!(file, "3 1  0 2 1 7").unwrap();
        }

        let graphs = read_temporal_graphs_from_file(filename).unwrap();

        assert_eq!(graphs.len(), 3);
        assert_eq!(graphs[0].vertex_count(), 3);
        assert_eq!(graphs[1].vertex_count(), 4);
        assert_eq!(graphs[2].vertex_count(), 3);

        // Cleanup
        let _ = fs::remove_file(filename);
    }
}
