use crate::sys_tools::get_cwd;
use chrono::{Datelike, Timelike};
use serde_derive::{Deserialize, Serialize};
use serialport::{available_ports, DataBits, SerialPortType, StopBits}; //FlowControl
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::sync::{mpsc, Arc, RwLock};
use std::{str, thread, time::Duration};
// use term_painter::Color::*;
// use term_painter::ToStyle;
const BAUDRATE: u32 = 115200;

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize, PartialOrd)]
pub enum SerState {
    KILL,
    ERROR,
    STOP,
    CREATED,
    INIT,
    BUSY,
    READ,
    WRITE,
    RESET,
    MONITOR,
    CLEAR,
    SLICE,
    IDLE, //logged in
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortInfo {
    pub num: String,
    pub man: String,
    pub desc: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct SerSys {
    pub state: SerState,
    pub threadstate: Arc<RwLock<SerState>>,
    pub status: HashMap<String, Vec<String>>,
    pub portlist: Vec<PortInfo>,
    pub port_sel: String,
    pub log: bool,
    #[serde(skip)]
    pub datain: Arc<RwLock<String>>,
    #[serde(skip)]
    pub dataout: Arc<RwLock<String>>,
    #[serde(skip)]
    threads: Vec<thread::JoinHandle<()>>,
    pub filename: String,
    pub msg: String,
}

impl SerSys {
    pub fn new() -> Self {
        Self {
            state: SerState::CREATED,
            threadstate: Arc::new(RwLock::new(SerState::CREATED)),
            status: HashMap::new(),
            portlist: vec![],
            port_sel: "-".to_string(),
            datain: Arc::new(RwLock::new("".to_string())),
            dataout: Arc::new(RwLock::new("".to_string())),
            threads: Vec::new(),
            log: false,
            filename: "".to_string(),
            msg: "".to_string(),
        }
    }
    pub fn listports(&mut self) {
        self.portlist.clear();
        let ppk2 = get_ppk2();
        match available_ports() {
            Ok(ports) => {
                match ports.len() {
                    0 => println!("\nNo ports found."),
                    1 => println!("\nFound 1 port:"),
                    n => println!("\nFound {} ports:", n),
                };
                for p in ports {
                    println!("  {}", p.port_name);

                    match p.port_type {
                        SerialPortType::UsbPort(info) => {
                            // println!("    Type: USB");

                            // let mfg = info.manufacturer.as_ref().map_or("", String::as_str);

                            // println!("COM{} ", p.port_name.to_string());

                            // println!("    VID:{:04x} PID:{:04x}", info.vid, info.pid);
                            // println!(
                            //     "     Serial Number: {}",
                            //     info.serial_number.as_ref().map_or("", String::as_str)
                            // );
                            // println!(
                            //     "Manuf: {}",
                            //     info.manufacturer.as_ref().map_or("", String::as_str)
                            // );
                            // println!(
                            //     "Product: {}",
                            //     info.product.as_ref().map_or("", String::as_str)
                            // );
                            let product = info.product.as_ref().map_or("", String::as_str);
                            if product.contains("USB Serial Port") {
                                self.portlist.push(PortInfo {
                                    num: p.port_name,
                                    desc: info.product.clone().unwrap_or("".to_string()),
                                    man: info.manufacturer.clone().unwrap_or("".to_string()),
                                });
                            } else {
                                println!("Port Skipped: \r\nCOM{} ", p.port_name.to_string());

                                println!("    VID:{:04x} PID:{:04x}", info.vid, info.pid);
                                println!(
                                    "     Serial Number: {}",
                                    info.serial_number.as_ref().map_or("", String::as_str)
                                );
                                println!(
                                    "Manuf: {}",
                                    info.manufacturer.as_ref().map_or("", String::as_str)
                                );
                                println!(
                                    "Product: {}",
                                    info.product.as_ref().map_or("", String::as_str)
                                );
                            }
                            // println!(
                            //     " Interface: {}",
                            //     info.
                            //         .as_ref()
                            //         .map_or("".to_string(), |x| format!("{:02x}", *x))
                            // );
                        }
                        SerialPortType::BluetoothPort => {
                            println!("    Type: Bluetooth");
                        }
                        SerialPortType::PciPort => {
                            println!("    Type: PCI");
                        }
                        SerialPortType::Unknown => {
                            println!("    Type: Unknown");
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("{:?}", e);
                eprintln!("Error listing serial ports");
            }
        }
        if ppk2.is_some() {
            let rem_port = ppk2.unwrap().clone();
            self.portlist.retain(|value| !value.num.eq(&rem_port));
        }
    }

    pub fn get_portlist(&self) -> Vec<String> {
        let mut list: Vec<String> = vec![];
        for it in self.portlist.iter() {
            list.push(it.num.clone());
        }
        return list;
    }
    pub fn select_port(&mut self, sel_prt: String) {
        // let mut relaunch_monitoring = false;
        if *self.threadstate.read().unwrap() > SerState::BUSY {
            *self.threadstate.write().unwrap() = SerState::KILL;
            thread::sleep(Duration::from_millis(100));
            // relaunch_monitoring = true;
        }
        self.port_sel = sel_prt;
        // if relaunch_monitoring {
        //     self.startmon();
        // }
    }

    pub fn get_state(&self) -> [SerState; 2] {
        return [self.state.clone(), *self.threadstate.read().unwrap()];
    }
    pub fn get_selport(&self) -> String {
        return self.port_sel.clone();
    }
    pub fn rereset_serial(&self) {
        let stop_bits = StopBits::One;
        let data_bits = DataBits::Eight;
        let baud_rate: u32 = 115200;
        let port_name = self.port_sel.clone();
        let builder = serialport::new(port_name.clone(), baud_rate)
            .stop_bits(stop_bits)
            .data_bits(data_bits)
            .dtr_on_open(true);

        // let mut port = TTYPort::open(&serialport::new(slave.name().unwrap(), 0)).expect("unable to open");
        let mut serial_buf: Vec<u8> = vec![0; 1000];
        match builder.timeout(Duration::from_millis(500)).open() {
            Ok(mut port) => match port.read(serial_buf.as_mut_slice()) {
                Ok(_) => {}
                Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                Err(e) => {
                    eprintln!("{:?}", e)
                }
            },
            Err(e) => {
                eprintln!("Failed to open \"{}\". Error: {}", &port_name, e);
            }
        }
        let builder = serialport::new(port_name.clone(), baud_rate)
            .stop_bits(stop_bits)
            .data_bits(data_bits)
            .dtr_on_open(false);
        match builder.timeout(Duration::from_millis(500)).open() {
            Ok(mut port) => match port.read(serial_buf.as_mut_slice()) {
                Ok(_) => {}
                Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                Err(e) => eprintln!("{:?}", e),
            },
            Err(e) => {
                eprintln!("Failed to open \"{}\". Error: {}", &port_name, e);
            }
        }
    }
    pub fn readserial_str(port_name: &String) -> String {
        // println!("reading :{:?}:", port_name);
        if !port_name.eq("-") {
            let stop_bits = StopBits::One;
            let data_bits = DataBits::Eight;
            let baud_rate: u32 = 115200;
            let builder = serialport::new(port_name.clone(), baud_rate)
                .stop_bits(stop_bits)
                .data_bits(data_bits)
                .dtr_on_open(false);

            // let mut port = TTYPort::open(&serialport::new(slave.name().unwrap(), 0)).expect("unable to open");
            match builder.timeout(Duration::from_millis(500)).open() {
                Ok(mut port) => {
                    let mut serial_buf: Vec<u8> = vec![0; 1000];
                    //println!("Receiving data on {} at {} baud:", &port_name, &baud_rate);
                    let mut dataout: String = "".to_string();

                    // while port.bytes_to_read().unwrap() > 0 {
                    // println!("bytes {:?} ", port.bytes_to_read().unwrap());
                    match port.read(serial_buf.as_mut_slice()) {
                        Ok(t) => {
                            // io::stdout().write_all(&serial_buf[..t]).unwrap();
                            dataout += &String::from_utf8_lossy(&serial_buf[..t]).to_string();
                            if dataout.len() > 0 {
                                // println!("data:{:?}", dataout);

                                return dataout;
                            }
                        }
                        Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                        Err(e) => eprintln!("{:?}", e),
                    }
                    // if dataout.contains("Entering WEB Server Mode"){
                    // break;
                    // }
                    // }
                }
                Err(e) => {
                    eprintln!("Failed to open \"{}\". Error: {}", &port_name, e);
                    // ::std::process::exit(1);
                }
            }
        } else {
            println!("select a Port");
        }
        return "".to_string();
    }

    pub fn startmon(&mut self, msgtx: mpsc::Sender<(String, String, String)>) {
        print!("\r\n[SER] Serial Monitoring Service Starting...");
        *self.threadstate.write().unwrap() = SerState::KILL;
        let _ = self.getdatain();
        let datain_c = Arc::clone(&self.datain);
        let dataout_c = Arc::clone(&self.dataout);
        *self.threadstate.write().unwrap() = SerState::MONITOR;
        let state_r = Arc::clone(&self.threadstate);
        // let datain_w = self.datain.write().unwrap();
        let builder = thread::Builder::new();
        let tmpselport = self.port_sel.clone();
        let path = format!("{}logs\\", get_cwd());
        let time = chrono::Utc::now();
        self.filename = format!(
            "{}serial_{}.{}.{}_{}h{}.log",
            path,
            time.year(),
            time.month(),
            time.day(),
            time.hour(),
            time.minute() + time.second() * 60
        );

        println!("New Log File{}", path);
        if !std::path::Path::new(&path).exists() {
            std::fs::create_dir(path).expect("path failed");
        }
        //  std::fs::create_dir_all(prefix).expect("creation failed");
        if let Err(_) = std::fs::File::create(&self.filename) {
            msgtx
                .send((
                    "Serial".into(),
                    "ERR".into(),
                    format!("Error Creating Serial Log file: {:?}", self.filename).to_string(),
                ))
                .unwrap();
        };
        //drop(data_file);
        println!("\r\n[SER] New Log File: {}", self.filename);
        let logfile = self.filename.to_string().clone();
        self.threads.push(
            builder
                .spawn(move || {
                    arc2readserial(datain_c, dataout_c, &tmpselport, state_r, logfile, msgtx)
                })
                .unwrap(),
        );
        print!("\r\n[SER] Serial Monitoring Launched");
    }

    pub fn stopmon(&mut self) {
        *self.threadstate.write().unwrap() = SerState::KILL;
        let mut ix = 0;
        while ix < self.threads.len() && self.threads.len() > 0 {
            if self.threads[ix].is_finished() {
                self.threads.remove(ix);
            }
            ix += 1;
        }
        self.state = SerState::KILL;
    }

    //drain serial in up to the last line feed
    pub fn getdatain(&mut self) -> String {
        // self.datain.write().unwrap().drain(..).collect()
        let index = self.datain.read().unwrap().rfind("\n");

        match index {
            Some(idx) => {
                // The index `idx` is the start of the "\r\n" sequence.
                // We want to drain everything *before* this point.
                // The length of "\r\n" is 2 bytes.
                let drain_until_byte = idx + 1;

                // 2. Use drain to remove everything up to and including the last "\r\n".
                // The range `..drain_until_byte` is everything from the start up to,
                // but not including, the determined index.
                self.datain
                    .write()
                    .unwrap()
                    .drain(..drain_until_byte)
                    .collect()
            }
            None => {
                // No "\r\n" found. Drain the entire string, leaving it empty.
                // This is equivalent to `std::mem::take(s)`.
                //std::mem::take(s)
                "".to_string()
            }
        }
    }
    pub fn getdataout(&mut self) -> String {
        self.dataout.write().unwrap().drain(..).collect()
    }
    pub fn writebuff(&mut self, msg: String) -> bool {
        if self.dataout.read().unwrap().is_empty() {
            *self.dataout.write().unwrap() = msg;
            return true;
        }
        false
    }
    pub fn sendserial(sys: &mut SerSys) {
        let port_name = sys.status.get("sel").unwrap().to_vec()[0].clone();
        if port_name != "-".to_string() {
            let msg = &sys.status.get("write").unwrap().to_vec()[0];
            let stop_bits = StopBits::One;
            let data_bits = DataBits::Eight;
            let baud_rate: u32 = 115200;

            let builder = serialport::new(port_name.clone(), baud_rate)
                .stop_bits(stop_bits)
                .data_bits(data_bits)
                .dtr_on_open(false);

            // println!("{:?}", &builder);
            match builder.timeout(Duration::from_millis(10)).open() {
                Ok(mut port) => {
                    println!(
                        "Writing '{}' to {} at {} baud ",
                        &msg, &port_name, &baud_rate
                    );

                    match port.write(msg.as_bytes()) {
                        Ok(_) => {
                            print!("Write{}", &msg);
                            std::io::stdout().flush().unwrap();
                        }
                        Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                        Err(e) => eprintln!("{:?}", e),
                    }
                }
                Err(e) => {
                    eprintln!("Failed to open \"{}\". Error: {}", &port_name, e);

                    // ::std::process::exit(1);
                }
            }
        } else {
            println!("select a Port");
        }
        //std::thread::sleep(Duration::from_millis((1000.0 / (rate as f32)) as u64));
    }
}

pub fn get_ppk2() -> Option<String> {
    match available_ports() {
        Ok(ports) => {
            for p in ports {
                match p.port_type {
                    SerialPortType::UsbPort(info) => {
                        if info
                            .product
                            .as_ref()
                            .map_or("", String::as_str)
                            .contains("nRF Connect USB")
                            && info.vid == 0x1915
                            && info.pid == 0xc00a
                        {
                            return Some(p.port_name);
                        }
                    }
                    _ => {}
                }
            }
        }
        Err(_) => {}
    }
    None
}

pub fn arc2readserial(
    dataout: Arc<RwLock<String>>,
    datain: Arc<RwLock<String>>,
    port_name: &String,
    serialctrl: Arc<RwLock<SerState>>,
    logfile: String,
    msgtx: mpsc::Sender<(String, String, String)>,
) {
    if *port_name != "-".to_string() {
        let mut loop_lock = 1;
        let stop_bits = StopBits::One;
        let data_bits = DataBits::Eight;
        let baud_rate: u32 = BAUDRATE;
        let mut serial_in: Vec<String> = vec![];

        let mut curstate = SerState::MONITOR;
        let builder = serialport::new(port_name.clone(), baud_rate)
            .stop_bits(stop_bits)
            .data_bits(data_bits)
            .dtr_on_open(false);
        match builder.timeout(Duration::from_millis(100)).open() {
            Ok(mut port) => {
                msgtx
                    .send((
                        "Serial".into(),
                        "INFO".into(),
                        "Serial Monitoring Started".to_string(),
                    ))
                    .unwrap();
                while loop_lock == 1 {
                    match curstate {
                        SerState::RESET => {
                            match port.write_data_terminal_ready(true) {
                                Ok(_) => {}
                                Err(e) => eprintln!("DTR set error: {:?}", e),
                            };
                            thread::sleep(Duration::from_millis(100));
                            match port.write_data_terminal_ready(true) {
                                Ok(_) => {}
                                Err(e) => eprintln!("DTR set error: {:?}", e),
                            };
                            thread::sleep(Duration::from_millis(100));
                            *serialctrl.write().unwrap() = SerState::MONITOR;
                        }
                        SerState::WRITE => {
                            let mut r_dataout = String::new();
                            r_dataout.extend(datain.write().unwrap().drain(..));
                            if r_dataout.len() > 0 {
                                match port.write(r_dataout.as_bytes()) {
                                    Ok(_) => {
                                        log::debug!("[Serial] serialwrite:{}", &r_dataout);
                                        std::io::stdout().flush().unwrap();
                                    }
                                    Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                                    Err(e) => eprintln!("{:?}", e),
                                }
                            }
                            *serialctrl.write().unwrap() = SerState::MONITOR;
                        }
                        SerState::MONITOR => {
                            let mut serial_buf = [0; 1500];
                            match &port.read(&mut serial_buf) {
                                Ok(t) => {
                                    let data_read =
                                        String::from_utf8_lossy(&serial_buf[..*t]).to_string();
                                    // Create a file
                                    let mut data_file = OpenOptions::new()
                                        .append(true)
                                        .open(&logfile)
                                        .expect("cannot open file");

                                    // Write to a file
                                    data_file.write(data_read.as_bytes()).expect("write failed");
                                    //send to ui
                                    serial_in.push(data_read);
                                }
                                Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {
                                    //   eprintln!("{:?}", e);
                                    // loop_lock = 0;
                                }
                                Err(e) => {
                                    eprintln!("{:?}", e);
                                    loop_lock = 0;
                                    *serialctrl.write().unwrap() = SerState::KILL;
                                }
                            }

                            if !serial_in.is_empty() {
                                let mut tmpstr = "".to_string();
                                let mut drain = serial_in.drain(..);
                                while drain.len() > 0 {
                                    tmpstr += &drain.next().unwrap().to_string();
                                }
                                dataout.write().unwrap().push_str(&tmpstr);
                            }
                        }
                        SerState::CLEAR => {
                            serial_in.clear();
                            datain.write().unwrap().clear();
                            dataout.write().unwrap().clear();
                            *serialctrl.write().unwrap() = SerState::MONITOR;
                        }

                        SerState::STOP | SerState::KILL | SerState::ERROR => {
                            loop_lock = 0;
                        }
                        SerState::IDLE | _ => {}
                    }
                    let state_r = serialctrl.read().unwrap();
                    curstate = *state_r;
                    thread::sleep(std::time::Duration::from_micros(100));
                }
            }
            Err(e) => {
                msgtx
                    .send((
                        "Serial".into(),
                        "ERR".into(),
                        format!("Failed to open \"{}\". Error: {}", &port_name, e).to_string(),
                    ))
                    .unwrap();
                log::error!("Failed to open \"{}\". Error: {}", &port_name, e);
                *serialctrl.write().unwrap() = SerState::ERROR;
                // ::std::process::exit(1);
            }
        }
    } else {
        println!("select a Port");
    }
    msgtx
        .send((
            "Serial".into(),
            "INFO".into(),
            "Serial Monitoring Stopped".to_string(),
        ))
        .unwrap();
    println!("MonitoringEnded");
}

#[cfg(test)]
mod services_tests {
    use serialport::{DataBits, StopBits}; //FlowControl
    use std::time::Duration;
    #[test]
    pub fn readserial_str() {
        // println!("reading :{:?}:", port_name);
        let port_name = "COM11".to_string();

        let stop_bits = StopBits::One;
        let data_bits = DataBits::Eight;
        let baud_rate: u32 = 115200;
        let builder = serialport::new(port_name.clone(), baud_rate)
            .stop_bits(stop_bits)
            .data_bits(data_bits)
            .dtr_on_open(true);

        // let mut port = TTYPort::open(&serialport::new(slave.name().unwrap(), 0)).expect("unable to open");
        let mut serial_buf: Vec<u8> = vec![0; 1000];
        match builder.timeout(Duration::from_millis(500)).open() {
            Ok(mut port) => match port.read(serial_buf.as_mut_slice()) {
                Ok(_) => {}
                Err(e) => eprintln!("{:?}", e),
            },
            Err(e) => {
                eprintln!("Failed to open \"{}\". Error: {}", &port_name, e);
            }
        }
        let builder = serialport::new(port_name.clone(), baud_rate)
            .stop_bits(stop_bits)
            .data_bits(data_bits)
            .dtr_on_open(false);
        match builder.timeout(Duration::from_millis(500)).open() {
            Ok(mut port) => match port.read(serial_buf.as_mut_slice()) {
                Ok(_) => {}
                Err(e) => eprintln!("{:?}", e),
            },
            Err(e) => {
                eprintln!("Failed to open \"{}\". Error: {}", &port_name, e);
            }
        }
    }
}
