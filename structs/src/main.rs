use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Config {
    pub types: HashMap<String, Vec<String>>,
    pub structs: HashMap<String, HashMap<String, String>>,
}
impl Default for Config {
    fn default() -> Config {
        Config {
            types: HashMap::new(),
            structs: HashMap::new(),
        }
    }
}
fn readjson(data: &mut Config, file: &str) -> u8 {
    match std::fs::read_to_string(file) {
        Ok(text) => {
            // print!("{:?}", text);
            *data = serde_json::from_str::<Config>(&text).unwrap();
            // print!("{:?}", data);
        } //sys = serde_json::from_str::<Report>(&text).unwrap(),
        Err(e) => {
            println!("Couldnt Find json {}", e);
            return 0;
        }
    }
    return 1;
    // print!("{:?}", sys.ids);
}

fn main() {
    let mut data = Config::default();
    readjson(&mut data, "config.json");
    for (key, value) in data.types.iter() {
        println!("{:?}", key);
    }
}
