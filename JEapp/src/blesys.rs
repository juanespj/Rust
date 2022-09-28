use btleplug::api::{
    Central, CharPropFlags, Manager as _, Peripheral, ScanFilter, WriteType::WithoutResponse,
};
use btleplug::platform::Manager;
use futures::executor::block_on;

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
// pub trait BLEctrl{
//     fn new() -> Self;
//     fn run(&self);
//     fn bletask(&self);
//     fn test();
// }

pub struct BLESys {
    local_name: String,
    value: f32,
    is_connected: bool,
    trigger: u8,
}
impl Default for BLESys {
    fn default() -> Self {
        Self {
            // Example stuff:
            local_name: "Feeder".to_string(),
            value: 2.7,
            is_connected: false,
            trigger: 0,
            // properties: (),
        }
    }
}

impl BLESys {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn run(cmd: u8) {
        //     //println!("blestarted {}", self.label);
        //     //  self.value = 4.0;

        //     // println!("blestarted {}", self.value);
        let mut rt = tokio::runtime::Runtime::new().unwrap();
        let future = Self::initble(cmd);
        rt.block_on(future);
    }

    fn bletask() {
        println!("BLElink ");
        // println!("blestarted {}", self.value);
    }
    fn test() {
        println!("BLElink ");
        // println!("blestarted {}", self.value);
    }
    pub async fn initble(cmd: u8) -> Result<(), Box<dyn Error>> {
        // pretty_env_logger::init();

        let manager = Manager::new().await?;
        let adapter_list = manager.adapters().await?;
        if adapter_list.is_empty() {
            println!("No Bluetooth adapters found");
        }

        for adapter in adapter_list.iter() {
            println!("Starting scan...");
            adapter
                .start_scan(ScanFilter::default())
                .await
                .expect("Can't scan BLE adapter for connected devices...");
            time::sleep(Duration::from_secs(2)).await;
            let peripherals = adapter.peripherals().await?;

            if peripherals.is_empty() {
                println!("->>> BLE peripheral devices were not found, sorry. Exiting...");
            } else {
                // All peripheral devices in range.
                for peripheral in peripherals.iter() {
                    let properties = peripheral.properties().await?;
                    let is_connected = peripheral.is_connected().await?;
                    let local_name = properties
                        .unwrap()
                        .local_name
                        .unwrap_or(String::from("(peripheral name unknown)"));
                    // println!(
                    //     "Peripheral {:?} is connected: {:?}",
                    //     &local_name, is_connected
                    // );
                    // Check if it's the peripheral we want.
                    if local_name.contains(PERIPHERAL_NAME_MATCH_FILTER) {
                        // println!("Found matching peripheral {:?}...", &local_name);
                        if !is_connected {
                            // Connect if we aren't already connected.
                            if let Err(err) = peripheral.connect().await {
                                println!("Error connecting to peripheral, skipping: {}", err);
                                continue;
                            }
                        }
                        let is_connected = peripheral.is_connected().await?;
                        println!(
                            "Now connected ({:?}) to peripheral {:?}.",
                            is_connected, &local_name
                        );
                        if is_connected {
                            println!("Discover peripheral {:?} services...", local_name);
                            peripheral.discover_services().await?;
                            for char in peripheral.characteristics() {
                                //   println!("Checking characteristic {:?}", characteristic);
                                // Subscribe to notifications from the characteristic with the selected
                                // UUID.
                                match cmd {
                                    1 => {
                                        if char.uuid == TIMESTAMP_CHARACTERISTIC_UUID {
                                            let mut now = SystemTime::now()
                                                .duration_since(UNIX_EPOCH)
                                                .expect("REASON")
                                                .as_secs();
                                            let mut ble_cmd: Vec<u8> = vec![];
                                            for i in 0..8 {
                                                ble_cmd.push(now as u8);
                                                now = now >> 8;
                                            }
                                            peripheral
                                                .write(&char, &ble_cmd, WithoutResponse)
                                                .await?;
                                        }
                                    }
                                    2 => {
                                        if char.uuid == TRIG_CHARACTERISTIC_UUID
                                            && char.properties.contains(CharPropFlags::NOTIFY)
                                        {
                                            println!("Trig {:?}", 1);
                                            let ble_cmd = vec![0x2];
                                            peripheral
                                                .write(&char, &ble_cmd, WithoutResponse)
                                                .await?;
                                            // peripheral.write(&characteristic, &word, WithoutResponse);
                                        };
                                    }
                                    0_u8 | 3_u8..=u8::MAX => todo!(),
                                }
                                // if characteristic.uuid == NOTIFY_CHARACTERISTIC_UUID
                                //     && characteristic.properties.contains(CharPropFlags::NOTIFY)
                                // {
                                //     println!(
                                //         "Subscribing to characteristic {:?}",
                                //         characteristic.uuid
                                //     );
                                //     peripheral.subscribe(&characteristic).await?;
                                //     // Print the first 4 notifications received.
                                //     let mut notification_stream =
                                //         peripheral.notifications().await?.take(4);
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
                            peripheral.disconnect().await?;
                        }
                        // } else {
                        //   println!("Skipping unknown peripheral {:?}", peripheral);
                    }
                }
            }
        }
        Ok(())
    }
}

// pub async fn find_feeder(central: &Adapter) -> Option<Peripheral> {
//     for p in central.peripherals().await.unwrap() {
//         if p.properties()
//             .await
//             .unwrap()
//             .unwrap()
//             .local_name
//             .iter()
//             .any(|name| name.contains("Find"))
//         {
//             println!("found DEV");
//             return Some(p);
//         }
//         // println!("{:?}",p);
//     }

//     None
// }

// pub async fn find_periph(adapter: &Adapter, mut list: Vec<String>) -> Result<(), Box<dyn Error>> {
//     list.clear();
//     let peripherals = adapter.peripherals().await?;
//     if peripherals.is_empty() {
//         eprintln!("->>> BLE peripheral devices were not found, sorry. Exiting...");
//     } else {
//         // All peripheral devices in range
//         for peripheral in peripherals.iter() {
//             let properties = peripheral.properties().await?;
//             let local_name = properties
//                 .unwrap()
//                 .local_name
//                 .unwrap_or(String::from("(peripheral name unknown)"));
//             list.push(local_name);
//         }
//     }

//     println!("peripherals {:?}", list);
//     Ok(())
// }
