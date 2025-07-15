use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
};

use crate::ShardManagerContainer;

#[command]
#[owners_only]
#[aliases("exit", "shutdown")]
async fn quit(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let uid = msg.author.id;

    if let Some(manager) = data.get::<ShardManagerContainer>() {
        let resp = msg.reply(ctx, "Are you sure you want to shutdown the bot?").await?;

        match msg.react(ctx, '✅').await {
            Ok(_) => {
                let _ = msg.react(ctx, '❌').await;
            }
            Err(why) => {
                msg.reply(ctx, format!("Error: {}", why)).await?;
            }
        }
        loop {
            let reactions = match msg.reaction_users(ctx, '✅', None, Some(uid)).await {
                Ok(v) => v,
                Err(why) => {
                    msg.reply(ctx, format!("Error: {}", why)).await?;
                    break;
                }
            };
            for user in reactions {
                if user.id == uid {
                    println!("Shutting down... {}", user.name);
                    manager.lock().await.shutdown_all().await;
                    break;
                }
            }
            let reactions = match msg.reaction_users(ctx, '❌', None, Some(uid)).await {
                Ok(v) => v,
                Err(why) => {
                    msg.reply(ctx, format!("Error: {}", why)).await?;
                    break;
                }
            };
            for user in reactions {
                if user.id == uid {
                    let _ = resp.delete(ctx).await;
                    break;
                }
            }

        }

    } else {
        msg.reply(ctx, "There was a problem getting the shard manager")
            .await?;

        return Ok(());
    }
    Ok(())
}
