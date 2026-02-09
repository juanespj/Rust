use eframe::egui;

use ftail::Ftail;
use log::LevelFilter;
use rust_embed::Embed;
// use std::backtrace::Backtrace;
use std::panic;
use sys_tools::{Config, readsys};
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

fn get_app_info() -> String {
    format!("JE Tools v{APP_VERSION}")
}

#[derive(Embed)]
#[folder = "$CARGO_MANIFEST_DIR/../assets"]
pub struct Asset;

fn main() {
    let mut syspr = Config::default();
    readsys(&mut syspr, "structs.json");
    // Log to stdout (if you run with `RUST_LOG=debug`).
    // tracing_subscriber::fmt::init();

    // let icon = image::open("assets/logo.png")
    //     .expect("Failed to open icon path")
    //     .to_rgba8();
    // let (icon_width, icon_height) = icon.dimensions();

    let logoicon = Asset::get("SymphonyLogoIcon.png").unwrap().data;

    let options = eframe::NativeOptions {
        multisampling: 0,
        renderer: eframe::Renderer::Glow,
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 800.0])
            .with_icon(
                eframe::icon_data::from_png_bytes(&logoicon[..]).expect("Failed to load icon"),
            ),

        depth_buffer: 0,

        ..Default::default()
    };
    if !std::path::Path::new("logs").exists() {
        std::fs::create_dir("logs").expect("path failed");
        println!("Logs folder created");
    }
    if let Err(res) = Ftail::new()
        .console(LevelFilter::Debug)
        .daily_file("logs", LevelFilter::Info)
        .init()
    {
        println!("Failed to initialize logger: {}", res);
    }
    panic::set_hook(Box::new(|panic_info| {
        // Log the panic information using the 'log' facade
        // You can access the payload and location of the panic.
        let msg = match panic_info.payload().downcast_ref::<&str>() {
            Some(s) => *s,
            None => match panic_info.payload().downcast_ref::<String>() {
                Some(s) => &s,
                None => "Box<Any>",
            },
        };

        let location = panic_info
            .location()
            .map(|loc| format!("{}:{}:{}", loc.file(), loc.line(), loc.column()))
            .unwrap_or_else(|| "unknown location".to_string());

        log::error!("PANIC DETECTED: at {} - {}", location, msg);
    }));

    let _s = eframe::run_native(
        &get_app_info(),
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);

            Ok(Box::new(RenderApp::new(cc, syspr)))
        }),
    );
}
