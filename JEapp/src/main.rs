
fn main() {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::init();
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1024.0, 610.0)),
        multisampling: 8,
        renderer: eframe::Renderer::Glow,
        depth_buffer: 24,
        ..Default::default()
    };    
    eframe::run_native(
        "Gcode-Leveling!",
        options,
        Box::new(|cc| Box::new(eframe_template::RenderApp::new(cc))),
    );
}

