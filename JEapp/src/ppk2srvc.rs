use crate::sersys::get_ppk2;

use egui::*;
use egui_plot::{Line, Plot, PlotPoints, *}; //Legend
use ppk2::{
    measurement::MeasurementMatch,
    types::{DevicePower, LogicPortPins, MeasurementMode, SourceVoltage},
    Ppk2,
};
use serde_derive::{Deserialize, Serialize};
use std::sync::{mpsc, Arc, RwLock};
use std::{
    collections::HashMap,
    sync::mpsc::RecvTimeoutError,
    thread,
    time::{Duration, Instant},
}; // PlotPoint,Polygon,Arrows,, Text
use tracing::error;

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize, PartialOrd)]
pub enum PPK2State {
    CREATED,
    DETECTED,
    IDLE,
    STOP,
    MONITOR,
    NOTFOUND,
    ERROR,
}
const BUFFSIZE: usize = 1200;

pub struct PPK2Srvc {
    pub state: Arc<std::sync::RwLock<PPK2State>>,
    pub port: String,
    pub data: Arc<RwLock<HashMap<String, Vec<f64>>>>,
    pub buffsize: Arc<RwLock<u32>>,
    pub arrix: u32,
    pub v_out: Arc<RwLock<u16>>,
    threads: Vec<thread::JoinHandle<()>>,
    ppkdetected: bool,
}
impl PPK2Srvc {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(PPK2State::CREATED)),
            port: "".to_string(),
            data: Arc::new(RwLock::new(HashMap::new())),
            buffsize: Arc::new(RwLock::new(BUFFSIZE as u32)),
            arrix: 0,
            threads: Vec::new(),
            v_out: Arc::new(RwLock::new(3600)),
            ppkdetected: false,
        }
    }

    pub fn init_ppk2(&mut self) -> bool {
        if *self.state.read().unwrap() != PPK2State::MONITOR {
            self.stop_monitor();
        }
        if self.threads.is_empty() {
            if let Some(port) = get_ppk2() {
                self.port = port;
                *self.state.write().unwrap() = PPK2State::IDLE;
                log::debug!("[SRVC] PPK2 Detected @ {}", self.port);
                self.ppkdetected = true;
                true
            } else {
                self.port = "".to_string();
                *self.state.write().unwrap() = PPK2State::NOTFOUND;
                log::debug!("[SRVC] PPK2 Not Detected ");
                false
            }
        } else {
            true
        }
    }

    pub fn start_monitor(&mut self, msgtx: mpsc::Sender<(String, String, String)>) -> bool {
        if *self.state.read().unwrap() != PPK2State::STOP {
            self.stop_monitor();
        }
        let map_c = Arc::clone(&self.data);
        let state_c = Arc::clone(&self.state);
        let buff_sz = Arc::clone(&self.buffsize);
        let sel_port = self.port.clone();
        let builder = thread::Builder::new();
        let vout = Arc::clone(&self.v_out.clone());

        self.data.write().unwrap().clear();
        self.threads.push(
            builder
                .spawn(move || monitor(sel_port, state_c, map_c, buff_sz, vout, msgtx))
                .unwrap(),
        );
        // if let Ok(rt) = tokio::runtime::Runtime::new() {
        //     let mut launched = false;

        //     let res = rt.block_on(checkppk2(Arc::clone(&self.state)));

        //     if !launched {
        //         println!("PPK2 Error");
        //         let mut ppkthread = self.threads.drain(..);
        //         if let Ok(thres) = ppkthread.next().unwrap().join() {
        //             println!("thread closed");
        //             return false;
        //         }
        //     }
        // }
        true
    }
    pub fn stop_monitor(&mut self) {
        *self.state.write().unwrap() = PPK2State::STOP;
        let mut ix = 0;
        while ix < self.threads.len() && self.threads.len() > 0 {
            if self.threads[ix].is_finished() {
                log::debug!("[SRVC] PPK2 Thread Finished And Removed");
                self.threads.remove(ix);
            }
            ix += 1;
        }
    }

    pub fn ui_plotres(&mut self, ui: &mut Ui) {
        if !self.data.read().unwrap().is_empty() {
            ui.horizontal(|ui| {
                if ui.button("1s").clicked() {
                    *self.buffsize.write().unwrap() = 500;
                }
                if ui.button("5s").clicked() {
                    *self.buffsize.write().unwrap() = 2500;
                }
                if ui.button("10s").clicked() {
                    *self.buffsize.write().unwrap() = 5000;
                }
                if ui.button("20s").clicked() {
                    *self.buffsize.write().unwrap() = 10000;
                }
                if ui.button("60s").clicked() {
                    *self.buffsize.write().unwrap() = 30000;
                }
                ui.add_space(20.0);
                ui.horizontal(|ui| {
                    ui.label("V Out:");
                    let vout = self.v_out.read().unwrap().clone();
                    let mut voutedit = vout.clone();
                    ui.add_sized(
                        [70.0, 20.0],
                        egui::DragValue::new(&mut voutedit).range(0..=5000),
                    );
                    if vout != voutedit {
                        *self.v_out.write().unwrap() = voutedit;
                    }
                });
            });
        }
    }
}

fn monitor(
    port: String,
    state: Arc<RwLock<PPK2State>>,
    map: Arc<RwLock<HashMap<String, Vec<f64>>>>,
    buff_sz: Arc<RwLock<u32>>,
    vout: Arc<RwLock<u16>>,
    msgtx: mpsc::Sender<(String, String, String)>,
) {
    let mut tries = 3;
    let mut monitoringthread: bool = true;
    let start = Instant::now();

    while tries > 0 {
        // Connect to PPK2 and initialize
        let mut ppk2 = if let Ok(out) = Ppk2::new(&port, MeasurementMode::Source) {
            tries = 0;
            *state.write().unwrap() = PPK2State::IDLE;
            out
        } else {
            tries -= 1;
            if tries == 0 {
                *state.write().unwrap() = PPK2State::ERROR;
                msgtx
                    .send((
                        "PPK2".into(),
                        "ERR".into(),
                        "PPK2 Failed to Start, Port might be busy?".to_string(),
                    ))
                    .unwrap();

                log::debug!("[SRVC] [ERR] PPK2 MODE ");
                break;
            }
            thread::sleep(std::time::Duration::from_millis(300));
            continue;
        };

        while monitoringthread {
            let v_set = vout.read().unwrap().clone();
            if ppk2
                .set_source_voltage(SourceVoltage::from_millivolts(v_set))
                .is_err()
            {
                log::debug!("\r\n[ERR] PPK2 Voltage ");
                *state.write().unwrap() = PPK2State::ERROR;
            }
            log::debug!("[SRVC] PPK Voltage out set {}", v_set);
            if ppk2.set_device_power(DevicePower::Enabled).is_err() {
                *state.write().unwrap() = PPK2State::ERROR;
                log::debug!("[ERR] PPK2 Power ");
            }
            if *state.read().unwrap() == PPK2State::ERROR {
                _ = ppk2.reset();
                log::debug!("[SRVC] PPK2 ERROR");
                return;
            }
            let mut samples = 1;

            ppk2 = if let Ok((rx, kill)) =
                ppk2.start_measurement_matching(LogicPortPins::default(), 3000)
            {
                // Set up sigkill handler.
                *state.write().unwrap() = PPK2State::MONITOR;
                msgtx
                    .send((
                        "PPK2".into(),
                        "INFO".into(),
                        "Monitoring Started".to_string(),
                    ))
                    .unwrap();
                log::debug!("[SRVC] PPK2 MONITOR Started!");
                // Receive measurements
                while samples > 0 {
                    if v_set != vout.read().unwrap().clone() {
                        msgtx
                            .send((
                                "PPK2".into(),
                                "INFO".into(),
                                format!("PPK2 Voltage changed {}", v_set).to_string(),
                            ))
                            .unwrap();
                        println!("\r\n[PPK2] Voltage changed ");
                        samples = 0;
                    }
                    if *state.read().unwrap() != PPK2State::MONITOR {
                        samples = 0;
                    }
                    let rcv_res = rx.recv_timeout(Duration::from_millis(2000));

                    use MeasurementMatch::*;
                    match rcv_res {
                        Ok(Match(m)) => {
                            let buffsize = buff_sz.read().unwrap().clone() as usize;
                            let mut datamap = map.write().unwrap();
                            datamap
                                .entry("Curr".to_string())
                                .or_insert_with(Vec::new)
                                .push((m.micro_amps) as f64);
                            let sample_time =
                                Instant::now().duration_since(start).as_micros() as f64;
                            datamap
                                .entry("Time".to_string())
                                .or_insert_with(Vec::new)
                                .push(sample_time / 1000.0);

                            for (_key, data) in datamap.iter_mut() {
                                if data.len() >= buffsize {
                                    data.drain(0..data.len() - (buffsize - 1));
                                }
                            }
                            // println!("Last chunk average: {:.4} μA", m.micro_amps);
                        }
                        Ok(NoMatch) => {
                            println!("No match in the last chunk of measurements");
                        }
                        Err(RecvTimeoutError::Disconnected) => {
                            msgtx
                                .send((
                                    "PPK2".into(),
                                    "ERR".into(),
                                    "PPK2 Disconnected".to_string(),
                                ))
                                .unwrap();
                            *state.write().unwrap() = PPK2State::ERROR;
                        }
                        Err(e) => {
                            *state.write().unwrap() = PPK2State::ERROR;
                            error!("Error receiving data: {e:?}");
                        }
                    }
                }
                kill().expect("ERROR")
                // if count > 100 {
                //     looplock = false;
                // }
            } else {
                return;
            };
            if *state.read().unwrap() != PPK2State::MONITOR {
                msgtx
                    .send((
                        "PPK2".into(),
                        "INFO".into(),
                        "PPK2 Monitoring Interrupted".to_string(),
                    ))
                    .unwrap();
                log::debug!("[SRVC] PPK2 Monitoring INTERRUPTED");
                monitoringthread = false;
            }
        }
        *state.write().unwrap() = PPK2State::STOP;
        msgtx
            .send((
                "PPK2".into(),
                "INFO".into(),
                "PPK2 Monitoring Stopped".to_string(),
            ))
            .unwrap();
        log::debug!("[SRVC] PPK2 Monitoring ENDED");

        // Start measuring.
        // let (rx, kill) = ppk2.start_measurement_matching(pins, 200)?;

        // Receive measurements
    }
}
pub fn ppk2_ui(
    ctx: &Context,
    ui: &mut Ui,
    data: &Box<HashMap<String, Vec<f64>>>,
    av_h: f32,
) -> bool {
    let w_height = if av_h < 250.0 { av_h } else { 250.0 };

    let w_width = 1000.0;
    let spacing = 20.0;
    let plotheight = w_height - spacing;
    let color = [
        Color32::LIGHT_BLUE,
        Color32::LIGHT_GREEN,
        Color32::LIGHT_GREEN,
        Color32::LIGHT_GREEN,
    ];
    let mut plotok = false;
    // ui.horizontal(|ui| {
    ui.add_space(30.0);

    if data.len() > 0 {
        ui.vertical(|ui| {
            // let x_axes = vec![AxisHints::default().label("sample")];
            // let nopoints = data.get("Curr").unwrap().len();
            // let x = if nopoints > 1000 {
            //     //average samples
            //     &data
            //         .get("Curr")
            //         .unwrap()
            //         .chunks(nopoints / 200)
            //         .map(|chunk| chunk.iter().sum::<f64>() / chunk.len() as f64)
            //         .collect::<Vec<f64>>()
            // } else {
            //     data.get("Curr").unwrap()
            // };

            let x = data.get("Curr").unwrap();
            //else {
            // };
            // let t = if nopoints > 1000 {
            //     &data
            //         .get("Time")
            //         .unwrap()
            //         .chunks(nopoints / 200)
            //         .map(|chunk| chunk.iter().sum::<f64>() / chunk.len() as f64)
            //         .collect::<Vec<f64>>()
            // } else {
            //     data.get("Time").unwrap()
            // };
            let t = data.get("Time").unwrap();
            let abs_max = x
                .iter()
                .max_by(|x, x2| x.abs().partial_cmp(&x2.abs()).unwrap())
                .unwrap();
            let mut scale = 1.0;
            let y_axes = if *abs_max > 1000.0 {
                scale = 0.001;
                vec![AxisHints::new_y()
                    .label("mA")
                    // .formatter(y_fmt)
                    .min_thickness(10.0)]
            } else {
                vec![AxisHints::new_y()
                    .label("uA")
                    // .formatter(y_fmt)
                    .min_thickness(10.0)]
            };
            let plot_lg = Plot::new(format!("pltlg"))
                .width(w_width * 0.8)
                .height(plotheight)
                .allow_zoom(true)
                .custom_y_axes(y_axes)
                .allow_drag(true)
                .allow_double_click_reset(true)
                .show_axes([true; 2])
                .include_y(0.0)
                //  .include_x(0.0)
                .show_background(false)
                //.legend(Legend::default())
                .coordinates_formatter(Corner::LeftBottom, CoordinatesFormatter::default());
            let mut ix = 0;

            // let _plotint =
            plot_lg.show(ui, |plot_ui| {
                let plt: PlotPoints = (0..x.len()).map(|i| [t[i], x[i] * scale]).collect();
                let planned_line = Line::new("trace", plt).color(color[ix]).width(0.4);
                plot_ui.line(planned_line.name("Current".to_string()));
                // plot_ui.set_plot_bounds(pltbounds);
                ix += 1;

                // plot_ui.text(Text::new(PlotPoint::new(10.0, 4.0), "angle"));
                //response
                // (
                //     plot_ui.pointer_coordinate(),
                //     plot_ui.pointer_coordinate_drag_delta(),
                //     plot_ui.plot_bounds(),
                //     plot_ui.plot_clicked(),
                // )
            });
        });
        ctx.request_repaint_after(Duration::from_millis(50));
        plotok = true;
    } else {
        ui.allocate_space(Vec2 {
            x: w_width,
            y: w_height,
        });
    }

    // });
    plotok
}

pub fn plot_metrics(ui: &mut Ui, data: &Box<HashMap<String, Vec<f64>>>) {
    ui.add_space(5.0);
    if data.contains_key("Curr") {
        let curr = data.get("Curr").unwrap();
        let abs_max = curr
            .iter()
            .max_by(|x, y| x.abs().partial_cmp(&y.abs()).unwrap())
            .unwrap();
        let abs_min = curr
            .iter()
            .min_by(|x, y| x.abs().partial_cmp(&y.abs()).unwrap())
            .unwrap();
        let avg = curr.iter().sum::<f64>() / curr.len() as f64;
        let mut scale = 1.0;
        let units = if *abs_max > 1000.0 {
            scale = 0.001;
            "mA".to_string()
        } else {
            "uA".to_string()
        };
        ui.vertical(|ui| {
            ui.label(RichText::new("Metrics:").font(FontId::proportional(14.0)));
            ui.horizontal(|ui| {
                ui.label(format!("Max: {:.2} {}", abs_max * scale, units));
                ui.add_space(10.0);
                ui.label(format!("Avg: {:.2} {}", avg * scale, units));
                ui.add_space(10.0);

                ui.label(format!("Min: {:.2} {}", abs_min * scale, units));
            });
        });
    } else {
        ui.allocate_space(Vec2 { x: 10.0, y: 25.0 });
    }
}

pub fn ppk2_low_power_detect(
    data: Box<HashMap<String, Vec<f64>>>,
    lowpowerlevel: f64,
) -> (bool, String) {
    let mut detect = false;
    let strout = if data.contains_key("Curr") {
        let curr = data.get("Curr").unwrap().into_iter().rev().take(100).rev();
        let abs_max = curr
            .clone()
            .max_by(|x, y| x.abs().partial_cmp(&y.abs()).unwrap())
            .unwrap();
        let abs_min = curr
            .clone()
            .min_by(|x, y| x.abs().partial_cmp(&y.abs()).unwrap())
            .unwrap();
        let avg = curr.clone().sum::<f64>() / (curr.len() as f64);
        let mut scale = 1.0;
        let units = if *abs_max > 1000.0 {
            scale = 0.001;
            "mA".to_string()
        } else {
            "uA".to_string()
        };
        if *abs_max > 0.0 {
            if avg < lowpowerlevel {
                detect = true;
            }
            format!(
                "Max: {:.2}{} Avg: {:.2}{}  Min: {:.2}{} ",
                abs_max * scale,
                units,
                avg * scale,
                units,
                abs_min * scale,
                units,
            )
        } else {
            "".to_string()
        }
    } else {
        "".to_string()
    };
    (detect, strout)
}
