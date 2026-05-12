use egui::{Color32, Pos2, Stroke, Vec2};

/// Radius of a vertex circle in logical pixels.
const VERTEX_RADIUS: f32 = 18.0;

/// Hit-test radius (slightly larger than visual radius for easier clicking).
const HIT_RADIUS: f32 = 22.0;

#[derive(Debug, Clone)]
struct Vertex {
    id: usize,
    pos: Pos2,
}

#[derive(Debug, Clone)]
struct Edge {
    from: usize, // vertex id
    to: usize,   // vertex id
}

#[derive(Default)]
pub struct GraphEditorApp {
    vertices: Vec<Vertex>,
    edges: Vec<Edge>,
    /// Index into `vertices` of the currently selected vertex, if any.
    selected: Option<usize>,
    /// Counter used to assign unique vertex IDs.
    next_id: usize,
}

impl GraphEditorApp {
    // ------------------------------------------------------------------
    // Public API (used by examples and future integrations)
    // ------------------------------------------------------------------

    /// Insert a vertex at `pos` and return its assigned ID.
    pub fn add_vertex(&mut self, pos: Pos2) -> usize {
        let id = self.next_id;
        self.vertices.push(Vertex { id, pos });
        self.next_id += 1;
        id
    }

    /// Insert an undirected edge between the vertices with IDs `from_id`
    /// and `to_id`. No-op if either ID does not exist or the edge is
    /// already present.
    pub fn add_edge(&mut self, from_id: usize, to_id: usize) {
        let from_exists = self.vertices.iter().any(|v| v.id == from_id);
        let to_exists = self.vertices.iter().any(|v| v.id == to_id);
        if !from_exists || !to_exists {
            return;
        }
        let already_exists = self.edges.iter().any(|e| {
            (e.from == from_id && e.to == to_id)
                || (e.from == to_id && e.to == from_id)
        });
        if !already_exists {
            self.edges.push(Edge { from: from_id, to: to_id });
        }
    }

    // ------------------------------------------------------------------
    // Private helpers
    // ------------------------------------------------------------------

    /// Return the index in `self.vertices` of the vertex hit by `pos`, if any.
    fn hit_vertex(&self, pos: Pos2) -> Option<usize> {
        self.vertices.iter().position(|v| {
            let dx = v.pos.x - pos.x;
            let dy = v.pos.y - pos.y;
            (dx * dx + dy * dy).sqrt() <= HIT_RADIUS
        })
    }

    /// Handle a left-click at canvas position `pos`.
    fn on_click(&mut self, pos: Pos2) {
        if let Some(hit_idx) = self.hit_vertex(pos) {
            match self.selected {
                None => {
                    // Select the clicked vertex.
                    self.selected = Some(hit_idx);
                }
                Some(sel_idx) if sel_idx == hit_idx => {
                    // Clicking the already-selected vertex deselects it.
                    self.selected = None;
                }
                Some(sel_idx) => {
                    // Create an edge between sel_idx and hit_idx (avoid duplicates).
                    let from_id = self.vertices[sel_idx].id;
                    let to_id = self.vertices[hit_idx].id;
                    self.add_edge(from_id, to_id);
                    self.selected = None;
                }
            }
        } else {
            // Empty space clicked — create a new vertex and deselect.
            self.add_vertex(pos);
            self.selected = None;
        }
    }
}

impl eframe::App for GraphEditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(Color32::from_rgb(245, 245, 240)))
            .show(ctx, |ui| {
                // ---- Input handling ------------------------------------------------
                let response = ui.allocate_rect(
                    ui.max_rect(),
                    egui::Sense::click(),
                );

                if response.clicked() {
                    if let Some(pos) = response.interact_pointer_pos() {
                        self.on_click(pos);
                    }
                }

                // ---- Painting ------------------------------------------------------
                let painter = ui.painter();

                // Draw edges first (beneath vertices).
                for edge in &self.edges {
                    let from_pos = self
                        .vertices
                        .iter()
                        .find(|v| v.id == edge.from)
                        .map(|v| v.pos);
                    let to_pos = self
                        .vertices
                        .iter()
                        .find(|v| v.id == edge.to)
                        .map(|v| v.pos);

                    if let (Some(fp), Some(tp)) = (from_pos, to_pos) {
                        painter.line_segment(
                            [fp, tp],
                            Stroke::new(2.5, Color32::from_rgb(80, 80, 80)),
                        );
                    }
                }

                // Draw vertices on top.
                for (idx, vertex) in self.vertices.iter().enumerate() {
                    let is_selected = self.selected == Some(idx);

                    let fill = if is_selected {
                        Color32::from_rgb(230, 120, 20) // orange highlight
                    } else {
                        Color32::from_rgb(1, 105, 111) // teal (matches Nexus primary)
                    };

                    // Outer ring for selected state.
                    if is_selected {
                        painter.circle_stroke(
                            vertex.pos,
                            VERTEX_RADIUS + 4.0,
                            Stroke::new(2.0, Color32::from_rgb(230, 120, 20)),
                        );
                    }

                    painter.circle_filled(vertex.pos, VERTEX_RADIUS, fill);

                    // Vertex ID label, centred inside the circle.
                    painter.text(
                        vertex.pos,
                        egui::Align2::CENTER_CENTER,
                        vertex.id.to_string(),
                        egui::FontId::proportional(13.0),
                        Color32::WHITE,
                    );
                }

                // ---- Status bar ----------------------------------------------------
                let status = match self.selected {
                    None => format!(
                        "{} vertices · {} edges — click to add a vertex",
                        self.vertices.len(),
                        self.edges.len()
                    ),
                    Some(idx) => format!(
                        "Vertex {} selected — click another vertex to connect",
                        self.vertices[idx].id
                    ),
                };

                // Overlay status text at the bottom of the canvas.
                painter.text(
                    response.rect.left_bottom() + Vec2::new(12.0, -12.0),
                    egui::Align2::LEFT_BOTTOM,
                    status,
                    egui::FontId::proportional(14.0),
                    Color32::from_rgb(80, 80, 80),
                );
            });
    }
}
