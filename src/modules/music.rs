use serenity::{
    builder::{CreateEmbedFooter, CreateMessage},
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
};

use super::utils::{remove_prefix_from_message, seconds_to_human};

use songbird::{input::Restartable, Call}; // for looping and yt searches (first result) (Restartable::*)
use std::{sync::Arc, time::Duration};
use tokio::sync::Mutex;

#[command]
#[only_in(guilds)]
async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();
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
    let guild = msg.guild(&ctx.cache).unwrap();
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
    let guild = msg.guild(&ctx.cache).unwrap();
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
    let guild = msg.guild(ctx).unwrap();
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

#[inline(always)]
async fn add_to_queue_url(
    url: String,
    handler: Arc<Mutex<Call>>,
    msg: &Message,
    ctx: &Context,
) -> CommandResult {
    let mut lock = handler.lock().await;
    let source = match Restartable::ytdl(url, true).await {
        Ok(source) => source.into(),
        Err(_why) => {
            msg.reply(&ctx.http, "Error sourcing ffmpeg (try the command again)").await?;
            return Ok(());
        }
    };
    lock.enqueue_source(source);
    Ok(())
}

#[inline(always)]
async fn add_to_queue_search(
    search: String,
    handler: Arc<Mutex<Call>>,
    msg: &Message,
    ctx: &Context,
) -> CommandResult {
    let mut lock = handler.lock().await;
    let source = match Restartable::ytdl_search(search, true).await {
        Ok(source) => source.into(),
        Err(why) => {
            msg.reply(&ctx.http, format!("Error sourcing ffmpeg: {why} (try the command again)"))
                .await?;
            return Ok(());
        }
    };
    lock.enqueue_source(source);
    Ok(())
}

#[command]
#[only_in(guilds)]
#[aliases("p")]
async fn play(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();
    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    #[cfg(debug_assertions)]
    let prefix = "d";
    #[cfg(not(debug_assertions))]
    let prefix = "s";

    let mut no_prefix = remove_prefix_from_message(&msg.content, prefix);
    let use_url = match no_prefix.split(" ").nth(1) {
        Some(possibly_url) => {
            let temp = possibly_url.starts_with("http");
            no_prefix = no_prefix
                .split(" ")
                .skip(1)
                .collect::<Vec<&str>>()
                .join(" ");
            temp
        }
        None => {
            msg.reply(ctx, "You need to specify a song (a youtube search or link)")
                .await?;
            return Ok(());
        }
    };

    if let Some(handler_lock) = manager.get(guild.id) {
        if use_url {
            add_to_queue_url(no_prefix, handler_lock, msg, ctx).await?;
        } else {
            add_to_queue_search(no_prefix, handler_lock, msg, ctx).await?;
        }

        msg.reply(&ctx.http, "Added song to queue.").await?;
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
        let handler = manager.join(guild.id, connect_to).await.0;
        if use_url {
            add_to_queue_url(no_prefix, handler, msg, ctx).await?;
        } else {
            add_to_queue_search(no_prefix, handler, msg, ctx).await?;
        }

        msg.reply(&ctx.http, "Added song to queue.").await?;
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn stop(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(ctx).unwrap();
    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation")
        .clone();

    if let Some(handler_lock) = manager.get(guild.id) {
        let handler = handler_lock.lock().await;
        handler.queue().stop();
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn skip(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(ctx).unwrap();
    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation")
        .clone();

    if let Some(handler_lock) = manager.get(guild.id) {
        let handler = handler_lock.lock().await;
        if let Err(why) = handler.queue().skip() {
            msg.reply(ctx, format!("Something went wrong: {why}"))
                .await?;
        }
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn pause(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(ctx).unwrap();
    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation")
        .clone();

    if let Some(handler_lock) = manager.get(guild.id) {
        let _ = handler_lock.lock().await.queue().pause();
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn resume(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(ctx).unwrap();
    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation")
        .clone();

    if let Some(handler_lock) = manager.get(guild.id) {
        let _ = handler_lock.lock().await.queue().resume();
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
#[aliases("np", "nowplaying", "playingnow", "pn")]
async fn playing(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(ctx).unwrap();
    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation")
        .clone();

    if let Some(handler_lock) = manager.get(guild.id) {
        match handler_lock.lock().await.queue().current() {
            Some(handler) => {
                let metadata = handler.metadata().to_owned();
                let track_info = handler.get_info().await.unwrap(); // there has to be a song playing
                msg.channel_id
                    .send_message(&ctx, |m: &mut CreateMessage| {
                        m.content("Now playing:").embed(|e| {
                            e.title(format!("{}", metadata.title.unwrap_or("?".to_string())))
                                .description(format!(
                                    "{} / {}",
                                    seconds_to_human(track_info.position.as_secs()),
                                    seconds_to_human(
                                        metadata
                                            .duration
                                            .unwrap_or(Duration::from_secs(0))
                                            .as_secs()
                                    )
                                ))
                                .image(
                                    metadata
                                        .thumbnail
                                        .unwrap_or("https://http.cat/404".to_owned()),
                                )
                                .url(
                                    metadata
                                        .source_url
                                        .unwrap_or("https://http.cat/404".to_string()),
                                )
                                .timestamp(
                                    metadata.date.unwrap_or("2004-06-08T16:04:23".to_string()),
                                )
                        })
                    })
                    .await?;
            }
            None => {
                msg.reply(ctx, "Nothing is playing currently").await?;
                return Ok(());
            }
        };
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
// r#loop since loop is a keyword but we want to use it as a command name
async fn r#loop(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let guild = msg.guild(ctx).unwrap();
    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation")
        .clone();
    match args.single() {
        Ok(arg) => {
            if let Some(handler_lock) = manager.get(guild.id) {
                handler_lock
                    .lock()
                    .await
                    .queue()
                    .current()
                    .unwrap()
                    .loop_for(arg)?;
            }
        }
        Err(_) => {
            if let Some(handler_lock) = manager.get(guild.id) {
                handler_lock
                    .lock()
                    .await
                    .queue()
                    .current()
                    .unwrap()
                    .enable_loop()?;
            }
        }
    };

    Ok(())
}

#[command]
#[only_in(guilds)]
#[aliases("disableloop", "deloop")]
async fn unloop(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(ctx).unwrap();
    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation")
        .clone();

    if let Some(handler_lock) = manager.get(guild.id) {
        if let Err(why) = handler_lock
            .lock()
            .await
            .queue()
            .current()
            .unwrap()
            .disable_loop()
        {
            msg.reply(ctx, format!("Something went wrong: {why}"))
                .await?;
        } else {
            msg.reply(ctx, "Disabling loop..").await?;
        }
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
#[aliases("q")]
async fn queue(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(ctx).unwrap();
    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation")
        .clone();

    if let Some(handler_lock) = manager.get(guild.id) {
        let queue = handler_lock.lock().await.queue().to_owned();
        let queuelen = queue.len();
        if queue.is_empty() {
            msg.reply(ctx, "Queue is empty").await?;
            return Ok(());
        }
        let current_q = queue.current_queue();
        let queue_msg = String::from_iter(
            current_q[..{
                if queuelen <= 10 {
                    queuelen
                } else {
                    10
                }
            }]
                .iter()
                .map(|trackhandle| {
                    let metadata = trackhandle.metadata().to_owned();
                    format!(
                        "**{}** ({})\n",
                        metadata.title.unwrap_or("?".to_string()),
                        seconds_to_human(
                            metadata
                                .duration
                                .unwrap_or(Duration::from_secs(0))
                                .as_secs()
                        ),
                    )
                }),
        );

        msg.channel_id
            .send_message(ctx, |m| {
                m.embed(|e| {
                    e.title("Current queue").description(queue_msg).footer(
                        |f: &mut CreateEmbedFooter| {
                            f.text(format!(
                                "In total {} songs in queue currently (showing the first ten only)",
                                queuelen
                            ))
                        },
                    )
                })
            })
            .await?;
    };

    Ok(())
}

#[command]
#[only_in(guilds)]
#[aliases("v", "vol")]
async fn volume(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(ctx).unwrap();

    #[cfg(debug_assertions)]
    let prefix = "d";
    #[cfg(not(debug_assertions))]
    let prefix = "s";

    let no_prefix = remove_prefix_from_message(&msg.content, prefix);
    // remove the command used (as it could be either "v", or "volume")
    let args = no_prefix
        .split(" ")
        .skip(1)
        .collect::<Vec<&str>>()
        .join(" ");

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice blah blah blah")
        .clone();
    let handler = match manager.get(guild.id) {
        Some(handler) => handler,
        None => {
            msg.reply(ctx, "Nothing is playing").await?;
            return Ok(());
        }
    };
    let current_song = match handler.lock().await.queue().current() {
        Some(song) => song,
        None => return Ok(()),
    };
    let current_volume = current_song.get_info().await.unwrap().volume;
    if args.len() < 1 {
        msg.reply(ctx, format!("Current volume is {}", current_volume * 100.0))
            .await?;
        return Ok(());
    }

    match args[1..].parse::<u8>() {
        Ok(num) => {
            let volume;
            if args.starts_with("+") {
                volume = (num as f32 / 100.0) + current_volume;
            } else if args.starts_with("-") {
                volume = current_volume - (num as f32 / 100.0);
            } else {
                if let Ok(num) = args.parse::<u8>() {
                    volume = num as f32 / 100.0;
                } else {
                    msg.reply(ctx, format!("Current volume is {}", current_volume * 100.0))
                        .await?;
                    return Ok(());
                }
            }
            current_song.set_volume(volume).unwrap();
        }
        Err(_) => {
            msg.reply(ctx, format!("Current volume is {}", current_volume * 100.0))
                .await?;
        }
    }

    Ok(())
}
