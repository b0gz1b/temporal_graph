use crate::{GraphState, TemporalGraph, TimeStep, VertexId};
use std::collections::HashSet;

/// Configuration for the label minimization algorithm
#[derive(Debug, Clone)]
pub struct MinimizationConfig {
    /// Maximum number of iterations before forced termination
    pub max_iterations: Option<usize>,

    /// Whether to track detailed statistics during execution
    pub track_statistics: bool,

    /// Whether to print debug information
    pub verbose: bool,
}

impl Default for MinimizationConfig {
    fn default() -> Self {
        Self {
            max_iterations: Some(10_000),
            track_statistics: false,
            verbose: false,
        }
    }
}

impl MinimizationConfig {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Builder method: set maximum iterations
    pub fn with_max_iterations(mut self, max: usize) -> Self {
        self.max_iterations = Some(max);
        self
    }

    /// Builder method: enable unlimited iterations
    pub fn unlimited_iterations(mut self) -> Self {
        self.max_iterations = None;
        self
    }

    /// Builder method: enable statistics tracking
    pub fn with_statistics(mut self) -> Self {
        self.track_statistics = true;
        self
    }

    /// Builder method: enable verbose output
    pub fn verbose(mut self) -> Self {
        self.verbose = true;
        self
    }
}

/// Statistics collected during algorithm execution
#[derive(Debug, Default, Clone)]
pub struct MinimizationStats {
    /// Number of iterations performed
    pub iterations: usize,

    /// Number of label transfers attempted
    pub transfers_attempted: usize,

    /// Number of successful label transfers
    pub transfers_successful: usize,

    /// Number of useless labels detected
    pub useless_labels_found: usize,

    /// Number of unique states visited
    pub states_visited: usize,
}

impl MinimizationStats {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Result of the minimization algorithm
#[derive(Debug, Clone)]
pub struct MinimizationResult {
    /// Whether the graph is minimal with respect to the algorithm
    pub is_minimal: bool,

    /// Statistics about the execution (if tracking was enabled)
    pub stats: Option<MinimizationStats>,

    /// Reason for termination
    pub termination_reason: TerminationReason,
}

/// Reason why the algorithm terminated
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TerminationReason {
    /// A cycle was detected - graph is minimal
    CycleDetected,

    /// A useless label was found - graph is not minimal
    UselessLabelFound,

    /// Maximum iterations reached
    MaxIterationsReached,
}

/// Main algorithm executor
pub struct LabelMinimizer<'a> {
    graph: &'a mut TemporalGraph,
    config: MinimizationConfig,
    stats: MinimizationStats,
    seen_states: HashSet<GraphState>,
}

impl<'a> LabelMinimizer<'a> {
    /// Create a new minimizer with default configuration
    pub fn new(graph: &'a mut TemporalGraph) -> Self {
        Self {
            graph,
            config: MinimizationConfig::default(),
            stats: MinimizationStats::new(),
            seen_states: HashSet::new(),
        }
    }

    /// Create a new minimizer with custom configuration
    pub fn with_config(graph: &'a mut TemporalGraph, config: MinimizationConfig) -> Self {
        Self {
            graph,
            config,
            stats: MinimizationStats::new(),
            seen_states: HashSet::new(),
        }
    }

    /// Run the label minimization algorithm
    pub fn run(&mut self) -> MinimizationResult {
        // Initialize with the starting state
        let initial_state = self.graph.to_state();
        self.seen_states.insert(initial_state);
        self.stats.states_visited = 1;

        if self.config.verbose {
            println!("Starting label minimization algorithm");
            println!();
            self.graph.print_state();
        }

        loop {
            self.stats.iterations += 1;
            if self.config.verbose {
                println!("\n=== Iteration {} ===", self.stats.iterations);
            }
            if self.should_terminate_iterations() {
                if self.config.verbose {
                    println!("Max iterations reached");
                }
                return MinimizationResult {
                    is_minimal: false,
                    stats: if self.config.track_statistics {
                        Some(self.stats.clone())
                    } else {
                        None
                    },
                    termination_reason: TerminationReason::MaxIterationsReached,
                };
            }
            let (u, v) = match self.find_wrappable_edge() {
                Some(edge) => edge,
                None => {
                    if self.config.verbose {
                        println!("No wrappable edge found - checking if useless label detected");
                    }
                    // No wrappable edge means we can't continue
                    // This is a stopping condition, need to determine if minimal
                    break;
                }
            };
            if self.config.verbose {
                println!("Found wrappable edge: ({}, {})", u, v);
            }
            let (w, x, t) = match self.find_min_incident_in_range(u, v) {
                Some(result) => result,
                None => {
                    if self.config.verbose {
                        println!("Warning: wrappable edge has no incident edges in range");
                    }
                    break;
                }
            };

            if self.config.verbose {
                println!(
                    "Found incident: w={} (neighbor), x={} (common vertex), t={}",
                    w, x, t
                );
            }
            // Determine the other endpoint of edge e
            let other_endpoint = if x == u { v } else { u };

            if self.config.verbose {
                println!("Other endpoint of e: {}", other_endpoint);
            }

            // Step 5: Transfer labels of neighbors of x through edge (x, other_endpoint)
            if self.config.verbose {
                println!(
                    "Transferring labels through edge ({}, {})",
                    x, other_endpoint
                );
            }
            let transferred = self.transfer_labels(x, other_endpoint);

            let (tmin, _tmax) = self.graph.get_edge_time_range(u, v).unwrap();
            if self.config.verbose {
                println!("Transferred {} labels", transferred);
            }
            if self.config.verbose {
                println!("Removing tmin={} from edge ({}, {})", tmin, u, v);
            }

            if self.config.verbose {
                println!("Adding tmin={} to edge ({}, {})", tmin, other_endpoint, w);
            }
            self.graph.add_edge(w, other_endpoint, tmin);

            let removed = self.graph.remove_edge_timestamp(u, v, tmin);
            if !removed {
                if self.config.verbose {
                    println!("Warning: failed to remove tmin");
                }
                break;
            }
            if self.config.verbose {
                println!();
                self.graph.print_state();
            }
            // Check if we've seen this state before (cycle detection)
            if self.has_seen_current_state() {
                if self.config.verbose {
                    println!("Cycle detected! Graph is minimal");
                }
                return MinimizationResult {
                    is_minimal: true,
                    stats: if self.config.track_statistics {
                        Some(self.stats.clone())
                    } else {
                        None
                    },
                    termination_reason: TerminationReason::CycleDetected,
                };
            }

            // Record the new state
            self.record_current_state();

            if self.config.verbose {
                println!(
                    "New state recorded (total states: {})",
                    self.stats.states_visited
                );
            }
        }

        // If we exit the loop without finding a cycle or useless label
        // We consider it minimal (no more transformations possible)
        if self.config.verbose {
            println!("Algorithm terminated");
            println!("Graph is not minimal (no cycling)");
            println!();
            self.graph.print_state();
        }
        MinimizationResult {
            is_minimal: false,
            stats: if self.config.track_statistics {
                Some(self.stats.clone())
            } else {
                None
            },
            termination_reason: TerminationReason::UselessLabelFound,
        }
    }

    /// Check if we've seen the current graph state before
    fn has_seen_current_state(&self) -> bool {
        let current_state = self.graph.to_state();
        self.seen_states.contains(&current_state)
    }

    /// Record the current graph state
    fn record_current_state(&mut self) {
        let current_state = self.graph.to_state();
        self.seen_states.insert(current_state);
        self.stats.states_visited += 1;
    }

    /// Check if maximum iterations have been reached
    fn should_terminate_iterations(&self) -> bool {
        if let Some(max) = self.config.max_iterations {
            self.stats.iterations >= max
        } else {
            false
        }
    }

    fn find_wrappable_edge(&self) -> Option<(VertexId, VertexId)> {
        self.graph.find_wrappable_edge()
    }
    fn find_min_incident_in_range(
        &self,
        u: VertexId,
        v: VertexId,
    ) -> Option<(VertexId, VertexId, TimeStep)> {
        self.graph.find_min_incident_in_range(u, v)
    }
    fn transfer_labels(&mut self, u: VertexId, v: VertexId) -> usize {
        let transferred = self.graph.transfer_labels_through_edge(u, v);

        if self.config.track_statistics {
            self.stats.transfers_attempted += 1;
            if transferred > 0 {
                self.stats.transfers_successful += 1;
            }
        }

        transferred
    }
}

// Convenience function for simple usage
impl TemporalGraph {
    /// Check if this temporal graph is label-minimal using default configuration
    pub fn is_label_minimal(&mut self) -> bool {
        let mut minimizer = LabelMinimizer::new(self);
        minimizer.run().is_minimal
    }

    /// Check if this temporal graph is label-minimal with custom configuration
    pub fn is_label_minimal_with_config(
        &mut self,
        config: MinimizationConfig,
    ) -> MinimizationResult {
        let mut minimizer = LabelMinimizer::with_config(self, config);
        minimizer.run()
    }

    /// Find an edge with multiple labels where an incident edge has a label between its min and max
    ///
    /// Returns Some((u, v)) if such an edge exists, None otherwise.
    ///
    /// Specifically, looks for edge {u,v} with |λ(uv)| ≥ 2 where:
    /// - tmin = min(λ(uv))
    /// - tmax = max(λ(uv))
    /// - ∃ incident edge e and t ∈ λ(e) such that tmin < t < tmax
    pub fn find_wrappable_edge(&self) -> Option<(VertexId, VertexId)> {
        // Iterate through all edges
        for ((u, v), edge) in &self.edges {
            // Check if edge has at least 2 labels
            if edge.timestamps.len() < 2 {
                continue;
            }

            // Get min and max timestamps for this edge
            let tmin = *edge.timestamps.iter().min().unwrap();
            let tmax = *edge.timestamps.iter().max().unwrap();

            // If min equals max, skip (shouldn't happen with len >= 2, but be safe)
            if tmin >= tmax {
                continue;
            }

            // Check all incident edges (edges that share vertex u or v)
            if self.has_incident_edge_in_range(*u, *v, tmin, tmax) {
                return Some((*u, *v));
            }
        }

        None
    }

    /// Helper: Check if there exists an incident edge to {u,v} with a timestamp in (tmin, tmax)
    fn has_incident_edge_in_range(
        &self,
        u: VertexId,
        v: VertexId,
        tmin: TimeStep,
        tmax: TimeStep,
    ) -> bool {
        // Check all edges in the graph
        for ((edge_u, edge_v), incident_edge) in &self.edges {
            // Skip the edge {u,v} itself
            if (*edge_u == u && *edge_v == v) || (*edge_u == v && *edge_v == u) {
                continue;
            }

            // Check if this edge is incident to {u,v}
            // An edge is incident if it shares at least one vertex
            let is_incident = *edge_u == u || *edge_u == v || *edge_v == u || *edge_v == v;

            if !is_incident {
                continue;
            }

            // Check if any timestamp of this incident edge is in the range (tmin, tmax)
            for &t in &incident_edge.timestamps {
                if t > tmin && t < tmax {
                    return true;
                }
            }
        }

        false
    }
    pub fn find_min_incident_in_range(
        &self,
        u: VertexId,
        v: VertexId,
    ) -> Option<(VertexId, VertexId, TimeStep)> {
        // Normalize the edge
        let (u_norm, v_norm) = if u <= v { (u, v) } else { (v, u) };

        // Get the edge and verify it has at least 2 timestamps
        let edge = self.edges.get(&(u_norm, v_norm))?;
        if edge.timestamps.len() < 2 {
            return None;
        }

        // Get tmin and tmax
        let tmin = *edge.timestamps.iter().min().unwrap();
        let tmax = *edge.timestamps.iter().max().unwrap();

        if tmin >= tmax {
            return None;
        }

        // Find all incident edges with timestamps in (tmin, tmax)
        let mut candidates: Vec<(VertexId, VertexId, TimeStep)> = Vec::new();

        for ((edge_u, edge_v), incident_edge) in &self.edges {
            // Skip the edge {u,v} itself
            if *edge_u == u_norm && *edge_v == v_norm {
                continue;
            }

            // Determine if this edge is incident and identify common vertex and neighbor
            let incident_info = if *edge_u == u || *edge_u == v {
                // edge_u is the common vertex, edge_v is the neighbor
                Some((*edge_u, *edge_v))
            } else if *edge_v == u || *edge_v == v {
                // edge_v is the common vertex, edge_u is the neighbor
                Some((*edge_v, *edge_u))
            } else {
                // Not incident
                None
            };

            if let Some((common_vertex, neighbor)) = incident_info {
                // Find timestamps in range (tmin, tmax)
                for &t in &incident_edge.timestamps {
                    if t > tmin && t < tmax {
                        candidates.push((neighbor, common_vertex, t));
                    }
                }
            }
        }

        // Return the candidate with minimum timestamp
        candidates.into_iter().min_by_key(|&(_, _, t)| t)
    }
    /// Helper method to get tmin and tmax for an edge
    pub fn get_edge_time_range(&self, u: VertexId, v: VertexId) -> Option<(TimeStep, TimeStep)> {
        let (u_norm, v_norm) = if u <= v { (u, v) } else { (v, u) };
        let edge = self.edges.get(&(u_norm, v_norm))?;

        if edge.timestamps.is_empty() {
            return None;
        }

        let tmin = *edge.timestamps.iter().min().unwrap();
        let tmax = *edge.timestamps.iter().max().unwrap();
        Some((tmin, tmax))
    }

    pub fn transfer_labels_through_edge(&mut self, u: VertexId, v: VertexId) -> usize {
        // Get tmin and tmax for edge {u,v}
        let (tmin, tmax) = match self.get_edge_time_range(u, v) {
            Some(range) => range,
            None => return 0, // Edge doesn't exist
        };

        // Find all neighbors of u (at any time)
        let neighbors_of_v = self.get_all_neighbors(v);

        let mut total_transferred = 0;

        // For each neighbor w of u (except v)
        for w in neighbors_of_v {
            if w == u {
                continue; // Skip v itself
            }
            // Find timestamps of {u,w} in range (tmin, tmax)
            let timestamps_to_transfer = self.get_edge_timestamps_in_range(v, w, tmin, tmax);

            if timestamps_to_transfer.is_empty() {
                continue;
            }

            // Remove these timestamps from {u,w}
            for &t in &timestamps_to_transfer {
                self.remove_edge_timestamp(v, w, t);
            }

            // Add these timestamps to {w,v}
            for &t in &timestamps_to_transfer {
                self.add_edge(w, u, t);
            }

            total_transferred += timestamps_to_transfer.len();
        }

        total_transferred
    }

    /// Get all neighbors of a vertex across all time steps
    pub fn get_all_neighbors(&self, vertex: VertexId) -> Vec<VertexId> {
        let mut neighbors = HashSet::new();

        for (u, v) in self.edges.keys() {
            if *u == vertex {
                neighbors.insert(*v);
            } else if *v == vertex {
                neighbors.insert(*u);
            }
        }

        neighbors.into_iter().collect()
    }

    /// Get timestamps of an edge that fall within range (tmin, tmax) - exclusive bounds
    fn get_edge_timestamps_in_range(
        &self,
        u: VertexId,
        v: VertexId,
        tmin: TimeStep,
        tmax: TimeStep,
    ) -> Vec<TimeStep> {
        let (u_norm, v_norm) = if u <= v { (u, v) } else { (v, u) };

        self.edges
            .get(&(u_norm, v_norm))
            .map(|edge| {
                edge.timestamps
                    .iter()
                    .filter(|&&t| t > tmin && t < tmax)
                    .copied()
                    .collect()
            })
            .unwrap_or_default()
    }
}
