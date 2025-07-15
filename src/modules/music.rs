use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
};

#[command]
#[only_in(guilds)]
async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let channel_id = guild
        .voice_states
        .get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            msg.reply(ctx, "You're not in any voice channel").await?;
            return Ok(());
        }
    };
    let manager = songbird::get(ctx)
        .await
        .expect("Songbird voice client placed in at initialisation")
        .clone();
    let _handler = manager.join(guild_id, connect_to).await;

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird voice client placed in at initialisation")
        .clone();

    let has_handler = manager.get(guild_id).is_some();
    if has_handler {
        if let Err(e) = manager.remove(guild_id).await {
            msg.channel_id
                .say(&ctx.http, format!("Failed: {:?}", e))
                .await?;
        }
        msg.channel_id.say(&ctx.http, "Left voice channel").await?;
    } else {
        msg.reply(ctx, "Not in a voice channel").await?;
    }

    Ok(())
}
#[command]
#[only_in(guilds)]
#[aliases("unmute")]
async fn mute(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let manager = songbird::get(ctx)
        .await
        .expect("Songbird voice client placed in at initialisation")
        .clone();

    match manager.get(guild.id) {
        Some(call) => {
            let mut lock = call.lock().await;
            let is_mute = lock.is_mute();
            lock.mute(!is_mute).await?;
        }
        None => {
            msg.reply(ctx, "Not in a voice channel").await?;
        }
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
#[aliases("undeafen")]
async fn deafen(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(ctx).await.unwrap();
    let manager = songbird::get(ctx)
        .await
        .expect("Songbird voice client placed in at initialisation")
        .clone();

    match manager.get(guild.id) {
        Some(call) => {
            let mut lock = call.lock().await;
            let is_deafened = lock.is_deaf();
            lock.deafen(!is_deafened).await?;
        }
        None => {
            msg.reply(ctx, "Not in a voice channel").await?;
        }
    }
    Ok(())
}
