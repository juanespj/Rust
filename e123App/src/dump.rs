// let plot = Plot::new("fplot")
//     .include_x(0.0)
//     .include_y(0.0)
//     .width(500.0)
//     .height(200.0)
//     .legend(Legend::default());

// if self.data_ready == 1 {
//     plot.show(ui, |plot_ui| {
//         let x = &self.dataset.dataf["Time"];
//         let y = &self.dataset.dataf["Tcalc"];
//         //  println!("{:?}",y);
//         //     let x = vec![0.0, 1.0, 2.0, 3.0, 4.0, 205.0];
//         // let y = vec![20.0, 4.0, 3.0, 2.0, 1.0, 0.0];
//         //println!("{:?}", x.iter().cloned().fold(0./0., f32::max));

//         let plt: PlotPoints =
//             (0..x.len()).map(|i| [x[i] as f64, y[i] as f64]).collect();

//         let planned_line = Line::new(plt);
//         plot_ui.line(planned_line);
//     });
//     //self.data_ready = 0;
// }

// let plot = Plot::new("lines")
//     .include_x(0.0)
//     .include_y(0.0)
//     .show_axes([false; 2])
//     .show_background(false)
//     .width(500.0)
//     .height(250.0)
//     .legend(Legend::default());
// plot.show(ui, |plot_ui| {
//     let x = [0.0, 1.0, 2.0, 3.0, 4.0, 5.0];
//     let y = [5.0, 4.0, 3.0, 2.0, 1.0, 0.0];

//     let plt: PlotPoints = (0..x.len()).map(|i| [x[i], y[i]]).collect();

//     let planned_line = Line::new(plt);
//     plot_ui.line(planned_line);
//     let sin: PlotPoints = (0..1000)
//         .map(|i| {
//             let x = i as f64 * 0.01;
//             [x, x.sin()]
//         })
//         .collect();
//     // println!("{:?}",sin );
//     let planned_line = Line::new(sin).fill(0.0);
//     plot_ui.line(planned_line);
//     let r: f64 = 5.0;

//     let circle: PlotPoints = (0..100)
//         .map(|i| {
//             let t = i as f64 * 0.01;
//             [0.0 + r * t.cos(), 0.0 + r * t.sin()]
//         })
//         .collect();
//     let planned_line = Line::new(circle);
//     plot_ui.line(planned_line);

//     // let planned_line = Line::new(series.into_iter().map(|x|x)).fill(0.0);
//     // let planned_line = Line::new(PlotPoints::from_iter(series.into_iter()));
//     // plot_ui.line(planned_line);
// });
