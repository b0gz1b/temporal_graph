mod app;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Temporal Graph Editor")
            .with_inner_size([1024.0, 768.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Temporal Graph Editor",
        options,
        Box::new(|_cc| Ok(Box::new(app::GraphEditorApp::default()))),
    )
}
