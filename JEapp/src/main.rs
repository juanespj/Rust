
// use eframe::IconData;
// use image;
fn main() {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    // tracing_subscriber::fmt::init();

    // let icon = image::open("assets/logo.png")
    //     .expect("Failed to open icon path")
    //     .to_rgba8();
    // let (icon_width, icon_height) = icon.dimensions();

    let options = eframe::NativeOptions {
        // icon_data: Some(IconData {
        //     rgba: icon.into_raw(),
        //     width: icon_width,
        //     height: icon_height,
        // }),
        initial_window_size: Some(egui::vec2(600.0, 800.0)),
        multisampling: 8,
        renderer: eframe::Renderer::Glow,
        depth_buffer: 24,
        ..Default::default()
    };

    let _launch = eframe::run_native(
        "DataCapture",
        options,
        Box::new(|cc| Box::new(jeapp::RenderApp::new(cc))),
    );
}
