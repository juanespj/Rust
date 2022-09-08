use egui::widgets::plot::{
    Arrows, Bar, BarChart, BoxElem, BoxPlot, BoxSpread, Corner, HLine, Legend, Line, LineStyle,
    MarkerShape, Plot, PlotImage, PlotPoint, PlotPoints, Points, Polygon, Text, VLine,
};
use egui::*;
/// We derive Deserialize/Serialize so we can persist app state on shutdown.
// #[derive(serde::Deserialize, serde::Serialize)]
// #[serde(default)]
// if we add new fields, give them default values when deserializing old state
use macroquad::prelude::*;
use std::f64::consts::TAU;
pub struct RenderApp {
    // Example stuff:
    label: String,
    angle: f32,
    // this how you opt-out of serialization of a member
    // #[serde(skip)]
    value: f32,
    picked_path: Option<String>,
}

impl Default for RenderApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            value: 2.7,
            angle: 0.2,
            picked_path: None,
        }
    }
}

impl RenderApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customized the look at feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        // if let Some(storage) = cc.storage {
        //     return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        // }

        Default::default()
    }
}

impl eframe::App for RenderApp {
    /// Called by the frame work to save state before shutdown.
    // fn save(&mut self, storage: &mut dyn eframe::Storage) {
    //     eframe::set_value(storage, eframe::APP_KEY, self);
    // }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let Self {
            label,
            value,
            angle,
            picked_path,
        } = self;

        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open file…").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_file() {
                            self.picked_path = Some(path.display().to_string());
                        }
                    }
                    if ui.button("Quit").clicked() {
                        _frame.close();
                    }
                });
            });
        });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Side Panel");

            ui.horizontal(|ui| {
                ui.label("Write something: ");
                ui.text_edit_singleline(label);
            });
            //file
            if let Some(picked_path) = &self.picked_path {
                ui.horizontal(|ui| {
                    ui.label("Picked file:");
                    ui.monospace(picked_path);
                });
            }
            ui.add(egui::Slider::new(value, 0.0..=10.0).text("value"));
            if ui.button("Increment").clicked() {
                *value += 1.0;
            }
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.label("The triangle is being painted using ");
                ui.hyperlink_to("three-d", "https://github.com/asny/three-d");
                ui.label(".");
            });

            //bottom
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.label("powered by ");
                    ui.hyperlink_to("egui", "https://github.com/emilk/egui");
                    ui.label(" and ");
                    ui.hyperlink_to(
                        "eframe",
                        "https://github.com/emilk/egui/tree/master/crates/eframe",
                    );
                    ui.label(".");
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's

            ui.heading("Preview");
            ui.hyperlink("https://github.com/emilk/eframe_template");
            ui.add(egui::github_link_file!(
                "https://github.com/emilk/eframe_template/blob/master/",
                "Source code."
            ));
            ui.label("Drag to rotate!");
            let mut plot = Plot::new("lines")
                .include_x(0.0)
                .include_y(0.0)
                .legend(Legend::default());
            plot.show(ui, |plot_ui| {
                let x = [0.0, 1.0, 2.0, 3.0, 4.0, 5.0];
                let y = [5.0, 4.0, 3.0, 2.0, 1.0, 0.0];

                let plt: PlotPoints = (0..x.len()).map(|i| [x[i], y[i]]).collect();

                let planned_line = Line::new(plt);
                plot_ui.line(planned_line);
                let sin: PlotPoints = (0..1000)
                    .map(|i| {
                        let x = i as f64 * 0.01;
                        [x, x.sin()]
                    })
                    .collect();
                // println!("{:?}",sin );
                let planned_line = Line::new(sin).fill(0.0);
                plot_ui.line(planned_line);
                let mut t: f64 = 0.0;
                let mut r: f64 = 5.0;

                let circle: PlotPoints = (0..100)
                    .map(|i| {
                        let t = i as f64 * 0.01;
                        [0.0 + r * t.cos(), 0.0 + r * t.sin()]
                    })
                    .collect();
                let planned_line = Line::new(circle);
                plot_ui.line(planned_line);

                // let planned_line = Line::new(series.into_iter().map(|x|x)).fill(0.0);
                // let planned_line = Line::new(PlotPoints::from_iter(series.into_iter()));
                // plot_ui.line(planned_line);
            });

            egui::warn_if_debug_build(ui);
        });

        if false {
            egui::Window::new("Window").show(ctx, |ui| {
                ui.label("Windows can be moved by dragging them.");
                ui.label("They are automatically sized based on contents.");
                ui.label("You can turn on resizing and scrolling if you like.");
                ui.label("You would normally chose either panels OR windows.");
            });
        }
    }
}

// fn circle() -> Line {

//     let n = 512;
//     let line_style= LineStyle::Solid;
//     let circle = (0..=n).map(|i| {
//         let t = remap(i as f64, 0.0..=(n as f64), 0.0..=TAU);
//         let r = 2.0;
//         let circle_center= Pos2::new(0.0, 0.0);
//         PlotPoint ::new(
//             r * t.cos() + circle_center.x as f64,
//             r * t.sin() + circle_center.y as f64,
//         )
//     });
//     Line::new(PlotPoints ::from_values_iter(circle))
//         .color(Color32::from_rgb(100, 200, 100))
//         .style(line_style)
//         .name("circle")
// }
