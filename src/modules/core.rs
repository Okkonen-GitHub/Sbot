use std::collections::HashSet;

use crate::modules::utils::*;

use serenity::{
    builder::{CreateEmbed, CreateMessage},
    framework::standard::{
        help_commands,
        macros::{command, help},
        Args, CommandGroup, CommandResult, HelpOptions,
    },
    model::{
        channel::{Message},
        id::UserId,
    },
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
#[aliases("stats")]
async fn info(ctx: &Context, msg: &Message) -> CommandResult {
    let latency = get_ping(ctx).await;

    //TODO reply with an embed with the bot's latency, cpu usage, memory usage, uptime, rust version, serenity version, and the number of shards

    let sysinfo = get_sys(false).await;

    let cpu_usage = sysinfo.get("cpu_usage").unwrap();
    let memory_usage = sysinfo.get("memory_usage").unwrap();
    let uptime = sysinfo.get("uptime").unwrap();
    
    let user = ctx.cache.current_user().await; // for the profile pic in the embed


    msg.channel_id
        .send_message(&ctx, |m: &mut CreateMessage| {
            m.content("testing")
            .embed(|e: &mut CreateEmbed| {
                e.title("Bot info")
                    .author(
                        | a| {
                            a.name("Info");
                            a.icon_url(user.avatar_url().unwrap_or("https://64.media.tumblr.com/126d5e1ad49ade5ff4b052d8441943aa/tumblr_py6pq79HeI1xny0zko1_540.png".to_string()))
                        }
                    )
                    .description("This is a test")
                    .field("Latency", latency, true)
                    .field("Uptime", &uptime, true)
                    .footer(
                        |f| {
                            f.text(format!("CPU: {}   RAM: {}", cpu_usage, memory_usage));
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
