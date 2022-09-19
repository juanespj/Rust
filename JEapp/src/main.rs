
mod appmod { pub mod btcomm; }
use std::error::Error;
#[tokio::main]
async fn main()-> Result<(), Box<dyn Error>>  {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    //tracing_subscriber::fmt::init();
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1024.0, 1024.0)),
        multisampling: 8,
        renderer: eframe::Renderer::Glow,
        depth_buffer: 24,
        ..Default::default()
    };
   // appmod::btcomm::initble().await.expect("error");
    eframe::run_native(
        "Gcode-Leveling!",
        options,
        Box::new(|cc| Box::new(jeapp::RenderApp::new(cc))),
    );
    Ok(())
}
