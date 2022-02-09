
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

pub async fn get_sys(full: Option<bool>) -> String{
    // Get cpu usage, cpu count, memory usage, uptime, rust version, serenity version, and the number of shards

    let mut sys = System::new_all();

    sys.refresh_all();
    
    let memory_usage = format!(
        "{}B / {}B ({:.1}%)",
        bytes_to_human(sys.used_memory()).await,
        bytes_to_human(sys.total_memory()).await,
        sys.used_memory() as f64 / sys.total_memory() as f64 * 100.0
    );
    let cpu_count = format!("{}", sys.processors().len());
    let mut cpu_usage = Vec::new();
    for core in sys.processors() {
        cpu_usage.push(
            core.cpu_usage()
        )
        
    }
    let mut cpu_usage_str = String::new();
    let mut avg = 0_f32;
    if let Some(is_full) = full{
        if is_full{
            for val in &cpu_usage{
                cpu_usage_str.push_str(&format!("\n{:.1}%", val));
            }
        }
    } else {
        for val in &cpu_usage {
            avg += val;
        }
        avg /= cpu_usage.len() as f32
    };
    if avg != 0_f32 {
        cpu_usage_str = avg.round().to_string();
    }

    if let Some(process) = sys.process(sys.processes_by_name("sbot").nth(0).unwrap().pid()) {
        let uptime = format!("{} s", &process.run_time());
        return format!("memory: {}\ncore:{}\ncpu usage: {}\nuptime: {}", memory_usage, cpu_count, cpu_usage_str, uptime);
    }
    let uptime = "? s".to_string();
    return format!("memory: {}\ncore:{}\ncpu usage: {}\nuptime: {}", memory_usage, cpu_count, cpu_usage_str, uptime);
}