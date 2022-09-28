use egui::widgets::plot::{Legend, Line, Plot, PlotPoints, Polygon};
//Arrows, Bar, BarChart, BoxElem, BoxPlot, BoxSpread, Corner, HLine,
//  MarkerShape,  PlotImage, PlotPoint,  Points, Text, VLine,  LineStyle,};

use egui::*;
pub mod data;
pub mod objects;
//pub use serial::SerialCtrl;
use crate::blesys;
/// We derive Deserialize/Serialize so we can persist app state on shutdown.
// #[derive(serde::Deserialize, serde::Serialize)]
// #[serde(default)]
// if we add new fields, give them default values when deserializing old state
//use macroquad::prelude::*;
use std::collections::HashMap;
use std::iter::Iterator;
use std::{thread, time::Duration};
pub struct RenderApp {
    // Example stuff:
    label: String,
    // this how you opt-out of serialization of a member
    // #[serde(skip)]
    ble: u8,
    //portoptions:Enum,
    data_ready: u8,
    picked_path: Option<String>,
    portlist: Vec<String>,
    port_sel: String,
    dataset: data::RawData,
    draw: u8,
    objectlist: Vec<objects::Obj3D>,
    // surflist: Vec<[[f64; 4]; 2]>,
    surflist: Vec<objects::Surf3D>,
    threads: Vec<thread::JoinHandle<()>>,
}

impl Default for RenderApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            ble: 0,
            picked_path: None,
            data_ready: 0,
            port_sel: "-".to_string(),
            portlist: vec![],
            dataset: data::RawData {
                diag: (HashMap::new()),
                dataf: (HashMap::new()),
            },
            draw: 0,
            objectlist: vec![],
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
        // if let Some(storage) = cc.storage {
        //     return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        // }

        Default::default()
    }
}

impl eframe::App for RenderApp {
    /// Called by the frame work to save state before shutdown.
    // fn save(&mut self, storage: &mut dyn eframe::Storage) {
    // eframe::set_value(storage, eframe::APP_KEY, self);
    // }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let Self {
            label,
            ble,
            picked_path,
            data_ready,
            dataset,
            portlist,
            port_sel,
            draw,
            objectlist,
            surflist,
            threads,
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
                            r: 2.0,
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
                        //self.draw = 1;
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
                            pos: [0.0, 0.0, 0.0],
                            r: 1.0,
                            alph: 0.0,
                            beta: 0.0,
                            gamm: 0.0,
                            points: [vec![], vec![]], //X Y points for render
                            scale: 1.0,
                            res: 100, //resolution
                        };

                        objects::draw_circle3d(&mut circle1);
                        self.objectlist.push(circle1);
                        self.draw = 1;
                    }
                });
                if ui.button("List COM…").clicked() {
                    listports(&mut self.portlist);
                    // dbg!();
                    // unreachable!();
                }
                ui.menu_button("BLE", |ui| {
                    if self.threads.len() > 0 {
                        while self.threads.len() > 0 {
                            let cur_thread = self.threads.remove(0); // moves it into cur_thread
                            cur_thread.join().unwrap();
                        }
                    }
                    if ui.button("Sync").clicked() {
                        let builder = thread::Builder::new();
                        self.threads
                            .push(builder.spawn(|| blesys::BLESys::run(1)).unwrap());
                    }
                    if ui.button("trig").clicked() {
                        let builder = thread::Builder::new();

                        self.threads
                            .push(builder.spawn(|| blesys::BLESys::run(2)).unwrap());
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
                let mut dataout: Vec<String> = vec![];
                if self.port_sel != "-" {
                    readserial(self.port_sel.clone(), 115200, &dataout);
                }
            }
            if ui.button("Send").clicked() {
                println!("{}", self.port_sel);
                let mut dataout: Vec<String> = vec![];
                if self.port_sel != "-" {
                    sendserial(self.port_sel.clone(), 115200, "r".to_string());
                }
                println!("read");
                std::thread::sleep(Duration::from_millis((1000.0) as u64));
                readserial(self.port_sel.clone(), 115200, &dataout);
            }

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
            if self.objectlist.len() > 0 || self.surflist.len() > 0 {
                let plot = Plot::new("preview")
                    .include_x(0.0)
                    .include_y(0.0)
                    .width(600.0)
                    .height(300.0)
                    .view_aspect(1.0)
                    .data_aspect(1.0)
                    .allow_scroll(false)
                    .allow_drag(false)
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

                            let planned_line =
                                Line::new(plt).color(Color32::from_rgb(100, 200, 100));
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
                        //   let rot:[f32;2]=[plot_ui.pointer_coordinate_drag_delta()[0],plot_ui.pointer_coordinate_drag_delta()[1] ];

                        // if rot[0] != 0.0{
                        //     self.objectlist[0].alph+=rot[0] as f64;
                        // }
                        // if rot[1] != 0.0{
                        //     self.objectlist[0].beta+=rot[1]as f64;
                        // }
                        // println!("coords{:?} ",plot_ui.pointer_coordinate_drag_delta()[0] );
                    }
                });
                //self.data_ready = 0;
            }

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

use serialport::{available_ports, DataBits, SerialPortType, StopBits};
use std::io::{self, Write};

fn listports(list: &mut Vec<String>) {
    let _ = &list.clear();
    match available_ports() {
        Ok(ports) => {
            match ports.len() {
                0 => println!("No ports found."),
                1 => println!("Found 1 port:"),
                n => println!("Found {} ports:", n),
            };
            for p in ports {
                println!("  {}", p.port_name);
                let _ = &list.push(p.port_name);
                match p.port_type {
                    SerialPortType::UsbPort(info) => {
                        println!("    Type: USB");
                        println!("    VID:{:04x} PID:{:04x}", info.vid, info.pid);
                        println!(
                            "     Serial Number: {}",
                            info.serial_number.as_ref().map_or("", String::as_str)
                        );
                        println!(
                            "      Manufacturer: {}",
                            info.manufacturer.as_ref().map_or("", String::as_str)
                        );
                        println!(
                            "           Product: {}",
                            info.product.as_ref().map_or("", String::as_str)
                        );
                        // println!(
                        //     "         Interface: {}",
                        //     info.interface
                        //         .as_ref()
                        //         .map_or("".to_string(), |x| format!("{:02x}", *x))
                        // );
                    }
                    SerialPortType::BluetoothPort => {
                        println!("    Type: Bluetooth");
                    }
                    SerialPortType::PciPort => {
                        println!("    Type: PCI");
                    }
                    SerialPortType::Unknown => {
                        println!("    Type: Unknown");
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("{:?}", e);
            eprintln!("Error listing serial ports");
        }
    }
}

fn readserial(port_name: String, baud_rate: u32, dataout: &Vec<String>) {
    let prt = port_name.clone();
    let port = serialport::new(prt, baud_rate)
        .timeout(Duration::from_millis(10))
        .open();

    match port {
        Ok(mut port) => {
            let mut serial_buf: Vec<u8> = vec![0; 1000];
            println!("Receiving data on {} at {} baud:", port_name, baud_rate);

            match port.read(serial_buf.as_mut_slice()) {
                Ok(t) => io::stdout().write_all(&serial_buf[..t]).unwrap(),
                Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                Err(e) => eprintln!("{:?}", e),
            }
        }
        Err(e) => {
            eprintln!("Failed to open \"{}\". Error: {}", &port_name, e);
            // ::std::process::exit(1);
        }
    }
}

fn sendserial(port_name: String, baud_rate: u32, string: String) {
    let stop_bits = StopBits::One;
    let data_bits = DataBits::Eight;

    let prt = port_name.clone();
    let builder = serialport::new(prt, baud_rate)
        .stop_bits(stop_bits)
        .data_bits(data_bits);

    println!("{:?}", &builder);
    let mut port = builder.open().unwrap_or_else(|e| {
        eprintln!("Failed to open \"{}\". Error: {}", port_name, e);
        ::std::process::exit(1);
    });

    println!(
        "Writing '{}' to {} at {} baud ",
        &string, &port_name, &baud_rate
    );

    match port.write(string.as_bytes()) {
        Ok(_) => {
            //print!("{}", &string);
            std::io::stdout().flush().unwrap();
        }
        Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
        Err(e) => eprintln!("{:?}", e),
    }

    //std::thread::sleep(Duration::from_millis((1000.0 / (rate as f32)) as u64));
}
