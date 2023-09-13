pub mod sersys;
use crate::sersys::{listports, SerState, SerSys};
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex, RwLock};
use std::{
    thread,
    time::Duration, //Instant
};

use strp::*;

pub struct Mesagging {
    ser_ch: (Sender<SerSys>, Receiver<SerSys>),
}
fn main() {
    let mut datain: Arc<RwLock<String>> = Arc::new(RwLock::new("".to_string()));
    let mut ser = SerSys::default();
    let mut msgs = Mesagging {
        ser_ch: mpsc::channel::<SerSys>(),
    };
    let mut threads: Vec<thread::JoinHandle<()>> = Vec::new();
    let mut portlist = listports();
    let mut port_sel = "".to_string();
    if portlist.len() == 2 {
        port_sel = portlist[1].num.clone();
    }

    println!("Port {}", port_sel);
    let (tx_ser, rx_ser): (Sender<SerSys>, Receiver<SerSys>) = mpsc::channel::<SerSys>();
    let (tx_a, rx_a): (Sender<SerSys>, Receiver<SerSys>) = mpsc::channel::<SerSys>();
    msgs.ser_ch.0 = tx_a;
    msgs.ser_ch.1 = rx_ser;

    ser.status
        .insert("sel".to_string(), vec![port_sel.to_string()]);

    ser.status
        .insert("write".to_string(), vec!["r".to_string()]);
    sersys::flushserial(&mut ser);
    sersys::sendserial(&mut ser);
    let datain_c = Arc::clone(&datain);
    let builder = thread::Builder::new();
    threads.push(
        builder
            .spawn(move || sersys::arcreadserial(datain_c, tx_ser, rx_a))
            .unwrap(),
    );
    ser.status.insert("START".to_string(), vec!["".to_string()]);
    let mut loop_lock = 1;
    let mut newmsg = 1;
    let mut datalocal = "".to_string();
    while loop_lock == 1 {
        match msgs.ser_ch.1.try_recv() {
            Ok(msg) => {
               
                if msg.status.contains_key("dataav") {
                    //   println!("fromSer: {:?}", self.sersys.status.get("read").unwrap());
                    // println!("\nfromSer: {:?}", r2);
                    // println!("\nfromSer: ");
                    ser.status.insert("ack".to_string(), vec!["".to_string()]);
                    newmsg = 1;
                    // self.logger
                    //     .status
                    //     .entry("read".to_string())
                    //     .and_modify(|value| {
                    //         *value =
                    //             vec![value[0].clone() + &self.sersys.status.get("read").unwrap()[0]]
                    //     })
                    //     .or_insert(vec![self.sersys.status.get("read").unwrap()[0].clone()]);
                }
                if msg.status.contains_key("START") {
                    println!("start");
                }
                if msg.status.contains_key("STOP") {
                    println!("end");
                    let r2 = datain.read().unwrap();
                    datalocal = (*r2).to_string();
                    log_process(&datalocal);
                    // log_process(&*r2.clone());
                    
                    loop_lock = 0;
                }
            }
            Err(_) => { /* handle sender disconnected */ } //Err(TryRecvError::Empty) => { /* handle no data available yet */ }
        }
        if newmsg == 1 {
            if let Err(_) = msgs.ser_ch.0.send(ser.clone()) {
                println!("Ser has stopped listening.")
            }
            ser.status.clear();
            newmsg = 0;
        }
    }
}

pub fn log_process(log: &str) {
    if log.contains("STARTLOG") {
        let mut data: HashMap<String, Vec<f64>> = HashMap::new();
        let (_, mut tmp) = log.split_once("STARTLOG").unwrap();

        if tmp.contains("STOPLOG") {
            (tmp, _) = tmp.split_once("STOPLOG").unwrap();

            // println!("rawSTOP:{:?}", raw);
        }
        // let cnt = tmp.matches("\r\n").count();
        let mut i = 0;
        for row in tmp.split("\r\n") {
            if row.len() > 0 {
                let cnt = row.matches(',').count();
                if cnt > 2 || row.chars().last().unwrap() != '<' {
                    println!("err:{:?}", row);
                    continue;
                }
                let matched: Result<(String, u32, f32), _> = try_scan!(row =>"{},{},{}<");
                match matched {
                    Ok((var, time, val)) => {
                          println!("D:{} => {}", var, val);

                        if (var == "PH" || var == "W") && val <= 1500.0 {
                            let tkey = format!("t{}", var);
                            let tout = time;
                            // if data.contains_key(&tkey) {
                            //     let last = data.get(&tkey).unwrap().len() - 1;
                            //     let tlast = data.get(&tkey).unwrap()[last].clone();

                            // if tout < (tlast * 1000.0) as u32 {
                            //     tout = (tlast * 1000.0) as u32 + 5;
                            // }
                            // }

                            data.entry(tkey)
                                .or_insert_with(Vec::new)
                                .push(tout as f64 / 1000.0);
                            data.entry(format!("{}", var))
                                .or_insert_with(Vec::new)
                                .push(val as f64);
                        }
                    }
                    Err(_e) => {
                        println!("{:?}", row);
                    }
                }
            }
            i += 1;
            // assert_eq!(a + b, c);d
        }
        println!("data:{:?}", data);
    }
    // if raw.contains("LG") {
    //     lightgate = true;
    // }
}
//     // let dt = 0.1;
//     // let mut obj = logctrl.objstate.get("rbb").unwrap().clone();
//     // let mut data = logctrl.data.clone();
//     let mut lightgate = false;
//     // let x_sp = 3.0 * signum(((logctrl.anim_state.step as f64) * PI / 50.0).cos()); //square
//     if logctrl.status.contains_key("read") {
//         let mut buff = "".to_string();
//         if logctrl.status.contains_key("buffer") {
//             buff = logctrl.status.get("buffer").unwrap()[0].clone();
//             println!("buffer:{:?}", buff);
//             logctrl.status.remove_entry("buffer");
//         }
//         //  buff +&
//         let mut raw: String = logctrl.status.get("read").unwrap()[0].clone();

//         // println!("raw:{:?}", raw);
//         if raw.contains("STOPLOG") {
//             raw = raw.split("STOPLOG").collect();
//             logctrl.state = LoggerState::IDLE;
//             // println!("rawSTOP:{:?}", raw);
//         }
//         if raw.contains("STARTLOG") {
//             logctrl.state = LoggerState::MONTORING;

//             raw = raw.split("STARTLOG").collect::<Vec<&str>>()[1].to_string();
//         }
//         // if raw.contains("LG") {
//         //     lightgate = true;
//         // }

//         logctrl.status.remove_entry("read");

//         // let mut rdr = csv::Reader::from_reader(raw.as_bytes());
//         // for result in rdr.deserialize(Record) {
//         //     match result {
//         //         Ok(record) => println!("{:?}", record),
//         //         Err(_) => { /* handle sender disconnected */ }
//         //     }
//         //     //   let record: Record = result?;
//         //
//         // }
//         //println!("{:?}", raw);
//         logctrl.data.clear();
//         // if raw.matches("\r\n").count() > 1 {
//         //     let last = raw.split("\r\n").last().unwrap();
//         //     if last.len() > 0 {
//         //         if !matches!(last.chars().last().unwrap(), '<') {
//         //             logctrl
//         //                 .status
//         //                 .entry("buffer".to_string())
//         //                 .or_insert_with(Vec::new)
//         //                 .push(last.to_string());
//         //         }
//         //     }
//         // }
//         let cnt = raw.matches("\r\n").count();
//         let mut i = 0;
//
//     }
// }
