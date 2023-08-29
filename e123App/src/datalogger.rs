use crate::appmod::objects;
use chrono::{NaiveDate, Timelike, Utc};
use egui::widgets::plot::{Arrows, Legend, Line, Plot, PlotPoint, PlotPoints, Polygon, Text};
use egui::Color32;
use egui::*;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc};
use std::{
    error::Error,
    str,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use strp::*;
use xlsxwriter::prelude::*;

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum LoggerState {
    CREATED,
    RUNNING,
    IDLE,
    MONTORING,
    KILL,
    RESET,
}

#[derive(Debug, Deserialize)]
struct Record {
    field: String,
    times: u32,
    data1: Option<u64>,
    deb1: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggerCtrl {
    pub state: LoggerState,
    #[serde(skip)]
    pub status: HashMap<String, Vec<String>>,
    #[serde(skip)]
    pub plt: HashMap<String, Vec<f64>>,
    pub cfg: HashMap<String, f64>,
    #[serde(skip)]
    pub data: HashMap<String, Vec<f64>>,
    #[serde(skip)]
    pub anim_state: objects::ObjAnim,
    // #[serde(skip)]
    // objstate: HashMap<String, HashMap<String, f64>>,
    // #[serde(skip)]
    // objectlist: Vec<objects::Obj3D>,
}

impl Default for LoggerCtrl {
    fn default() -> Self {
        Self {
            state: LoggerState::CREATED,
            status: HashMap::new(),
            plt: HashMap::new(),
            anim_state: objects::ObjAnim {
                steps: 0,
                step: 0,
                state: 0,
            },
            // objectlist: vec![],
            // objstate: HashMap::new(),
            data: HashMap::new(),
            cfg: HashMap::from([
                ("w".to_string(), 15.0),
                ("h".to_string(), 0.10),
                ("a".to_string(), 0.00),
            ]),
        }
    }
}

// use core::f64::consts::PI;
// use num::signum;

pub fn log_plot(logctrl: &mut LoggerCtrl) {
    // let dt = 0.1;
    // let mut obj = logctrl.objstate.get("rbb").unwrap().clone();
    // let mut data = logctrl.data.clone();
    let mut lightgate = false;
    // let x_sp = 3.0 * signum(((logctrl.anim_state.step as f64) * PI / 50.0).cos()); //square
    if logctrl.status.contains_key("read") {
        let mut buff = "".to_string();
        if logctrl.status.contains_key("buffer") {
            buff = logctrl.status.get("buffer").unwrap()[0].clone();
            println!("buffer:{:?}", buff);
            logctrl.status.remove_entry("buffer");
        }
        //  buff +&
        let mut raw: String = logctrl.status.get("read").unwrap()[0].clone();

        // println!("raw:{:?}", raw);
        if raw.contains("STOPLOG") {
            raw = raw.split("STOPLOG").collect();
            logctrl.state = LoggerState::IDLE;
            // println!("rawSTOP:{:?}", raw);
        }
        if raw.contains("STARTLOG") {
            logctrl.state = LoggerState::MONTORING;

            raw = raw.split("STARTLOG").collect::<Vec<&str>>()[1].to_string();
        }
        // if raw.contains("LG") {
        //     lightgate = true;
        // }

        logctrl.status.remove_entry("read");

        // let mut rdr = csv::Reader::from_reader(raw.as_bytes());
        // for result in rdr.deserialize(Record) {
        //     match result {
        //         Ok(record) => println!("{:?}", record),
        //         Err(_) => { /* handle sender disconnected */ }
        //     }
        //     //   let record: Record = result?;
        //
        // }
        //println!("{:?}", raw);
        logctrl.data.clear();
        // if raw.matches("\r\n").count() > 1 {
        //     let last = raw.split("\r\n").last().unwrap();
        //     if last.len() > 0 {
        //         if !matches!(last.chars().last().unwrap(), '<') {
        //             logctrl
        //                 .status
        //                 .entry("buffer".to_string())
        //                 .or_insert_with(Vec::new)
        //                 .push(last.to_string());
        //         }
        //     }
        // }
        let cnt = raw.matches("\r\n").count();
        let mut i = 0;
        for row in raw.split("\r\n") {
            if row.len() > 0 {
                let cnt = row.matches(',').count();
                if cnt > 2 || row.chars().last().unwrap() != '<' {
                    println!("err:{:?}", row);
                    continue;
                }
                let matched: Result<(String, u32, f32), _> = try_scan!(row =>"{},{},{}<");
                match matched {
                    Ok((var, time, val)) => {
                        //  println!("D:{} => {}", var, val);

                        if (var == "PH" || var == "W") && val <= 1500.0 {
                            let tkey = format!("t{}", var);
                            let mut tout = time;
                            if logctrl.data.contains_key(&tkey) {
                                let last = logctrl.data.get(&tkey).unwrap().len() - 1;
                                let tlast = logctrl.data.get(&tkey).unwrap()[last].clone();

                                if tout < (tlast * 1000.0) as u32 {
                                    tout = (tlast * 1000.0) as u32 + 5;
                                }
                            }
                            logctrl
                                .data
                                .entry(tkey)
                                .or_insert_with(Vec::new)
                                .push(tout as f64 / 1000.0);
                            logctrl
                                .data
                                .entry(format!("{}", var))
                                .or_insert_with(Vec::new)
                                .push(val as f64);
                        }
                    }
                    Err(_e) => {
                        if i == cnt {
                            logctrl
                                .status
                                .entry("buffer".to_string())
                                .or_insert_with(Vec::new)
                                .push(row.to_string());
                        }
                    }
                }
            }
            i += 1;
            // assert_eq!(a + b, c);d
        }
    }
}

pub fn log_gui(ctx: &Context, ui: &mut Ui, logctrl: &mut LoggerCtrl) {
    ui.separator();
    if ui.button("Save LOGS").clicked() {
        write_xlsx(logctrl.clone());
    }
    if ui.button("show data").clicked() {
        println!("{:?}", &logctrl.data);
    }
    ui.separator();
    let data = logctrl.data.clone();
    let nokeys = data.keys().len() / 2;
    let w_height = ui.available_height();
    let w_width = ui.available_width();

    for key in data.keys() {
        if key.chars().nth(0).unwrap() != 't' {
            let plot = Plot::new(format!("plt{}", key))
                .width(w_width * 0.8)
                .height(w_height / nokeys as f32)
                .auto_bounds_x()
                .auto_bounds_y()
                .allow_zoom(true)
                .allow_drag(true)
                .show_axes([true; 2])
                .show_background(false)
                .legend(Legend::default());

            plot.show(ui, |plot_ui| {
                // if key.contains("LG1") {
                //     let t = data.get("tLG1").unwrap();
                //     let x = data.get("LG1").unwrap();
                //     let plt: PlotPoints = (0..x.len()).map(|i| [t[i], x[i]]).collect();

                //     let planned_line = Line::new(plt).color(Color32::from_rgb(150, 255, 150));
                //     plot_ui.line(planned_line.name("%Obscuration"));
                // } else {
                let t = data.get(&format!("t{}", key)).unwrap();
                let x = data.get(key).unwrap();
                let plt: PlotPoints = (0..x.len()).map(|i| [t[i], x[i]]).collect();
                let planned_line = Line::new(plt).color(Color32::from_rgb(255, 50, 50));
                plot_ui.line(planned_line.name(key));
                // }
            });

            ui.separator();
        }
    }

    if logctrl.state == LoggerState::MONTORING {
        ctx.request_repaint_after(std::time::Duration::from_millis(500));
    }
}

pub fn write_xlsx(sys: LoggerCtrl) -> bool {
    let today = Utc::now().date_naive();
    let time = Utc::now().time();
    let filename = format!(
        "{}_h{}m{}s{}_log.xlsx",
        today,
        time.hour(),
        time.minute(),
        time.second()
    );
    let mut success = false;
    match Workbook::new(&filename) {
        Ok(workbook) => match workbook.add_worksheet(None) {
            Ok(mut sheet1) => {
                let mut cur_row = 0;

                let mut cur_col = 0;
                for item in sys.data.keys() {
                    let mut ix = 0;
                    let _errchk = sheet1.write_string(
                        cur_row,
                        cur_col,
                        &item,
                        Some(&Format::new().set_font_color(FormatColor::Black)),
                    );
                    ix += 1;

                    for item in sys.data.get(item).unwrap() {
                        write_num_cell(&mut sheet1, cur_row + ix, cur_col, item.clone());
                        ix += 1;
                    }
                    cur_col += 1;
                }

                success = true;
                let _errchk = workbook.close();
            }
            Err(_) => {
                println!("Error saving!");
            }
        },
        Err(_) => {
            println!("Error saving!")
        }
    }
    success
}

fn write_str_cell(sheet: &mut Worksheet<'_>, row: u32, col: u16, data: &String) {
    let _errchk = sheet.write_string(
        row,
        col,
        data,
        Some(&Format::new().set_font_color(FormatColor::Black)),
    );
}
fn write_num_cell(sheet: &mut Worksheet<'_>, row: u32, col: u16, data: f64) {
    let _errchk = sheet.write_number(
        row,
        col,
        data,
        Some(&Format::new().set_font_color(FormatColor::Black)),
    );
}
fn write_bool_cell(sheet: &mut Worksheet<'_>, row: u32, col: u16, data: bool) {
    let _errchk = sheet.write_boolean(
        row,
        col,
        data,
        Some(&Format::new().set_font_color(FormatColor::Black)),
    );
}
