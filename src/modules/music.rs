// use ::serenity::all::CreateEmbedFooter;
use poise::serenity_prelude as serenity;
use std::str::from_utf8;

use super::utils::seconds_to_human;

use crate::{Context, Error};

use songbird::{input::YoutubeDl, Call}; // for looping and yt searches (first result) (Restartable::*)
use std::{process::Command, sync::Arc, time::Duration};
use tokio::{sync::Mutex, time::interval};

#[poise::command(prefix_command, slash_command)]
// #[poise::]
pub async fn join(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();

    let channel_id = ctx
        .guild()
        .unwrap()
        .voice_states
        .get(&ctx.author().id)
        .and_then(|voice_state| voice_state.channel_id);

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            ctx.reply("You're not in any voice channel").await?;
            return Ok(());
        }
    };
    ctx.data().songbird.join(guild_id, connect_to).await?;

    Ok(())
}

#[poise::command(prefix_command, slash_command)]
// #[only_in(guilds)]
pub async fn leave(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();

    match ctx.data().songbird.get(guild_id) {
        Some(_) => {
            let _a = ctx.data().songbird.remove(guild_id).await;
        }
        None => (),
    }

    Ok(())
}
#[poise::command(prefix_command, slash_command, aliases("unmute"))]
// #[only_in(guilds)]
// #[aliases("unmute")]
pub async fn mute(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();

    match ctx.data().songbird.get(guild_id) {
        Some(call) => {
            let mut lock = call.lock().await;
            let is_mute = lock.is_mute();
            lock.mute(!is_mute).await?;
        }
        None => {
            ctx.reply("Not in a voice channel").await?;
        }
    }

    Ok(())
}

#[poise::command(prefix_command, slash_command, aliases("undeafen"))]
// #[only_in(guilds)]
pub async fn deafen(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();

    match ctx.data().songbird.get(guild_id) {
        Some(call) => {
            let mut lock = call.lock().await;
            let is_deafened = lock.is_deaf();
            lock.deafen(!is_deafened).await?;
        }
        None => {
            ctx.reply("Not in a voice channel").await?;
        }
    }
    Ok(())
}

// if url has a pattern that seems like a playlist, try to add the whole playlist to queue (one at
// a time)
#[inline(always)]
async fn add_to_queue_url(
    url: String,
    handler: Arc<Mutex<Call>>,
    ctx: Context<'_>,
) -> Result<(), Error> {
    // will return a list of urls if it's a playlist
    // or a lenght 1 list if it's just a song
    fn extract_urls(url: &str) -> Vec<String> {
        let ytdl_args = [
            "--ignore-config",
            "--no-warning",
            "--skip-download",
            "--flat-playlist",
            "-I",
            "0:50",
            "--print",
            "webpage_url",
            url,
        ];
        let cmd = Command::new("yt-dlp")
            .args(&ytdl_args)
            .output()
            .expect("Yt-dlp not found");
        let s = from_utf8(&cmd.stdout).expect("Not parsed");

        let mut result = Vec::with_capacity(10);
        for line in s.lines() {
            result.push(line.to_owned());
        }

        result
    }
    let client = reqwest::Client::new();

    let mut lock = handler.lock().await;
    let mut successses = 0;

    let urls = extract_urls(&url);

    for url in urls {
        let source = YoutubeDl::new(client.clone(), url);
        successses += 1;
        lock.enqueue(source.into()).await;
    }
    ctx.reply(format!("Added {successses} song(s) to queue."))
        .await?;
    Ok(())
}

#[inline(always)]
async fn add_to_queue_search(
    search: String,
    handler: Arc<Mutex<Call>>,
    ctx: Context<'_>,
) -> Result<(), Error> {
    let client = reqwest::Client::new();
    let mut lock = handler.lock().await;
    let source = YoutubeDl::new_search(client, search);
    let queue = lock.enqueue_input(source.into()).await;
    queue.play()?;

    ctx.reply("Added song to queue.").await?;
    Ok(())
}

#[poise::command(prefix_command, slash_command, aliases("p"))]
// #[only_in(guilds)]
pub async fn play(
    ctx: Context<'_>,
    #[description = "A youtube search or a youtube url"]
    #[rest]
    source: String,
) -> Result<(), Error> {
    // let guild = ctx.guild().unwrap();
    let guild_id = ctx.guild_id().unwrap();

    let use_url = source.starts_with("http");

    if let Some(handler_lock) = ctx.data().songbird.get(guild_id) {
        if use_url {
            add_to_queue_url(source, handler_lock, ctx).await?;
        } else {
            add_to_queue_search(source, handler_lock, ctx).await?;
        }
    } else {
        let channel_id = ctx
            .guild()
            .unwrap()
            .voice_states
            .get(&ctx.author().id)
            .and_then(|voice_state| voice_state.channel_id);

        let connect_to = match channel_id {
            Some(channel) => channel,
            None => {
                ctx.reply("You're not in any voice channel").await?;
                return Ok(());
            }
        };
        let handler = ctx.data().songbird.join(guild_id, connect_to).await?;
        if use_url {
            add_to_queue_url(source, handler, ctx).await?;
        } else {
            add_to_queue_search(source, handler, ctx).await?;
            ctx.reply("Added song to queue.").await?;
        }
    }
    auto_leave(ctx).await?;
    Ok(())
}

// Todo: Cache servers where auto_leave is already awaited
async fn auto_leave(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();

    if let Some(handler_lock) = ctx.data().songbird.get(guild_id) {
        const FIVE_MINS: u64 = 5 * 60;
        let mut interval = interval(Duration::from_secs(FIVE_MINS));
        loop {
            interval.tick().await;
            if handler_lock.lock().await.queue().is_empty() {
                let _ = ctx.data().songbird.remove(guild_id).await;
                break;
            }
        }
    }
    Ok(())
}

#[poise::command(prefix_command, slash_command)]
// #[only_in(guilds)]
pub async fn stop(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let manager = ctx.data().songbird.clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        handler.queue().stop();
    }

    Ok(())
}

#[poise::command(prefix_command, slash_command)]
// #[only_in(guilds)]
pub async fn skip(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();

    if let Some(handler_lock) = ctx.data().songbird.get(guild_id) {
        let handler = handler_lock.lock().await;
        if let Err(why) = handler.queue().skip() {
            ctx.reply(format!("Something went wrong: {why}")).await?;
        }
    }

    Ok(())
}

#[poise::command(prefix_command, slash_command)]
// #[only_in(guilds)]
pub async fn pause(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();

    if let Some(handler_lock) = ctx.data().songbird.get(guild_id) {
        let _ = handler_lock.lock().await.queue().pause();
    }

    Ok(())
}

#[poise::command(prefix_command, slash_command)]
// #[only_in(guilds)]
pub async fn resume(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();

    if let Some(handler_lock) = ctx.data().songbird.get(guild_id) {
        let _ = handler_lock.lock().await.queue().resume();
    }

    Ok(())
}

// #[poise::command(prefix_command, slash_command)]
// // #[only_in(guilds)]
// // #[aliases("np", "nowplaying", "playingnow", "pn")]
// async fn playing(ctx: Context<'_>) -> Result<(), Error> {
//     let guild_id = ctx.guild_id().unwrap();
//
//     if let Some(handler_lock) = ctx.data().songbird.get(guild_id) {
//         match handler_lock.lock().await.queue().current() {
//             Some(handler) => {
//                 let metadata = handler.get_info().await?.to_owned;
//                 let track_info = handler.get_info().await.unwrap(); // there has to be a song playing
//                 ctx.channel_id()
//                     .send_message(&ctx, |m: &mut serenity::CreateMessage| {
//                         m.content("Now playing:").embed(|e| {
//                             e.title(format!("{}", metadata.title.unwrap_or("?".to_string())))
//                                 .description(format!(
//                                     "{} / {}",
//                                     seconds_to_human(track_info.position.as_secs()),
//                                     seconds_to_human(
//                                         metadata
//                                             .duration
//                                             .unwrap_or(Duration::from_secs(0))
//                                             .as_secs()
//                                     )
//                                 ))
//                                 .image(
//                                     metadata
//                                         .thumbnail
//                                         .unwrap_or("https://http.cat/404".to_owned()),
//                                 )
//                                 .url(
//                                     metadata
//                                         .source_url
//                                         .unwrap_or("https://http.cat/404".to_string()),
//                                 )
//                                 .timestamp(
//                                     metadata.date.unwrap_or("2004-06-08T16:04:23Z".to_string()),
//                                 )
//                         })
//                     })
//                     .await?;
//             }
//             None => {
//                 ctx.reply("Nothing is playing currently").await?;
//                 return Ok(());
//             }
//         };
//     }
//
//     Ok(())
// }

#[poise::command(prefix_command, slash_command)]
// #[only_in(guilds)]
// r#loop since loop is a keyword but we want to use it as a command name
pub async fn r#loop(ctx: Context<'_>, times: Option<usize>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    match times {
        Some(arg) => {
            if let Some(handler_lock) = ctx.data().songbird.get(guild_id) {
                handler_lock
                    .lock()
                    .await
                    .queue()
                    .current()
                    .unwrap()
                    .loop_for(arg)?;
            }
        }
        None => {
            if let Some(handler_lock) = ctx.data().songbird.get(guild_id) {
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

#[poise::command(prefix_command, slash_command)]
// #[only_in(guilds)]
// #[aliases("disableloop", "deloop")]
pub async fn unloop(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();

    if let Some(handler_lock) = ctx.data().songbird.get(guild_id) {
        if let Err(why) = handler_lock
            .lock()
            .await
            .queue()
            .current()
            .unwrap()
            .disable_loop()
        {
            ctx.reply(format!("Something went wrong: {why}")).await?;
        } else {
            ctx.reply("Disabling loop..").await?;
        }
    }

    Ok(())
}

#[poise::command(prefix_command, slash_command)]
// #[only_in(guilds)]
// #[aliases("q")]
pub async fn queue(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();

    if let Some(handler_lock) = ctx.data().songbird.get(guild_id) {
        let queue = handler_lock.lock().await.queue().to_owned();
        let queuelen = queue.len();
        if queue.is_empty() {
            ctx.reply("Queue is empty").await?;
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
                .map(|_trackhandle| format!("**{}** ({})\n", "?".to_string(), "??")),
        );

        ctx.send(
            poise::CreateReply::default().embed(
                serenity::CreateEmbed::default()
                    .title("Current queue")
                    .description(queue_msg)
                    .footer(serenity::CreateEmbedFooter::new(format!(
                        "In total {} songs in queue currently (showing the first ten only)",
                        queuelen
                    ))),
            ),
        )
        .await?;
    };

    Ok(())
}

#[poise::command(prefix_command, slash_command, aliases("v", "vol"))]
// #[only_in(guilds)]
pub async fn volume(ctx: Context<'_>, #[rest] volume: Option<String>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();

    let handler = match ctx.data().songbird.get(guild_id) {
        Some(handler) => handler,
        None => {
            ctx.reply("Nothing is playing").await?;
            return Ok(());
        }
    };
    let current_song = match handler.lock().await.queue().current() {
        Some(song) => song,
        None => return Ok(()),
    };
    let current_volume = current_song.get_info().await.unwrap().volume;
    match volume {
        Some(vol) => match vol.parse::<u8>() {
            Ok(num) => {
                current_song.set_volume(num as f32 / 100.0).unwrap();
            }
            Err(_) => {
                let new_vol;
                if let Ok(num) = vol[1..].parse::<u8>() {
                    if vol.starts_with("+") {
                        new_vol = (num as f32 / 100.0) + current_volume;
                    } else if vol.starts_with("-") {
                        new_vol = current_volume - (num as f32 / 100.0);
                    } else {
                        ctx.reply("You messed up somthing").await?;
                        return Err(Error::from("Parsing volume failed"));
                    }

                    current_song.set_volume(new_vol).unwrap();
                }
            }
        },
        None => {
            ctx.reply(format!("Current volume is {}", current_volume * 100.0))
                .await?;
        }
    }

    Ok(())
}
