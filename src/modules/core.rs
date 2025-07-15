use std::collections::HashSet;

use crate::ShardManagerContainer; // impl in main.rs

use crate::modules::utils::*;

use serenity::{
    builder::{CreateEmbed, CreateMessage},
    client::bridge::gateway::ShardId,
    framework::standard::{
        help_commands,
        macros::{command, help},
        Args, CommandGroup, CommandResult, HelpOptions,
    },
    model::{
        channel::{Embed, Message},
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

    msg.channel_id
        .send_message(&ctx, |m: &mut CreateMessage| {
            m.content("testing").embed(|e: &mut CreateEmbed| {
                e.title("Bot info")
                    .description("This is a test")
                    .field("Latency", latency, true);

                e
            });
            m
        })
        .await?;

    Ok(())
}
