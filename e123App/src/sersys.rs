// use futures::executor::block_on;
use serde_derive::{Deserialize, Serialize};
use serialport::{available_ports, DataBits, SerialPortType, StopBits};
use std::collections::HashMap;
use std::io::{self, Write};
// use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc, Mutex};
use std::time::Instant;
use std::{
    // error::Error,
    str,
    time::Duration, //{, SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum SerState {
    CREATED,
    BUSY,
    READ,
    WRITE,
    SCAN,
    MONITOR,
    IDLE, //logged in
    KILL,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortInfo {
    pub num: String,
    pub man: String,
    pub desc: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerSys {
    pub state: SerState,
    pub status: HashMap<String, Vec<String>>,
    pub portlist: Vec<PortInfo>,
}
impl Default for SerSys {
    fn default() -> Self {
        Self {
            state: SerState::CREATED,
            status: HashMap::new(),
            portlist: vec![],
        }
    }
}

impl SerSys {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn startserv(tx_ser: mpsc::Sender<SerSys>, rx_a: mpsc::Receiver<SerSys>) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let future = Self::sertask(tx_ser, rx_a);
        let _r = rt.block_on(future);
    }

    pub async fn arcsertask() {
        let mut sys = SerSys::default();

        sys.portlist = self::listports();
        // self::listports(&mut sys);

        let mut newmsg: u8 = 0;
        let mut loop_lock: u8 = 0;
        let mut now = Instant::now();
        while loop_lock == 0 {
            // loop_lock = 1;
            match sys.state {
                SerState::MONITOR => {
                    self::readserial(&mut sys);

                    if sys.status.contains_key("read") {
                        let tmpmsg = sys.status.get("read").unwrap()[0].clone();

                        // if tmpmsg.matches("\r\n").count() > 1 {
                        //     let last = tmpmsg.split("\r\n").last().unwrap();
                        //     println!("last{:?}", last);
                        //     if last.len() > 0 {
                        //         println!(
                        //         ":{:?}",
                        //         last.chars().last().unwrap()
                        //     );
                        //     }

                        // }
                        if tmpmsg.contains("START") {}
                        if tmpmsg.contains("STOP") {
                            sys.state = SerState::IDLE;
                        }
                        // sys.status.remove("read");
                        newmsg = 1;
                    }
                }
                _ => {}
            }
        }
        println!(">SERIAL STOP.");
    }

    pub async fn sertask(
        tx_ser: mpsc::Sender<SerSys>,
        rx_a: mpsc::Receiver<SerSys>,
    ) -> color_eyre::Result<()> {
        let mut sys = SerSys::default();

        sys.portlist = self::listports();
        // self::listports(&mut sys);

        let mut newmsg: u8 = 0;
        let mut loop_lock: u8 = 0;
        let mut now = Instant::now();
        while loop_lock == 0 {
            match rx_a.try_recv() {
                //when message from app is received
                Ok(msg) => {
                    println!("fromAPP: {:?}", msg.state);
                    match msg.state {
                        SerState::CREATED => {
                            sys.state = SerState::IDLE;
                            newmsg = 1;
                        }
                        SerState::WRITE => {
                            sys.status = msg.status.clone();
                            if sys.status.contains_key("write") {
                                self::sendserial(&mut sys);
                                sys.status.remove("write");
                                newmsg = 1;
                            }
                            sys.state = SerState::IDLE;
                        }
                        SerState::READ => {
                            sys.status = msg.status.clone();

                            self::readserial(&mut sys);
                            if sys.status.contains_key("read") {
                                newmsg = 1;
                            }
                            sys.state = SerState::IDLE;
                        }
                        SerState::MONITOR => {
                            sys.status = msg.status.clone();
                            sys.state = SerState::MONITOR;
                        }
                        SerState::SCAN => {
                            sys.state = SerState::SCAN;
                            sys.portlist = self::listports();
                            if sys.status.contains_key("list") {
                                newmsg = 1;
                            }
                        }
                        SerState::IDLE => {}
                        SerState::KILL => loop_lock = 1,
                        _ => {}
                    }
                }
                Err(_) => { /* handle sender disconnected */ }
            }
            // loop_lock = 1;
            match sys.state {
                SerState::MONITOR => {
                    self::readserial(&mut sys);

                    if sys.status.contains_key("read") {
                        let tmpmsg = sys.status.get("read").unwrap()[0].clone();

                        // if tmpmsg.matches("\r\n").count() > 1 {
                        //     let last = tmpmsg.split("\r\n").last().unwrap();
                        //     println!("last{:?}", last);
                        //     if last.len() > 0 {
                        //         println!(
                        //         ":{:?}",
                        //         last.chars().last().unwrap()
                        //     );
                        //     }
                        // }
                        if tmpmsg.contains("START") {}
                        if tmpmsg.contains("STOP") {
                            sys.state = SerState::IDLE;
                            println!("read:{:?}", sys.status.get("read").unwrap());
                        }
                        // sys.status.remove("read");
                        newmsg = 1;
                    }
                }
                _ => {}
            }
            if newmsg == 1 && now.elapsed().as_millis() > 100 {
                if let Err(_) = tx_ser.send(sys.clone()) {
                    println!("App not listening.")
                }
                // if sys.status.contains_key("read") {
                //     sys.status.remove("read");
                // }
                now = Instant::now();
                newmsg = 0;
            }
        }
        println!(">SERIAL STOP.");
        Ok(())
    }
}

pub fn listports() -> Vec<PortInfo> {
    let mut list: Vec<PortInfo> = vec![];
    match available_ports() {
        Ok(ports) => {
            match ports.len() {
                0 => println!("No ports found."),
                1 => println!("Found 1 port:"),
                n => println!("Found {} ports:", n),
            };
            for p in ports {
                //       println!("  {}", p.port_name);

                match p.port_type {
                    SerialPortType::UsbPort(info) => {
                        // println!("    Type: USB");
                        // println!("    VID:{:04x} PID:{:04x}", info.vid, info.pid);
                        // println!(
                        //     "     Serial Number: {}",
                        //     info.serial_number.as_ref().map_or("", String::as_str)
                        // );
                        let mfg = info.manufacturer.as_ref().map_or("", String::as_str);
                        list.push(PortInfo {
                            num: p.port_name,
                            desc: info.product.clone().unwrap_or("".to_string()),
                            man: info.manufacturer.clone().unwrap_or("".to_string()),
                        });
                        if mfg.contains("Cypress") {}
                        if mfg.contains("STMicroelectronics") {}
                        // println!(
                        //     "      Manufacturer: {}",
                        //     info.manufacturer.as_ref().map_or("", String::as_str)
                        // );
                        // println!(
                        //     "           Product: {}",
                        //     info.product.as_ref().map_or("", String::as_str)
                        // );
                        // println!(
                        //     "         Interface: {}",
                        //     info.interface
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
            if list.len() > 0 {
                //   println!("{:?}",list);
                return list;
            }
        }
        Err(e) => {
            eprintln!("{:?}", e);
            eprintln!("Error listing serial ports");
        }
    }
    return vec![];
}

fn readserial(sys: &mut SerSys) {
    let port_name = sys.status.get("sel").unwrap().to_vec()[0].clone();
    if port_name != "-".to_string() {
        let baud_rate: u32 = 115200;
        let port = serialport::new(port_name.clone(), baud_rate)
            .timeout(Duration::from_millis(10))
            .open();

        match port {
            Ok(mut port) => {
                let mut serial_buf: Vec<u8> = vec![0; 100];
                //println!("Receiving data on {} at {} baud:", &port_name, &baud_rate);
                let mut dataout: String = "".to_string();
                match port.read(serial_buf.as_mut_slice()) {
                    Ok(t) => {
                        //io::stdout().write_all(&serial_buf[..t]).unwrap();
                        dataout = String::from_utf8_lossy(&serial_buf[..t]).to_string();
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                    Err(e) => eprintln!("{:?}", e),
                }

                if dataout.len() > 0 {
                    //   println!("data:{:?}", dataout);
                    // sys.status.insert(
                    //     "read".to_string(),
                    //     vec![dataout],
                    // );
                    // sys.status
                    //     .entry("read".to_string())
                    //     .or_insert_with(Vec::new)
                    //     .
                    //     .push(dataout);
                    sys.status
                        .entry("read".to_string())
                        .and_modify(|value| *value = vec![value[0].clone() + &dataout])
                        .or_insert(vec![dataout]);
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
}

pub fn arcreadserial(sys: &SerSys, dataout: Arc<Mutex<String>>, ser_state: Arc<Mutex<u8>>) {
    let port_name = sys.status.get("sel").unwrap().to_vec()[0].clone();
    if port_name != "-".to_string() {
        let baud_rate: u32 = 115200;
        let port = serialport::new(port_name.clone(), baud_rate)
            .timeout(Duration::from_millis(10))
            .open();
        let mut serial_buf: Vec<u8> = vec![0; 100];
        //println!("Receiving data on {} at {} baud:", &port_name, &baud_rate);
        let mut dataouttmp: String = "".to_string();
        let mut looplock = true;

        match port {
            Ok(mut port) => {
                while looplock {
                    match &port.read(serial_buf.as_mut_slice()) {
                        Ok(t) => {
                            //io::stdout().write_all(&serial_buf[..t]).unwrap();
                            dataouttmp += &String::from_utf8_lossy(&serial_buf[..*t]).to_string();
                        }
                        Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                        Err(e) => eprintln!("{:?}", e),
                    }

                    if dataouttmp.len() > 0 {
                        if let Ok(mut data) = dataout.try_lock() {
                            // Step 2: check that data is not yet assigned.
                            data.push_str(&dataouttmp);
                        }

                        // println!("data:{:?}", dataout);
                    }
                    if let Ok( state) = ser_state.try_lock() {
                        // Step 2: check that data is not yet assigned.
                        if ser_state.lock().unwrap() > 1 {
                            looplock = false;
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to open \"{}\". Error: {}", &port_name, e);
                looplock = false;
                // ::std::process::exit(1);
            }
        }
    } else {
        println!("select a Port");
    }
}

fn sendserial(sys: &mut SerSys) {
    let port_name = sys.status.get("sel").unwrap().to_vec()[0].clone();
    if port_name != "-".to_string() {
        let msg = &sys.status.get("write").unwrap().to_vec()[0];
        let stop_bits = StopBits::One;
        let data_bits = DataBits::Eight;
        let baud_rate: u32 = 115200;

        let builder = serialport::new(port_name.clone(), baud_rate)
            .stop_bits(stop_bits)
            .data_bits(data_bits);

        println!("{:?}", &builder);
        let mut port = builder.open().unwrap_or_else(|e| {
            eprintln!("Failed to open \"{}\". Error: {}", port_name, e);
            ::std::process::exit(1);
        });

        println!(
            "Writing '{}' to {} at {} baud ",
            &msg, &port_name, &baud_rate
        );

        match port.write(msg.as_bytes()) {
            Ok(_) => {
                //print!("{}", &string);
                std::io::stdout().flush().unwrap();
            }
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
            Err(e) => eprintln!("{:?}", e),
        }
    } else {
        println!("select a Port");
    }
    //std::thread::sleep(Duration::from_millis((1000.0 / (rate as f32)) as u64));
}
