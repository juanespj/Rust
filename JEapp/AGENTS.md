# JEapp Agent Instructions

## Purpose
This file helps AI coding agents understand the JEapp repository quickly and make productive changes without guessing the architecture.

## Build and run
- Native GUI: `cargo build` and `cargo run --release`
- The project is a Rust GUI app built with `eframe` / `egui`
- If a web frontend is added later, the repository README mentions `trunk serve` for WASM builds

## Key code organization
- `src/main.rs` boots the native app and constructs `RenderApp`
- `src/lib.rs` exports the main app and subsystem modules
- `src/appmod/` contains the core application state, UI model, and persisted app data
- `src/blesys.rs`, `src/cnc.rs`, `src/datalogger.rs`, `src/ltspicesim.rs`, `src/rbbsim.rs`, `src/sersys.rs`, `src/services.rs`, `src/sys_tools.rs` are domain-specific modules for BLE, CNC, logging, simulations, serial systems, and OS utilities
- `assets/` holds embedded resources and web service worker assets
- `ltspice/` is a separate crate/subproject focused on LTSpice-related simulation functionality

## Important conventions
- `RenderApp` is the main GUI state holder and uses `serde` serialization for persistent state
- Fields that should not be persisted are marked `#[serde(skip)]`
- Concurrency is implemented with `std::sync::mpsc`, `Arc<RwLock<...>>`, and async-friendly patterns in services
- The app uses `HashMap<String, f64>` heavily for dynamic simulation parameters and runtime configuration
- Platform-specific dependencies (e.g. `windows` crate) are gated by `cfg(target_os = "windows")`

## What agents should do
- Prefer small, localized refactors that preserve GUI state and app startup behavior
- Keep `RenderApp` serialization compatibility in mind if changing field names or defaults
- Avoid broad GUI redesigns without understanding `egui` event handling and paint/repaint logic
- When adding features, follow the existing module structure rather than moving unrelated functionality into `src/main.rs`

## References
- See `README.md` for higher-level usage notes and web deployment guidance
