use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc};
use std::{
    error::Error,
    str,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use egui::widgets::plot::{Arrows, Legend, Line, Plot, PlotPoint, PlotPoints, Polygon, Text};
use egui::Color32;
use egui::*;

use crate::appmod::objects;

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum RbbState {
    CREATED,
    RUNNING,
    IDLE, //logged in
    KILL,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RbbCtrl {
    pub state: RbbState,
    #[serde(skip)]
    pub status: HashMap<String, Vec<String>>,
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
                ("a".to_string(), 0.00), //incline
                ("r".to_string(), 0.5),
                ("x".to_string(), 0.0),
                ("xv".to_string(), 0.0),
                ("xa".to_string(), 0.0),
                ("m".to_string(), 1.0),
            ]),
        }
    }
}


pub fn rbb_gui(ctx: &Context, ui: &mut Ui, rbbctrl: &mut RbbCtrl) {
    ui.horizontal(|ui| {
        if ui.button("rbb").clicked() {
            let mut rbb: HashMap<String, f64> = HashMap::from([
                ("w".to_string(), 15.0),
                ("h".to_string(), 0.10),
                ("a".to_string(), 0.00),
                ("r".to_string(), 0.5),
                ("x".to_string(), 0.0),
                ("xv".to_string(), 0.0),
                ("xa".to_string(), 0.0),
                ("m".to_string(), 1.0),
            ]);
            objects::draw_rbb(&mut rbb, &mut rbbctrl.objectlist);
            rbbctrl.anim_state.state = 0;
            if rbbctrl.objstate.contains_key("rbb") {
                rbbctrl
                    .objstate
                    .entry("rbb".to_string())
                    .and_modify(|k| *k = rbb);
            } else {
                objects::draw_rbb(&mut rbb, &mut rbbctrl.objectlist);
                rbbctrl.objstate.insert("rbb".to_string(), rbb);
            }
        }
        if ui.button("Animate_rbb").clicked() {
            rbbctrl.anim_state.state = 1;
            rbbctrl.anim_state.steps = 500;
            rbbctrl.anim_state.step = 0;
        }
    });
    if rbbctrl.objectlist.len() > 0 {
        let plot = Plot::new("preview")
            .include_x(0.0)
            .include_y(0.0)
            .width(600.0)
            .height(300.0)
            .view_aspect(1.0)
            .data_aspect(1.0)
            .allow_zoom(true)
            .allow_drag(true)
            .show_axes([false; 2])
            .show_background(false)
            .legend(Legend::default());

        plot.show(ui, |plot_ui| {
            if rbbctrl.objectlist.len() > 0 {
                for obj in rbbctrl.objectlist.iter() {
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
                 // let n = 100;
                // let mut sin_values: Vec<_> = (0..=n)
                //     .map(|i| remap(i as f64, 0.0..=n as f64, -TAU..=TAU))
                //     .map(|i| [i, i.sin()])
                //     .collect();
                // let line = Line::new(sin_values.split_off(n / 2)).fill(-1.5);

                // plot_ui.line(line.name("Line with fill"));
            }
            let rbb = rbbctrl.objstate.get("rbb").unwrap().clone();
            let beam_ang = rbb.get("a").unwrap();
            plot_ui.arrows(Arrows::new(
                PlotPoints::from([-2.0 * beam_ang.sin(), 2.0 * beam_ang.cos()]),
                PlotPoints::from([-0.2*beam_ang.sin(), 0.2*beam_ang.cos()]),
            ));
            // let s = format!(
            //     "document.body.style.zoom='{:.2}'",
            //     (new_zoom as f32) / 100.0
            // );
            let angle=format!("angle: {:.2}",rbb.get("a").unwrap());
            plot_ui.text(Text::new(PlotPoint::new(10.0, 4.0),angle ));//.name("Text")
               
            //   plot_ui.text("angle:");
        });
        //self.data_ready = 0;
    }
    if rbbctrl.anim_state.state == 1 {
        self::rbb_anim(rbbctrl);
        if rbbctrl.objstate.contains_key("rbb") {
            // if self.anim_state.step==0{
            //     self.timer=Instant::now();
            // }

            // let enabled = ctx.input().time - disabled_time > 2.0;
            // if !enabled {
            ctx.request_repaint();
            // }
            rbbctrl.anim_state.step += 1;
        } else {
            rbbctrl.anim_state.state = 0;
        }
        if rbbctrl.anim_state.step >= rbbctrl.anim_state.steps {
            rbbctrl.anim_state.state = 0;
        }
    }
}

use core::f64::consts::PI;
use num::signum;

pub fn rbb_anim(rbbctrl: &mut RbbCtrl) {
    let dt = 0.1;
    let mut rbb = rbbctrl.objstate.get("rbb").unwrap().clone();
    let x_sp = 3.0 * signum(((rbbctrl.anim_state.step as f64) * PI / 50.0).cos()); //square

    let a_out = 0.05 * ((rbbctrl.anim_state.step as f64) / 10.0).cos();

    let xv_out = rbb.get("xv").unwrap() - rbb.get("xa").unwrap() * dt;
    let x_out = rbb.get("x").unwrap() + xv_out * dt - 0.5 * rbb.get("xa").unwrap() * dt * dt;
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
