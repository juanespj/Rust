extern crate graphics;
extern crate ndarray;
extern crate opengl_graphics;
extern crate piston;

pub const PI: f64 = 3.14159265358979323846264338327950288f64; // 3.1415926535897931f64

use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::WindowSettings;

use ndarray::{arr1, arr2, Array1};

pub struct Tube {
    pub pos: [f64; 2],
    pub r: f64,
    pub alph: f64,
    pub beta: f64,
    pub gamm: f64,
    pub len: f64,
}

pub struct App {
    pub gl: GlGraphics, // OpenGL drawing backend.
    pub rotation: f64,  // Rotation for the square.
                        //pub window: &W,
}

pub fn draw_circle(&mut self, args: &RenderArgs, pos: [f64; 2], r: f64) {
    self.gl.draw(args.viewport(), |c, gl| {
        use graphics::*;
        let orange = [1.0, 0.5, 0.0, 1.0];

        let c = c.trans(0.0, 0.0); //origin
        const NO_POINTS: usize = 100;
        let mut t: f64 = 0.0;
        let mut arr_x: [f64; NO_POINTS] = [0.0; NO_POINTS];
        let mut arr_y: [f64; NO_POINTS] = [0.0; NO_POINTS];
        //let transform = c.transform.trans(0.0, 0.0).rot_rad(0.0).trans(0.0, 0.0);
        for i in 0..NO_POINTS {
            t += 2.0 * PI / (NO_POINTS as f64);
            arr_x[i] = pos[0] + r * t.cos();
            arr_y[i] = pos[1] + r * t.sin();
        }
        for i in 0..NO_POINTS - 1 {
            graphics::line(
                orange,
                1.2,
                [arr_x[i], arr_y[i], arr_x[i + 1], arr_y[i + 1]],
                c.transform,
                gl,
            );
        }
    })
}

impl App {
    pub fn draw(&mut self, args: &RenderArgs, arr_x: [f64; 5], arr_y: [f64; 5]) {
        use graphics::*;
        let orange = [1.0, 0.5, 0.0, 1.0];

        self.gl.draw(args.viewport(), |c, gl| {
            let c = c.trans(0.0, 0.0); //origin
                                       //let transform = c.transform.trans(0.0, 0.0).rot_rad(0.0).trans(0.0, 0.0);
            for i in 0..4 {
                line(
                    orange,
                    1.2,
                    [arr_x[i], arr_y[i], arr_x[i + 1], arr_y[i + 1]],
                    c.transform,
                    gl,
                );
            }
        });
    }

    

    pub fn draw_elipse(&mut self, args: &RenderArgs, tube: &Tube) {
        self.gl.draw(args.viewport(), |c, gl| {
            use graphics::*;
            let orange = [1.0, 0.5, 0.0, 1.0];

            let c = c.trans(0.0, 0.0); //origin
            const NO_POINTS: usize = 100;
            let mut t: f64 = 0.0;

            let rx = tube.r * tube.beta.cos();
            let ry = tube.r * tube.beta.sin();
            let mut arr_x: [f64; NO_POINTS] = [0.0; NO_POINTS];
            let mut arr_y: [f64; NO_POINTS] = [0.0; NO_POINTS];
            //let transform = c.transform.trans(0.0, 0.0).rot_rad(0.0).trans(0.0, 0.0);
            for i in 0..NO_POINTS {
                t += 2.0 * PI / (NO_POINTS as f64);
                arr_x[i] =
                    tube.pos[0] + rx * t.cos() * tube.alph.cos() - ry * t.sin() * tube.alph.sin();
                arr_y[i] =
                    tube.pos[1] + rx * t.sin() * tube.alph.sin() + ry * t.sin() * tube.alph.cos();
            }
            for i in 0..NO_POINTS - 1 {
                graphics::line(
                    orange,
                    1.2,
                    [arr_x[i], arr_y[i], arr_x[i + 1], arr_y[i + 1]],
                    c.transform,
                    gl,
                );
            }
        })
    }

    pub fn draw_tube(&mut self, args: &RenderArgs, tube: &Tube) {
        self.gl.draw(args.viewport(), |c, gl| {
            use graphics::*;
            let orange = [1.0, 0.5, 0.0, 1.0];

            let c = c.trans(0.0, 0.0); //origin
            const NO_POINTS: usize = 100;
            let mut t: f64 = 0.0;

            let rx = tube.r * tube.beta.cos();
            let ry = tube.r * tube.beta.sin();
            let mut arr_x: [f64; NO_POINTS] = [0.0; NO_POINTS];
            let mut arr_y: [f64; NO_POINTS] = [0.0; NO_POINTS];
            //let transform = c.transform.trans(0.0, 0.0).rot_rad(0.0).trans(0.0, 0.0);
            for i in 0..NO_POINTS {
                t += 2.0 * PI / (NO_POINTS as f64);
                arr_x[i] =
                    tube.pos[0] + rx * t.cos() * tube.alph.cos() - ry * t.sin() * tube.alph.sin();
                arr_y[i] =
                    tube.pos[1] + rx * t.sin() * tube.alph.sin() + ry * t.sin() * tube.alph.cos();
            }
            for i in 0..NO_POINTS - 1 {
                graphics::line(
                    orange,
                    0.8,
                    [arr_x[i], arr_y[i], arr_x[i + 1], arr_y[i + 1]],
                    c.transform,
                    gl,
                );
            }
            let offset_x = tube.len * tube.alph.cos() * tube.beta.sin();
            let offset_y = tube.len * tube.alph.sin() * tube.beta.cos();
            for i in 0..NO_POINTS - 1 {
                graphics::line(
                    orange,
                    0.8,
                    [
                        arr_x[i] + offset_x,
                        arr_y[i] + offset_y,
                        arr_x[i + 1] + offset_x,
                        arr_y[i + 1] + offset_y,
                    ],
                    c.transform,
                    gl,
                );
            }
            let points = [0, 25, 75];
            for i in points {
                graphics::line(
                    orange,
                    0.8,
                    [arr_x[i], arr_y[i], arr_x[i] + offset_x, arr_y[i] + offset_y],
                    c.transform,
                    gl,
                );
            }
        })
    }

    pub fn draw_tube_matrix(&mut self, args: &RenderArgs, tube: &Tube) {
        self.gl.draw(args.viewport(), |c, gl| {
            use graphics::*;
            let orange = [1.0, 0.5, 0.0, 1.0];

            let c = c.trans(0.0, 0.0); //origin
            const NO_POINTS: usize = 100;
            let mut t: f64 = 0.0;
            let mut u = 0.0;

            let mut xyz;
            let pos_vect = arr1(&[tube.pos[0], tube.pos[1], 0.0]);
            let rot_alpha = arr2(&[
                [1.0, 0.0, 0.0],
                [0.0, tube.alph.cos(), -tube.alph.sin()],
                [0.0, tube.alph.sin(), tube.alph.cos()],
            ]);
            let rot_beta = arr2(&[
                [tube.beta.cos(), 0.0, tube.beta.sin()],
                [0.0, 1.0, 0.0],
                [-tube.beta.sin(), 0.0, tube.beta.cos()],
            ]);

            let rot_gamm = arr2(&[
                [tube.alph.cos(), -tube.alph.sin(), 0.0],
                [tube.alph.sin(), tube.alph.cos(), 0.0],
                [0.0, 0.0, 1.0],
            ]);
            let rot_mat = rot_alpha.dot(&rot_beta).dot(&rot_gamm);
            let mut arr_x: [f64; NO_POINTS] = [0.0; NO_POINTS];
            let mut arr_y: [f64; NO_POINTS] = [0.0; NO_POINTS];
            let mut arr_x2: [f64; NO_POINTS] = [0.0; NO_POINTS];
            let mut arr_y2: [f64; NO_POINTS] = [0.0; NO_POINTS];
            let mut new_vect;
            //let transform = c.transform.trans(0.0, 0.0).rot_rad(0.0).trans(0.0, 0.0);
            for i in 0..NO_POINTS {
                t += 2.0 * PI / (NO_POINTS as f64);
                xyz = arr1(&[tube.r * t.cos(), tube.r * t.sin(), u]);
                new_vect = rot_mat.dot(&xyz) + &pos_vect;
                arr_x[i] = new_vect[0];
                arr_y[i] = new_vect[1];
            }
            for i in 0..NO_POINTS - 1 {
                graphics::line(
                    orange,
                    0.8,
                    [arr_x[i], arr_y[i], arr_x[i + 1], arr_y[i + 1]],
                    c.transform,
                    gl,
                );
            }
            
            u = tube.len;
            for i in 0..NO_POINTS {
                t += 2.0 * PI / (NO_POINTS as f64);
                xyz = arr1(&[tube.r * t.cos(), tube.r * t.sin(), u]);
                new_vect = rot_mat.dot(&xyz) + &pos_vect;
                arr_x2[i] = new_vect[0];
                arr_y2[i] = new_vect[1];
            }
            for i in 0..NO_POINTS - 1 {
                graphics::line(
                    orange,
                    0.8,
                    [arr_x2[i], arr_y2[i], arr_x2[i + 1], arr_y2[i + 1]],
                    c.transform,
                    gl,
                );
            }

            let points = [0, 20, 40, 60, 80];
            for i in points {
                graphics::line(
                    orange,
                    0.8,
                    [arr_x[i], arr_y[i], arr_x2[i], arr_y2[i]],
                    c.transform,
                    gl,
                );
            }
        })
    }

    pub fn draw_ui(&mut self, args: &RenderArgs) {
        use graphics::*;
        let orange = [1.0, 0.5, 0.0, 1.0];
        let black = [0.0, 0.0, 0.0, 1.0];
        let red = [1.0, 0.0, 0.0, 1.0];
        self.gl.draw(args.viewport(), |c, gl| {
            let c = c.trans(0.0, 0.0);
            let rect = math::margin_rectangle([20.0, 20.0, 400.0, 400.0], 0 as f64 * 5.0);
            //graphics::rectangle(orange, rect, c.transform, gl);
            Rectangle::new_border(black, 1.0).draw(rect, &c.draw_state, c.transform, gl);
        });
    }
}

// fn draw_rectangles<G: Graphics>(cursor: [f64; 2], window: &dyn Window, c: &Context, g: &mut G) {
//     let size = window.size();
//     let draw_size = window.draw_size();
//     let zoom = 0.2;
//     let offset = 30.0;

//     let rect_border = graphics::Rectangle::new_border([1.0, 0.0, 0.0, 1.0], 1.0);

//     // Cursor.
//     let cursor_color = [0.0, 0.0, 0.0, 1.0];
//     let zoomed_cursor = [offset + cursor[0] * zoom, offset + cursor[1] * zoom];
//     graphics::ellipse(
//         cursor_color,
//         graphics::ellipse::circle(zoomed_cursor[0], zoomed_cursor[1], 4.0),
//         c.transform,
//         g,
//     );

//     // User coordinates.
//     rect_border.draw(
//         [
//             offset,
//             offset,
//             size.width as f64 * zoom,
//             size.height as f64 * zoom,
//         ],
//         &c.draw_state,
//         c.transform,
//         g,
//     );
//     let rect_border = graphics::Rectangle::new_border([0.0, 0.0, 1.0, 1.0], 1.0);
//     rect_border.draw(
//         [
//             offset + size.width as f64 * zoom,
//             offset,
//             draw_size.width as f64 * zoom,
//             draw_size.height as f64 * zoom,
//         ],
//         &c.draw_state,
//         c.transform,
//         g,
//     );
// }

// fn draw_axis_values<W: Window, G: Graphics>(
//     axis_values: &mut AxisValues,
//     window: &W,
//     c: &Context,
//     g: &mut G,
// ) {
//     let window_height = window.size().height as f64;
//     let max_axis_height = 200.0;
//     let offset = 10.0;
//     let top = window_height - (max_axis_height + offset);
//     let color = [1.0, 0.0, 0.0, 1.0];
//     let width = 10.0;
//     let mut draw = |i, v: f64| {
//         let i = i as f64;
//         let height = (v + 1.0) / 2.0 * max_axis_height;
//         let rect = [
//             offset + i * (width + offset),
//             top + max_axis_height - height,
//             width,
//             height,
//         ];
//         graphics::rectangle(color, rect, c.transform, g);
//     };
//     for (i, &v) in axis_values.values().enumerate() {
//         draw(i, v);
//     }
// }
