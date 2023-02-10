use btleplug::api::{
    Central, CharPropFlags, Manager as _, Peripheral, ScanFilter, WriteType::WithoutResponse,
};
use btleplug::platform::{Adapter, Manager};
use color_eyre::eyre::Result;
use futures::executor::block_on;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc};
use std::{
    error::Error,
    str, thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::time;
use uuid::Uuid;

/// Only devices whose name contains this string will be tried.
const PERIPHERAL_NAME_MATCH_FILTER: &str = "Feeder";
/// UUID of the characteristic for which we should subscribe to notifications.
const TRIG_CHAR_UUID: Uuid = Uuid::from_u128(0x357a1221_2ae4_4b08_8ea1_06fa478234cb);
const TIMESTAMP_CHAR_UUID: Uuid = Uuid::from_u128(0x65F52242_2AF2_473B_A939_57E5753C92B5);
const QUANTITY_CHAR_UUID: Uuid = Uuid::from_u128(0xbe9d244a_169e_4e1e_a4a1_b661f0f14da6);
const SPD_CHAR_UUID: Uuid = Uuid::from_u128(0xF055B698_DFA9_4D83_820B_893566B0F6C9);

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum BLEState {
    CREATED,
    READY,
    BUSY,
    SCAN,
    FIND,
    IDLE, //logged in
    KILL,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BLESys {
    pub state: BLEState,
    pub status: HashMap<String, Vec<String>>,
    #[serde(skip)]
    adapter_list: Vec<Adapter>,
    pub blepersel: String,
    pub blelist: Vec<String>,
}

impl Default for BLESys {
    fn default() -> Self {
        Self {
            state: BLEState::CREATED,
            status: HashMap::new(),
            adapter_list: vec![],
            blepersel: "-".to_string(),
            blelist: vec![],
        }
    }
}

impl BLESys {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn startserv(tx_ble: mpsc::Sender<BLESys>, rx_a: mpsc::Receiver<BLESys>) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let future = Self::bletask(tx_ble, rx_a);
        rt.block_on(future);
    }

    pub async fn bletask(
        tx_ble: mpsc::Sender<BLESys>,
        rx_a: mpsc::Receiver<BLESys>,
    ) -> color_eyre::Result<()> {
        let mut sys = BLESys::default();
        let manager = Manager::new().await?;

        sys.adapter_list = manager.adapters().await?;
        if sys.adapter_list.is_empty() {
            eprintln!("No Bluetooth adapters found");
        }

        let mut newmsg: u8 = 0;
        let mut loop_lock: u8 = 0;
        let mut prev_state=sys.state.clone();
        while loop_lock == 0 {
           
            if prev_state!= sys.state{
                match sys.state {
                    BLEState::CREATED => {
                        sys.state = BLEState::SCAN;
                        newmsg = 1;
                    }
                    BLEState::FIND => {
                        if sys.status.contains_key("sel") {
                            let sel = sys.status.get("sel").unwrap().to_vec();
                            sys.status.insert("sel".to_string(), sel);
                            self::find_per(&mut sys).await?;
                        }
                        sys.state = BLEState::IDLE;
                        newmsg = 1;
                    }
                    BLEState::SCAN => {
                        sys.state = BLEState::SCAN;
                        self::list_periph(&mut sys).await?;
                        if sys.status.contains_key("periph") {
                            newmsg = 1;
                        }
                    }
                    BLEState::IDLE => { match rx_a.try_recv()  {
                        Ok(msg) => {
                            println!("fromAPP: {:?}", msg.state);
                            sys.state=msg.state;
                        }
                        Err(_) => { /* handle sender disconnected */ }
                    }}
                    BLEState::KILL => loop_lock = 1,
                    _ => {}
                }
            }
            // loop_lock = 1;
            if newmsg == 1 {
                if let Err(_) = tx_ble.send(sys.clone()) {
                    println!("App not listening.")
                }
                newmsg = 0;
            }
        }
        println!("BLE STOP.");
        Ok(())
    }
}

async fn find_per(sys: &mut BLESys) -> color_eyre::Result<()> {
    let per_tofind = &sys.status.get("sel").unwrap().to_vec()[0];
    for adapter in sys.adapter_list.iter() {
        println!("Starting scan on {}...", adapter.adapter_info().await?);
        adapter
            .start_scan(ScanFilter::default())
            .await
            .expect("Can't scan BLE adapter for connected devices...");
        time::sleep(Duration::from_secs(3)).await;
        for per in adapter.peripherals().await.unwrap() {
            let properties = per.properties().await?;
            let is_connected = per.is_connected().await?;
            let local_name = properties
                .unwrap()
                .local_name
                .unwrap_or(String::from("(peripheral name unknown)"));
            if per
                .properties()
                .await
                .unwrap()
                .unwrap()
                .local_name
                .iter()
                .any(|name| name.contains(per_tofind))
            {
                println!("found DEV {:?}", per);
                if !is_connected {
                    // Connect if we aren't already connected.
                    if let Err(err) = per.connect().await {
                        eprintln!("Error connecting to peripheral, skipping: {}", err);
                        continue;
                    }
                }
                let is_connected = per.is_connected().await?;
                println!(
                    "Now connected ({:?}) to peripheral {:?}.",
                    is_connected, &local_name
                );
                if is_connected {
                    println!("Discover peripheral {:?} services...", sys.blepersel);
                    per.discover_services().await?;
                    let mut found = 0;
                    for characteristic in per.characteristics() {
                        found = 0;
                        match characteristic.uuid {
                            TIMESTAMP_CHAR_UUID => {
                                println!("Timestamp");
                                found = 1;
                            }
                            QUANTITY_CHAR_UUID => {
                                println!("Quantity");
                                found = 1;
                            }
                            SPD_CHAR_UUID => {
                                println!("Speed");
                                found = 1;
                            }
                            TRIG_CHAR_UUID => {
                                println!("Trigger");
                                found = 1;
                            }
                            _ => {}
                        }
                        if found == 0 {
                            println!("Checking characteristic {:?}", characteristic);
                        }else{

                        }

                        // Subscribe to notifications from the characteristic with the selected
                        // UUID.
                        // if characteristic.uuid == NOTIFY_CHARACTERISTIC_UUID
                        //     && characteristic.properties.contains(CharPropFlags::NOTIFY)
                        // {
                        //     println!("Subscribing to characteristic {:?}", characteristic.uuid);
                        //     per.subscribe(&characteristic).await?;
                        //     // Print the first 4 notifications received.
                        //     let mut notification_stream =
                        //     per.notifications().await?.take(4);
                        //     // Process while the BLE connection is not broken or stopped.
                        //     while let Some(data) = notification_stream.next().await {
                        //         println!(
                        //             "Received data from {:?} [{:?}]: {:?}",
                        //             local_name, data.uuid, data.value
                        //         );
                        //     }
                        // }
                    }
                    println!("Disconnecting from peripheral {:?}...", local_name);
                    per.disconnect().await?;
                }
            }
            // println!("{:?}",p);
        }
    }

    Ok(())
}

async fn list_periph(sys: &mut BLESys) -> color_eyre::Result<()> {
    let mut list: Vec<String> = vec![];
    let mut listconn: Vec<String> = vec![];
    for adapter in sys.adapter_list.iter() {
        println!("Starting scan on {}...", adapter.adapter_info().await?);
        adapter
            .start_scan(ScanFilter::default())
            .await
            .expect("Can't scan BLE adapter for connected devices...");
        time::sleep(Duration::from_secs(3)).await;
        let peripherals = adapter.peripherals().await?;
        if peripherals.is_empty() {
            eprintln!("->>> BLE peripheral devices were not found, sorry. Exiting...");
        } else {
            // All peripheral devices in range
            for peripheral in peripherals.iter() {
                let properties = peripheral.properties().await?;
                let is_connected = peripheral.is_connected().await?;
                let local_name = properties
                    .unwrap()
                    .local_name
                    .unwrap_or(String::from("unknown"));

                if !local_name.contains("unknown") {
                    list.push(local_name.clone());
                }
                if is_connected {
                    listconn.push(local_name);
                }

                // println!(
                //     "Peripheral {:?} is connected: {:?}",
                //     local_name, is_connected
                // );
                // println!("{:?}", local_name);
            }

            // All peripheral devices in range
        }
    }

    sys.status.insert("periph".to_string(), list);
    if listconn.len() > 0 {
        sys.status.insert("conn".to_string(), listconn);
    }
    // println!("SCAN {:?}", sys.status);
    Ok(())
}

use egui::*;
pub fn ble_gui(ui: &mut Ui, blectrl: &mut BLESys) -> u8 {
    // ui.menu_button("BLE", |ui| {
    // if self.threads.len() > 0 {
    //     while self.threads.len() > 0 {
    //         let cur_thread = self.threads.remove(0); // moves it into cur_thread
    //         cur_thread.join().unwrap();
    //     }
    // }
    // });
    let mut msg = 0;

    if ui.button("Scan").clicked() {
        blectrl.state = BLEState::SCAN;
        msg = 1;
    }
    ui.separator();
    ComboBox::from_label("BLE Port")
        .selected_text(blectrl.blepersel.to_string())
        .show_ui(ui, |ui| {
            for i in 0..blectrl.blelist.len() {
                ui.selectable_value(
                    &mut blectrl.blepersel,
                    (*blectrl.blelist[i]).to_string(),
                    blectrl.blelist[i].to_string(),
                );
            }
        });
    if ui.button("Connect").clicked() {
        if blectrl.blepersel != "".to_string() {
            blectrl
                .status
                .insert("sel".to_string(), vec![blectrl.blepersel.clone()]);
            blectrl.state = BLEState::FIND;
            msg = 1;
        }
    }
    return msg;
}

// async fn per_chara(,sys: &mut BLESys)-> color_eyre::Result<()>{
//     peripheral:&mut  dyn Peripheral
//         println!("Discover peripheral {:?} services...", sys.blepersel);
//         peripheral.discover_services().await?;
//         for char in peripheral.characteristics() {
//             //   println!("Checking characteristic {:?}", characteristic);
//             // Subscribe to notifications from the characteristic with the selected
//             // UUID.
//             match cmd {
//                 1 => {
//                     if char.uuid == TIMESTAMP_CHARACTERISTIC_UUID {
//                         let mut now = SystemTime::now()
//                             .duration_since(UNIX_EPOCH)
//                             .expect("REASON")
//                             .as_secs();
//                         let mut ble_cmd: Vec<u8> = vec![];
//                         for i in 0..8 {
//                             ble_cmd.push(now as u8);
//                             now = now >> 8;
//                         }
//                         peripheral.write(&char, &ble_cmd, WithoutResponse).await?;
//                     }
//                 }
//                 2 => {
//                     if char.uuid == TRIG_CHARACTERISTIC_UUID
//                         && char.properties.contains(CharPropFlags::NOTIFY)
//                     {
//                         println!("Feed Trig ");
//                         let ble_cmd = vec![0x2];
//                         peripheral.write(&char, &ble_cmd, WithoutResponse).await?;
//                         // peripheral.write(&characteristic, &word, WithoutResponse);
//                     };
//                 }
//                 0_u8 | 3_u8..=u8::MAX => todo!(),
//             }
//             // if characteristic.uuid == NOTIFY_CHARACTERISTIC_UUID
//             //     && characteristic.properties.contains(CharPropFlags::NOTIFY)
//             // {
//             //     println!(
//             //         "Subscribing to characteristic {:?}",
//             //         characteristic.uuid
//             //     );
//             //     peripheral.subscribe(&characteristic).await?;
//             //     // Print the first 4 notifications received.
//             //     let mut notification_stream =
//             //         peripheral.notifications().await?.take(4);
//             //     // Process while the BLE connection is not broken or stopped.
//             //     while let Some(data) = notification_stream.next().await {
//             //         println!(
//             //             "Received data from {:?} [{:?}]: {:?}",
//             //             local_name, data.uuid, data.value
//             //         );
//             //     }
//             // }

//         println!("Disconnecting from peripheral {:?}...", sys.blepersel);
//         peripheral.disconnect().await?;
//     }
//     Ok(())
// }
