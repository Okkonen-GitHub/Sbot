use std::{collections::HashSet, time::Instant};

use crate::modules::utils::*;
use poise::{serenity_prelude as serenity, CreateReply};

use crate::{Context, Error};

// #[help]
// #[command_not_found_text = "Command not found: {}"]
// #[max_levenshtein_distance(3)]
// #[indention_prefix = ">"]
// #[lacking_permissions = "Hide"]
// #[lacking_role = "Nothing"]
// #[wrong_channel = "Strike"]
#[poise::command(prefix_command, slash_command, track_edits)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help for"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    command: Option<String>,
) -> Result<(), Error> {
    poise::builtins::help(
        ctx,
        command.as_deref(),
        poise::builtins::HelpConfiguration::default(),
    )
    .await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn about(ctx: Context<'_>) -> Result<(), Error> {
    ctx.reply(format!("A pretty normal bot")).await?;

    Ok(())
}

#[poise::command(prefix_command, slash_command, aliases("latency"))]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    ctx.reply(format!("Bot latency: {} ms", ctx.ping().await.as_millis()))
        .await?;

    Ok(())
}

#[poise::command(slash_command, prefix_command, aliases("bping"))]
pub async fn betterping(ctx: Context<'_>) -> Result<(), Error> {
    let latency = ctx.ping().await.as_millis();
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
    let message = ctx.reply(&latency).await?;
    let elapsed = duration.elapsed().as_millis();

    let post_latency = format!("POST latency: {} ms", elapsed);
    let full_latency = format!("{}\n{}", latency, post_latency);
    message
        .edit(ctx, CreateReply::default().content(full_latency))
        .await?;

    Ok(())
}

#[poise::command(slash_command, prefix_command, aliases("up"))]
pub async fn uptime(ctx: Context<'_>) -> Result<(), Error> {
    let sysinfo = get_sys(false).await;
    let uptime: String = seconds_to_human(sysinfo.get("uptime").unwrap().parse::<u64>().unwrap());
    ctx.reply(format!("{uptime}")).await?;
    Ok(())
}
#[poise::command(slash_command, prefix_command, aliases("stats"))]
pub async fn info(ctx: Context<'_>) -> Result<(), Error> {
    const BOT_VERSION: &str = env!("CARGO_PKG_VERSION");

    let latency = ctx.ping().await.as_millis();

    //TODO rust version, serenity version, and the number of shards

    let sysinfo = get_sys(false).await;

    // let cpu_usage = sysinfo.get("cpu_usage").unwrap();
    let memory_usage = sysinfo.get("memory_usage").unwrap();
    let uptime = seconds_to_human(sysinfo.get("uptime").unwrap().parse::<u64>().unwrap());

    let guilds = ctx.cache().guilds().len();

    ctx.send(poise::CreateReply::default().embed(
            serenity::CreateEmbed::default()
                .color(serenity::Color::PURPLE)
                .author(
                    serenity::CreateEmbedAuthor::new("Info")
                        // .icon_url(ctx.cache().current_user().avatar_url()
                        // .unwrap_or("https://64.media.tumblr.com/126d5e1ad49ade5ff4b052d8441943aa/tumblr_py6pq79HeI1xny0zko1_540.png".to_string()))
                )
                .description(format!("Version: {}", BOT_VERSION))
                .field("Latency", latency.to_string(), true)
                .field("Uptime", &uptime, true)
                .field("Guilds", guilds.to_string(), true)
                .footer(
                        serenity::CreateEmbedFooter::new(format!("RAM: {}", memory_usage))
                        .icon_url("https://media.discordapp.net/attachments/514213558549217330/514345278669848597/8yx98C.gif"))
        ))
        .await?;

    Ok(())
}

#[poise::command(slash_command, prefix_command, aliases("inful", "infofull"))]
pub async fn fullinfo(ctx: Context<'_>) -> Result<(), Error> {
    const BOT_VERSION: &str = env!("CARGO_PKG_VERSION");

    let latency = ctx.ping().await.as_millis();

    let sysinfo = get_sys(true).await;

    let memory_usage = sysinfo.get("memory_usage").unwrap();
    let uptime = seconds_to_human(sysinfo.get("uptime").unwrap().parse::<u64>().unwrap());

    // let user = ctx.cache().current_user(); // for the profile pic in the embed

    let cpu_usage = sysinfo.get("cpu_usage").unwrap();
    let os_info = sysinfo.get("os_info").unwrap();
    let thread_count = sysinfo.get("thread_count").unwrap();

    let guilds = ctx.cache().guilds().len();
    ctx.send(
        CreateReply::default().embed(
            serenity::CreateEmbed::default()
                .color(serenity::Color::PURPLE)
                .timestamp(serenity::Timestamp::now())
                .author(serenity::CreateEmbedAuthor::new("Bot Info")) // .icon_url(ctx.cache().current_user().avatar_url().unwrap()))
                .description(format!("Version: {}", BOT_VERSION))
                .field("Latency:", latency.to_string(), true)
                .field("Uptime:", uptime, true)
                .field("Guilds:", guilds.to_string(), true)
                .field("Threads: ", thread_count, true)
                .field("CPU usage:", cpu_usage, true)
                .field("OS: ", os_info, true)
                .footer(serenity::CreateEmbedFooter::new(format!("RAM: {}", memory_usage))
                    .icon_url("https://media.discordapp.net/attachments/514213558549217330/514345278669848597/8yx98C.gif")),
        )
    ).await?;
    Ok(())
}
