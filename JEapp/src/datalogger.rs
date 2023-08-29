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
use xlsxwriter::prelude::*;

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum LoggerState {
    CREATED,
    RUNNING,
    IDLE, //logged in
    KILL,
    RESET,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggerCtrl {
    pub state: LoggerState,
    #[serde(skip)]
    pub status: HashMap<String, Vec<String>>,
    pub plt: HashMap<String, Vec<f64>>,
    pub cfg: HashMap<String, f64>,

    pub data: HashMap<String, Vec<f64>>,
    #[serde(skip)]
    pub anim_state: objects::ObjAnim,
    #[serde(skip)]
    objstate: HashMap<String, HashMap<String, f64>>,
    #[serde(skip)]
    objectlist: Vec<objects::Obj3D>,
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
            objectlist: vec![],
            objstate: HashMap::new(),
            data: HashMap::new(),
            cfg: HashMap::from([
                ("w".to_string(), 15.0),
                ("h".to_string(), 0.10),
                ("a".to_string(), 0.00),
            ]),
        }
    }
}

use core::f64::consts::PI;
use num::signum;

pub fn log_plot(logctrl: &mut LoggerCtrl) {
    // let dt = 0.1;
    // let mut obj = logctrl.objstate.get("rbb").unwrap().clone();
    // let mut data = logctrl.data.clone();
    let mut lightgate = false;
    // let x_sp = 3.0 * signum(((logctrl.anim_state.step as f64) * PI / 50.0).cos()); //square
    if logctrl.status.contains_key("read") {
        let mut raw: String = logctrl.status.get("read").unwrap()[0].clone();
        // println!("raw:{:?}", raw);
        if raw.contains("STOPLOG") {
            raw = raw.split("STOPLOG").collect();
            // println!("rawSTOP:{:?}", raw);
        }
        if raw.contains("STARTLOG") {
            raw = raw.split("STARTLOG").collect::<Vec<&str>>()[1].to_string();
        }
        if raw.contains("LG") {
            lightgate = true;
        }

        for row in raw.split("\r\n") {
            if row.contains("\n\r") {
                continue;
            }

            let rowd = row.split(",").collect::<Vec<&str>>();
            println!("{:?}", rowd);
            if rowd.len() >= 3 {
                let val = rowd[0].split(" ").collect::<Vec<&str>>()[1]
                    .to_string()
                    .parse::<f64>()
                    .unwrap()
                    / 1000.0;
                // print!("{}t{:?} : ", rowd[1], val);
                logctrl
                    .data
                    .entry(format!("t{}", rowd[1]))
                    .or_insert_with(Vec::new)
                    .push(val);
                // .and_modify(|value| value = value.push(val))
                // .or_insert(vec![val]);
                let num = rowd[2].parse::<f64>();
                match num {
                    Ok(val) => {
                        println!("{}", val);
                        logctrl
                            .data
                            .entry(format!("{}", rowd[1]))
                            .or_insert_with(Vec::new)
                            .push(val);
                    }
                    Err(_) => {}
                }
            }
            if rowd.len() == 4 {
                if row.contains("LW") {
                    let val = rowd[3].parse::<f64>().unwrap();
                    logctrl
                        .data
                        .entry(format!("V{}", rowd[1]))
                        .or_insert_with(Vec::new)
                        .push(val);
                }
            }
        }
    }
}

pub fn log_gui(ctx: &Context, ui: &mut Ui, logctrl: &mut LoggerCtrl) {
    ui.separator();
    if ui.button("Save LOGS").clicked() {
        write_xlsx(logctrl.clone());
    }
    // if ui.button("Open Satfile…").clicked() {
    //     //  let filename ="./11.17.17Device 3.xlsx".to_string();
    //     // data::processdata(filename, &mut self.dataset);
    //     if let Some(path) = rfd::FileDialog::new().pick_file() {
    //         cncctrl.picked_path = path.display().to_string();
    //         processdata(path.display().to_string(), &mut cncctrl.dataset)
    //     }
    // }
    ui.separator();
    let data = logctrl.data.clone();

    let w_height = ui.available_height();
    let w_width = ui.available_width();
    for key in data.keys() {
        if key.chars().nth(0).unwrap() != 't' {
            let plot = Plot::new(format!("plt{}", key))
                .width(w_width * 0.8)
                .height(w_height / 6.0)
                .auto_bounds_x()
                .auto_bounds_y()
                .allow_zoom(true)
                .allow_drag(true)
                .show_axes([true; 2])
                .show_background(false)
                .legend(Legend::default());

            plot.show(ui, |plot_ui| {
                if key.contains("LG1") {
                    let t = data.get("tLG1").unwrap();
                    let x = data.get("LG1").unwrap();
                    let plt: PlotPoints = (0..x.len()).map(|i| [t[i], x[i]]).collect();

                    let planned_line = Line::new(plt).color(Color32::from_rgb(150, 255, 150));
                    plot_ui.line(planned_line.name("%Obscuration"));
                } else {
                    let t = data.get(&format!("t{}", key)).unwrap();
                    let x = data.get(key).unwrap();
                    let plt: PlotPoints = (0..x.len()).map(|i| [t[i], x[i]]).collect();

                    let planned_line = Line::new(plt).color(Color32::from_rgb(255, 50, 50));
                    plot_ui.line(planned_line.name(key));
                }
            });

            ui.separator();
        }
    }

    if logctrl.anim_state.state == 1 {
        if logctrl.objstate.contains_key("read") {
            ctx.request_repaint();
            logctrl.anim_state.step += 1;
        } else {
            logctrl.anim_state.state = 0;
        }
        if logctrl.anim_state.step >= logctrl.anim_state.steps {
            logctrl.anim_state.state = 0;
        }
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
