use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
};
use songbird::input::Restartable; // for looping and yt searches (first result) (Restartable::*)

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

#[command]
#[only_in(guilds)]
#[aliases("p")]
async fn play(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    //TODO Add queue, stop, pause, resume (unpause), looping

    let url = match args.single::<String>() {
        Ok(url) => url,
        Err(_) => {
            msg.reply(ctx, "Must provide an URL to a video or audio")
                .await?;
            return Ok(()); // maybe at some point impl pause and this would just unpause or say above (if not paused)
        }
    };
    if !url.starts_with("http") {
        msg.reply(
            ctx,
            "Doesn't look like a valid URL (use `https://youtube.com` instead of `youtube.com`)",
        )
        .await?;
        return Ok(());
    }
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild.id) {
        let mut handler = handler_lock.lock().await;

        let source = match Restartable::ytdl(url, true).await {
            Ok(source) => source.into(),
            Err(why) => {
                println!("Err starting source: {:?}", why);

                msg.reply(&ctx.http, "Error sourcing ffmpeg").await?;

                return Ok(());
            }
        };
        handler.play_source(source);

        msg.reply(&ctx.http, "Playing song").await?;
    } else {
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
        let handler = manager.join(guild.id, connect_to).await.0;
        let mut lock = handler.lock().await;
        let source = match Restartable::ytdl(url, true).await {
            Ok(source) => source.into(),
            Err(why) => {
                println!("Err starting source: {:?}", why);

                msg.reply(&ctx.http, "Error sourcing ffmpeg").await?;

                return Ok(());
            }
        };
        lock.play_source(source);

        msg.reply(&ctx.http, "Playing song").await?;
    }

    Ok(())
}
