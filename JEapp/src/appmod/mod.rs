use egui::widgets::plot::{Arrows, Legend, Line, Plot, PlotPoint, PlotPoints, Polygon, Text};
// Bar, BarChart, BoxElem, BoxPlot, BoxSpread, Corner, HLine,
//  MarkerShape,  PlotImage,  Points, Text, VLine,  LineStyle,};
// use core::f64::consts::PI;
use egui::*;
pub mod data;
pub mod objects;
use crate::datalogger::{log_plot, LoggerCtrl, LoggerState};
//pub use serial::SerialCtrl;
use crate::blesys::{self, BLEState, BLESys};
use crate::cnc::{CNCCtrl, CNCState};
use crate::ltspicesim::{SimCtrl, SimState};
use crate::rbbsim::{RbbCtrl, RbbState};
use crate::sersys::{SerState, SerSys};
/// We derive Deserialize/Serialize so we can persist app state on shutdown.
use device_query::{DeviceQuery, DeviceState, Keycode};
// use num::signum;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
// use std::iter::Iterator;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::{
    thread,
    time::Duration, //Instant
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
#[derive(serde::Deserialize, serde::Serialize)]
pub struct AppsOpen {
    ble: bool,
    rbb: bool,
    logger: bool,
    ltsim: bool,
    cnc: bool,
}
// if we add new fields, give them default values when deserializing old state
// use macroquad::prelude::*;
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct RenderApp {
    // Example stuff:
    label: String,
    // this how you opt-out of serialization of a member
    #[serde(skip)]
    data_ready: u8,
    timer: Duration,
    // #[serde(skip)]
    apps: AppsOpen,
    picked_path: String,
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
    ltsctrl: SimCtrl,
    #[serde(skip)]
    logger: LoggerCtrl,
    #[serde(skip)]
    cnc: CNCCtrl,

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
            picked_path: "".to_string(),
            apps: AppsOpen {
                ble: false,
                rbb: false,
                ltsim: false,
                logger: true,
                cnc: false,
            },
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
            ltsctrl: SimCtrl::default(),
            cnc: CNCCtrl::default(),
            logger: LoggerCtrl::default(),
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
            ltsctrl,
            cnc,
            logger,
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
                    if ui.button("LTSpice").clicked() {
                        self.apps.ltsim = !self.apps.ltsim;
                    }
                    if ui.button("CNC").clicked() {
                        self.apps.cnc = !self.apps.cnc;
                    }
                    if ui.button("RBB").clicked() {
                        self.apps.rbb = !self.apps.cnc;
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
                }
            });
        });

        egui::SidePanel::left("side_panel")
            .exact_width(150.0)
            .show(ctx, |ui| {
                ui.heading("App Control");
                ui.label("DataFolder");
                if ui.button("Path").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        self.picked_path = path.display().to_string();
                    }
                }
                if self.picked_path.len() > 0 {
                    ui.label(&self.picked_path);
                }

                ui.separator();
                ComboBox::from_label("COM Port")
                    .selected_text(self.port_sel.to_string())
                    .width(60.0)
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
                if ui.button("Monitor").clicked() {
                    println!("{}", self.port_sel);

                    if self.port_sel != "-" {
                        self.sersys
                            .status
                            .insert("sel".to_string(), vec![self.port_sel.clone()]);
                        self.sersys.state = SerState::MONITOR;
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
                        // ui.label("powered by ");
                        // ui.hyperlink_to("egui", "https://github.com/emilk/egui");
                        // ui.label(" and ");
                        // ui.hyperlink_to(
                        //     "eframe",
                        //     "https://github.com/emilk/egui/tree/master/crates/eframe",
                        // );
                        ui.label(".");
                    });
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            if self.apps.ble {
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
            }
            egui::Window::new("🔧 LightGATE")
                // .auto_sized()
                
                .anchor(Align2::LEFT_TOP, [2.0, 2.0])
                .vscroll(true)
                .open(&mut self.apps.logger)
                .show(ctx, |ui| {
                    crate::datalogger::log_gui(ctx, ui, &mut self.logger);
                });
            if self.apps.rbb {
                egui::Window::new("🔧 RBB")
                    .auto_sized()
                    .anchor(Align2::LEFT_TOP, [2.0, 2.0])
                    .vscroll(true)
                    .open(&mut self.apps.rbb)
                    .show(ctx, |ui| {
                        crate::rbbsim::rbb_gui(ctx, ui, &mut self.rbbctrl);
                    });
            }
            if self.apps.ltsim {
                egui::Window::new("🔧 LTSim")
                    .auto_sized()
                    // .anchor(Align2::LEFT_TOP, [2.0, 2.0])
                    .vscroll(true)
                    .open(&mut self.apps.ltsim)
                    .show(ctx, |ui| {
                        crate::ltspicesim::lts_gui(ctx, ui, &mut self.ltsctrl);
                    });
            }
            if self.apps.cnc {
                egui::Window::new("🔧 CNC")
                    .auto_sized()
                    // .anchor(Align2::LEFT_TOP, [2.0, 2.0])
                    .vscroll(true)
                    .open(&mut self.apps.cnc)
                    .show(ctx, |ui| {
                        crate::cnc::main_gui(ctx, ui, &mut self.cnc);
                    });
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
                Keycode::S => {
                    // println!("{:?}", self.sersys);
                    if self.sersys.state == SerState::IDLE {
                        let mut msg = "BTN\r\nFirst pass time\r\n2023/6/22 (Thursday) -31-3-3\r\nWritting to filename-2262023_9-3-3.csv\r\nFile created ok, proceed to record\r\n\n\rSTARTLOG\r\nRead: 495,1,3.15\r\n".to_string();
                        let msg1 = "Read: 562,LG1,-6.32,44.47\r\nRead: 562,Temp,29.00\r\nRead: 683,1,3.08\r\n44.42\r\nRead: 749,LG1,-6.18,44.42\r\n".to_string();
                        let msg2 ="Read: 749,Temp,29.00\r\nRead: 870,1,3.08\r\n44.39\r\nRead: 936,LG1,-6.09,44.39\r\nRead: 936,Temp,28.75\r\n".to_string();
                        let msg3 ="Read: 1057,1,3.15\r\n44.29\r\nRead: 1123,LG1,-5.80,44.29\r\nRead: 1123,Temp,29.25\r\nRead: 1244,1,3.08\r\n44.29\r\nRead: 1310,LG1,-5.80,44.29\r\nRead: 1310,Temp,28.25\r\nRead: 1431,1,3.08\r\n44.31\r\nRead: 1497,LG1,-5.87,44.31\r\nRead: 1497,Temp,29.25\r\nRead: 1618,1,3.08\r\n44.23\r\nRead: 1687,LG1,-5.64,44.23\r\nRead: 1687,Temp,29.50\r\nRead: 1808,1,3.15\r\n44.28\r\nRead: 1874,LG1,-5.78,44.28\r\nRead: 1874,Temp,28.50\r\nRead: 1995,1,3.08\r\n44.29\r\nRead: 2061,LG1,-5.80,44.29\r\nRead: 2061,Temp,28.75\r\nRead: 2182,1,3.08\r\n44.31\r\nRead: 2248,LG1,-5.87,44.31\r\nRead: 2248,Temp,28.75\r\nRead: 2369,1,3.08\r\n44.20\r\nRead: 2435,LG1,-5.53,44.20\r\nRead: 2435,Temp,28.25\r\nRead: 2556,1,3.15\r\n44.14\r\nRead: 2622,LG1,-5.37,44.14\r\nRead: 2622,Temp,28.75\r\nRead: 2743,1,3.08\r\n44.11\r\nRead: 2809,LG1,-5.28,44.11\r\nRead: 2809,Temp,28.50\r\nRead: 2930,1,3.08\r\n44.06\r\nRead: 2996,LG1,-5.14,44.06\r\nRead: 2999,Temp,29.00\r\nRead: 3119,1,3.15\r\n44.03\r\nRead: 3185,LG1,-5.05,44.03\r\nRead: 3185,Temp,29.00\r\nRead: 3306,1,3.08\r\n44.02\r\nRead: 3372,LG1,-5.03,44.02\r\nRead: 3372,Temp,28.75\r\nRead: 3493,1,3.22\r\n43.99\r\nRead: 3559,LG1,-4.94,43.99\r\nRead: 3559,Temp,28.75\r\nRead: 3680,1,3.22\r\n44.03\r\nRead: 3746,LG1,-5.05,44.03\r\nRead: 3746,Temp,29.25\r\nRead: 3867,1,3.08\r\n44.13\r\nRead: 3933,LG1,-5.35,44.13\r\nRead: 3933,Temp,29.00\r\nRead: 4054,1,2.86\r\n44.40\r\nRead: 4120,LG1,-6.11,44.40\r\nRead: 4120,Temp,29.00\r\nRead: 4241,1,4.03\r\n34.88\r\nRead: 4307,LG1,21.37,34.88\r\nRead: 4307,Temp,29.25\r\nRead: 4430,1,166.26\r\n28.15\r\nRead: 4496,LG1,40.82,28.15\r\nRead: 4496,Temp,29.75\r\nRead: 4617,1,250.12\r\n30.98\r\nRead: 4683,LG1,32.63,30.98\r\nRead: 4683,Temp,33.50\r\nRead: 4804,1,271.58\r\n34.80\r\nRead: 4870,LG1,21.59,34.80\r\nRead: 4870,Temp,31.50\r\nRead: 4991,1,297.73\r\n37.29\r\nRead: 5057,LG1,14.42,37.29\r\nRead: 5057,Temp,31.25\r\nRead: 5178,1,340.65\r\n38.77\r\nRead: 5244,LG1,10.13,38.77\r\nRead: 5244,Temp,31.50\r\nRead: 5365,1,361.08\r\n39.78\r\nRead: 5431,LG1,7.22,39.78\r\nRead: 5431,Temp,33.50\r\nRead: 5552,1,417.55\r\n40.48\r\nRead: 5618,LG1,5.21,40.48\r\nRead: 5618,Temp,35.00\r\nRead: 5741,1,420.92\r\n41.27\r\nRead: 5807,LG1,2.91,41.27\r\nRead: 5807,Temp,35.00\r\nRead: 5928,1,394.26\r\n41.39\r\nRead: 5994,LG1,2.57,41.39\r\nRead: 5994,Temp,34.75\r\nRead: 6115,1,360.21\r\n41.27\r\nRead: 6181,LG1,2.93,41.27\r\nRead: 6181,Temp,34.25\r\nRead: 6302,1,330.32\r\n41.17\r\nRead: 6368,LG1,3.20,41.17\r\nRead: 6368,Temp,34.50\r\nRead: 6489,1,156.30\r\n41.26\r\nRead: 6555,LG1,2.96,41.26\r\nRead: 6555,Temp,33.75\r\nRead: 6676,1,3.52\r\n41.82\r\nRead: 6742,LG1,1.33,41.82\r\nRead: 6742,Temp,33.50\r\nRead: 6863,1,3.00\r\n42.41\r\nRead: 6929,LG1,-0.36,42.41\r\nRead: 6929,Temp,31.50\r\nRead: 7050,1,3.08\r\n42.52\r\nRead: 7119,LG1,-0.68,42.52\r\nRead: 7119,Temp,31.50\r\nRead: 7240,1,3.15\r\n44.34\r\nRead: 7306,LG1,-5.96,44.34\r\nRead: 7306,Temp,31.50\r\nRead: 7427,1,3.15\r\n44.63\r\nRead: 7493,LG1,-6.77,44.63\r\nRead: 7493,Temp,29.75\r\nRead: 7614,1,3.08\r\n44.65\r\nRead: 7680,LG1,-6.84,44.65\r\nRead: 7680,Temp,30.00\r\nRead: 7801,1,3.15\r\n44.97\r\nRead: 7867,LG1,-7.76,44.97\r\nRead: 7867,Temp,30.50\r\nRead: 7988,1,3.15\r\n45.05\r\nRead: 8054,LG1,-7.99,45.05\r\nRead: 8054,Temp,30.75\r\nRead: 8175,1,3.15\r\n45.28\r\nRead: 8241,LG1,-8.66,45.28\r\nRead: 8241,Temp,30.75\r\nRead: 8362,1,3.00\r\n45.06\r\nRead: 8428,LG1,-8.03,45.06\r\nRead: 8431,Temp,30.00\r\nRead: 8551,1,3.22\r\n44.96\r\nRead: 8617,LG1,-7.74,44.96\r\nRead: 8617,Temp,30.75\r\nRead: 8738,1,3.08\r\n44.48\r\nRead: 8804,LG1,-6.36,44.48\r\nRead: 8804,Temp,29.50\r\nRead: 8925,1,3.22\r\n44.45\r\nRead: 8991,LG1,-6.27,44.45\r\nRead: 8991,Temp,30.25\r\nRead: 9112,1,3.37\r\n44.73\r\nRead: 9178,LG1,-7.06,44.73\r\nRead: 9178,Temp,29.75\r\nRead: 9299,1,3.37\r\n38.24\r\nRead: 9365,LG1,11.67,38.24\r\nRead: 9365,Temp,30.00\r\nRead: 9486,1,4.32\r\n22.84\r\nRead: 9552,LG1,56.14,22.84\r\nRead: 9552,Temp,30.50\r\nRead: 9673,1,116.97\r\n25.20\r\nRead: 9739,LG1,49.33,25.20\r\nRead: 9739,Temp,32.25\r\nRead: 9862,1,90.23\r\n29.62\r\nRead: 9928,LG1,36.55,29.62\r\nRead: 9928,Temp,31.50\r\nRead: 10049,1,82.10\r\n31.19\r\nRead: 10115,LG1,32.04,31.19\r\nRead: 10115,Temp,31.50\r\nRead: 10236,1,91.26\r\n33.63\r\nRead: 10302,LG1,24.98,33.63\r\nRead: 10302,Temp,30.25\r\nRead: 10423,1,154.32\r\n35.63\r\nRead: 10489,LG1,19.22,35.63\r\nRead: 10489,Temp,30.50\r\nRead: 10610,1,204.20\r\n38.80\r\nRead: 10676,LG1,10.06,38.80\r\nRead: 10676,Temp,34.75\r\nRead: 10797,1,258.40\r\n40.88\r\nRead: 10863,LG1,4.04,40.88\r\nRead: 10863,Temp,34.25\r\nRead: 10984,1,266.31\r\n42.00\r\nRead: 11050,LG1,0.81,42.00\r\nRead: 11050,Temp,34.25\r\nRead: 11173,1,255.10\r\n42.20\r\nRead: 11239,LG1,0.25,42.20\r\nRead: 11239,Temp,34.50\r\nRead: 11360,1,281.25\r\n42.29\r\nRead: 11426,LG1,-0.02,42.29\r\nRead: 11426,Temp,34.25\r\nRead: 11547,1,364.45\r\n42.34\r\nRead: 11613,LG1,-0.18,42.34\r\nRead: 11613,Temp,34.00\r\nRead: 11734,1,527.42\r\n42.20\r\nRead: 11800,LG1,0.25,42.20\r\nRead: 11800,Temp,34.75\r\nRead: 11921,1,56.62\r\n42.26\r\nRead: 11987,LG1,0.07,42.26\r\nRead: 11987,Temp,34.25\r\nRead: 12108,1,3.15\r\n42.52\r\nRead: 12174,LG1,-0.68,42.52\r\nRead: 12174,Temp,29.50\r\nRead: 12295,1,3.15\r\n42.63\r\nRead: 12361,LG1,-1.02,42.63\r\nRead: 12361,Temp,33.00\r\nRead: 12484,1,4.03\r\n42.52\r\nRead: 12550,LG1,-0.68,42.52\r\nRead: 12550,Temp,31.75\r\nRead: 12671,1,3.15\r\n42.65\r\nRead: 12737,LG1,-1.06,42.65\r\nRead: 12737,Temp,32.00\r\nRead: 12858,1,2.93\r\n42.74\r\nRead: 12924,LG1,-1.33,42.74\r\nRead: 12924,Temp,31.00\r\nRead: 13045,1,3.08\r\n42.92\r\nRead: 13111,LG1,-1.85,42.92\r\nRead: 13111,Temp,31.25\r\nRead: 13232,1,3.15\r\n43.03\r\nRead: 13298,LG1,-2.17,43.03\r\nRead: 13298,Temp,30.75\r\nRead: 13419,1,3.15\r\n43.15\r\nRead: 13485,LG1,-2.50,43.15\r\nRead: 13485,Temp,31.50\r\nRead: 13606,1,3.15\r\n43.27\r\nRead: 13672,LG1,-2.84,43.27\r\nRead: 13672,Temp,31.25\r\nRead: 13795,1,3.22\r\n43.31\r\nRead: 13861,LG1,-2.98,43.31\r\nRead: 13861,Temp,30.50\r\nRead: 13982,1,3.15\r\n43.29\r\nRead: 14048,LG1,-2.91,43.29\r\nRead: 14048,Temp,29.50\r\nRead: 14169,1,3.22\r\n43.35\r\nRead: 14235,LG1,-3.09,43.35\r\nRead: 14235,Temp,29.50\r\nRead: 14356,1,3.15\r\n43.55\r\nRead: 14422,LG1,-3.66,43.55\r\nRead: 14422,Temp,29.50\r\nRead: 14543,1,3.08\r\n43.61\r\nRead: 14609,LG1,-3.84,43.61\r\nRead: 14609,Temp,30.50\r\nRead: 14730,1,3.08\r\n43.58\r\nRead: 14796,LG1,-3.75,43.58\r\nRead: 14796,Temp,30.50\r\nRead: 14917,1,3.15\r\n43.64\r\nRead: 14983,LG1,-3.93,43.64\r\nRead: 14983,Temp,31.25\r\nRead: 15106,1,3.22\r\n43.63\r\nRead: 15172,LG1,-3.88,43.63\r\nRead: 15172,Temp,31.00\r\nRead: 15293,1,3.30\r\n43.54\r\nRead: 15359,LG1,-3.63,43.54\r\nRead: 15359,Temp,31.50\r\nRead: 15480,1,3.00\r\n43.51\r\nRead: 15546,LG1,-3.54,43.51\r\nRead: 15546,Temp,30.75\r\nRead: 15667,1,3.08\r\n43.48\r\nRead: 15733,LG1,-3.45,43.48\r\nRead: 15733,Temp,30.25\r\nRead: 15854,1,3.08\r\n43.52\r\nRead: 15920,LG1,-3.57,43.52\r\nRead: 15920,Temp,31.25\r\nRead: 16041,1,3.15\r\n43.58\r\nRead: 16107,LG1,-3.75,43.58\r\nRead: 16107,Temp,30.75\r\nRead: 16228,1,3.08\r\n43.59\r\nRead: 16294,LG1,-3.79,43.59\r\nRead: 16294,Temp,29.50\r\nRead: 16417,1,3.15\r\n43.60\r\nRead: 16483,LG1,-3.81,43.60\r\nRead: 16483,Temp,30.25\r\nRead: 16604,1,3.08\r\n43.61\r\nRead: 16670,LG1,-3.84,43.61\r\nRead: 16670,Temp,30.25\r\nRead: 16791,1,3.22\r\n43.74\r\nRead: 16857,LG1,-4.22,43.74\r\nRead: 16857,Temp,29.25\r\nRead: 16978,1,3.08\r\n43.86\r\nRead: 17044,LG1,-4.56,43.86\r\nRead: 17044,Temp,30.00\r\nRead: 17165,1,3.15\r\n43.93\r\nRead: 17231,LG1,-4.76,43.93\r\nRead: 17231,Temp,30.25\r\nRead: 17352,1,3.15\r\n44.03\r\nRead: 17418,LG1,-5.05,44.03\r\nRead: 17418,Temp,30.50\r\nRead: 17539,1,3.15\r\n44.18\r\nRead: 17605,LG1,-5.48,44.18\r\nRead: 17605,Temp,29.75\r\nRead: 17728,1,3.22\r\n38.48\r\nRead: 17794,LG1,10.99,38.48\r\nRead: 17794,Temp,31.25\r\nRead: 17915,1,108.11\r\n31.16\r\nRead: 17981,LG1,32.11,31.16\r\nRead: 17981,Temp,33.00\r\nRead: 18102,1,156.45\r\n31.91\r\nRead: 18168,LG1,29.97,31.91\r\nRead: 18168,Temp,31.25\r\nRead: 18289,1,228.74\r\n35.62\r\nRead: 18355,LG1,19.25,35.62\r\nRead: 18355,Temp,34.50\r\nRead: 18476,1,287.55\r\n39.24\r\nRead: 18542,LG1,8.78,39.24\r\nRead: 18542,Temp,34.50\r\nRead: 18663,1,352.59\r\n41.17\r\nRead: 18729,LG1,3.20,41.17\r\nRead: 18729,Temp,34.00\r\nRead: 18850,1,400.49\r\n42.01\r\nRead: 18916,LG1,0.79,42.01\r\nRead: 18919,Temp,33.75\r\nRead: 19039,1,438.50\r\n42.29\r\nRead: 19105,LG1,-0.02,42.29\r\nRead: 19105,Temp,34.75\r\nRead: 19226,1,441.72\r\n42.27\r\nRead: 19292,LG1,0.02,42.27\r\nRead: 19292,Temp,34.25\r\nRead: 19413,1,446.19\r\n42.33\r\nRead: 19479,LG1,-0.14,42.33\r\nRead: 19479,Temp,34.75\r\nRead: 19600,1,496.00\r\n42.26\r\nRead: 19666,LG1,0.07,42.26\r\nRead: 19666,Temp,35.25\r\nRead: 19787,1,505.74\r\n42.09\r\nRead: 19853,LG1,0.54,42.09\r\nRead: 19853,Temp,37.00\r\nRead: 19974,1,446.34\r\n41.92\r\nRead: 20040,LG1,1.04,41.92\r\nRead: 20040,Temp,36.50\r\nRead: 20161,1,282.71\r\n41.84\r\nRead: 20227,LG1,1.29,41.84\r\nRead: 20230,Temp,35.75\r\nRead: 20350,1,15.53\r\n42.09\r\nRead: 20416,LG1,0.56,42.09\r\nRead: 20416,Temp,33.75\r\nRead: 20537,1,3.22\r\n41.84\r\nRead: 20603,LG1,1.26,41.84\r\nRead: 20603,Temp,31.00\r\nRead: 20724,1,3.15\r\n41.65\r\nRead: 20790,LG1,1.83,41.65\r\nRead: 20790,Temp,30.50\r\nRead: 20911,1,3.15\r\n41.68\r\nRead: 20977,LG1,1.74,41.68\r\nRead: 20977,Temp,33.00\r\nRead: 21098,1,3.08\r\n41.55\r\nRead: 21164,LG1,2.10,41.55\r\nRead: 21164,Temp,32.50\r\nRead: 21285,1,3.15\r\n41.53\r\nRead: 21351,LG1,2.17,41.53\r\nRead: 21351,Temp,32.50\r\nRead: 21472,1,3.15\r\n41.66\r\nRead: 21538,LG1,1.81,41.66\r\nRead: 21541,Temp,31.50\r\nRead: 21661,1,3.15\r\n41.77\r\nRead: 21727,LG1,1.47,41.77\r\nRead: 21727,Temp,31.50\r\nRead: 21848,1,3.15\r\n41.81\r\nRead: 21914,LG1,1.35,41.81\r\nRead: 21914,Temp,31.25\r\nRead: 22035,1,3.15\r\n42.05\r\nRead: 22101,LG1,0.65,42.05\r\nRead: 22101,Temp,31.75\r\n".to_string();
                        let msg4= "Read: 22222,1,3.15\r\n41.57\r\nRead: 22288,LG1,2.05,41.57\r\nRead: 22288,Temp,31.25\r\nRead: 22409,1,3.30\r\n41.43\r\nRead: 22475,LG1,2.46,41.43\r\nRead: 22475,Temp,30.75\r\nRead: 22596,1,3.15\r\n40.53\r\nRead: 22662,LG1,5.05,40.53\r\nRead: 22662,Temp,32.00\r\nRead: 22783,1,3.22\r\n40.65\r\nRead: 22849,LG1,4.72,40.65\r\nRead: 22849,Temp,32.00\r\nRead: 22972,1,3.37\r\n40.89\r\nRead: 23038,LG1,4.02,40.89\r\nRead: 23038,Temp,31.25\r\nRead: 23159,1,2.93\r\n41.00\r\nRead: 23225,LG1,3.70,41.00\r\nRead: 23225,Temp,30.75\r\nBTN\r\nSTOPLOG BTN\r\n".to_string();

                        if self.data_ready == 2 {
                            msg = msg1;
                            println!("msg#{:?}", self.data_ready);
                        }
                        if self.data_ready == 3 {
                            msg = msg2;
                            println!("msg#{:?}", self.data_ready);
                        }
                        if self.data_ready == 4 {
                            msg = msg3;
                        }
                        if self.data_ready == 5 {
                            msg = msg4;
                        }
                        if self.data_ready < 6 {
                            self.logger
                                .status
                                .entry("read".to_string())
                                .and_modify(|value| *value = vec![value[0].clone() + &msg])
                                .or_insert(vec![msg]);

                            log_plot(&mut self.logger);
                            println!("data{:?}", self.logger.data);
                            self.data_ready += 1;
                        }
                        self.logger.status.remove("read");
                    }
                    // self.cmd = CMDapp::UpdPrev;
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
                self.sersys = data.clone();
                if self.sersys.status.contains_key("read") {
                    // println!("fromSer: {:?}", self.sersys.status.get("read").unwrap());

                    self.logger
                        .status
                        .entry("read".to_string())
                        .and_modify(|value| {
                            *value =
                                vec![value[0].clone() + &self.sersys.status.get("read").unwrap()[0]]
                        })
                        .or_insert(vec![self.sersys.status.get("read").unwrap()[0].clone()]);

                    log_plot(&mut self.logger.clone());
                }
                if self.sersys.status.contains_key("start") {
                    println!("start");
                }
                if self.sersys.status.contains_key("end") {
                    println!("end");
                }
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
