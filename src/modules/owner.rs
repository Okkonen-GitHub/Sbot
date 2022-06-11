use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
};
use super::activities::set_status;
use std::sync::Arc;
use crate::{ShardManagerContainer, ShuttingDown};

#[command]
#[owners_only]
#[aliases("exit", "shutdown")]
async fn quit(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let cmd_user_id = msg.author.id;

    if let Some(manager) = data.get::<ShardManagerContainer>() {
        let mut resp = msg
            .reply(ctx, "Are you sure you want to shutdown the bot?")
            .await?;

        match resp.react(ctx, '✅').await {
            Ok(_) => {
                let _ = resp.react(ctx, '❌').await;
            }
            Err(why) => {
                msg.reply(ctx, format!("Error: {}", why)).await?;
            }
        }

        let start_time = std::time::Instant::now();
        'outer: loop {
            let elapsed = start_time.elapsed().as_secs();
            if elapsed >= 60 {
                break 'outer;
            }
            // sleep(Duration::from_millis(250)).await;
            let reactions = match resp.reaction_users(ctx, '✅', None, None).await {
                Ok(v) => v,
                Err(why) => {
                    msg.reply(ctx, format!("Error: {}", why)).await?;
                    break;
                }
            };
            // println!("clicked yes: {:?}", reactions);
            for user in reactions {
                // println!("user: {:?}", user.name);
                if user.id == cmd_user_id {
                    println!("Shutting down... {}", user.name);
                    // too lazy to import them just for this
                    let activity = serenity::model::gateway::Activity::playing("Shutting down...");
                    let status = serenity::model::prelude::OnlineStatus::DoNotDisturb;
                    let fuck = Arc::new(ctx.clone());
                    let passed_context = Arc::clone(&fuck);
                    set_status(passed_context , activity, status).await;
                    
                    data.get::<ShuttingDown>().unwrap().store(true, std::sync::atomic::Ordering::Relaxed);

                    manager.lock().await.shutdown_all().await;
                    break 'outer;
                }
            }
            let reactions = match resp.reaction_users(ctx, '❌', None, None).await {
                Ok(v) => v,
                Err(why) => {
                    msg.reply(ctx, format!("Error: {}", why)).await?;
                    break;
                }
            };
            // println!("clicked no: {:?}", reactions);
            for user in reactions {
                if user.id == cmd_user_id {
                    let _ = resp.delete(ctx).await;
                    msg.reply(ctx, "Shutdown cancelled.").await?;
                    break 'outer;
                }
            }
        }
        // Here no reactions have been added in 60 seconds, so we delete all reactions to the message
        // and edit the message.
        println!("No reactions added, deleting all reactions...");
        resp.delete_reactions(ctx).await?;
        resp.edit(ctx, |m| {
            m.content("Are you sure you want to shutdown the bot? (Cancelled.)")
        })
        .await?;
    } else {
        msg.reply(ctx, "There was a problem getting the shard manager")
            .await?;
    }
    Ok(())
}
