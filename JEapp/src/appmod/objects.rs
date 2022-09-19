pub struct Obj3D {
    pub pos: [f64; 3],
    pub r: f64,
    pub alph: f64,
    pub beta: f64,
    pub gamm: f64,
    pub points: [Vec<f64>; 2], //X Y points for render
    pub scale: f64,
    pub res: usize, //resolution
}

pub struct Surf3D {
    pub pos: [f64; 3],
    pub r: f64,
    pub alph: f64,
    pub beta: f64,
    pub gamm: f64,
    pub points_raw:[Vec<f64>; 3],
    pub points: Vec<[[f64; 4]; 2]>, //X Y points for rectangles render
    pub scale: f64,
    pub res: usize, //resolution
    
}

use ndarray::{arr1, arr2, Array1};
use std::num::*;
pub const PI: f64 = 3.14159265358979323846264338327950288f64; // 3.1415926535897931f64

pub fn draw_circle2d(obj: &mut Obj3D) {
    obj.points[0].clear();
    obj.points[1].clear();
    //let c = c.trans(0.0, 0.0); //origin
    let mut t: f64 = 0.0;
    //let transform = c.transform.trans(0.0, 0.0).rot_rad(0.0).trans(0.0, 0.0);
    for i in 0..obj.res + 1 {
        t += 2.0 * PI / (obj.res as f64);
        obj.points[0].push(obj.pos[0] + obj.r * t.cos());
        obj.points[1].push(obj.pos[1] + obj.r * t.sin());
    }
}

pub fn draw_circle3d(obj: &mut Obj3D) {
    let u = 0.0;
    obj.points[0].clear();
    obj.points[1].clear();
    let pos_vect = arr1(&[obj.pos[0], obj.pos[1], 0.0]);
    let rot_alpha = arr2(&[
        [1.0, 0.0, 0.0],
        [0.0, obj.alph.cos(), -obj.alph.sin()],
        [0.0, obj.alph.sin(), obj.alph.cos()],
    ]);
    let rot_beta = arr2(&[
        [obj.beta.cos(), 0.0, obj.beta.sin()],
        [0.0, 1.0, 0.0],
        [-obj.beta.sin(), 0.0, obj.beta.cos()],
    ]);

    let rot_gamm = arr2(&[
        [obj.alph.cos(), -obj.alph.sin(), 0.0],
        [obj.alph.sin(), obj.alph.cos(), 0.0],
        [0.0, 0.0, 1.0],
    ]);
    let rot_mat = rot_alpha.dot(&rot_beta).dot(&rot_gamm);
    let mut t: f64 = 0.0;
    //let mut new_vect;
    //let transform = c.transform.trans(0.0, 0.0).rot_rad(0.0).trans(0.0, 0.0);
    for i in 0..obj.res + 1 {
        t += 2.0 * PI / (obj.res as f64);
        let xyz = arr1(&[obj.r * t.cos(), obj.r * t.sin(), u]);
        let new_vect = rot_mat.dot(&xyz) + &pos_vect;
        obj.points[0].push(new_vect[0]);
        obj.points[1].push(new_vect[1]);
    }
}

pub fn draw_3dmesh(data: &mut [Vec<f64>; 3], obj: &mut Obj3D) {
    obj.points[0].clear();
    obj.points[1].clear();

    let pos_vect = arr1(&[obj.pos[0], obj.pos[1], 0.0]);
    let rot_alpha = arr2(&[
        [1.0, 0.0, 0.0],
        [0.0, obj.alph.cos(), -obj.alph.sin()],
        [0.0, obj.alph.sin(), obj.alph.cos()],
    ]);
    let rot_beta = arr2(&[
        [obj.beta.cos(), 0.0, obj.beta.sin()],
        [0.0, 1.0, 0.0],
        [-obj.beta.sin(), 0.0, obj.beta.cos()],
    ]);

    let rot_gamm = arr2(&[
        [obj.alph.cos(), -obj.alph.sin(), 0.0],
        [obj.alph.sin(), obj.alph.cos(), 0.0],
        [0.0, 0.0, 1.0],
    ]);
    let rot_mat = rot_alpha.dot(&rot_beta).dot(&rot_gamm);

    //let mut new_vect;
    //let transform = c.transform.trans(0.0, 0.0).rot_rad(0.0).trans(0.0, 0.0);
    for i in 0..data[0].len() {
        let xyz = arr1(&[data[0][i], data[1][i], data[2][i]]);
        let new_vect = rot_mat.dot(&xyz) + &pos_vect;
        obj.points[0].push(new_vect[0]);
        obj.points[1].push(new_vect[1]);
    }
}

fn point_dist(x1: f64, x2: f64, y1: f64, y2: f64) -> f64 {
    (f64::powf(x2 - x1, 2.0) + f64::powf(y2 - y1, 2.0)).sqrt()
}

pub fn draw_3dmesh_surf(
   // data: &mut [Vec<f64>; 3],
    obj: &mut Surf3D//,
    //surflist: &mut Vec<[[f64; 4]; 2]>,
) {
    //surflist :&mut Vec<Obj3D>
    //, surflist: &mut Vec<egui::Pos2>
    obj.points.clear();   

    for i in 0..obj.points_raw[0].len() {
        //  last_d= d_max;
        for j in 0..obj.points_raw[0].len() - i - 1 {
            let d = point_dist(obj.points_raw[0][0], obj.points_raw[0][j], obj.points_raw[1][0], obj.points_raw[1][j]);
            let d1 = point_dist(obj.points_raw[0][0], obj.points_raw[0][j + 1], obj.points_raw[1][0], obj.points_raw[1][j + 1]);
            if d1 < d {
                obj.points_raw[0].swap(j, j + 1);
                obj.points_raw[1].swap(j, j + 1);
                obj.points_raw[2].swap(j, j + 1);
            }
            //println!("d: {:?}", d);
        }
    }

    let mut x_vals: Vec<f64> = vec![];
    for i in 0..obj.points_raw[0].len() {
        let x = obj.points_raw[0][i].round();
        if !x_vals.contains(&x) {
            x_vals.push(x);
        }
    }
    // println!("data{:?} ", data[0]);
    // println!("xvals{:?} ", data[0].len());
     //println!("xvals{:?} ", x_vals);
    //organize 3d points into matrix format
    let mut meschvec: Vec<Vec<[f64; 3]>> = vec![];

    for i in 0..x_vals.len() {
        meschvec.push(vec![]);
        for j in 0..obj.points_raw[0].len() {
            if obj.points_raw[0][j].round() == x_vals[i].round() {
                meschvec[i].push([obj.points_raw[0][j], obj.points_raw[1][j], obj.points_raw[2][j]]);
                // println!("{:?} ix {}", obj.points_raw[0][j],i);
            }
        }
    }

    //println!("data{:?} ", meschvec);

    //  println!("data{:?} ", surflist);

    // let x = [0.0, 1.0, 1.0, 0.0];
    //                 let y = [0.0, 0.0, 1.0, 1.0];
    // let mut current_x: f64 = data[0][0];
    // let mut mesh: Vec<Vec<usize>> = vec![];
    //sort distance
    // for i in 0..data[0].len() {
    //     //  last_d= d_max;
    //     for j in 0..data[0].len() - i - 1 {
    //         let d = point_dist(data[0][0], data[0][j], data[1][0], data[1][j]);
    //         let d1 = point_dist(data[0][0], data[0][j + 1], data[1][0], data[1][j + 1]);
    //         if d1 < d {
    //             data[0].swap(j, j + 1);
    //             data[1].swap(j, j + 1);
    //             data[2].swap(j, j + 1);
    //         }
    //         //println!("d: {:?}", d);
    //     }
    // }
//flat surfaces
    // print!("{:?}", mesh);
    // for i in 0..mesh.len() - 1 {
    //     for j in 1..mesh[0].intoiter() {
    //         println!("d: {:?}", i);
    //          let x = [data[0][i], data[0][i+1], data[0][i+1],data[0][i]];
    //          let y = [data[1][i], data[1][i], data[0][i+1], data[0][i+1]];

    //         surflist.push();
    //     }
    // }

    let pos_vect = arr1(&[obj.pos[0], obj.pos[1], 0.0]);
    let rot_alpha = arr2(&[
        [1.0, 0.0, 0.0],
        [0.0, obj.alph.cos(), -obj.alph.sin()],
        [0.0, obj.alph.sin(), obj.alph.cos()],
    ]);
    let rot_beta = arr2(&[
        [obj.beta.cos(), 0.0, obj.beta.sin()],
        [0.0, 1.0, 0.0],
        [-obj.beta.sin(), 0.0, obj.beta.cos()],
    ]);

    let rot_gamm = arr2(&[
        [obj.alph.cos(), -obj.alph.sin(), 0.0],
        [obj.alph.sin(), obj.alph.cos(), 0.0],
        [0.0, 0.0, 1.0],
    ]);
    let rot_mat = rot_alpha.dot(&rot_beta).dot(&rot_gamm);
   // println!("flat{:?} ", meschvec);
    for i in 0..meschvec.len() {
        for j in 0..meschvec[i].len() {
            let xyz = arr1(&[meschvec[i][j][0], meschvec[i][j][1], meschvec[i][j][2]*obj.scale]);//Z scale
            let new_vect = rot_mat.dot(&xyz) + &pos_vect;
            meschvec[i][j][0]=new_vect[0];
            meschvec[i][j][1]=new_vect[1];
            meschvec[i][j][2]=new_vect[2]; 
        }
    }
   // println!("3d{:?} ", meschvec);
    for i in 0..meschvec.len()-1 {
        for j in 0..meschvec[i].len()-1 {
            obj.points.push([
                [
                    meschvec[i][j][0],
                    meschvec[i][j + 1][0],
                    meschvec[i + 1][j + 1][0],
                    meschvec[i + 1][j][0],
                ], //XY j // &[Vec<[f64;4]>;2]
                [
                    meschvec[i][j][1],
                    meschvec[i][j + 1][1],
                    meschvec[i + 1][j + 1][1],
                    meschvec[i + 1][j][1],
                ],
            ]);
        }
    }
}
