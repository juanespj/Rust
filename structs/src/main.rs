use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ItemInfo {
    pub tag: String,
    pub data_type: String,
}
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Config {
    pub types: HashMap<String, Vec<String>>,
    pub structs: HashMap<String, HashMap<String, ItemInfo>>,
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

type CutsElem = HashMap<String, Vec<String>>;
type ProjDet = HashMap<String, Vec<CutsElem>>;

fn initialize_structs<T>(config: &Config, typevar: &str, typedmap: &mut HashMap<String, T>) {
    // let mut initialized_structs: HashMap<String, HashMap<String, DynamicType>> = HashMap::new();

    for (struct_name, fields) in &config.structs {
       let mut var =  match typevar {
            "u32" => "0".to_string(),
            "f32" => "0.0".to_string(),
            "$" | "Date" | "String" => "".to_string(),            
            // "CutsElem" => DynamicType::F32(0.0),
            // Handle vectors of types
            t => {
                if t.starts_with("Vec<") && t.ends_with(">") {
                    let inner_type = &t[4..t.len() - 1];
                    println!("Vec ->{:?}", t);
                }
                println!("not mapped ->{:?}", t);
            }
        }
        //    let mut initialized_fields: HashMap<String, DynamicType> = HashMap::new();

        for (field_name, item_info) in fields {}

        //  initialized_structs.insert(struct_name.clone(), initialized_fields);
    }

    // initialized_structs
}

fn typecheck<T>(data: Vec<T>) {}
fn main() {
    let mut data = Config::default();
    if readjson(&mut data, "cutsconfig.json") == 1 {
        //   let cfg = initialize_structs(&data);
        // for (struct_name, fields) in cfg {
        //     println!("Struct: {}", struct_name);
        //     for (field_name, field_value) in fields {
        //         println!("  Field: {} => {:?}", field_name, field_value);
        //     }
        // }
        //   println!("{}", cfg.get("CutsProject").unwrap());
    };
}
