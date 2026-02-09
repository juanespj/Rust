use egui::Color32;
use egui::*;
use egui_plot::{Line, Plot, PlotPoints, *}; //Legend
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, mpsc};
use std::{
    error::Error,
    str,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use crate::appmod::objects;

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum RbbState {
    CREATED,
    RUNNING,
    IDLE, //logged in
    KILL,
    RESET,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RbbCtrl {
    pub state: RbbState,
    #[serde(skip)]
    pub status: HashMap<String, Vec<String>>,
    pub plt: HashMap<String, Vec<f64>>,
    pub rbb: HashMap<String, f64>,
    #[serde(skip)]
    pub anim_state: objects::ObjAnim,
    #[serde(skip)]
    objstate: HashMap<String, HashMap<String, f64>>,
    #[serde(skip)]
    objectlist: Vec<objects::Obj3D>,
}

impl Default for RbbCtrl {
    fn default() -> Self {
        Self {
            state: RbbState::CREATED,
            status: HashMap::new(),
            plt: HashMap::new(),
            anim_state: objects::ObjAnim {
                steps: 0,
                step: 0,
                state: 0,
            },
            objectlist: vec![],
            objstate: HashMap::new(),

            rbb: HashMap::from([
                ("w".to_string(), 15.0),
                ("h".to_string(), 0.10),
                ("a".to_string(), 0.00),
                ("av".to_string(), 0.00),
                ("aa".to_string(), 0.00),
                ("r".to_string(), 0.5),
                ("x".to_string(), 0.0),
                ("xv".to_string(), 0.0),
                ("xa".to_string(), 0.0),
                ("m".to_string(), 1.0),
                ("Izz".to_string(), 10.0),
                ("Tmax".to_string(), 5.0),
            ]),
        }
    }
}

pub fn rbb_gui(ctx: &Context, ui: &mut Ui, rbbctrl: &mut RbbCtrl) {
    let mut rbb = rbbctrl.rbb.clone();
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            if ui.button("rbb").clicked() {
                rbbctrl.rbb.insert(
                    "j".to_string(),
                    rbb.get("w").unwrap() * rbb.get("h").unwrap().powf(3.0) / 12.0,
                );
                objects::draw_rbb(&mut rbb, &mut rbbctrl.objectlist);
                rbbctrl.anim_state.state = 0;
                if rbbctrl.objstate.contains_key("rbb") {
                    rbbctrl
                        .objstate
                        .entry("Reset".to_string())
                        .and_modify(|k| *k = rbb.clone());
                } else {
                    objects::draw_rbb(&mut rbb, &mut rbbctrl.objectlist);
                    rbbctrl.objstate.insert("rbb".to_string(), rbb.clone());
                }
            }
            if ui.button("Run").clicked() {
                rbbctrl.anim_state.state = 1;
                rbbctrl.anim_state.steps = 500;
                rbbctrl.anim_state.step = 0;
            }
            ui.horizontal(|ui| {
                ui.label("W: ");
                ui.add(egui::DragValue::new(rbbctrl.rbb.get_mut("w").unwrap()).speed(0.1));
                ui.end_row();
                ui.label("A: ");
                let mut ang = rbbctrl.rbb.get("a").unwrap() * 180.0 / PI;
                ui.add(egui::DragValue::new(&mut ang).speed(0.1));
                rbbctrl
                    .rbb
                    .entry("a".to_string())
                    .and_modify(|k| *k = ang / 180.0 * PI);
                ui.end_row();
            });
        });

        if rbbctrl.objectlist.len() > 0 {
            // let plot = Plot::new("preview")
            //     .include_x(0.0)
            //     .include_y(0.0)
            //     .width(600.0)
            //     .height(200.0)
            //     .view_aspect(1.0)
            //     .data_aspect(1.0)
            //     .allow_zoom(true)
            //     .allow_drag(true)
            //     .show_axes([false; 2])
            //     .show_background(false)
            //     .legend(Legend::default())
            //     .center_y_axis(true);

            // plot.show(ui, |plot_ui| {
            //     if rbbctrl.objectlist.len() > 0 {
            //         for obj in rbbctrl.objectlist.iter() {
            //             let x = &obj.points[0];
            //             let y = &obj.points[1];
            //             let plt: PlotPoints =
            //                 (0..x.len()).map(|i| [x[i] as f64, y[i] as f64]).collect();

            //             let planned_line = Line::new(plt).color(Color32::from_rgb(
            //                 obj.color[0],
            //                 obj.color[1],
            //                 obj.color[2],
            //             ));
            //             plot_ui.line(planned_line);
            //         }
            //     }
            //     let rbb = rbbctrl.objstate.get("rbb").unwrap().clone();
            //     let beam_ang = rbb.get("a").unwrap();
            //     plot_ui.arrows(Arrows::new(
            //         PlotPoints::from([2.0 * beam_ang.sin(), -2.0 * beam_ang.cos()]),
            //         PlotPoints::from([0.2 * beam_ang.sin(), -0.2 * beam_ang.cos()]),
            //     ));

            //     let angle = format!("angle: {:.2}", rbb.get("a").unwrap());
            //     plot_ui.text(Text::new(PlotPoint::new(10.0, 4.0), angle)); //.name("Text")
            // });
        }
    });
    if rbbctrl.anim_state.state == 1 {
        self::rbb_anim(rbbctrl);
        if rbbctrl.objstate.contains_key("rbb") {
            ctx.request_repaint();
            rbbctrl.anim_state.step += 1;
        } else {
            rbbctrl.anim_state.state = 0;
        }
        if rbbctrl.anim_state.step >= rbbctrl.anim_state.steps {
            rbbctrl.anim_state.state = 0;
        }
    }
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            // ComboBox::from("plt")
            //      .selected_text("var".to_string())
            //     .show_ui(ui, |ui| {
            //         // for i in 0..self.portlist.len() {
            //         //     ui.selectable_value(
            //         //         &mut self.port_sel,
            //         //         (*self.portlist[i]).to_string(),
            //         //         self.portlist[i].to_string(),
            //         //     );
            //         // }
            //     });
        });
        // let plot = Plot::new("plt")
        //     .include_x(0.0)
        //     .include_y(0.0)
        //     .width(600.0)
        //     .height(180.0)
        //     .view_aspect(1.0)
        //     .data_aspect(1.0)
        //     .allow_zoom(true)
        //     .allow_drag(true)
        //     .show_axes([true; 2])
        //     .show_background(false)
        //     .legend(Legend::default())
        //     .center_y_axis(true);

        // plot.show(ui, |plot_ui| {
        //     if rbbctrl.plt.contains_key("x") {
        //         let t = rbbctrl.plt.get("t").unwrap();
        //         let x = rbbctrl.plt.get("x").unwrap();
        //         let plt: PlotPoints = (0..x.len()).map(|i| [t[i], x[i]]).collect();

        //         let planned_line = Line::new(plt).color(Color32::from_rgb(150, 255, 150));
        //         plot_ui.line(planned_line.name("x"));

        //         let e = rbbctrl.plt.get("e").unwrap();
        //         let plt: PlotPoints = (0..x.len()).map(|i| [t[i], e[i]]).collect();

        //         let planned_line = Line::new(plt).color(Color32::from_rgb(255, 50, 50));
        //         plot_ui.line(planned_line.name("e"));
        //     }
        // });
    });
}

use core::f64::consts::PI;
use num::signum;

pub fn rbb_anim(rbbctrl: &mut RbbCtrl) {
    let dt = 0.1;
    let mut rbb = rbbctrl.objstate.get("rbb").unwrap().clone();
    let x_sp = 3.0 * signum(((rbbctrl.anim_state.step as f64) * PI / 50.0).cos()); //square

    if !rbb.contains_key("xold") {
        rbb.entry("xold".to_string()).or_insert(0.0);
        rbb.entry("aold".to_string()).or_insert(0.0);
    }
    // rbb.entry("aa".to_string()).or_insert(default)
    // rbb.get("av")
    // ("aa".to_string(), 0.00),
    // let a_out = 0.05 * ((rbbctrl.anim_state.step as f64) / 10.0).cos();
    let a_out = 0.05 * ((rbbctrl.anim_state.step as f64) / 10.0).cos();
    let xv_out = rbb.get("xv").unwrap() - rbb.get("xa").unwrap() * dt;
    let x_out = rbb.get("x").unwrap() + xv_out * dt - 0.5 * rbb.get("xa").unwrap() * dt * dt;
    rbb.entry("xold".to_string()).or_insert(x_out.clone());
    rbb.entry("aold".to_string()).or_insert(a_out.clone());
    rbbctrl
        .plt
        .entry("t".to_string())
        .or_insert_with(Vec::new)
        .push(dt * rbbctrl.anim_state.step as f64);

    rbbctrl
        .plt
        .entry("x".to_string())
        .or_insert_with(Vec::new)
        .push(x_out);

    rbbctrl
        .plt
        .entry("e".to_string())
        .or_insert_with(Vec::new)
        .push(x_sp - x_out);

    rbb.entry("a".to_string()).and_modify(|k| *k = a_out);
    rbb.entry("xa".to_string())
        .and_modify(|k| *k = 9.81 * a_out.sin());
    rbb.entry("xv".to_string()).and_modify(|k| *k = xv_out);
    rbb.entry("x".to_string()).and_modify(|k| *k = x_out);

    objects::draw_rbb(&mut rbb, &mut rbbctrl.objectlist);
    rbbctrl
        .objstate
        .entry("rbb".to_string())
        .and_modify(|k| *k = rbb);
}
