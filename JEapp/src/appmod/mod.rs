use egui::widgets::plot::{Legend, Line, Plot, PlotPoints, Polygon};
//Arrows, Bar, BarChart, BoxElem, BoxPlot, BoxSpread, Corner, HLine,
//  MarkerShape,  PlotImage, PlotPoint,  Points, Text, VLine,  LineStyle,};
// use core::f64::consts::PI;
use egui::*;
pub mod data;
pub mod objects;

//pub use serial::SerialCtrl;
use crate::blesys::{self,  BLEState, BLESys};
use crate::rbbsim::{RbbCtrl, };//RbbState
use crate::sersys::{SerState, SerSys};
/// We derive Deserialize/Serialize so we can persist app state on shutdown.
use device_query::{DeviceQuery, DeviceState, Keycode};
// use num::signum;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::iter::Iterator;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::{
    thread,
    time::{Duration, },//Instant
};

pub struct Mesagging {
    ble_ch: (Sender<BLESys>, Receiver<BLESys>),
    ser_ch: (Sender<SerSys>, Receiver<SerSys>),
}

#[derive(PartialEq, Serialize, Deserialize)]
enum CMDapp {
    Idle,
    BLEmsg,
    UpdPrev,
    Sermsg,
}

pub struct AppsOpen {
    ble:bool,
    rbb:bool,    
}
// if we add new fields, give them default values when deserializing old state
// use macroquad::prelude::*;
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct RenderApp {
    // Example stuff:
    label: String,
    // this how you opt-out of serialization of a member
    data_ready: u8,
    timer: Duration,
    #[serde(skip)]
    apps:AppsOpen,
    picked_path: Option<String>,
    #[serde(skip)]
    sersys: SerSys,
    #[serde(skip)]
    portlist: Vec<String>,
    #[serde(skip)]
    port_sel: String,
    #[serde(skip)]
    dataset: data::RawData,
    #[serde(skip)]
    device_state: DeviceState,
    #[serde(skip)]
    anim_state: objects::ObjAnim,
    draw: u8,
    #[serde(skip)]
    msgs: Mesagging,
    #[serde(skip)]
    blectrl: BLESys,
    #[serde(skip)]
    rbbctrl: RbbCtrl,
    #[serde(skip)]
    cmd: CMDapp,
    #[serde(skip)]
    objstate: HashMap<String, HashMap<String, f64>>,
    #[serde(skip)]
    objectlist: Vec<objects::Obj3D>,
    // surflist: Vec<[[f64; 4]; 2]>,
    #[serde(skip)]
    surflist: Vec<objects::Surf3D>,
    #[serde(skip)]
    threads: Vec<thread::JoinHandle<()>>,
}

impl Default for RenderApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            cmd: CMDapp::Idle,
            picked_path: None,
            apps:AppsOpen {ble:true,rbb:false},
            timer: Duration::new(0, 0),
            data_ready: 0,
            device_state: DeviceState::new(),
            sersys: SerSys::default(),
            port_sel: "-".to_string(),
            portlist: vec![],
            dataset: data::RawData {
                diag: (HashMap::new()),
                dataf: (HashMap::new()),
            },
            blectrl: BLESys::default(),
            rbbctrl: RbbCtrl::default(),       
            msgs: Mesagging {
                ble_ch: mpsc::channel::<BLESys>(),
                ser_ch: mpsc::channel::<SerSys>(),
            },
            anim_state: objects::ObjAnim {
                steps: 0,
                step: 0,
                state: 0,
            },
            draw: 0,
            objectlist: vec![],
            objstate: HashMap::new(),
            surflist: vec![],
            threads: Vec::new(),
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
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for RenderApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let Self {
            label,
            timer,
            blectrl,
            rbbctrl,
            apps,
            device_state,
            msgs,
            picked_path,
            data_ready,
            dataset,
            cmd,
            sersys,
            portlist,
            port_sel,
            anim_state,
            draw,
            objectlist,
            objstate,
            surflist,
            threads,
        } = self;

        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        // egui::Window::new("🔧 Settings")
        //     .vscroll(true)
        //     .show(ctx, |ui| {
        //         ctx.settings_ui(ui);
        //     });

        #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        _frame.close();
                    }
                });
                ui.menu_button("CNC", |ui| {
                    if ui.button("Open OGF…").clicked() {
                        let filename = "./4014iso".to_string();
                        data::process_ogf(filename);

                        // if let Some(path) = rfd::FileDialog::new().pick_file() {
                        //     self.picked_path = Some(path.display().to_string());
                        //    data::processdata(path.display().to_string())
                    }
                    if ui.button("Open RPF…").clicked() {
                        let filename = "./probe.txt".to_string();
                        let mut mesh = objects::Surf3D {
                            pos: [0.0, 0.0, 0.0],
                            param: HashMap::from([("r".to_string(), 2.0)]),
                            alph: 0.5,
                            beta: 0.5,
                            gamm: 0.5,
                            points_raw: [vec![], vec![], vec![]],
                            points: vec![], //X Y points for render
                            scale: 10.0,
                            res: 100, //resolution
                        };
                        // let mut meshRAW: [Vec<f64>; 3] = [vec![], vec![], vec![]];
                        data::process_raw_probe_file(filename, &mut mesh.points_raw);
                        // println!("mesh{:?} ", mesh.points_raw);
                        // objects::draw_3dmesh(&mut meshRAW, &mut mesh);
                        objects::draw_3dmesh_surf(&mut mesh);

                        self.surflist.push(mesh);

                        // if let Some(path) = rfd::FileDialog::new().pick_file() {
                        //     self.picked_path = Some(path.display().to_string());
                        //    data::processdata(path.display().to_string())
                    }
                    if ui.button("Open Satfile…").clicked() {
                        //  let filename ="./11.17.17Device 3.xlsx".to_string();
                        // data::processdata(filename, &mut self.dataset);
                        if let Some(path) = rfd::FileDialog::new().pick_file() {
                            self.picked_path = Some(path.display().to_string());
                            data::processdata(path.display().to_string(), &mut self.dataset)
                        }
                    }
                    if ui.button("draw").clicked() {
                        let mut circle1 = objects::Obj3D {
                            tag: "circle".to_string(),
                            pos: [0.0, 0.0, 0.0],
                            param: HashMap::from([("r".to_string(), 1.0)]),
                            alph: 0.0,
                            beta: 0.0,
                            gamm: 0.0,
                            points: [vec![], vec![]], //X Y points for render
                            scale: 1.0,
                            res: 100, //resolution
                            color: [250, 100, 50],
                        };

                        objects::draw_circle3d(&mut circle1);
                        self.objectlist.push(circle1);
                        self.draw = 1;
                    }
                });
                if ui.button("List COM…").clicked() {
                    if self.sersys.state == SerState::CREATED {
                        let (tx_ser, rx_ser): (Sender<SerSys>, Receiver<SerSys>) =
                            mpsc::channel::<SerSys>();
                        let (tx_a, rx_a): (Sender<SerSys>, Receiver<SerSys>) =
                            mpsc::channel::<SerSys>();
                        self.msgs.ser_ch.0 = tx_a;
                        self.msgs.ser_ch.1 = rx_ser;
                        //self keeps tx_a and rx_w
                        let builder = thread::Builder::new();
                        self.threads.push(
                            builder
                                .spawn(move || SerSys::startserv(tx_ser, rx_a))
                                .unwrap(),
                        );
                        self.cmd = CMDapp::Sermsg;
                    }

                    //   listports(&mut self.portlist);
                    // dbg!();
                    // unreachable!();
                }
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
            // ui.add(egui::Slider::new(value, 0.0..=10.0).text("value"));
            // if ui.button("Increment").clicked() {
            //     *value += 1;
            // }
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.label("The triangle ");
                ui.hyperlink_to("three-d", "https://github.com/asny/three-d");
                ui.label(".");
                ui.end_row();
            });
            ComboBox::from_label("COM Port")
                .selected_text(self.port_sel.to_string())
                .show_ui(ui, |ui| {
                    for i in 0..self.portlist.len() {
                        ui.selectable_value(
                            &mut self.port_sel,
                            (*self.portlist[i]).to_string(),
                            self.portlist[i].to_string(),
                        );
                    }
                });

            if ui.button("Read").clicked() {
                println!("{}", self.port_sel);

                if self.port_sel != "-" {
                    self.sersys
                        .status
                        .insert("sel".to_string(), vec![self.port_sel.clone()]);
                    self.sersys.state = SerState::READ;
                    self.cmd = CMDapp::Sermsg;
                }
            }
            if ui.button("Send").clicked() {
                if self.port_sel != "-" {
                    self.sersys
                        .status
                        .insert("sel".to_string(), vec![self.port_sel.clone()]);
                    self.sersys
                        .status
                        .insert("write".to_string(), vec!["p".to_string()]);
                    self.sersys.state = SerState::WRITE;
                    self.cmd = CMDapp::Sermsg;
                }
                println!("{}", self.port_sel);
                //let mut dataout: Vec<String> = vec![];
            }

            ui.separator();

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
            egui::Window::new(" BLE")
                .enabled(true)
                .open(&mut self.apps.ble)
                .vscroll(true)
                .show(ctx, |ui| {
                    if ui.button("Start").clicked() {
                        if self.blectrl.state == BLEState::CREATED {
                            let (tx_ble, rx_ble): (Sender<BLESys>, Receiver<BLESys>) =
                                mpsc::channel::<BLESys>();
                            let (tx_a, rx_a): (Sender<BLESys>, Receiver<BLESys>) =
                                mpsc::channel::<BLESys>();
                            self.msgs.ble_ch.0 = tx_a;
                            self.msgs.ble_ch.1 = rx_ble;
                            //self keeps tx_a and rx_w
                            let builder = thread::Builder::new();
                            self.threads.push(
                                builder
                                    .spawn(move || BLESys::startserv(tx_ble, rx_a))
                                    .unwrap(),
                            );
                            self.cmd = CMDapp::BLEmsg;
                        }
                        println!("sync.")
                    }

                    let msg = blesys::ble_gui(ui, &mut self.blectrl);
                    if msg == 1 {
                        self.cmd = CMDapp::BLEmsg;
                    }
                });

            egui::Window::new("🔧 RBB")
                .auto_sized()
                .anchor(Align2::LEFT_TOP, [2.0, 2.0])
                .vscroll(true)
                .open(&mut self.apps.rbb)
                .show(ctx, |ui| {
                    crate::rbbsim::rbb_gui(ctx, ui, &mut self.rbbctrl);
                });

            ui.heading("Preview");
            if self.objectlist.len() > 0 || self.surflist.len() > 0 {
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
                    if self.surflist.len() > 0 {
                        let rot: [f64; 2] = [
                            plot_ui.pointer_coordinate_drag_delta()[0] as f64,
                            plot_ui.pointer_coordinate_drag_delta()[1] as f64,
                        ];
                        if rot[0] != 0.0 || rot[1] != 0.0 {
                            let mut i = 0;
                            while i < self.surflist.len() {
                                self.surflist[i].alph = rot[0] * 0.5 + self.surflist[i].alph;
                                self.surflist[i].beta = rot[1] * 0.5 + self.surflist[i].beta;
                                objects::draw_3dmesh_surf(&mut self.surflist[i]);
                                i += 1;
                            }
                        }
                        for obj in self.surflist.iter() {
                            for surf in obj.points.iter() {
                                let x = &surf[0];
                                let y = &surf[1];
                                let plt: PlotPoints = (0..x.len()).map(|i| [x[i], y[i]]).collect();

                                let planned_surf =
                                    Polygon::new(plt).color(Color32::from_rgb(100, 200, 100));
                                plot_ui.polygon(planned_surf);
                            }
                        }
                    
                    }
                });
                //self.data_ready = 0;
            }
        });
    }

    fn post_rendering(&mut self, _window_size_px: [u32; 2], _frame: &eframe::Frame) {
        let keys: Vec<Keycode> = self.device_state.get_keys();
        for key in keys.iter() {
            match key {
                Keycode::A => {
                    if self.objstate.contains_key("rbb") {
                        let mut rbb = self.objstate.get("rbb").unwrap().clone();
                        rbb.entry("a".to_string()).and_modify(|k| *k += 0.01);
                        objects::draw_rbb(&mut rbb, &mut self.objectlist);
                        self.objstate
                            .entry("rbb".to_string())
                            .and_modify(|k| *k = rbb);
                    }
                    self.cmd = CMDapp::UpdPrev;
                }
                Keycode::D => {
                    if self.objstate.contains_key("rbb") {
                        let mut rbb = self.objstate.get("rbb").unwrap().clone();
                        rbb.entry("a".to_string()).and_modify(|k| *k += -0.01);
                        objects::draw_rbb(&mut rbb, &mut self.objectlist);
                        self.objstate
                            .entry("rbb".to_string())
                            .and_modify(|k| *k = rbb);
                    }
                    self.cmd = CMDapp::UpdPrev;
                }
                Keycode::W => {
                    if self.objstate.contains_key("rbb") {
                        let mut rbb = self.objstate.get("rbb").unwrap().clone();
                        rbb.entry("x".to_string()).and_modify(|k| *k += -0.01);
                        objects::draw_rbb(&mut rbb, &mut self.objectlist);
                        self.objstate
                            .entry("rbb".to_string())
                            .and_modify(|k| *k = rbb);
                    }
                    self.cmd = CMDapp::UpdPrev;
                }
                // Keycode::Escape => todo!(),
                // Keycode::Space => todo!(),
                // Keycode::Enter => todo!(),
                _ => println!("Pressed key: {:?}", key),
            }
        }

        match self.msgs.ble_ch.1.try_recv() {
            Ok(data) => {
                println!("fromBLE: {:?}", data.status);
                self.blectrl = data.clone();
                if self.blectrl.status.contains_key("periph") {
                    self.blectrl.blelist = self.blectrl.status.get("periph").unwrap().to_vec()
                }
            }
            Err(_) => { /* handle sender disconnected */ } //Err(TryRecvError::Empty) => { /* handle no data available yet */ }
        }
        match self.msgs.ser_ch.1.try_recv() {
            Ok(data) => {
                println!("fromSer: {:?}", data.status);
                self.sersys = data.clone();
                if self.sersys.status.contains_key("list") {
                    self.portlist = self.sersys.status.get("list").unwrap().to_vec()
                }
            }
            Err(_) => { /* handle sender disconnected */ } //Err(TryRecvError::Empty) => { /* handle no data available yet */ }
        }
        match self.cmd {
            CMDapp::BLEmsg => {
                if let Err(_) = self.msgs.ble_ch.0.send(self.blectrl.clone()) {
                    println!("BLE has stopped listening.")
                }
                thread::sleep(Duration::from_millis(200));
                self.cmd = CMDapp::Idle;
            }
            CMDapp::Sermsg => {
                if let Err(_) = self.msgs.ser_ch.0.send(self.sersys.clone()) {
                    println!("Ser has stopped listening.")
                }
                thread::sleep(Duration::from_millis(200));
                self.cmd = CMDapp::Idle;
            }
            CMDapp::Idle => (),

            CMDapp::UpdPrev => {
                //self.greetold = self.ui.greet.clone();

                self.cmd = CMDapp::Idle;
            }
        }
    }

    fn on_close_event(&mut self) -> bool {
        true
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {}

    fn auto_save_interval(&self) -> std::time::Duration {
        std::time::Duration::from_secs(30)
    }

    fn max_size_points(&self) -> egui::Vec2 {
        egui::Vec2::INFINITY
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> egui::Rgba {
        // NOTE: a bright gray makes the shadows of the windows look weird.
        // We use a bit of transparency so that if the user switches on the
        // `transparent()` option they get immediate results.
        egui::Color32::from_rgba_unmultiplied(12, 12, 12, 180).into()

        // _visuals.window_fill() would also be a natural choice
    }

    fn persist_native_window(&self) -> bool {
        true
    }

    fn persist_egui_memory(&self) -> bool {
        true
    }

    fn warm_up_enabled(&self) -> bool {
        false
    }
}

pub fn rbb_gui(ui: &mut Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("The triangle ");
        ui.hyperlink_to("three-d", "https://github.com/asny/three-d");
        ui.label(".");
        ui.end_row();
    });
}
