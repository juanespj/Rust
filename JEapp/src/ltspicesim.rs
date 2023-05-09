use crate::appmod::objects;
use core::f64::consts::PI;
use egui::widgets::plot::{Arrows, Legend, Line, Plot, PlotPoint, PlotPoints, Polygon, Text};
use egui::Color32;
use egui::*;
use num::signum;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::ffi::OsStr;
use std::fmt::format;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc};
use std::{
    error::Error,
    process::Command,
    str, thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use sysinfo::{System, SystemExt};

use crate::ltspice::results::{DataType, PeakType};
use crate::ltspice::SteppedSimulation;

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
    pub sim: HashMap<String, String>,
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
                ("Rfoil".to_string(), "15.0".to_string()),
                ("Lfoil".to_string(), "0.10".to_string()),
                ("Lcoup".to_string(), "1".to_string()),
                // ("K".to_string(), "L1 L2 1".to_string()),
                ("Ctank".to_string(), "1e-9".to_string()),
            ]),
        }
    }
}
use std::fs::File;
use std::io::Write;

pub fn lts_gui(ctx: &Context, ui: &mut Ui, simctrl: &mut SimCtrl) {
    //let mut sim = simctrl.sim.clone();
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            if ui.button("run").clicked() {
                let pocpath =
                    PathBuf::from(r"C:\Users\Admin\OneDrive - BAT\Saturn\Simulations\PoC4Full\");
                self::runltspice(&pocpath, "PoC4Full.asc", simctrl);
                // let snspath = PathBuf::from(r"C:\Users\Admin\OneDrive - BAT\Saturn\Simulations\Sense\");
                // self::runltspice(&snspath, "sense.asc", &mut simctrl.clone());
            }

            ui.horizontal(|ui| {
                for (key, value) in simctrl.sim.iter_mut() {
                    if key != "K" && key != "Lcoup" {
                        ui.vertical(|ui| {
                            ui.label(key.to_string());
                            let mut f = value.parse::<f32>().unwrap();
                            ui.add(egui::DragValue::new(&mut f).speed(0.1));
                            *value = format!("{}", f);
                        });
                    }
                }
                let mut foil_on: bool = false;
                // if simctrl.sim.get("K").unwrap().contains("L1 L2")  {
                //     foil_on = true;
                // }
                if ui
                    .selectable_label(foil_on, "with Foil".to_string())
                    .clicked()
                {
                    if foil_on == false {
                        simctrl
                            .sim
                            .entry("K".to_string())
                            .and_modify(|k| *k = "L1 L2 1".to_string());
                        simctrl
                            .sim
                            .entry("Lcoup".to_string())
                            .and_modify(|k| *k = "1".to_string());
                    } else {
                        simctrl
                            .sim
                            .entry("K".to_string())
                            .and_modify(|k| *k = "L3 L2 1".to_string());
                        simctrl
                            .sim
                            .entry("Lcoup".to_string())
                            .and_modify(|k| *k = "1".to_string());
                    }
                }
                ui.end_row();
            });
            for (key, value) in simctrl.sim.iter_mut() {
                if key != "K" {
                    ui.label(format!(".param {} {}", key, value.parse::<f32>().unwrap()).as_str());
                } else {
                    ui.label(format!(".param {} {:?}", key, value).as_str());
                }
            }
        });
    });
    let plot = Plot::new("plt")
        .include_x(0.0)
        .include_y(0.0)
        .width(600.0)
        .height(180.0)
        .view_aspect(1.0)
        .data_aspect(1.0)
        .allow_zoom(true)
        .allow_drag(true)
        .show_axes([true; 2])
        .show_background(false)
        .legend(Legend::default())
        .center_y_axis(true);

    plot.show(ui, |plot_ui| {
        if simctrl.plt.contains_key("t") {
            let t = simctrl.plt.get("t").unwrap();
            let x = simctrl.plt.get("sense").unwrap();
            let plt: PlotPoints = (0..x.len()).map(|i| [t[i], x[i]]).collect();

            let planned_line = Line::new(plt).color(Color32::from_rgb(150, 255, 150));
            plot_ui.line(planned_line.name("sense"));

            // let e = simctrl.plt.get("e").unwrap();
            // let plt: PlotPoints = (0..x.len()).map(|i| [t[i], e[i]]).collect();

            // let planned_line = Line::new(plt).color(Color32::from_rgb(255, 50, 50));
            // plot_ui.line(planned_line.name("e"));
        }
    });
}

fn chk_running(name: &str) -> bool {
    let s = System::new_all();
    for _process in s.processes_by_name(name) {
        print!("\r{:?} - running", name);
        return true;
    }
    println!("{:?} - not found", name);
    return false;
}

pub fn runltspice(filedir: &PathBuf, name: &str, simctrl: &mut SimCtrl) {
    //     //println!("blestarted {}", self.label);
    //     //  self.value = 4.0;
    let ltspice = "C:\\Program Files\\LTC\\LTspiceXVII\\XVIIx64.exe";
    let wdstate = chk_running("XVIIx64.exe");
    let prevpath = env::current_dir().unwrap();
    let filename = name.to_string();
    assert!(env::set_current_dir(filedir).is_ok());
    println!("curr{:?}", env::current_dir().unwrap());
    //Update params
    let mut txtout: String = "".to_string();
    for (key, value) in simctrl.sim.iter_mut() {
        if key != "K" {
            txtout
                .push_str(format!(".param {} {}\r\n", key, value.parse::<f32>().unwrap()).as_str());
        } else {
            txtout.push_str(format!(".param {} {:?}\r\n", key, value).as_str());
        }
    }

    match File::create("param.txt") {
        Ok(mut file) => match file.write_all(txtout.as_bytes()) {
            Ok(_) => println!("Params Saved"),
            Err(e) => println!("error: {:?}", e),
        },
        Err(e) => println!("error: {:?}", e),
    }
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
        // println!("running");
    }
    println!("finished");
    //read results

    // match File::open(format!("{}.raw", name.split_at(name.len()-4).0)) {
    //     Ok(file) => {
    //         let reader = BufReader::new(file);
    //         for line in reader.lines() {
    //             match line {
    //                 Ok(text) => println!("{}", text),
    //                 Err(e) => {},
    //             };
    //         }
    //     }
    //     Err(_) => {}
    // }

    let model_name = Path::new(name.split_at(name.len() - 4).0);

    let results = SteppedSimulation::from_files(
        model_name.with_extension("raw").as_path(),
        model_name.with_extension("log").as_path(),
    );

    //Get all Variables that are in the Results of the Simulation
    let vars = results.available_variables();
    let steps = results.available_steps();
    for var in vars.iter() {
        if var.name == "time" {
            println!("Time: {:?}", var)
        }
        if var.name.contains("sense") {
            println!("var: {:?}", var)
        }
        //println!("vars {:?}", var.name);
    }
    let vout = results.get_variable_for_name("V(sense)").unwrap();
    println!("VOUT {:?}", vout);
    let vout = results.get_results(11);
    // println!("results {:?}", results.get_results(11));

    simctrl
        .plt
        .insert("t".to_string(), results.get_results(0));
    simctrl
        .plt
        .insert("sense".to_string(), results.get_results(11));
    // simctrl
    // .plt
    // .entry("x".to_string())
    // .or_insert_with(Vec::new)
    // .push(x_out);
    // simctrl.objectlist;
    //

    //     simctrl
    //     .plt
    //     .entry("x".to_string())
    //     .or_insert_with(Vec::new)
    //     .push(x_out);

    assert!(env::set_current_dir(&prevpath).is_ok());
}
