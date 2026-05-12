//! Demo example for the `graph_editor` crate.
//!
//! Launches the GUI pre-loaded with a small demo graph:
//! five vertices arranged roughly as a pentagon, connected
//! as a cycle (0-1-2-3-4-0) plus one chord (1-3).
//!
//! Run with:
//!   cargo run --example graph_editor_demo -p graph_editor
//!
//! Interaction:
//!   - Left click on empty space  → create a new vertex
//!   - Left click on a vertex     → select it (turns orange)
//!   - Left click on another      → draw an edge, deselect
//!   - Left click the same vertex → deselect

use graph_editor::app::GraphEditorApp;
use std::f32::consts::TAU;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Graph Editor — Demo")
            .with_inner_size([1024.0, 768.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Graph Editor — Demo",
        options,
        Box::new(|_cc| {
            // Build a pentagon demo graph.
            // Centre of the canvas (approximate for a 1024×768 window).
            let cx = 512.0_f32;
            let cy = 384.0_f32;
            let r = 200.0_f32;
            let n = 5usize;

            let mut app = GraphEditorApp::default();

            // Add 5 vertices equally spaced on a circle.
            // We start at the top (−π/2) so vertex 0 is at the top.
            for i in 0..n {
                let angle = TAU * (i as f32) / (n as f32) - std::f32::consts::FRAC_PI_2;
                let pos = egui::pos2(cx + r * angle.cos(), cy + r * angle.sin());
                app.add_vertex(pos);
            }

            // Cycle edges: 0-1, 1-2, 2-3, 3-4, 4-0
            for i in 0..n {
                app.add_edge(i, (i + 1) % n);
            }

            // One chord: 1-3
            app.add_edge(1, 3);

            Ok(Box::new(app))
        }),
    )
}
