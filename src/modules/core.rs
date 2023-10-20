use std::{collections::HashSet, time::Instant};

use crate::modules::utils::*;

use serenity::{
    builder::{CreateEmbed, CreateMessage},
    framework::standard::{
        help_commands,
        macros::{command, help},
        Args, CommandGroup, CommandResult, HelpOptions,
    },
    model::{channel::Message, id::UserId},
    prelude::*,
};

#[help]
#[command_not_found_text = "Command not found: {}"]
#[max_levenshtein_distance(3)]
#[indention_prefix = ">"]
#[lacking_permissions = "Hide"]
#[lacking_role = "Nothing"]
#[wrong_channel = "Strike"]
async fn c_help(
    ctx: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    let _ = help_commands::with_embeds(ctx, msg, args, help_options, groups, owners).await;
    Ok(())
}

#[command]
async fn about(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(&ctx, format!("A pretty normal bot")).await?;

    Ok(())
}

#[command]
#[aliases("latency")]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    let latency = get_ping(ctx).await;
    msg.reply(ctx, format!("Bot latency: {}", latency)).await?;

    Ok(())
}

#[command]
#[aliases("bping")]
async fn betterping(ctx: &Context, msg: &Message) -> CommandResult {
    let latency = get_ping(ctx).await;
    let get_latency = {
        let now = Instant::now();
        // `let _` to supress any errors.
        let _ = reqwest::get("https://discordapp.com/api/v6/gateway").await;
        now.elapsed().as_millis() as f64
    };
    // "Websocket latency: 121 ms"
    let latency = format!(
        "Websocket latency: {}\nGET latency: {} ms",
        latency, get_latency
    );
    let duration = Instant::now();
    let mut message = msg.reply(ctx, &latency).await?;
    let elapsed = duration.elapsed().as_millis();

    let post_latency = format!("POST latency: {} ms", elapsed);
    let full_latency = format!("{}\n{}", latency, post_latency);
    message.edit(&ctx, |m| m.content(full_latency)).await?;

    Ok(())
}

#[command]
#[aliases("up")]
async fn uptime(ctx: &Context, msg: &Message) -> CommandResult {
    let sysinfo = get_sys(false).await;
    let uptime: String = seconds_to_human(sysinfo.get("uptime").unwrap().parse::<u64>().unwrap());
    msg.reply(ctx, format!("{uptime}")).await?;
    Ok(())
}

#[command]
#[aliases("stats")]
async fn info(ctx: &Context, msg: &Message) -> CommandResult {
    const BOT_VERSION: &str = env!("CARGO_PKG_VERSION");

    let latency = get_ping(ctx).await;

    //TODO rust version, serenity version, and the number of shards

    let sysinfo = get_sys(false).await;

    // let cpu_usage = sysinfo.get("cpu_usage").unwrap();
    let memory_usage = sysinfo.get("memory_usage").unwrap();
    let uptime = seconds_to_human(sysinfo.get("uptime").unwrap().parse::<u64>().unwrap());

    let user = ctx.cache.current_user(); // for the profile pic in the embed

    let guilds = ctx.cache.guilds().len();

    msg.channel_id
        .send_message(&ctx, |m: &mut CreateMessage| {
            m.embed(|e: &mut CreateEmbed| {
                e.author(
                    | a| {
                        a.name("Info");
                        a.icon_url(user.avatar_url().unwrap_or("https://64.media.tumblr.com/126d5e1ad49ade5ff4b052d8441943aa/tumblr_py6pq79HeI1xny0zko1_540.png".to_string()))
                    }
                )
                .description(format!("Version: {}", BOT_VERSION))
                .field("Latency", latency, true)
                .field("Uptime", &uptime, true)
                .field("Guilds", &guilds, true)
                .footer(
                    |f| {
                        f.text(format!("RAM: {}", memory_usage));
                        f.icon_url("https://media.discordapp.net/attachments/514213558549217330/514345278669848597/8yx98C.gif")
                    },
                );
                e
            });
            m
        })
        .await?;

    Ok(())
}

#[command]
#[aliases("inful", "infofull")]
async fn fullinfo(ctx: &Context, msg: &Message) -> CommandResult {
    const BOT_VERSION: &str = env!("CARGO_PKG_VERSION");

    let latency = get_ping(ctx).await;

    let sysinfo = get_sys(true).await;

    let memory_usage = sysinfo.get("memory_usage").unwrap();
    let uptime = seconds_to_human(sysinfo.get("uptime").unwrap().parse::<u64>().unwrap());

    let user = ctx.cache.current_user(); // for the profile pic in the embed

    let cpu_usage = sysinfo.get("cpu_usage").unwrap();
    let os_info = sysinfo.get("os_info").unwrap();
    let thread_count = sysinfo.get("thread_count").unwrap();

    let guilds = ctx.cache.guilds().len();
    msg.channel_id
        .send_message(&ctx, |m: &mut CreateMessage| {
            m.embed(|e: &mut CreateEmbed| {
                e.author(
                        | a| {
                            a.name("Bot Info:");
                            a.icon_url(user.avatar_url().unwrap_or("https://64.media.tumblr.com/126d5e1ad49ade5ff4b052d8441943aa/tumblr_py6pq79HeI1xny0zko1_540.png".to_string()))
                        }
                    )
                    .description(format!("Version: {}", BOT_VERSION))
                    .field("Latency:", latency, true)
                    .field("Uptime:", &uptime, true)
                    .field("Guilds:", &guilds, true)
                    .field("Threads: ", &thread_count, true)
                    .field("CPU usage:", &cpu_usage, true)
                    .field("OS: ", &os_info, true)
                    .footer(
                        |f| {
                            f.text(format!("RAM: {}", memory_usage));
                            f.icon_url("https://media.discordapp.net/attachments/514213558549217330/514345278669848597/8yx98C.gif")
                        },
                    );
                e
            });
            m
        })
        .await?;

    Ok(())
}
