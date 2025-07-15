use std::{
    collections::{HashMap, HashSet},
    env,
    ffi::OsStr,
    os::unix::ffi::OsStrExt,
    path::PathBuf, // time::Duration,
};
// use serenity::builder::{CreateMessage, CreateEmbed};
// use crate::ShardManagerContainer;

// use serenity::{
//     client::{bridge::gateway::ShardId, Context},
//     http::Http,
//     model::id::UserId,
// };
use poise::serenity_prelude as serenity;

use crate::{Context, Error};

use sysinfo::{System, MINIMUM_CPU_UPDATE_INTERVAL};

pub fn bytes_to_human(mut bytes: u64) -> String {
    let symbols: [char; 9] = ['\0', 'K', 'M', 'G', 'T', 'P', 'E', 'Z', 'Y'];

    let mut i = 0;

    while bytes >= 1024 {
        // could use a bitsift here but too lazy to do that
        bytes /= 1024;
        i += 1;
    }
    let unit = symbols[i];

    format!("{}{}", bytes, unit)
}

pub fn seconds_to_human(mut secs: u64) -> String {
    let mut hours = 0;
    let mut mins = 0;
    let mut days = 0;

    while secs >= 86400 {
        days += 1;
        secs -= 86400;
    }

    while secs >= 3600 {
        hours += 1;
        secs -= 3600;
    }

    while secs >= 60 {
        mins += 1;
        secs -= 60;
    }
    if days > 0 {
        format!("{}d {}h", days, hours)
    } else if hours > 0 {
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

// pub async fn get_ping(ctx: Context<'_>) -> String {
//     let latency = {
//         let data_read = ctx.data().read().await;
//         let shard_manager = data_read.get::<ShardManagerContainer>().unwrap();
//
//         let manager = shard_manager.lock().await;
//         let runners = manager.runners.lock().await;
//
//         let runner = runners.get(&ShardId(ctx.shard_id)).unwrap();
//
//         if let Some(duration) = runner.latency {
//             format!("{:.2} ms", duration.as_millis())
//         } else {
//             "? ms".to_string()
//         }
//     };
//
//     latency
// }

pub async fn get_sys(full: bool) -> HashMap<&'static str, String> {
    // Get cpu usage, cpu count, memory usage, uptime, rust version, serenity version, and the number of shards

    let mut sys = System::new_all();

    sys.refresh_all();

    let mut sys_info: HashMap<&str, String> = HashMap::new();
    sys_info.insert(
        "memory_usage",
        format!(
            "{}B / {}B ({:.1}%)",
            bytes_to_human(sys.used_memory()),
            bytes_to_human(sys.total_memory()),
            sys.used_memory() as f64 / sys.total_memory() as f64 * 100.0
        ),
    );

    // full system information (see full info command)
    if full {
        let mut cpu_usage = Vec::new();
        sys.refresh_cpu_all();
        tokio::time::sleep(MINIMUM_CPU_UPDATE_INTERVAL).await;
        sys.refresh_cpu_all();
        for core in sys.cpus() {
            cpu_usage.push(core.cpu_usage())
        }
        sys_info.insert(
            "os_info",
            sysinfo::System::long_os_version().unwrap_or(String::from("?")),
        );

        sys_info.insert("thread_count", format!("{}", sys.cpus().len()));

        // big brain functional programming
        let cpu_usage_str =
            String::from_iter(cpu_usage.iter().map(|usage| format!(" {:.1}%", usage)));
        sys_info.insert("cpu_usage", cpu_usage_str);
    }
    let proc = sys.processes_by_name(OsStr::from_bytes(b"sbot")).next();
    if let Some(process) = proc {
        sys_info.insert("uptime", format!("{}", &process.run_time()));
    } else {
        sys_info.insert("uptime", "? s".to_string());
    }

    sys_info
}

// Returns a tuple containing a hashset of bot owners' ids and the bot's id
pub async fn get_owners(token: &str) -> (HashSet<serenity::UserId>, serenity::UserId) {
    let http = serenity::Http::new(&token);

    // fetch your bot's owners and id
    let (owners, bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            if let Some(team) = info.team {
                for member in team.members {
                    owners.insert(member.user.id);
                }
            } else {
                owners.insert(info.owner.unwrap().id);
            }
            match http.get_current_user().await {
                Ok(bot_id) => (owners, bot_id.id),
                Err(why) => panic!("Could not access the bot id: {:?}", why),
            }
        }
        Err(why) => panic!("Could not access application info: {:?}", why),
    };
    return (owners, bot_id);
}

// removes prefix and possibly whitespace from the beginning of a string
pub fn remove_prefix_from_message(message: &String, prefix: &str) -> String {
    if message.starts_with(prefix) {
        let message = message[prefix.len()..].to_string();
        let message = message.trim_start(); // trim whitespace from the beginning (if there is any)
        message.to_string()
    } else {
        // should never happen
        message.to_owned()
    }
}
