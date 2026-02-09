use crate::appmod::{AppState, Config, RenderApp};

use crate::ppk2srvc::PPK2State;
use crate::sersys::SerState;
use crate::services::launch_services;
use crate::services::{KeyShortcuts, ServiceCtrl, ServicesRequest, req};
use eframe::egui::{Color32, FontId, RichText, text::LayoutJob};
use egui::{ComboBox, Context, TextFormat, Ui};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration, Instant};
use std::{env, path::Path}; //mpsc, SystemTime
const SWT_BASIC_PARAMS: &[&str] = &["hw_ver", "fwv", "rak_fwv", "obs"];

pub fn get_serial_ports(services: Arc<RwLock<ServiceCtrl>>, port_sel: &mut (String, Vec<String>)) {
    let serstate = services.write().unwrap().get_state();
    port_sel.1 = services.write().unwrap().serial_get_port_list();
    if port_sel.1.len() > 0 && port_sel.0 == "-".to_string() {
        if port_sel.1.len() == 2 {
            port_sel.0 = port_sel.1[1].clone();
        } else {
            port_sel.0 = port_sel.1[0].clone();
        }
    }
    services
        .write()
        .unwrap()
        .serial_svc
        .write()
        .unwrap()
        .select_port(port_sel.0.clone());
    log::debug!("[SRVC] Selport {:?} State {:?}", port_sel.0, serstate);

    // self.services
    //     .write()
    //     .unwrap()
    //     .req(ServicesRequest::STARTSERIALMONITOR);
}
pub fn readsys(data: &mut Config, file: &str) -> u8 {
    match std::fs::read_to_string(file) {
        Ok(text) => {
            // print!("{:?}", text);
            *data = serde_json::from_str::<Config>(&text).unwrap();
            // print!("{:?}", data.types);
        } //sys = serde_json::from_str::<Report>(&text).unwrap(),
        Err(e) => {
            println!("Couldnt Find json {}", e);
            return 0;
        }
    }
    return 1;
    // print!("{:?}", sys.ids);
}
pub fn serial_port_ui(ui: &mut Ui, app: &mut RenderApp) {
    ui.horizontal(|ui| {
        ui.label("Port:");
        if let Ok(selport) = app.services.try_write() {
            app.port_sel.0 = selport.serial_get_sel_port();
        }

        let mut tmpselport = app.port_sel.0.clone();
        ComboBox::from_id_salt("COM PortSel")
            .selected_text(tmpselport.to_string())
            .width(70.0)
            .show_ui(ui, |ui| {
                for i in 0..app.port_sel.1.len() {
                    ui.selectable_value(
                        &mut tmpselport,
                        (*app.port_sel.1[i]).to_string(),
                        app.port_sel.1[i].to_string(),
                    );
                }
            });
        if tmpselport != app.port_sel.0 {
            app.services
                .write()
                .unwrap()
                .serial_svc
                .write()
                .unwrap()
                .select_port(tmpselport);
        }

        if ui.button("↺").on_hover_text("Refresh COM ports").clicked() {
            app.port_sel.1 = app.services.write().unwrap().serial_get_port_list();
        }
    });
    if let Some(val) = app.services_state.get("SerState") {
        if ["KILL", "ERROR", "STOP", "CREATED"].contains(&val.as_str()) {
            if ui.button("Launch Serial Monitor").clicked() {
                start_monitor_read(app);
            }
        } else {
            if ui.button("Stop Serial Monitor").clicked() {
                req(&app.srvcrequest.0, ServicesRequest::STOPSERIALMONITOR);
            }
        }
    }
    ui.add_space(5.0);
    if ui.button("Reset Serial Port").clicked() {
        req(&app.srvcrequest.0, ServicesRequest::RESETSERIAL);
    }
    ui.add_space(5.0);
}

// pub fn messagewindow(
//     ctx: &Context,
//     openwindows: &mut WindowsOpen,
//     windowmsg: &mut HashMap<String, String>,
// ) {
//     let mut msgwind = openwindows.msg.clone();

//     egui::Window::new("Info:")
//         .auto_sized()
//         .open(&mut msgwind)
//         .show(ctx, |ui| {
//             if windowmsg.contains_key("MSG") {
//                 ui.label(windowmsg.get("MSG").unwrap());
//             }
//             if windowmsg.contains_key("FlashMSG") {
//                 ui.colored_label(Color32::RED, "Error:");
//                 ui.colored_label(Color32::RED, windowmsg.get("FlashMSG").unwrap());
//             }
//             if windowmsg.contains_key("path") {
//                 let path = Path::new(windowmsg.get("path").unwrap());
//                 ui.label(format!("{:?}", path.parent().unwrap()));
//             }
//             if windowmsg.contains_key("ERR") {
//                 ui.colored_label(Color32::RED, "Error:");
//                 ui.colored_label(Color32::RED, windowmsg.get("ERR").unwrap());
//             }
//             if windowmsg.contains_key("PPK2") {
//                 ui.colored_label(
//                     Color32::RED,
//                     RichText::new("PPK2 Error").font(FontId::proportional(20.0)),
//                 );
//                 ui.colored_label(Color32::RED, windowmsg.get("PPK2").unwrap());
//             }
//             if ui.button("OK").clicked() {
//                 openwindows.msg = false;
//                 windowmsg.clear();
//             }
//         });
//     if msgwind == false {
//         openwindows.msg = false;
//     }
// }

pub fn get_cwd() -> String {
    match env::current_exe() {
        Ok(exe_path) => {
            let mut cwd = exe_path.parent().unwrap().to_str().unwrap().to_string();
            if cwd.to_lowercase().contains("debug")
                && cwd.to_lowercase().contains("deps")
                && cwd.to_lowercase().contains("target")
            {
                cwd = exe_path
                    .parent()
                    .unwrap()
                    .parent()
                    .unwrap()
                    .parent()
                    .unwrap()
                    .parent()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string()
                    + "\\";
            } else if cwd.to_lowercase().contains("debug") && cwd.to_lowercase().contains("target")
            {
                //  println!("Debugging Mod Path");
                cwd = exe_path
                    .parent()
                    .unwrap()
                    .parent()
                    .unwrap()
                    .parent()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string()
                    + "\\";
            } else if cwd.to_lowercase().contains("release")
                && cwd.to_lowercase().contains("target")
            {
                //    println!("Debugging Mod Path");
                cwd = exe_path
                    .parent()
                    .unwrap()
                    .parent()
                    .unwrap()
                    .parent()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string()
                    + "\\";
            } else {
                cwd = cwd + "\\";
            }

            print!("app_cwd..:{:?} -> {:?}", exe_path, cwd);
            return cwd;
        }
        Err(e) => println!("failed to get current exe path: {e}"),
    };
    "".to_string()
}

pub fn app_init(app: &mut RenderApp) {
    // the `error.log` will only contain this message
    app.cwd = get_cwd();

    launch_services(app);
    thread::sleep(std::time::Duration::from_millis(20));
    get_serial_ports(Arc::clone(&app.services), &mut app.port_sel);
    app.state = AppState::Idle;

    req(&app.srvcrequest.0, ServicesRequest::LAUNCHPRINTER);
}

pub fn start_monitor_read(app: &mut RenderApp) {
    if *app
        .services
        .write()
        .unwrap()
        .serial_svc
        .write()
        .unwrap()
        .threadstate
        .read()
        .unwrap()
        != SerState::MONITOR
    {
        req(&app.srvcrequest.0, ServicesRequest::STARTSERIALMONITOR);
        thread::sleep(Duration::from_secs(1));
    }

    app.timedelta = Some(Instant::now());

    app.state = AppState::Cfg;

    thread::sleep(std::time::Duration::from_millis(100));
    req(&app.srvcrequest.0, ServicesRequest::DELAYEDREADSWT);

    app.state = AppState::Cfg;
}

pub fn ppk2_basic_control(ui: &mut Ui, app: &mut RenderApp) {
    let ppkstate = app
        .services
        .write()
        .unwrap()
        .ppk2
        .write()
        .unwrap()
        .state
        .write()
        .unwrap()
        .clone();
    if ppkstate != PPK2State::MONITOR {
        if btn(ui, "PPK2 Power On", 14.0) {
            // *self.services.write().unwrap().ppk2_kill.write().unwrap() = false;
            req(&app.srvcrequest.0, ServicesRequest::MONITORPPK2);
        }
        return;
    } else if ppkstate == PPK2State::MONITOR {
        // if app.ppkplt.contains_key("Curr") {
        //     let vout = app
        //         .services
        //         .write()
        //         .unwrap()
        //         .ppk2
        //         .write()
        //         .unwrap()
        //         .v_out
        //         .read()
        //         .unwrap()
        //         .clone();
        //     ui.add_space(5.0);
        //     ui.label("PPK Monitoring");
        //     ui.indent("ppkuiindent", |ui| {
        //         let curr = app.ppkplt.get("Curr").unwrap();
        //         let abs_max = curr
        //             .iter()
        //             .max_by(|x, y| x.abs().partial_cmp(&y.abs()).unwrap())
        //             .unwrap();

        //         let avg = curr.iter().sum::<f64>() / curr.len() as f64;
        //         let mut scale = 1.0;
        //         let units = if *abs_max > 1000.0 {
        //             scale = 0.001;
        //             "mA".to_string()
        //         } else {
        //             "uA".to_string()
        //         };
        //         ui.horizontal(|ui| {
        //             ui.label(format!("V: {:.2} V", vout as f32 / 1000.0));
        //             ui.label(format!("I: {:.2} {}", avg * scale, units));
        //         });
        //     });
        // }
    }

    if btn(ui, "PPK2 STOP", 14.0) {
        req(&app.srvcrequest.0, ServicesRequest::STOPPPK2);
    }
}

pub fn large_btn(ui: &mut egui::Ui, label: &str) -> bool {
    ui.add_sized(
        [80.0, 30.0],
        egui::Button::new(egui::RichText::new(label).strong().size(16.0)),
    )
    .clicked()
}
pub fn btn(ui: &mut egui::Ui, label: &str, size: f32) -> bool {
    ui.add_sized(
        [80.0, 30.0],
        egui::Button::new(egui::RichText::new(label).strong().size(size)).corner_radius(2.0),
    )
    .clicked()
}

pub const TOAST_SIZE: f32 = 20.0;

pub fn msg_toast_handling(app: &mut RenderApp) {
    if let Ok((source, msgtype, msg)) = app.msgchan.1.try_recv() {
        match msgtype.as_str() {
            "OK" => {
                log::info!("{} : {:?}", source, msg);
                app.toasts
                    .success(egui::RichText::new(&msg).size(TOAST_SIZE))
                    .duration(Some(Duration::from_secs(5)))
                    .show_progress_bar(true)
                    .closable(true);
            }
            "MSG" => {
                log::info!("{} : {:?}", source, msg);
                app.toasts
                    .info(egui::RichText::new(&msg).size(TOAST_SIZE))
                    .duration(Some(Duration::from_secs(10)))
                    .show_progress_bar(true)
                    .closable(true);
            }
            "INFO" => {
                log::info!("{} : {:?}", source, msg);
                app.toasts
                    .info(egui::RichText::new(&msg).size(15.0))
                    .duration(Some(Duration::from_secs(5)))
                    .show_progress_bar(true)
                    .closable(true);
            }
            "ERR" => {
                app.toasts
                    .error(egui::RichText::new(&msg).size(TOAST_SIZE))
                    .duration(Some(Duration::from_secs(15)))
                    .show_progress_bar(true)
                    .closable(true);
                log::error!("\r\n{} ERRORMSG {}", source, msg);
            }
            _ => {
                log::error!("{} MessageType not mapped <{:?}> {}", source, msgtype, msg)
            }
        }
    }
}

use genrs_lib::{EncodingFormat, encode_key, generate_key};
pub fn generate_appkey() -> String {
    let key = generate_key(16);
    if let Ok(encoded_key) = encode_key(key, EncodingFormat::Hex) {
        return encoded_key;
    }
    log::error!("Error generating AppKey");
    "ERROR".to_string()
}

pub fn filter_non_hex_in_place(s: &mut String) {
    // The retain method iterates over the characters of the String.
    // It keeps a character ('c') only if the closure returns 'true'.
    // `is_ascii_hexdigit()` checks for '0'-'9', 'a'-'f', and 'A'-'F'.
    s.retain(|c| c.is_ascii_hexdigit());
}
