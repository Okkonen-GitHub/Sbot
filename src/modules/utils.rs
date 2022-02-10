use std::collections::HashMap;

use crate::ShardManagerContainer;

use serenity::client::{bridge::gateway::ShardId, Context};

use sysinfo::{System, SystemExt, ProcessorExt, ProcessExt};


async fn bytes_to_human(mut bytes: u64) -> String {
    let mut unit = 'K';

    if bytes >= 1024 {
        bytes /= 1024;
        unit = 'M';
    }

    if bytes >= 1024 {
        bytes /= 1024;
        unit = 'G';
    }

    if bytes >= 1024 {
        bytes /= 1024;
        unit = 'T';
    }

    if bytes >= 1024 {
        bytes /= 1024;
        unit = 'P';
    }

    if bytes >= 1024 {
        bytes /= 1024;
        unit = 'E';
    }

    format!("{}{}", bytes, unit)
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
    
    let mut cpu_usage = Vec::new();
    for core in sys.processors() {
        cpu_usage.push(
            core.cpu_usage()
        )     
    }

    if full {
        sys_info.insert("thread_count", format!("{}", sys.processors().len()));

        let cpu_usage_str = String::from_iter(cpu_usage.iter().map(|usage| format!("{:.1}%", usage)));
        sys_info.insert("cpu_usage", cpu_usage_str);
        // for val in &cpu_usage{
        //     cpu_usage_str.push_str(&format!("\n{:.1}%", val));
        // }
    } else {
        let mut avg = 0.0;

        for val in &cpu_usage {
            avg += val;
        }
        avg /= cpu_usage.len() as f32;

        sys_info.insert("cpu_usage", format!("{:.1}%", avg));
    }

    if let Some(process) = sys.process(sys.processes_by_name("sbot").nth(0).unwrap().pid()) {
        sys_info.insert("uptime", format!("{} s", &process.run_time()));
    } else {
        sys_info.insert("uptime", "? s".to_string());
    }

    sys_info
}
