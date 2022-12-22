#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::collections::BTreeMap;
pub mod objects;
use device_query::{DeviceQuery, DeviceState, Keycode};
use num::signum;
use eframe::{egui, NativeOptions};
use egui::*;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::iter::Iterator;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::{
    thread,
    time::{Duration, Instant},
};
use egui_dock::{DockArea, NodeIndex, Style, Tree};
/// We identify tabs by the title of the file we are editing.
type Title = String;

fn main() {
    let options = NativeOptions::default();
    eframe::run_native(
        "Text editor examples",
        options,
        Box::new(|_cc| Box::new(MyApp::default())),
    );
}

struct Buffers {
    buffers: BTreeMap<Title, String>,

}

impl egui_dock::TabViewer for Buffers {
    type Tab = Title;
    
    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Title) {
        // let text = self.buffers.entry(title.clone()).or_default();
        // egui::TextEdit::multiline(text)
        //     .desired_width(f32::INFINITY)
        //     .show(ui);
        match tab.as_str() {
            "BLE" => {
                egui::menu::bar(ui, |ui| {
                    ui.menu_button("File", |ui| if ui.button("Quit").clicked() {});
                });
                ui.spacing_mut().item_spacing.y = 10.0;
                ui.heading("BLE:");
            }

            "Serial" => {
                ui.spacing_mut().item_spacing.y = 10.0;
                ui.heading("SERIAL:");
                if self.objectlist.len() > 0  {
                    let plot = Plot::new("preview")
                        .include_x(0.0)
                        .include_y(0.0)
                        .width(600.0)
                        .height(300.0)
                        .view_aspect(1.0)
                        .data_aspect(1.0)
                        .allow_zoom(true)
                        .allow_drag(true)
                        .show_axes([false; 2])
                        .show_background(false)
                        .legend(Legend::default());
    
                    plot.show(ui, |plot_ui| {
                        if self.objectlist.len() > 0 {
                            for obj in self.objectlist.iter() {
                                let x = &obj.points[0];
                                let y = &obj.points[1];
                                let plt: PlotPoints =
                                    (0..x.len()).map(|i| [x[i] as f64, y[i] as f64]).collect();
    
                                let planned_line = Line::new(plt).color(Color32::from_rgb(
                                    obj.color[0],
                                    obj.color[1],
                                    obj.color[2],
                                ));
                                plot_ui.line(planned_line);
                            }
                        }
                    });}
            }
            _ => {
                ui.label(format!("Content of {tab}"));
            }
        }
    }

    fn title(&mut self, title: &mut Title) -> egui::WidgetText {
        egui::WidgetText::from(&*title)
    }
}

struct MyApp {
    buffers: Buffers,
    tree: egui_dock::Tree<String>,
    #[serde(skip)]
    objstate: HashMap<String, HashMap<String, f64>>,
    #[serde(skip)]
    objectlist: Vec<objects::Obj3D>,
}

impl Default for MyApp {
    fn default() -> Self {
         let mut buffers = BTreeMap::default();
        buffers.insert(
            "BLE".to_owned(),
            "".to_owned(),
        );
        buffers.insert("Files".to_owned(), "".to_owned(),);
        buffers.insert(
            "Serial".to_owned(),
            "".to_owned(),
           // include_str!("../README.md").to_owned(),
        );
        let mut tree = egui_dock::Tree::new(vec![
            "Files".to_owned(),
            "BLE".to_owned(),
            "Serial".to_owned(),
        ]);
        let [a, b] = tree.split_right(NodeIndex::root(), 0.3, vec!["tab3".to_owned()]);
        let [_, _] = tree.split_below(a, 0.7, vec!["tab4".to_owned()]);
        //let [_, _] = tree.split_below(b, 0.5, vec!["tab5".to_owned()]);
        
        // let tree = egui_dock::Tree::new(vec!["README.md".to_owned(), "CHANGELOG.md".to_owned()]);

        Self {
           buffers: Buffers { buffers },
            tree,
            objectlist: vec![],
            objstate: HashMap::new(),
        }
    }
}

impl eframe::App for MyApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("documents").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| if ui.button("Quit").clicked() {});
            });
            ui.spacing_mut().item_spacing.y = 10.0;
            ui.heading("Main Panel:");
            for title in self.buffers.buffers.keys() {
                
                let tab_location = self.tree.find_tab(title);
                let is_open = tab_location.is_some();
                if ui.selectable_label(is_open, title).clicked() {
                    if let Some((node_index, tab_index)) = tab_location {
                        self.tree.set_active_tab(node_index, tab_index);
                    } else {
                        // Open the file for editing:
                        self.tree.push_to_focused_leaf(title.clone());
                    }
                }
            }
            if ui.button("reset_rbb").clicked() {
                let mut rbb: HashMap<String, f64> = HashMap::from([
                    ("w".to_string(), 15.0),
                    ("h".to_string(), 0.10),
                    ("a".to_string(), 0.00),
                    ("r".to_string(), 0.5),
                    ("x".to_string(), 0.0),
                    ("xv".to_string(), 0.0),
                    ("xa".to_string(), 0.0),
                    ("m".to_string(), 1.0),
                ]);
                objects::draw_rbb(&mut rbb, &mut self.objectlist);
                self.objstate
                    .entry("rbb".to_string())
                    .and_modify(|k| *k = rbb);
                self.draw = 1;
            }
        });

        egui_dock::DockArea::new(&mut self.tree)
            .style(egui_dock::Style::from_egui(ctx.style().as_ref()))
            .show(ctx, &mut self.buffers);
    }
}
