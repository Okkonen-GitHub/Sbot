use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
};

use tokio::time::{sleep, Duration};

use crate::ShardManagerContainer;

#[command]
#[owners_only]
#[aliases("exit", "shutdown")]
async fn quit(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let uid = serenity::model::id::UserId(872455985497788456);
    println!("uid: {}", uid);
    if let Some(manager) = data.get::<ShardManagerContainer>() {
        let resp = msg.reply(ctx, "Are you sure you want to shutdown the bot?").await?;

        match resp.react(ctx, '✅').await {
            Ok(_) => {
                let _ = resp.react(ctx, '❌').await;
            }
            Err(why) => {
                msg.reply(ctx, format!("Error: {}", why)).await?;
            }
        }
        for _ in 0..240 {
            println!("sleeping");
            sleep(Duration::from_millis(250)).await;
            let reactions = match resp.reaction_users(ctx, '✅', None, Some(uid)).await {
                Ok(v) => v,
                Err(why) => {
                    msg.reply(ctx, format!("Error: {}", why)).await?;
                    break;
                }
            };
            println!("clicked yes: {:?}", reactions);
            for user in reactions {
                println!("user: {:?}", user.name);
                if user.id == uid {
                    println!("Shutting down... {}", user.name);
                    manager.lock().await.shutdown_all().await;
                    break;
                }
            }
            let reactions = match resp.reaction_users(ctx, '❌', None, Some(uid)).await {
                Ok(v) => v,
                Err(why) => {
                    msg.reply(ctx, format!("Error: {}", why)).await?;
                    break;
                }
            };
            println!("clicked no: {:?}", reactions);
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
