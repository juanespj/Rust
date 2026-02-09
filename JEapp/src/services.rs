use crate::RenderApp;
use crate::ppk2srvc::*;
use crate::sersys::*;
use chrono::{DateTime, Utc};
use device_query::{DeviceQuery, DeviceState, Keycode};
use egui::*;
use itertools::Itertools;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{BufReader, prelude::*};
use std::os::windows::process::CommandExt;
use std::process::{Command, Stdio};
use std::sync::{Arc, RwLock, mpsc};
use std::{
    mem, str, thread,
    time::{Duration, SystemTime},
};

use sysinfo::System;
use sysinfo::SystemExt;
// use term_painter::Attr::*;
use sysinfo::ProcessExt;
use term_painter::Color::*;
use term_painter::ToStyle;
#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize, PartialOrd)]
pub enum ServicesRequest {
    TRANSFERSWT,
    READSWT,
    DELAYEDREADSWT,
    SWTDEEPSLEEP,
    SWTERASELORACRED,
    WRITESWT,
    SETSWTDEFAULTS,
    INITSERIAL,
    STARTSERIALMONITOR,
    STOPSERIALMONITOR,
    RESETSERIAL,
    CLEARSERIAL,
    STARTFLASHER,
    STARTCLEANFLASH,
    LAUNCHPRINTER,
    CONFIGPRINTER,
    STOPPRINTER,
    SENDPRINT,
    STOP,
    INITPPK2,
    MONITORPPK2,
    STOPPPK2,
}
#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize, PartialOrd)]
pub enum ServicesSysState {
    STOPPED,
    INIT,
    RUNNING,
    KILL,
}
#[derive(Debug, Clone, PartialEq)]
pub enum KeyShortcuts {
    NULL,
    DEVEUI,       //[Keycode::D, Keycode::LControl]
    APPKEY,       //[Keycode::A, Keycode::LControl]
    UPDATECSV,    //[Keycode::S, Keycode::LControl]
    FLASH,        //[Keycode::F, Keycode::LControl, Keycode::LShift]
    CLEARFLASH,   //[Keycode::E, Keycode::LControl]
    LOWPOWER,     //[Keycode::D, Keycode::LControl, Keycode::LShift]
    PRINTLABEL,   //[Keycode::P, Keycode::LControl]
    FINDFREE,     //[Keycode::X, Keycode::LControl],
    TRANSFERCRED, //[Keycode::T, Keycode::LControl],
    READSWT,      //[Keycode::R, Keycode::LControl],
    WFNEXT,
    WFBACK,
}

pub struct ServiceCtrl {
    pub keysct: Arc<RwLock<KeyShortcuts>>,
    device_state: DeviceState,

    pub state: Arc<RwLock<ServicesSysState>>,
    pub serial_svc: Arc<RwLock<SerSys>>,
    pub threads: Vec<thread::JoinHandle<()>>,
    pub paths: HashMap<String, String>,
    pub ppk2: Arc<RwLock<PPK2Srvc>>,
    pub ppk2_started: bool,
}

impl Default for ServiceCtrl {
    fn default() -> Self {
        let service_ctrl = Self {
            paths: HashMap::new(),
            device_state: DeviceState::new(),
            keysct: Arc::new(RwLock::new(KeyShortcuts::NULL)),
            state: Arc::new(RwLock::new(ServicesSysState::INIT)),
            serial_svc: Arc::new(RwLock::new(SerSys::new())),
            threads: Vec::new(),
            ppk2: Arc::new(RwLock::new(PPK2Srvc::new())),
            ppk2_started: false,
        };
        service_ctrl
    }
}
impl ServiceCtrl {
    pub fn get_keys(&mut self) -> KeyShortcuts {
        self.keysct.read().unwrap().clone()
    }
    pub fn set_keys(&mut self, keys: KeyShortcuts) {
        //  println!("Set Keys: {:?}", keys);
        *self.keysct.write().unwrap() = keys.clone();
    }

    pub fn get_states(&self) -> HashMap<String, String> {
        HashMap::from([
            (
                "SerState".into(),
                format!(
                    "{:?}",
                    self.serial_svc.read().unwrap().threadstate.read().unwrap()
                ),
            ),
            (
                "PPK2".into(),
                format!("{:?}", self.ppk2.read().unwrap().state.read().unwrap()),
            ),
        ])
    }
    pub fn get_datain_len(&mut self) -> usize {
        // self.serial_svc
        //     .write()
        //     .unwrap()
        //     .datain
        //     .read()
        //     .unwrap()
        //     .len()
        if let Some(index) = self
            .serial_svc
            .write()
            .unwrap()
            .datain
            .read()
            .unwrap()
            .rfind("\r\n")
        {
            index
        } else {
            0
        }
    }
    pub fn get_state(&mut self) -> ServicesSysState {
        self.state.read().unwrap().clone()
    }
    pub fn set_state(&mut self, state: ServicesSysState) {
        *self.state.write().unwrap() = state;
    }

    pub fn get_paths(&self) -> HashMap<String, String> {
        self.paths.clone()
    }

    pub fn get_ppk_data(&mut self) -> HashMap<String, Vec<f64>> {
        self.ppk2.write().unwrap().data.read().unwrap().clone()
    }

    pub fn serial_write(&mut self, msg: &str) {
        let mut serial = self.serial_svc.write().unwrap();
        serial.writebuff(msg.to_string());
        *serial.threadstate.write().unwrap() = SerState::WRITE;
    }
    pub fn serial_get_state(&mut self) -> [SerState; 2] {
        self.serial_svc.write().unwrap().get_state().clone()
    }

    pub fn serial_stop_mon(&mut self) {
        let mut serial = self.serial_svc.write().unwrap();
        serial.stopmon();
        *serial.threadstate.write().unwrap() = SerState::STOP;
    }
    pub fn serial_start_mon(&mut self) {}
    pub fn serial_get_port_list(&self) -> Vec<String> {
        let mut ser = self.serial_svc.write().unwrap();
        ser.listports();
        ser.get_portlist()
    }
    pub fn serial_get_sel_port(&self) -> String {
        self.serial_svc.write().unwrap().get_selport().clone()
    }
}

pub fn req(servreq_rx: &mpsc::Sender<ServicesRequest>, req: ServicesRequest) {
    if let Err(req_res) = servreq_rx.send(req) {
        log::debug!("[SRVC] Request : {:?} {:?}", req, req_res);
    } else {
        log::debug!("[SRVC] Request : {:?} OK", req,);
    }
}

pub fn launch_services(app: &mut RenderApp) {
    let builder = thread::Builder::new();
    let msgtx = app.msgchan.0.clone();
    let serv_arc = Arc::clone(&app.services);
    let (_, dummy_rx) = mpsc::channel();

    // 2. Replace the real Receiver with the dummy_rx, and take ownership of the real one
    // This moves the real Receiver out of the struct.
    let real_rx = mem::replace(&mut app.srvcrequest.1, dummy_rx);
    app.threads.push(
        builder
            .spawn(move || service_handle(serv_arc, real_rx, msgtx))
            .unwrap(),
    );
    print!("\r\n[SRVC] Services Thread Running")
}

const DEVICESHORTCUTS: &str =
    "Ctrl+Shift+F,Flash!\nCtrl+E,Clear Flash\nCtrl+Shift+D,Toggle DeepSleep";
const APPSHORTCUTS: &str = "Ctrl+D,Jump to DEVEUI\nCtrl+A,Jump to APPKEY and Generate if Empty\nCtrl+F,Find Free\nCtrl+F,Print Label\nCtrl+S,Update CSV\nCtrl+T,Transfer Credentials\nCtrl+R,Read SWT";

pub fn keyshortcuts_guide_ui(ui: &mut Ui) {
    ui.label(RichText::new("Device Controls").color(Color32::from_rgb(110, 255, 110)));
    egui::Grid::new("Dev_shrotcuts_guide")
        .num_columns(12)
        .spacing(egui::Vec2::new(5.0, 5.0))
        .striped(true)
        .show(ui, |ui| {
            let lines = DEVICESHORTCUTS.split("\n").collect::<Vec<&str>>();
            for line in lines {
                let it = line.split(",").collect::<Vec<&str>>();
                let elems = it[0].split("+").collect::<Vec<&str>>();
                let mut i = elems.len();
                ui.horizontal(|ui| {
                    for elem in elems {
                        ui.label(RichText::new(elem).color(Color32::from_rgb(110, 110, 225)));
                        if i > 1 {
                            ui.label("+");
                        }
                        i -= 1;
                    }
                });
                ui.label(RichText::new(it[1]).italics());
                ui.end_row();
            }
        });
    ui.label(RichText::new("App Controls").color(Color32::from_rgb(110, 255, 110)));
    let lines = APPSHORTCUTS.split("\n").collect::<Vec<&str>>();
    egui::Grid::new("App_shrotcuts_guide")
        .num_columns(12)
        .spacing(egui::Vec2::new(5.0, 5.0))
        .striped(true)
        .show(ui, |ui| {
            for line in lines {
                let it = line.split(",").collect::<Vec<&str>>();
                let elems = it[0].split("+").collect::<Vec<&str>>();
                let mut i = elems.len();
                ui.horizontal(|ui| {
                    for elem in elems {
                        ui.label(RichText::new(elem).color(Color32::from_rgb(110, 110, 225)));
                        if i > 1 {
                            ui.label("+");
                        }
                        i -= 1;
                    }
                });
                ui.label(RichText::new(it[1]).italics());
                ui.end_row();
            }
        });
}

pub fn service_handle(
    serv_arc: Arc<RwLock<ServiceCtrl>>,
    servreq_rx: mpsc::Receiver<ServicesRequest>,
    msgtx: mpsc::Sender<(String, String, String)>,
) {
    serv_arc
        .write()
        .unwrap()
        .set_state(ServicesSysState::RUNNING);
    loop {
        if serv_arc.is_poisoned() {
            msgtx
                .send((
                    "SRVC".into(),
                    "ERR".into(),
                    "Services Thread Poisoned".to_string(),
                ))
                .unwrap();
            log::error!("[SRVC] Services Thread Poisoned");
            serv_arc.clear_poison();
        } else {
            if (*serv_arc.read().unwrap().state.read().unwrap()) == ServicesSysState::KILL {
                break;
            }
        }
        let vec_keys = serv_arc.write().unwrap().device_state.get_keys();
        let keys = vec_keys.as_slice();
        if !keys.is_empty() {
            // println!("DEVUI: {:?}", keys);
            let keyout = match keys {
                [Keycode::D, Keycode::LControl] => KeyShortcuts::DEVEUI,
                [Keycode::A, Keycode::LControl] => KeyShortcuts::APPKEY,
                [Keycode::S, Keycode::LControl] => KeyShortcuts::UPDATECSV,
                [Keycode::E, Keycode::LControl] => KeyShortcuts::CLEARFLASH,
                [Keycode::F, Keycode::LShift, Keycode::LControl] => KeyShortcuts::FLASH,
                [Keycode::D, Keycode::LShift, Keycode::LControl] => KeyShortcuts::LOWPOWER,
                [Keycode::F, Keycode::LControl] => KeyShortcuts::FINDFREE,
                [Keycode::T, Keycode::LControl] => KeyShortcuts::TRANSFERCRED,
                [Keycode::R, Keycode::LControl] => KeyShortcuts::READSWT,
                [Keycode::P, Keycode::LControl] => KeyShortcuts::PRINTLABEL,
                [Keycode::N, Keycode::LControl] => KeyShortcuts::WFNEXT,
                [Keycode::B, Keycode::LControl] => KeyShortcuts::WFBACK,
                _ => {
                    // The underscore '_' acts as a catch-all for any other case
                    KeyShortcuts::NULL
                }
            };
            serv_arc.write().unwrap().set_keys(keyout);
        }

        let mut b_launchflasher = false;
        let mut b_launchflashclear = false;
        let mut loginfo = "".to_string();
        if let Ok(request) = servreq_rx.recv() {
            Blue.with(|| {
                log::debug!("Request received: {:?}", request);
            });
            match request {
                ServicesRequest::READSWT => {
                    loginfo = "Read SWT Requested".to_string();
                    serv_arc.write().unwrap().serial_write("c");
                }
                ServicesRequest::DELAYEDREADSWT => {
                    thread::sleep(Duration::from_millis(300));
                    loginfo = "Delayed Read SWT Requested".to_string();
                    serv_arc.write().unwrap().serial_write("c");
                }
                ServicesRequest::SWTDEEPSLEEP => {
                    loginfo = "SWT Enable Deep Sleep Request".to_string();
                    serv_arc.write().unwrap().serial_write("d");
                }
                ServicesRequest::SWTERASELORACRED => {
                    loginfo = "SWT Clear Lora Request".to_string();
                    serv_arc.write().unwrap().serial_write("l");
                }
                ServicesRequest::TRANSFERSWT => {
                    loginfo = "Transfer SWT Requested".to_string();
                    let service = serv_arc.write().unwrap();
                    let serial = service.serial_svc.write().unwrap();
                    *serial.threadstate.write().unwrap() = SerState::WRITE;
                }
                ServicesRequest::WRITESWT => {
                    loginfo = "WRITE SWT Requested".to_string();
                    serv_arc.write().unwrap().serial_write("u");
                }
                ServicesRequest::SETSWTDEFAULTS => {
                    loginfo = "SWT Set ResetDefaults".to_string();
                    serv_arc.write().unwrap().serial_write("r");
                }
                ServicesRequest::STARTSERIALMONITOR => {
                    loginfo = "Launch Monitor Requested".to_string();
                    let mut service = serv_arc.write().unwrap();
                    let [serialctrl, serthread] = service.serial_svc.read().unwrap().get_state();

                    if serialctrl == serthread {
                        if serialctrl != SerState::INIT
                            && serialctrl != SerState::KILL
                            && serialctrl != SerState::ERROR
                        {
                            service.serial_stop_mon();
                        }
                    }
                    service.serial_svc.write().unwrap().startmon(msgtx.clone());
                }
                ServicesRequest::STOPSERIALMONITOR => {
                    loginfo = "Stop Monitor Requested".to_string();
                    serv_arc.write().unwrap().serial_stop_mon();
                }

                ServicesRequest::INITSERIAL => {
                    loginfo = "Serial Init".to_string();
                    serv_arc.write().unwrap().serial_get_port_list();
                }
                ServicesRequest::RESETSERIAL => {
                    loginfo = "Serial Init".to_string();
                    let states = serv_arc.write().unwrap().get_states();
                    let service = serv_arc.write().unwrap();

                    let serial = service.serial_svc.write().unwrap();
                    if states.get("SerState").unwrap() == "MONITOR" {
                        println!("Serial Reset Requested while Monitoring");

                        *serial.threadstate.write().unwrap() = SerState::RESET;
                    } else {
                        serial.rereset_serial();
                        println!("Serial Reset Requested");
                    }
                }
                ServicesRequest::CLEARSERIAL => {
                    loginfo = "Serial Clear".to_string();
                    let service = serv_arc.write().unwrap();
                    let serial = service.serial_svc.write().unwrap();
                    *serial.threadstate.write().unwrap() = SerState::CLEAR;
                }
                ServicesRequest::STOP => {
                    *serv_arc.write().unwrap().state.write().unwrap() = ServicesSysState::KILL;

                    serv_arc.write().unwrap().serial_stop_mon();
                    serv_arc
                        .write()
                        .unwrap()
                        .ppk2
                        .write()
                        .unwrap()
                        .stop_monitor();
                    chk_running("node", true);
                }

                ServicesRequest::MONITORPPK2 => {
                    if *serv_arc
                        .write()
                        .unwrap()
                        .ppk2
                        .write()
                        .unwrap()
                        .state
                        .read()
                        .unwrap()
                        != PPK2State::MONITOR
                    {
                        loginfo = "Monitor PPK2".to_string();

                        serv_arc.write().unwrap().ppk2.write().unwrap().init_ppk2();
                        if serv_arc
                            .write()
                            .unwrap()
                            .ppk2
                            .write()
                            .unwrap()
                            .start_monitor(msgtx.clone())
                            == false
                        {
                            log::debug!("\r\n[SRVC] PPK2 Monitor Failed");
                        }
                    } else {
                        log::debug!("\r\n[SRVC] PPK2 Monitor Already running");
                    }
                }
                ServicesRequest::STOPPPK2 => {
                    log::debug!("PPK2 Stopped");
                    loginfo = "PPK2 Stopped".to_string();
                    serv_arc
                        .write()
                        .unwrap()
                        .ppk2
                        .write()
                        .unwrap()
                        .stop_monitor();
                }
                _ => {} //ServicesRequest::NONE => {}
            }
        }
        if !loginfo.is_empty() {
            let datetime: DateTime<Utc> = SystemTime::now().into();
            Blue.with(|| {
                log::debug!("\r\n{} {}", datetime.format("%T"), loginfo);
            });
        }

        thread::sleep(std::time::Duration::from_micros(200));
    }
    let services = serv_arc.write().unwrap();
    *services
        .serial_svc
        .write()
        .unwrap()
        .threadstate
        .write()
        .unwrap() = SerState::KILL;
    let datetime: DateTime<Utc> = SystemTime::now().into();
    Red.with(|| {
        log::debug!("\r\n{} Services Thread DOWN", datetime.format("%T"));
    });
}

pub fn servic_gui(ui: &mut Ui, sys_arc: Arc<RwLock<ServiceCtrl>>) {
    let states = sys_arc.write().unwrap().get_states();
    //   ui.vertical(|ui| {
    for (serv, state) in states.iter().sorted_by_key(|x| x.0) {
        ui.horizontal(|ui| {
            ui.label(serv);
            ui.colored_label(Color32::WHITE, state);
        });
    }

    //  });
}

pub fn chk_running(name: &str, stop: bool) -> bool {
    let s = System::new_all();
    for process in s.processes_by_name(name.as_ref()) {
        println!("{:?} - running", name);
        if stop {
            process.kill();
            println!("{:?} - killed", name);
        }
        return true;
    }
    println!("{:?} - not running", name);
    return false;
}

pub fn run_service(service: &str, path: String, cmd: String, check: bool) -> bool {
    let mut wdstate = false;
    let success = Arc::new(RwLock::new(false));
    if check {
        wdstate = chk_running(service, false);
    }

    if wdstate == false {
        println!("launching {}", service);
        let args = cmd.clone() + &path;
        let c_success = Arc::clone(&success);
        thread::spawn(
            move || {
                // let res = Command::new("dir") //.//chromedriver.exe
                //     .output()
                //     .expect("failed to execute process");
                // println!("{:?}", res.stdout.as_slice());
                let mut child = Command::new("cmd")
                    .raw_arg(args)
                    // .arg("/c")
                    // .arg(&cmd)
                    // .arg("start")
                    // .arg("--prefix D:/Projects/Symphony/zpl-to-rest-swt/")
                    .stdout(Stdio::piped())
                    .stdin(Stdio::piped())
                    .spawn()
                    .expect("error");
                // "python//n//r"
                // Create a handle and writer for the stdin of the second process
                // let mut outstdin = child.stdin.take().unwrap();
                // outstdin.write(idf_tool.as_bytes()).unwrap();
                // // let mut writer = BufWriter::new(&mut outstdin);
                thread::sleep(Duration::from_secs(1));
                // Loop over the output from the first process
                let mut waiting = true;
                let mut err = false;
                while waiting {
                    if let Some(ref mut stdout) = child.stdout {
                        for line in BufReader::new(stdout).lines() {
                            let l: String = line.unwrap();
                            println!("{}", l);
                            if l.contains("Websocket is listening on port") {
                                waiting = false;
                                println!("Node Service ready.. ");
                            }
                            if l.contains("npm ERR!") {
                                waiting = false;
                                println!("Node Service Error.. ");
                                err = true;
                                break;
                            }
                        }
                    }
                    if let Err(e) = child.kill() {
                        err = true;
                        println!("error stoping.{}", e);
                    };
                }
                if !err {
                    *c_success.write().unwrap() = true;
                }
            }, // some work here
        );
        println!();
        if !*success.read().unwrap() {
            println!("LaunchError");
            return false;
        }
        //println!("Launched Driver {}", service);

        let mut timeout = 3;
        while chk_running(service, false) == false {
            thread::sleep(std::time::Duration::from_millis(100));
            timeout -= 1;
            if timeout == 0 {
                break;
            }
            println!("Waiting for {}", cmd);
        }

        println!("Launched [{}]", cmd);
    }
    return *success.read().unwrap();
}

pub fn srvclog(loginfo: String) {
    let datetime: DateTime<Utc> = SystemTime::now().into();
    Blue.with(|| {
        println!("\r\n{} {}", datetime.format("%T"), loginfo);
    });
}
