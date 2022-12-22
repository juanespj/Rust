use calamine::{open_workbook, DataType, Reader, Xlsx};
use std::collections::HashMap;

pub struct RawData {
    pub diag: HashMap<String, String>,
    pub dataf: HashMap<String, Vec<f32>>,
}

pub fn processdata(name: String, ds: &mut RawData) {
    println!("type: {:?}", name);
    let mut excel: Xlsx<_> = open_workbook(name).unwrap();
    // let mut diag: HashMap<String, String> = HashMap::new();
    let mut header: Vec<String> = vec![];
    let mut value: Vec<String> = vec![];
    // let mut dataf: HashMap<String, Vec<f32>> = HashMap::new();
    // let mut dataf: HashMap<&String, Vec<f64>> = HashMap::new();
    if let Some(Ok(r)) = excel.worksheet_range("Sheet1") {
        for (i, row) in r.rows().enumerate() {
            if i == 0 || i == 2 || i == 4 {
                // let mut items = row.split(",");
                for s in row {
                    if s != &DataType::Empty {
                        header.push(s.to_string());
                    }
                }
                // println!("{:?}", header)
                // println!("row={:?}, row[0]={:?}", row, );
            } else if i == 1 || i == 3 {
                // let mut items = row.split(",");
                for s in row {
                    if s != &DataType::Empty {
                        //let f: f64 = s.to_string().parse().unwrap();
                        value.push(s.to_string());
                    }
                }
                // println!("{:?}", value)
                // println!("row={:?}, row[0]={:?}", row, );
                for i in 0..header.len() {
                    &ds.diag.insert(header[i].to_string(), value[i].to_string());
                }
                header.clear();
                value.clear();
            } else if i == 6 {
                println!("{:?}", &ds.diag);
                for s in row {
                    if s != &DataType::Empty {
                        header.push(s.to_string());
                    }
                }

                let emptyvec: Vec<f32> = vec![0.0];
                let plt = header
                    .clone()
                    .into_iter()
                    .map(|x: String| (x, emptyvec.clone()));

                ds.dataf = HashMap::from_iter(plt);
                // println!("{:?}", dataf);
            } else if i > 7 {
                //let mut ix = 0;
                let mut ix = 0;
                for k in header.iter_mut() {
                    let f: f32 = row[ix].to_string().parse().unwrap();

                    ds.dataf
                        .entry(k.to_string())
                        .or_insert_with(Vec::new)
                        .push(f);

                    ix += 1;
                }
            }
        }
    }
}

use std::fs;
pub fn process_ogf(name: String) {
    let contents = fs::read_to_string(name).expect("Should have been able to read the file");
    let mut header: Vec<String> = vec![];
    let mut value: Vec<String> = vec![];
   // println!("{:?}", contents);
    // if let Some(Ok(r)) = excel.worksheet_range("Sheet1") {

    //     for (i, row) in r.rows().enumerate() {

    //             // println!("{:?}", dataf);
    //         } else if i > 7 {
    //             //let mut ix = 0;
    //             let mut ix = 0;
    //             for k in header.iter_mut() {
    //                 let f: f32 = row[ix].to_string().parse().unwrap();

    //                 ds.dataf.entry(k.to_string()).or_insert_with(Vec::new).push(f);

    //                 ix += 1;
    //             }
    //         }
    //     }

    // }
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

// fn newg_word(in:Char,&word:char){

// }
