use super::activities::set_status;
use crate::{Context, Error};
use poise::serenity_prelude as sernity;
use std::sync::Arc;

#[command(prefix_command, slash_command, aliases("exit", "shutdown"))]
async fn quit(ctx: Context<'_>) -> Result<(), Error> {
    let data = ctx.data();
    let cmd_user_id = ctx.author().id;

    let mut resp = ctx
        .reply("Are you sure you want to shutdown the bot?")
        .await?;

    match resp.into_message.(ctx, '✅').await {
        Ok(_) => {
            resp.react(ctx, '❌').await?;
        }
        Err(why) => {
            ctx.reply(format!("Error: {}", why)).await?;
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
                let fuck = Arc::new(ctx.clone()); // annoying but necessary pointer trickery since set_status() needs an Arc<Context>
                let passed_context = Arc::clone(&fuck);
                set_status(passed_context, activity, status).await;

                // set shutting down flag. Doesn't need to be set back to false, since bot will inevitably shut down
                data.get::<ShuttingDown>()
                    .unwrap()
                    .store(true, std::sync::atomic::Ordering::Relaxed);

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
    Ok(())
}
