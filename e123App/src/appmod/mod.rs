// use egui::widgets::plot::{Arrows, Legend, Line, Plot, PlotPoint, PlotPoints, Polygon, Text};
// Bar, BarChart, BoxElem, BoxPlot, BoxSpread, Corner, HLine,
//  MarkerShape,  PlotImage,  Points, Text, VLine,  LineStyle,};
// use core::f64::consts::PI;
use egui::*;
pub mod data;
pub mod objects;
use crate::datalogger::{ log_process, LoggerCtrl}; //, LoggerState
const VERSION: &str = "LightGate v0.01";
use crate::sersys::{self, listports, SerState, SerSys};
/// We derive Deserialize/Serialize so we can persist app state on shutdown.
use device_query::{DeviceQuery, DeviceState, Keycode};
// use num::signum;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
// use std::iter::Iterator;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::{
    thread,
    time::Duration, //Instant
};

pub struct Mesagging {
    ser_ch: (Sender<SerSys>, Receiver<SerSys>),
}

#[derive(PartialEq, Serialize, Deserialize)]
enum CMDapp {
    Idle,
    UpdPrev,
    Sermsg,
}
#[derive(serde::Deserialize, serde::Serialize)]
pub struct AppsOpen {
    logger: bool,
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
    datain: Arc<RwLock<String>>,
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
    logger: LoggerCtrl,
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
            apps: AppsOpen { logger: true },
            timer: Duration::new(0, 0),
            data_ready: 0,
            datain: Arc::new(RwLock::new("".to_string())),
            device_state: DeviceState::new(),
            sersys: SerSys::default(),
            port_sel: "-".to_string(),
            portlist: vec![],
            dataset: data::RawData {
                diag: (HashMap::new()),
                dataf: (HashMap::new()),
            },
            logger: LoggerCtrl::default(),
            msgs: Mesagging {
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
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        let Self {
            label: _,
            timer: _,
            sersys: _,
            logger: _,
            apps: _,
            device_state: _,
            msgs: _,
            picked_path: _,
            data_ready: _,
            dataset: _,
            datain: _,
            cmd: _,
            portlist: _,
            port_sel: _,
            anim_state: _,
            draw: _,
            objectlist: _,
            objstate: _,
            surflist: _,
            threads: _,
        } = self;

        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        //

        #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        _frame.close();
                    }
                });

                // if ui.button("List COM…").clicked() {
                if self.sersys.state == SerState::CREATED {
                    self.sersys.portlist = listports();
                    if self.sersys.portlist.len() == 1 {
                        self.port_sel = self.sersys.portlist[0].num.clone();
                    }
                    // let (tx_ser, rx_ser): (Sender<SerSys>, Receiver<SerSys>) =
                    //     mpsc::channel::<SerSys>();
                    // let (tx_a, rx_a): (Sender<SerSys>, Receiver<SerSys>) =
                    //     mpsc::channel::<SerSys>();
                    // self.msgs.ser_ch.0 = tx_a;
                    // self.msgs.ser_ch.1 = rx_ser;
                    // //self keeps tx_a and rx_w
                    // let builder = thread::Builder::new();
                    // self.threads.push(
                    //     builder
                    //         .spawn(move || SerSys::startserv(tx_ser, rx_a))
                    //         .unwrap(),
                    // );
                    // self.cmd = CMDapp::Sermsg;
                    self.sersys.state = SerState::IDLE
                }
                // }
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
                    let (tx_ser, rx_ser): (Sender<SerSys>, Receiver<SerSys>) =
                        mpsc::channel::<SerSys>();
                    let (tx_a, rx_a): (Sender<SerSys>, Receiver<SerSys>) =
                        mpsc::channel::<SerSys>();
                    self.msgs.ser_ch.0 = tx_a;
                    self.msgs.ser_ch.1 = rx_ser;
                    println!("{}", self.port_sel);

                    self.sersys
                        .status
                        .insert("sel".to_string(), vec![self.port_sel.to_string()]);

                    self.sersys
                        .status
                        .insert("write".to_string(), vec!["r".to_string()]);
                    sersys::sendserial(&mut self.sersys);
                    let datain_c = Arc::clone(&self.datain);
                    let builder = thread::Builder::new();
                    self.threads.push(
                        builder
                            .spawn(move || sersys::arcreadserial(datain_c, tx_ser, rx_a))
                            .unwrap(),
                    );
                    self.sersys
                        .status
                        .insert("START".to_string(), vec!["".to_string()]);
                    //self keeps tx_a and rx_w
                    // let builder = thread::Builder::new();
                    // self.threads.push(
                    //     builder
                    //         .spawn(move || SerSys::startserv(tx_ser, rx_a))
                    //         .unwrap(),
                    // );

                    // sersys::arcreadserial(&self.sersys, self.datain.clone(), self.ser_state);
                    // if self.port_sel != "-" {
                    //     self.sersys
                    //         .status
                    //         .insert("sel".to_string(), vec![self.port_sel.clone()]);
                    //     self.sersys.state = SerState::MONITOR;
                    self.cmd = CMDapp::Sermsg;
                    // }
                    self.sersys.state = SerState::MONITOR;
                }

                if ui.button("Clear").clicked() {
                    self.logger.data.clear();

                    // self.logger.status

                    self.cmd = CMDapp::Sermsg;
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
                        ui.spacing_mut().item_spacing.x = 1.0;
                        ui.colored_label(Color32::LIGHT_GRAY, VERSION);
                        ui.label(" by ");
                        ui.hyperlink_to("Everything123", "http://everything123.co.uk/");
                    });
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("LightGate");
            // egui::Window::new("🔧 Settings")
            //     // .auto_sized()
            //     .anchor(Align2::LEFT_TOP, [2.0, 2.0])
            //     .vscroll(true)
            //     .open(&mut self.apps.logger)
            //     .show(ctx, |ui| {
            // crate::datalogger::log_gui(ctx, ui, &mut self.logger);
            // });
            crate::datalogger::log_gui(ctx, ui, &mut self.logger);

        });
        if  self.sersys.state==SerState::MONITOR {
            ctx.request_repaint();
        }
    }

    fn post_rendering(&mut self, _window_size_px: [u32; 2], _frame: &eframe::Frame) {
        let keys: Vec<Keycode> = self.device_state.get_keys();
        for key in keys.iter() {
            match key {
                Keycode::A => {

                    // self.cmd = CMDapp::UpdPrev;
                }
                Keycode::D => {

                    // self.cmd = CMDapp::UpdPrev;
                }
                Keycode::W => {
                    self.cmd = CMDapp::UpdPrev;
                }
                Keycode::S => {
                    // println!("{:?}", self.sersys);
                    // if self.sersys.state == SerState::IDLE {
                    // }
                    // self.cmd = CMDapp::UpdPrev;
                }
                // Keycode::Escape => todo!(),
                // Keycode::Space => todo!(),
                // Keycode::Enter => todo!(),
                _ => println!("Pressed key: {:?}", key),
            }
        }
       
    
        match self.msgs.ser_ch.1.try_recv() {
            Ok(msg) => {
                // self.sersys = data.clone();
                // if self.sersys.status.contains_key("read") {
                //   println!("fromSer: {:?}", self.sersys.status.get("read").unwrap());

                // self.logger
                //     .status
                //     .entry("read".to_string())
                //     .and_modify(|value| {
                //         *value =
                //             vec![value[0].clone() + &self.sersys.status.get("read").unwrap()[0]]
                //     })
                //     .or_insert(vec![self.sersys.status.get("read").unwrap()[0].clone()]);

                // log_plot(&mut self.logger);
                // }
                if msg.status.contains_key("dataav") {
                    //   println!("fromSer: {:?}", self.sersys.status.get("read").unwrap());
                    // println!("\nfromSer: {:?}", r2);
                    // println!("\nfromSer: ");
                    let r2 = self.datain.read().unwrap();
                    self.logger.raw = (*r2).to_string();
                    if self.logger.raw.len() > 0 {
                        log_process(&mut self.logger);
                    }
                    self.sersys.state = SerState::MONITOR;
                    self.sersys
                        .status
                        .insert("ack".to_string(), vec!["".to_string()]);
                    self.cmd = CMDapp::Sermsg;
                }
                if msg.status.contains_key("START") {
                    println!("start");
                }
                if msg.status.contains_key("STOP") {
                    self.sersys.state = SerState::IDLE;
                    println!("end");
                }
         
                // if self.sersys.status.contains_key("list") {
                //     self.portlist = self.sersys.status.get("list").unwrap().to_vec()
                // }
            }
            Err(_) => { /* handle sender disconnected */ } //Err(TryRecvError::Empty) => { /* handle no data available yet */ }
        }
        match self.cmd {
            CMDapp::Sermsg => {
                if let Err(_) = self.msgs.ser_ch.0.send(self.sersys.clone()) {
                    println!("Ser has stopped listening.")
                }
                self.sersys.status.clear();
                //  thread::sleep(Duration::from_millis(200));
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
