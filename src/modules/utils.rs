use std::{collections::HashMap, fs, path::PathBuf, env};

use crate::ShardManagerContainer;

use serenity::client::{bridge::gateway::ShardId, Context};

use sysinfo::{System, SystemExt, ProcessorExt, ProcessExt};

use serde_json;


async fn bytes_to_human(mut bytes: u64) -> String {

    let symbols: [char; 8] = ['K','M', 'G', 'T', 'P', 'E', 'Z', 'Y'];

    let mut i = 0;

    while bytes > 1024 {
        bytes /= 1024;
        i += 1;
    }
    let unit = symbols[i];

    format!("{}{}", bytes, unit)
}

pub async fn seconds_to_human(mut secs: u64) -> String {
    
    let mut hours = 0;
    let mut mins = 0;

    while secs >= 3600 {
        hours += 1;
        secs -= 3600;
    }

    while secs >= 60 {
        mins += 1;
        secs -= 60;
    }
    if hours > 0 {
        format!("{}h {}m", hours, mins)
    } else if mins > 0 {
        format!("{}m {}s", mins, secs)
    } else {
        format!("{}s", secs)
    }
}

pub fn get_pwd() -> PathBuf {
    env::current_dir().unwrap()
}

pub async fn get_ping(ctx: &Context) -> String {
    let latency = {
        let data_read = ctx.data.read().await;
        let shard_manager = data_read.get::<ShardManagerContainer>().unwrap();

        let manager = shard_manager.lock().await;
        let runners = manager.runners.lock().await;

        let runner = runners.get(&ShardId(ctx.shard_id)).unwrap();

        if let Some(duration) = runner.latency {
            format!("{:.2} ms", duration.as_millis())
        } else {
            "? ms".to_string()
        }
    };

    latency
}

pub async fn get_sys(full: bool) -> HashMap<&'static str, String> {
    // Get cpu usage, cpu count, memory usage, uptime, rust version, serenity version, and the number of shards

    let mut sys = System::new_all();

    sys.refresh_all();
    
    let mut sys_info: HashMap<&str, String> = HashMap::new();

    sys_info.insert("memory_usage", format!(
        "{}B / {}B ({:.1}%)",
        bytes_to_human(sys.used_memory()).await,
        bytes_to_human(sys.total_memory()).await,
        sys.used_memory() as f64 / sys.total_memory() as f64 * 100.0
    ));
    
    
    if full {
        let mut cpu_usage = Vec::new();
        for core in sys.processors() {
            cpu_usage.push(
                core.cpu_usage()
            )     
        }
        sys_info.insert("os_info", sys.long_os_version().unwrap_or(String::from("?")));

        sys_info.insert("thread_count", format!("{}", sys.processors().len()));

        let cpu_usage_str = String::from_iter(cpu_usage.iter().map(|usage| format!("{:.1}%", usage)));
        sys_info.insert("cpu_usage", cpu_usage_str);
        // for val in &cpu_usage{
        //     cpu_usage_str.push_str(&format!("\n{:.1}%", val));
        // }
    }

    if let Some(process) = sys.process(sys.processes_by_name("sbot").nth(0).unwrap().pid()) {
        sys_info.insert("uptime", format!("{}", &process.run_time()));
    } else {
        sys_info.insert("uptime", "? s".to_string());
    }

    sys_info
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

    pub async fn set(&self, key: &str, value: serde_json::Value) {
        let file = fs::read_to_string(&self.path);

        if let Ok(file) = file {
            println!("HELLO {}", &file);
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
