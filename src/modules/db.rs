
use std::{fs, path::PathBuf};

use serde_json;


use super::suggestions::Suggestion;

struct Guild {
    suggestions: Vec<Suggestion>,
    suggestion_channel: u64,
    welcome_channel: u64,
    // prefix: String,

    // muted_role: Option<u64>,
    // muted_role_name: Option<String>, // coming later

}

pub struct JsonDb {
    path: PathBuf,
}

impl JsonDb {
    pub fn new(path: PathBuf) -> Self {
        JsonDb { path }
    }

    pub async fn get(&self, key: &str) -> Option<serde_json::Value> {
        let file = fs::read_to_string(&self.path);

        if let Ok(file) = file {
            let json: serde_json::Value = serde_json::from_str(&file).unwrap();

            if let Some(val) = json.get(key) {
                Some(val.clone())
            } else {
                None
            }
        } else {
            None
        }
    }
    // need to fix `key` to accept any serde::Value or whatever can be used to index a json object.. but that's not that simple
    pub async fn set(&self, key: &str, value: serde_json::Value) {
        let file = fs::read_to_string(&self.path);

        if let Ok(file) = file {
            // println!("HELLO {}", &file);
            let mut json: serde_json::Value = serde_json::from_str(&file).unwrap();

            json[key] = value;

            let json_str = serde_json::to_string(&json).unwrap();

            fs::write(&self.path, json_str).unwrap();
        }
    }
    pub async fn get_all(&self) -> Option<serde_json::Map<String, serde_json::Value>> {
        let file = fs::read_to_string(&self.path);

        if let Ok(file) = file {
            let json: serde_json::Value = serde_json::from_str(&file).unwrap();

            Some(json.as_object().unwrap().to_owned())
        } else {
            None
        }
    }
}
