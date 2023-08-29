extern crate kiss3d;
extern crate nalgebra as na;

use kiss3d::camera::{ArcBall, FirstPerson};
use kiss3d::light::Light;
use kiss3d::resource::Mesh;
use kiss3d::window::Window;
use na::{Point3, UnitQuaternion, Vector3};
use std::cell::RefCell;
use std::fs;
use std::rc::Rc;

// fn modpoint3(vec:Point3<f32>){
// let dist =
// }
fn main() {
    let eye = Point3::new(100.0f32, 100.0, 100.0);
    let at = Point3::origin();
    let mut first_person = FirstPerson::new(eye, at);
    let mut arc_ball = ArcBall::new(eye, at);
    let mut use_arc_ball = true;
    let mut window = Window::new("Kiss3d: custom_mesh");
    let mut data: [Vec<f32>; 3] = [vec![], vec![], vec![]];
    process_raw_probe_file("probe.txt".to_string(), &mut data);

    let mut vertices: Vec<Point3<f32>> = vec![];
    let mut indices: Vec<Point3<u16>> = vec![];

    for ix in 0..data[0].len() {
        let point = Point3::new(data[0][ix], data[1][ix], data[2][ix] * 10.0);
        vertices.push(point);
    }
//    let mut drain = vertices.drain(..);

    let mut vsorted = vec![vertices[0]];
    vertices.remove(0);
    while vertices.len() >0 {
        let mut dist = 0.0;
        let mut closest = 0;
        for (i,pt) in vertices.iter().enumerate() {
            let v = pt.clone() - vsorted[vsorted.len()-1];
            let norm = &v.dot(&v).sqrt();
            if dist < *norm || dist == 0.0 {
                dist = *norm;
                closest = i;
            }

        //     // dist= ().abs();
        }
        vsorted.push(vertices[closest]);
    vertices.remove(closest);
       
    }
    // let a = Point3::new(-1.0, -1.0, 0.0);
    // let b = Point3::new(1.0, -1.0, 0.0);
    // let c = Point3::new(0.0, 1.0, 0.0);
    // let d = Point3::new(0.0, 1.0, 1.0);
    // let e = Point3::new(0.0, -1.0, 1.0);
    // vertices = vec![a, b, c, d, e];

    let mut i = 0;
    let mut ix = 0;
    // window.render_with_camera(&mut arc_ball);
    window.set_line_width(2.0);

    window.set_light(Light::StickToCamera);
    window.set_point_size(10.0);
    while window.render() {
        i = 0;
        ix = 0;
        for pt in &vsorted {
            window.draw_point(pt, &Point3::new(1.0, 1.0, 0.0));
            i += 1;
            if i > 1 {
                window.draw_line(
                    &vsorted[ix - 1],
                    &vsorted[ix],
                    &Point3::new(1.0, 0.0, pt.z * 10.0),
                );
            }
            ix += 1;
        }
        // window.render_with_camera(&mut first_person);
        // window.draw_line(&b, &c, &Point3::new(0.0, 1.0, 0.0));
        // window.draw_line(&c, &a, &Point3::new(0.0, 0.0, 1.0));

        // window.draw_planar_line(
        //     &Point2::new(-100.0, -200.0),
        //     &Point2::new(100.0, -200.0),
        //     &Point3::new(1.0, 1.0, 1.0),
        // );
    }
    // for ix in 0..data[0].len() {
    //     for iy in 0..data[1].len() {
    //         if
    //         vertices.push(Point3::new(data[0][ix], data[1][ix], data[2][ix]));
    //         i += 1;
    //     }
    // }

    // println!("{:?}", data[0]);
    // // let
    // // let indices = vec![
    // //     Point3::new(0u16, 1, 2),
    // //     Point3::new(0u16, 3, 4),
    // //     Point3::new(2, 3, 1),
    // //     Point3::new(1, 2, 4),
    // // ];

    // let mesh = Rc::new(RefCell::new(Mesh::new(
    //     vertices, indices, None, None, false,
    // )));
    // let mut c = window.add_mesh(mesh, Vector3::new(1.0, 1.0, 1.0));

    // c.set_color(1.0, 0.0, 0.0);
    // c.enable_backface_culling(false);

    // window.set_light(Light::StickToCamera);

    // let rot = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), 0.014);

    // while window.render() {
    //     c.prepend_to_local_rotation(&rot);
    // }
}

pub fn process_raw_probe_file(filename: String, data: &mut [Vec<f32>; 3]) {
    println!("{:?}", filename);

    let contents = fs::read_to_string(filename).expect("Should have been able to read the file");
    data[0].clear();
    data[1].clear();
    data[2].clear();
    for line in contents.lines() {
        let mut triplet: Vec<f32> = Vec::new();

        for var in line.split(",") {
            let f: f32 = match var.parse() {
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

// pub fn draw_3dmesh(data: &mut [Vec<f64>; 3], obj: &mut Obj3D) {
//     obj.points[0].clear();
//     obj.points[1].clear();

//     let pos_vect = arr1(&[obj.pos[0], obj.pos[1], 0.0]);

//     //let mut new_vect;
//     //let transform = c.transform.trans(0.0, 0.0).rot_rad(0.0).trans(0.0, 0.0);
//     for i in 0..data[0].len() {
//         let xyz = arr1(&[data[0][i], data[1][i], data[2][i]]);
//         let new_vect = rot_mat.dot(&xyz) + &pos_vect;
//         obj.points[0].push(new_vect[0]);
//         obj.points[1].push(new_vect[1]);
//     }
// }
