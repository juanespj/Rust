use core::f64::consts::PI;
use egui::widgets::plot::{Arrows, Legend, Line, Plot, PlotPoint, PlotPoints, Polygon, Text};
use egui::Color32;
use egui::*;
use num::signum;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::ffi::OsStr;
use std::path::Path;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc};
use std::{
    error::Error,
    process::Command,
    str, thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use sysinfo::{System, SystemExt};

use crate::appmod::objects;

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum SimState {
    CREATED,
    RUNNING,
    IDLE, //logged in
    KILL,
    RESET,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimCtrl {
    pub state: SimState,
    #[serde(skip)]
    pub status: HashMap<String, Vec<String>>,
    pub plt: HashMap<String, Vec<f64>>,
    pub sim: HashMap<String, f64>,
    #[serde(skip)]
    pub anim_state: objects::ObjAnim,
    #[serde(skip)]
    objstate: HashMap<String, HashMap<String, f64>>,
    #[serde(skip)]
    objectlist: Vec<objects::Obj3D>,
}

impl Default for SimCtrl {
    fn default() -> Self {
        Self {
            state: SimState::CREATED,
            status: HashMap::new(),
            plt: HashMap::new(),
            anim_state: objects::ObjAnim {
                steps: 0,
                step: 0,
                state: 0,
            },
            objectlist: vec![],
            objstate: HashMap::new(),

            sim: HashMap::from([
                ("rfoil".to_string(), 15.0),
                ("lfoil".to_string(), 0.10),
                ("lcoup".to_string(), 0.00),
                ("K".to_string(), 0.00),
            ]),
        }
    }
}

pub fn lts_gui(ctx: &Context, ui: &mut Ui, simctrl: &mut SimCtrl) {
    let mut sim = simctrl.sim.clone();
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            if ui.button("run").clicked() {
                let snspath =
                    Path::new("C:\\Users\\Admin\\OneDrive - BAT\\Saturn\\Simulations\\Sense\\");
                self::runltspice(snspath, "sense.asc");
                let pocpath =
                    Path::new("C:\\Users\\Admin\\OneDrive - BAT\\Saturn\\Simulations\\PoC4Full\\");
                self::runltspice(pocpath, "PoC4Full.asc");
            }

            ui.horizontal(|ui| {
                for (key, value) in simctrl.sim.iter_mut() {
                    ui.label(key.to_string());
                    if key != "K" {
                        let mut tmp = value.clone();
                        ui.add(egui::DragValue::new(&mut tmp).speed(0.1));
                        *value = tmp.clone();
                    } else {
                        if ui
                            .selectable_label(false, "with Foil".to_string())
                            .clicked()
                        {
                            
                        }
                    }
                }

                ui.end_row();
            });
        });
    });
}

fn chk_running(name: &str) -> bool {
    let s = System::new_all();
    for _process in s.processes_by_name(name) {
        println!("{:?} - running", name);
        return true;
    }
    println!("{:?} - not found", name);
    return false;
}

pub fn runltspice(filedir: &Path, name: &str) {
    //     //println!("blestarted {}", self.label);
    //     //  self.value = 4.0;
    let ltspice = "C:\\Program Files\\LTC\\LTspiceXVII\\XVIIx64.exe";
    let wdstate = chk_running("XVIIx64.exe");
    let prevpath = env::current_dir().unwrap();
    let filename = name.to_string();
    assert!(env::set_current_dir(&filedir).is_ok());
    if wdstate == false {
        println!("launching");
        thread::spawn(move || {
            Command::new(ltspice)
                .args(["-Run", "-b", &filename])
                .output()
                .expect("failed to execute process")
        });
    }
    while chk_running("XVIIx64.exe") == true {
        thread::sleep(Duration::from_millis(100));
        // timeout -= 1;
        // if timeout == 0 {
        //     break;
        // }
        println!("running");
    }
    println!("finished");
    assert!(env::set_current_dir(&prevpath).is_ok());
}
