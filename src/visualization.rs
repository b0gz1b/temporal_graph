use crate::{TemporalGraph, TimeStep};
use graphviz_rust::dot_generator::*;
use graphviz_rust::dot_structures::*;
use graphviz_rust::printer::{DotPrinter, PrinterContext};
use graphviz_rust::cmd::{CommandArg, Format};
use graphviz_rust::exec;
use std::fs::File;
use std::io::Error;
use std::io::Write;

impl TemporalGraph {
    /// Generate DOT format showing all edges with timestamp labels
    pub fn to_dot_with_time_labels(&self) -> Graph {
        let mut stmts = Vec::new();
        
        // Add default node style
        stmts.push(stmt!(node!("node"; attr!("shape", "circle"), attr!("style", "filled"), attr!("fillcolor", "lightblue"))));
        
        // Add all vertices
        for vertex in &self.vertices {
            stmts.push(stmt!(node!(vertex.to_string())));
        }
        
        // Add edges with timestamp labels
        for ((u, v), edge) in &self.edges {
            // Sort timestamps for consistent display
            let mut times: Vec<TimeStep> = edge.timestamps.iter().copied().collect();
            times.sort_unstable();
            
            // Format label: show as comma-separated list
            let label = if times.len() <= 5 {
                // Show all times if not too many
                times.iter()
                    .map(|t| t.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            } else {
                // Show range if many timestamps
                format!("{}..{} ({} times)", times[0], times[times.len()-1], times.len())
            };
            
            stmts.push(stmt!(edge!(
                node_id!(u.to_string()) => node_id!(v.to_string());
                attr!("label", esc label)
            )));
        }
        
        // Build graph structure
        Graph::Graph {
            id: id!("temporal_graph"),
            strict: true,
            stmts,
        }
    }
    
    /// Save full temporal graph with edge labels
    pub fn save_with_labels(&self, filename: &str) -> std::io::Result<()> {
        let dot_graph = self.to_dot_with_time_labels();
        let dot_string = dot_graph.print(&mut PrinterContext::default());
        
        // Write DOT file
        let dot_filename = format!("{}.dot", filename);
        let mut file = File::create(&dot_filename)?;
        file.write_all(dot_string.as_bytes())?;
        
        // Convert to PNG using graphviz command
        exec(
            dot_graph,
            &mut PrinterContext::default(),
            vec![
                CommandArg::Format(Format::Png),
                CommandArg::Output(format!("{}.png", filename)),
            ],
        ).map_err(Error::other)?;
        
        println!("Saved temporal graph visualization to {}.png", filename);
        Ok(())
    }
    
    /// Generate DOT format for the graph at a specific time (for snapshots)
    pub fn to_dot_at_time(&self, time: TimeStep) -> Graph {
        let mut stmts = Vec::new();
        
        stmts.push(stmt!(node!("node"; attr!("shape", "circle"))));
        
        for vertex in &self.vertices {
            stmts.push(stmt!(node!(vertex.to_string())));
        }
        
        let active_edges = self.edges_at_time(time);
        for (u, v) in active_edges {
            stmts.push(stmt!(edge!(
                node_id!(u.to_string()) => node_id!(v.to_string())
            )));
        }
        
        Graph::Graph {
            id: id!("temporal_graph"),
            strict: true,
            stmts,
        }
    }
    
    pub fn save_snapshot(&self, time: TimeStep, filename: &str) -> std::io::Result<()> {
        let dot_graph = self.to_dot_at_time(time);
        let dot_string = dot_graph.print(&mut PrinterContext::default());
        
        let dot_filename = format!("{}.dot", filename);
        let mut file = File::create(&dot_filename)?;
        file.write_all(dot_string.as_bytes())?;
        
        exec(
            dot_graph,
            &mut PrinterContext::default(),
            vec![
                CommandArg::Format(Format::Png),
                CommandArg::Output(format!("{}.png", filename)),
            ],
        ).map_err(Error::other)?;
        
        println!("Saved visualization to {}.png", filename);
        Ok(())
    }
    
    pub fn save_timeline_panels(&self, output_prefix: &str) -> std::io::Result<()> {
        let mut all_times: Vec<TimeStep> = self.edges
            .values()
            .flat_map(|edge| edge.timestamps.iter())
            .copied()
            .collect();
        all_times.sort_unstable();
        all_times.dedup();
        
        for &time in &all_times {
            let filename = format!("{}_{}", output_prefix, time);
            self.save_snapshot(time, &filename)?;
        }
        
        println!("Generated {} timeline panels", all_times.len());
        Ok(())
    }
}
