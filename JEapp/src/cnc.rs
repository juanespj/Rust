use crate::appmod::data::processdata;
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
pub enum CNCState {
    CREATED,
    RUNNING,
    IDLE, //logged in
    KILL,
    RESET,
}

#[derive(Serialize, Deserialize)]
pub struct CNCCtrl {
    pub state: CNCState,
    #[serde(skip)]
    picked_path: String,
    pub status: HashMap<String, Vec<String>>,
    pub plt: HashMap<String, Vec<f64>>,
    pub cfg: HashMap<String, f64>,
    draw: u8,
    #[serde(skip)]
    surflist: Vec<objects::Surf3D>,
    pub data: HashMap<String, Vec<f64>>,
    #[serde(skip)]
    pub anim_state: objects::ObjAnim,
    #[serde(skip)]
    objstate: HashMap<String, HashMap<String, f64>>,
    #[serde(skip)]
    objectlist: Vec<objects::Obj3D>,
}

impl Default for CNCCtrl {
    fn default() -> Self {
        Self {
            picked_path: "".to_string(),
            state: CNCState::CREATED,
            status: HashMap::new(),
            plt: HashMap::new(),
            draw: 0,
            anim_state: objects::ObjAnim {
                steps: 0,
                step: 0,
                state: 0,
            },
            objectlist: vec![],
            surflist: vec![],
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

pub fn main_gui(ctx: &Context, ui: &mut Ui, cncctrl: &mut CNCCtrl) {
    if ui.button("Open OGF…").clicked() {
        let filename = "./4014iso".to_string();
        process_ogf(filename);

        // if let Some(path) = rfd::FileDialog::new().pick_file() {
        //     self.picked_path = Some(path.display().to_string());
        //    data::processdata(path.display().to_string())
    }
    if ui.button("Open RPF…").clicked() {
        let filename = "./probe.txt".to_string();
        let mut mesh = objects::Surf3D {
            pos: [0.0, 0.0, 0.0],
            param: HashMap::from([("r".to_string(), 2.0)]),
            alph: 0.9,
            beta: 0.5,
            gamm: 0.5,
            points_raw: [vec![], vec![], vec![]],
            points: vec![], //X Y points for render
            scale: 10.0,
            res: 100, //resolution
        };
        // let mut meshRAW: [Vec<f64>; 3] = [vec![], vec![], vec![]];
        process_raw_probe_file(filename, &mut mesh.points_raw);
        // println!("mesh{:?} ", mesh.points_raw);
        // objects::draw_3dmesh(&mut meshRAW, &mut mesh);
        objects::draw_3dmesh_surf(&mut mesh);

        cncctrl.surflist.push(mesh);
      
        // if let Some(path) = rfd::FileDialog::new().pick_file() {
        //     self.picked_path = Some(path.display().to_string());
        //    data::processdata(path.display().to_string())
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
        let mut trap = objects::Obj3D {
            tag: "trap".to_string(),
            pos: [0.0, 0.0, 0.0],
            param: HashMap::from([
                ("h".to_string(), 3.0),
                ("w".to_string(), 5.0),
                ("a".to_string(), 0.0),
                ("s".to_string(), 0.0),
            ]),
            alph: 0.0,
            beta: 0.0,
            gamm: 0.0,
            points: [vec![], vec![]], //X Y points for render
            scale: 1.0,
            res: 100, //resolution
            color: [250, 100, 50],
        };
        objects::draw_trap(&mut trap);
        objects::draw_circle3d(&mut circle1);
        cncctrl.objectlist.push(circle1);
        cncctrl.draw = 1;
    }
    ui.separator();
    ui.heading("Preview");
    ui.horizontal(|ui| {
        if cncctrl.objectlist.len() > 0 || cncctrl.surflist.len() > 0 {
            let plot = Plot::new("preview")
                // .include_x(0.0)
                // .include_y(0.0)
                .width(200.0)
                .auto_bounds_x()
                .auto_bounds_y()
                .height(200.0)
                .view_aspect(1.0)
                .data_aspect(1.0)
                .allow_zoom(true)
                .allow_drag(true)
                // .show_axes([true; 2])
                // .show_background(false)
                .legend(Legend::default());

            plot.show(ui, |plot_ui| {
                if cncctrl.objectlist.len() > 0 {
                    for obj in cncctrl.objectlist.iter() {
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
                if cncctrl.surflist.len() > 0 {
                    let rot: [f64; 2] = [
                        plot_ui.pointer_coordinate_drag_delta()[0] as f64,
                        plot_ui.pointer_coordinate_drag_delta()[1] as f64,
                    ];
                    if rot[0] != 0.0 || rot[1] != 0.0 {
                        let mut i = 0;
                        while i < cncctrl.surflist.len() {
                            cncctrl.surflist[i].alph = rot[0] * 0.5 + cncctrl.surflist[i].alph;
                            cncctrl.surflist[i].beta = rot[1] * 0.5 + cncctrl.surflist[i].beta;
                            objects::draw_3dmesh_surf(&mut cncctrl.surflist[i]);
                            i += 1;
                        }
                    }
                    for obj in cncctrl.surflist.iter() {
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
            // self.data_ready = 0;
            
        }
    });
}

use std::fs;
// use calamine::{open_workbook, DataType, Reader, Xlsx};
pub fn process_ogf(name: String) {
    //
    let contents = fs::read_to_string(name).expect("Should have been able to read the file");
    let header: Vec<String> = vec![];
    let value: Vec<String> = vec![];
    println!("{:?}", contents);
}

pub fn process_raw_probe_file(filename: String, data: &mut [Vec<f64>; 3]) {
    println!("{:?}", filename);

    let contents = fs::read_to_string(filename).expect("Should have been able to read the file");
    data[0].clear();
    data[1].clear();
    data[2].clear();
    for line in contents.lines() {
        let mut triplet: Vec<f64> = Vec::new();

        for var in line.split(",") {
            let f: f64 = match var.parse() {
                Ok(v) => v,
                Err(_) => 0.0, // or whatever error handling
            };
            triplet.push(f);
        }
        data[0].push(triplet[0]);
        data[1].push(triplet[1]);
        data[2].push(triplet[2]);
    }
    //print!("{:?}", data[2]);
    // rendermesh(x, y, z);
}
