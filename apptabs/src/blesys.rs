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
    str,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::time;
use uuid::Uuid;

/// Only devices whose name contains this string will be tried.
const PERIPHERAL_NAME_MATCH_FILTER: &str = "Feeder";
/// UUID of the characteristic for which we should subscribe to notifications.
const TRIG_CHARACTERISTIC_UUID: Uuid = Uuid::from_u128(0x357a1221_2ae4_4b08_8ea1_06fa478234cb);
const TIMESTAMP_CHARACTERISTIC_UUID: Uuid = Uuid::from_u128(0x65F52242_2AF2_473B_A939_57E5753C92B5);

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
}
impl Default for BLESys {
    fn default() -> Self {
        Self {
            state: BLEState::CREATED,
            status: HashMap::new(),
            adapter_list: vec![],
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
        while loop_lock == 0 {
            match rx_a.try_recv() {
                Ok(msg) => {
                    println!("fromAPP: {:?}", msg.state);
                    match msg.state {
                        BLEState::CREATED => {
                            sys.state = BLEState::IDLE;
                            newmsg = 1;
                        }
                        BLEState::FIND => {
                            if msg.status.contains_key("sel") {
                                let sel = msg.status.get("sel").unwrap().to_vec();
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
                        BLEState::IDLE => {}
                        BLEState::KILL => loop_lock = 1,
                        _ => {}
                    }
                }
                Err(_) => { /* handle sender disconnected */ }
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
                //return Some(p);
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

// fn per_chara(){

//         println!("Discover peripheral {:?} services...", local_name);
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

//         println!("Disconnecting from peripheral {:?}...", local_name);
//         peripheral.disconnect().await?;
//     }
// }
