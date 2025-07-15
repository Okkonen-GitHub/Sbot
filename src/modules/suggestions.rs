use serde::{Serialize, Deserialize};
use serde_json::Value;
use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandResult},
    model::{channel::Message, id::{ChannelId, MessageId}}, builder::{CreateEmbed, CreateMessage, CreateEmbedAuthor},
};
use super::db::*;
use super::utils::{get_pwd, remove_prefix_from_message};

#[derive(Serialize, Deserialize)]
pub struct Suggestion {
    pub submitter_id: u64,
    pub suggestion: String,
    pub timestamp: String,
    pub id: u64,
    pub message_id: u64,
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
            let suggestion_channel = ChannelId(suggestion_channel.as_u64().unwrap());
            
            // to avoid duplicates
            let mut suggestion_id = 0;
            for suggestion in data["suggestions"].as_array().unwrap() {
                let current = suggestion["id"].as_u64().unwrap();
                if current > suggestion_id {
                    suggestion_id = current;
                }
            }
            suggestion_id += 1; // add one to the current highest id

            #[cfg(debug_assertions)]
            let bot_prefix = "d";
            #[cfg(not(debug_assertions))]
            let bot_prefix = "s";
            
            let no_prefix = remove_prefix_from_message(&msg.content, bot_prefix);
            let suggestion_content = no_prefix.split(" ").skip(1).collect::<Vec<&str>>().join(" ");
            

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
            {
                // save to db
                let suggestion = Suggestion {
                    submitter_id: msg.author.id.0,
                    suggestion: suggestion_content.clone(),
                    timestamp: msg.timestamp.to_rfc3339(),
                    id: suggestion_id,
                    message_id: added_suggestion.id.0
                };
                let suggestions = data["suggestions"].as_array_mut().unwrap();
                suggestions.push(serde_json::to_value(suggestion).unwrap());
                db.set(&guild_id.to_string(), data).await;
            }

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
#[aliases("removesuggestion", "rs", "deletesuggestion", "remove", "delete", "ds")]
async fn remove_suggestion(ctx: &Context, msg: &Message) -> CommandResult {
    // check if used in a guild
    let guild_id = match msg.guild_id {
        Some(id) => id.0,
        None => {
            msg.reply(ctx, "You must be in a server to use this command.").await?;
            return Ok(());
        }
    };
    // get the suggestion id
    #[cfg(debug_assertions)]
    let bot_prefix = "d";
    #[cfg(not(debug_assertions))]
    let bot_prefix = "s";
    let no_prefix = remove_prefix_from_message(&msg.content, bot_prefix);
    let suggestion_id = match no_prefix.split(" ").nth(1).unwrap().parse::<u64>() {
        Ok(id) => id,
        Err(_) => {
            msg.reply(ctx, "Invalid suggestion id").await?;
            return Ok(());
        }
    };
    // check if the user is the same as the original submitter
    let db = JsonDb::new(get_pwd().join("data/guilds.json"));
    let data = db.get(&guild_id.to_string()).await;
    match data {
        Some(data) => {
            let mut suggestions = data["suggestions"].as_array().unwrap().to_owned();
            let suggestion = suggestions.iter().find(|s| s["id"].as_u64().unwrap() == suggestion_id);
            match suggestion {
                Some(suggestion) => {
                    if suggestion["submitter_id"].as_u64().unwrap() == msg.author.id.0 {
                        // then delete the suggestion message
                        let suggestion_channel = ChannelId(data.get("suggestion_channel").unwrap().as_u64().unwrap());
                        let msg_id = MessageId(suggestion["message_id"].as_u64().unwrap());
                        suggestion_channel.delete_message(ctx, msg_id).await?;

                        // remove the suggestion from the db
                        let index = suggestions.iter().position(|s| s["id"].as_u64().unwrap() == suggestion_id).unwrap();
                        suggestions.remove(index);
                        let new_data = serde_json::json!({
                            "suggestion_channel": suggestion_channel.0,
                            "suggestions": suggestions,
                            "welcome_channel": data["welcome_channel"],
                        });
                        db.set(&guild_id.to_string(), new_data).await;
                        msg.reply(ctx, "Suggestion removed").await?;
                    }
                }
                None => {
                    msg.reply(ctx, "Invalid suggestion id").await?;
                }
            }
        },
        None => {
            msg.reply(ctx, "No previous suggesions").await?;
            return Ok(());
        }
    }
    Ok(())
}

// `<prefix> editsuggestion [suggestion_id] [edited suggestion]`
#[command]
#[aliases("editsuggestion", "es", "esuggestion")]
async fn edit_suggestion(ctx: &Context, msg: &Message) -> CommandResult {
    // check if command was used in a guild
    let guild_id = match msg.guild_id {
        Some(id) => id.0,
        None => {
            msg.reply(ctx, "You must be in a server to use this command.").await?;
            return Ok(());
        }
    };

    #[cfg(debug_assertions)]
    let bot_prefix = "d";
    #[cfg(not(debug_assertions))]
    let bot_prefix = "s";

    let no_prefix = remove_prefix_from_message(&msg.content, bot_prefix);
    // println!("{:?}", no_prefix.split(" "));
    // get the suggestion id and the edited suggestion from the message
    let suggestion_id = match no_prefix.split(" ").nth(1) {
        Some(id) => match id.parse::<u64>() {
            Ok(id) => id,
            Err(_) => {
                msg.reply(ctx, "Invalid suggestion id (must be a number)").await?;
                return Ok(());
            }
        },
        None => {
            msg.reply(ctx, "You must specify a suggestion id.").await?;
            return Ok(());
        }
    };
    let edited_suggestion = no_prefix.split(" ").skip(2).collect::<Vec<&str>>().join(" ");

    // find the suggestion by id
    let db = JsonDb::new(get_pwd().join("data/guilds.json"));
    let data = db.get(&guild_id.to_string()).await;
    match data {
        Some(data) => {
            let suggestions = data["suggestions"].as_array().unwrap();
            let mut suggestion = match suggestions.iter().find(|s| s["id"].as_u64().unwrap() == suggestion_id) {
                Some(s) => s.to_owned(),
                None => {
                    msg.reply(ctx, "Suggestion not found").await?;
                    return Ok(());
                }
            };
            // check if the submitter is the same as the author of the message
            if suggestion["submitter_id"].as_u64().unwrap() != msg.author.id.0 {
                msg.reply(ctx, "You can only edit your own suggestions.").await?;
                return Ok(());
            }
            // edit the suggestion
            suggestion["suggestion"] = Value::String(edited_suggestion.clone());
            let new_data = serde_json::json!({
                "suggestion_channel": data["suggestion_channel"].as_u64().unwrap(),
                "suggestions": suggestions,
                "welcome_channel": data["welcome_channel"].as_u64()
            });
            // save to db
            db.set(&guild_id.to_string(), new_data).await; // something is broken here but I'm too sleepy fix it now (doesn't update the suggestion)

            // edit the message in the suggestion channel
            let message_id = suggestion["message_id"].as_u64().unwrap();
            let suggestion_channel = ChannelId(data.get("suggestion_channel").unwrap().as_u64().unwrap());

            suggestion_channel.edit_message(ctx, message_id, |m| {
                m.embed(|e: &mut CreateEmbed| {
                    e.title(format!("Suggestion #{} (edited)", suggestion_id))
                    .description(format!("{}", edited_suggestion))
                    .timestamp(msg.timestamp)
                    .author(|a: &mut CreateEmbedAuthor| {
                        a.name(msg.author.name.clone())
                        .icon_url(msg.author.avatar_url().unwrap_or("".to_string()))
                    });
                    e
                });
                m
            }).await?;
            msg.reply(ctx, "Suggestion edited").await?;
        },
        None => {
            msg.reply(ctx, "No suggestions in this guild from before").await?;
            return Ok(());
        }
    }
    Ok(())
}


//TODO only server admins should be able to use this command
//TODO implement checks: #[check(Admin)] or #[admin_only]
// Almost fully made by copilot but pretty basic - just slightly different to edit_suggestion
#[command]
#[aliases("acceptsuggestion", "as", "accept")]
async fn accept_suggestion(ctx: &Context, msg: &Message) -> CommandResult {
    // check if command was used in a guild
    let guild_id = match msg.guild_id {
        Some(id) => id.0,
        None => {
            msg.reply(ctx, "You must be in a server to use this command.").await?;
            return Ok(());
        }
    };
    // get the suggestion id from the message
    #[cfg(debug_assertions)]
    let bot_prefix = "d";
    #[cfg(not(debug_assertions))]
    let bot_prefix = "s";

    let no_prefix = remove_prefix_from_message(&msg.content, bot_prefix);
    let suggestion_id = match no_prefix.split(" ").nth(1) {
        Some(id) => match id.parse::<u64>() {
            Ok(id) => id,
            Err(_) => {
                msg.reply(ctx, "Invalid suggestion id (must be a number)").await?;
                return Ok(());
            }
        },
        None => {
            msg.reply(ctx, "You must specify a suggestion id.").await?;
            return Ok(());
        }
    };
    // get the message id
    let db = JsonDb::new(get_pwd().join("data/guilds.json"));
    let data = db.get(&guild_id.to_string()).await;
    match data {
        Some(data) => {
            let suggestions = data["suggestions"].as_array().unwrap();
            match suggestions.iter().find(|s| s["id"].as_u64().unwrap() == suggestion_id) {
                Some(s) => {
                    let suggestion = s.to_owned();
                    // edit the message in the suggestion channel
                    let channel_id = ChannelId(data.get("suggestion_channel").unwrap().as_u64().unwrap());
                    let message_id = suggestion["message_id"].as_u64().unwrap();
                    channel_id.edit_message(ctx, message_id, |m| {
                        m.embed(|e: &mut CreateEmbed| {
                            e.title(format!("Suggestion #{} (accepted)", suggestion_id))
                            .description(format!("{}", suggestion["suggestion"].as_str().unwrap()))
                            .timestamp(msg.timestamp)
                            .author(|a: &mut CreateEmbedAuthor| {
                                a.name(msg.author.name.clone())
                                .icon_url(msg.author.avatar_url().unwrap_or("".to_string()))
                            });
                            e
                        });
                        m
                    }).await?;
                    // idc to add some accepted boolean to the database
                },
                None => {
                    msg.reply(ctx, "Suggestion not found").await?;
                    return Ok(());
                }
            };
        },
        None => {
            msg.reply(ctx, "No suggestions in this guild from before").await?;
            return Ok(());
        }
    }
    Ok(())
}

//TODO only server admins should be able to use this command
//TODO implement checks: #[check(Admin)] or #[admin_only]
#[command]
#[aliases("suggestions")]
async fn set_suggestion_channel(ctx: &Context, msg: &Message) -> CommandResult {
    // check if user has specified a channel (a channel id or a channel mention, too lazy to implement search by channel name)
    // if not just use current channel

    #[cfg(debug_assertions)]
    let bot_prefix = "d";
    #[cfg(not(debug_assertions))]
    let bot_prefix = "s";

    let no_prefix = remove_prefix_from_message(&msg.content, bot_prefix);
    let channel = match no_prefix.split(" ").nth(1) {
        Some(id) => match id.parse::<u64>() {
            Ok(id) => ChannelId(id),
            Err(_) => {
                // here user has either mentioned a channel or has fucked up something
                let id = id.trim_start_matches("<#").trim_end_matches(">"); // remove <# and >, leaving only numbers if it is a channel mention
                match id.parse::<u64>() {
                    Ok(id) => ChannelId(id),
                    Err(_) => {
                        msg.reply(ctx, "Invalid channel id").await?;
                        return Ok(());
                    }
                }
            }
        },
        None => {
            msg.channel_id
        }
    };


    if let Some(guild_id) = msg.guild_id {
        let path = get_pwd().join("data/guilds.json");
        let db = JsonDb::new(path);
        if let Some(mut guild_data) = db.get(&guild_id.to_string()).await {
            guild_data["suggestion_channel"] = serde_json::json!(channel.0);
            db.set(&guild_id.to_string(), guild_data).await;
        } else {
            let guild_data = serde_json::json!({
                "suggestion_channel": channel.0, // Will later just use the Guild struct to serialize this, maybe
                "suggestions": [],
                "welcome_channel": None::<u64>,
            });
            db.set(&guild_id.to_string(), guild_data).await;
        }
    }
    msg.reply(ctx, "Suggestion channel set.").await?;
    Ok(())
}