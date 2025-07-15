use std::collections::HashSet;

use crate::ShardManagerContainer; // impl in main.rs

use serenity::{
    client::bridge::gateway::ShardId,
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
    msg.reply(ctx, format!("Bot latency: {}", latency)).await?;

    Ok(())
}
