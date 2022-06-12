use serde::Serialize;
use serde_json::Value;
use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandResult},
    model::{channel::{Message}, id::ChannelId}, builder::{CreateEmbed, CreateMessage, CreateEmbedAuthor},
};
use super::db::*;
use super::utils::get_pwd;

#[derive(Serialize)]
pub struct Suggestion {
    pub submitter: String,
    pub suggestion: String,
    pub timestamp: String,
    pub id: String,
}


// `<prefix> suggest my suggestion`
#[command]
async fn suggest(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = match msg.guild_id {
        Some(id) => id.0,
        None => {
            msg.reply(ctx, "You must be in a server to use this command.").await?;
            return Ok(());
        }
    };
    let db = JsonDb::new(get_pwd().join("data/guilds.json"));
    let data = db.get(&guild_id.to_string()).await;
    match data {
        Some(mut data) => {
            // println!("{:?}", data);
            // unwrap hell
            let suggestion_channel = data.get("suggestion_channel").unwrap();
            let suggestion_channel = ChannelId(suggestion_channel.as_str().unwrap().parse::<u64>().unwrap());
            

            let suggestion_id = (data["suggestions"].as_array().unwrap_or(&Vec::<Value>::new()).len() + 1) as u64;

            let suggestion_content = &msg.content.split(" ").skip(2).collect::<Vec<&str>>().join(" ");
            
            {
                // save to db
                let suggestion = Suggestion {
                    submitter: msg.author.name.clone(),
                    suggestion: suggestion_content.clone(),
                    timestamp: msg.timestamp.to_rfc3339(),
                    id: suggestion_id.to_string(),
                };
                let suggestions = data["suggestions"].as_array_mut().unwrap();
                suggestions.push(serde_json::to_value(suggestion).unwrap());
                db.set(&guild_id.to_string(), data).await;
            }

            let added_suggestion = suggestion_channel
                .send_message(ctx, |m: &mut CreateMessage| {
                    m.embed(|e: &mut CreateEmbed| {
                        e.title(format!("Suggestion #{}", suggestion_id))
                        .description(format!("{}", suggestion_content))
                        .timestamp(msg.timestamp)
                        .author(|a: &mut CreateEmbedAuthor| {
                            a.name(msg.author.name.clone())
                            .icon_url(msg.author.avatar_url().unwrap_or("".to_string()))
                        });
                        e
                    });
                    m
                }).await?;

            // why does this all work?

            match added_suggestion.react(ctx, '✅').await {
                Ok(_) => {
                    let _ = added_suggestion.react(ctx, '❌').await;
                }
                Err(why) => {
                    msg.reply(ctx, format!("Error: {}", why)).await?;
                }
            }
        },
        None => {
            msg.reply(ctx, "No suggestion channel set").await?;
            return Ok(());
        }

    }

    
    Ok(())
}


#[command]
#[aliases("suggestions")]
async fn set_suggestion_channel(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id;
    let channel = msg.channel_id;
    if let Some(guild_id) = guild_id {
        let path = get_pwd().join("data/guilds.json");
        let db = JsonDb::new(path);
        if let Some(mut guild_data) = db.get(&guild_id.to_string()).await {
            guild_data["suggestion_channel"] = serde_json::Value::String(channel.to_string());
            db.set(&guild_id.to_string(), guild_data).await;
        } else {
            let guild_data = serde_json::json!({
                "suggestion_channel": channel.to_string(),
                "suggestions": [],
            });
            db.set(&guild_id.to_string(), guild_data).await;
        }
    }
    msg.reply(ctx, "Suggestion channel set.").await?;
    Ok(())
}