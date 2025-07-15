use super::{db::*, utils::{get_pwd, remove_prefix_from_message}, suggestions::Suggestion};
use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandResult},
    model::{channel::Message, id::ChannelId, guild::Member},
};

// this could probably be inlined (#[inline(always)]) because it is only used in one place
// too lasy to make the message embedded
pub async fn say_hello(ctx: &Context, member: &Member) {
    let guild_id = member.guild_id;
    // get the welcome channel, if any, then send a welcome message there, otherwise do nothing
    let db = JsonDb::new(get_pwd().join("data/guilds.json"));
    match db.get(&guild_id.0.to_string()).await {
        Some(data) => {
            let welcome_channel = data["welcome_channel"].as_u64();
            if let Some(welcome_channel) = welcome_channel {
                // check if there is a custom message, otherwise use default
                let content = if let Some(welcome_msg) = data["welcome_message"].as_str() {
                    // custom message components are {user_tag}, {user_name}, {guild_name}. Maybe more some day (never)
                    // so we just .replace them in the string
                    welcome_msg.to_owned()
                        .replace("{guild_name}", &guild_id.name(ctx).unwrap_or("?".to_owned()))
                        .replace("{user_name}", &member.user.name)
                        .replace("{user_tag}", &member.user.tag())
                        // pretty easy to add more
                } else {
                    format!("Welcome to {}, {}", guild_id.name(ctx).unwrap_or("?".to_string()), member.user.name)
                };
                // use `let _` to ignore any errors. 
                let _ = ChannelId(welcome_channel).say(ctx, content).await;
            } // if welcome channel is not set then we don't need to do anything
        }
        None => {
            // do nothing, no channel set
        }
    }
}


// <prefix> setwelcomemessage Welcome to {guild_name}, {user_name}!
// custom message components are {user_tag}, {user_name}, {guild_name}. Maybe more some day (never)
#[command]
#[aliases("welcomemessage", "setwelcomemsg", "setwelcomemessage", "sms")]
async fn set_welcome_message(ctx: &Context, msg: &Message) -> CommandResult {
    let db = JsonDb::new(get_pwd().join("data/guilds.json"));
    let guild_id = match msg.guild_id {
        Some(id) => id,
        None => {
            msg.channel_id.say(&ctx.http, "This command can only be used in a server.").await?;
            return Ok(());
        }
    };
    
    // remove prefix
    #[cfg(debug_assertions)]
    let prefix = "d";
    #[cfg(not(debug_assertions))]
    let prefix = "s";

    let no_prefix = remove_prefix_from_message(&msg.content, prefix);
    let welcome_message = no_prefix.split(" ").skip(1).collect::<Vec<&str>>().join(" ");
    match db.get(&guild_id.to_string()).await {
        Some(data) => {
            // edit the previous welcome message
            let new_data = serde_json::json!({
                "suggestion_channel": data["suggestion_channel"],
                "suggestions": data["suggestions"],
                "welcome_channel": data["welcome_channel"],
                "welcome_message": welcome_message,
            });
            db.set(&guild_id.to_string(), new_data).await;
            msg.reply(ctx, "Welcome message updated.").await?;
        },
        None => {
            let new_data = serde_json::json!({
                "suggestion_channel": None::<u64>,
                "suggestions": Vec::<Suggestion>::new(),
                "welcome_channel": None::<u64>,
                "welcome_message": welcome_message,
            });
            db.set(&guild_id.to_string(), new_data).await;
            msg.reply(ctx, "Welcome message set, now you should set a welcome channel (`setwelcomechannel`)").await?;
        }
    }
    
    Ok(())
}

#[command]
#[aliases("welcomechannel", "setwelcomechannel", "smc")]
async fn set_welcome_channel(ctx: &Context, msg: &Message) -> CommandResult {
    let db = JsonDb::new(get_pwd().join("data/guilds.json"));
    let guild_id = match msg.guild_id {
        Some(id) => id,
        None => {
            msg.channel_id.say(&ctx.http, "This command can only be used in a server.").await?;
            return Ok(());
        }
    };

    #[cfg(debug_assertions)]
    let bot_prefix = "d";
    #[cfg(not(debug_assertions))]
    let bot_prefix = "s";

    // determine what channel should be used (current or a mentioned channel)
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
            // if no channel is mentioned, use the current channel
            msg.channel_id
        }
    };
    let data = db.get(&guild_id.0.to_string()).await;
    match data {
        Some(data) => {
            // update the db
            let new_data = serde_json::json!({
                "suggestion_channel": data["suggestion_channel"], // might be None, should be fine. same for suggestions
                "suggestions": data["suggestions"],
                "welcome_channel": channel.0,
                "welcome_message": data["welcome_message"]
            });
            db.set(&guild_id.0.to_string(), new_data).await;
            msg.channel_id.say(&ctx.http, &format!("Welcome channel set to {}", channel)).await?;
        }
        None => {
            // update the db
            let new_data = serde_json::json!({
                "suggestion_channel": None::<u64>, 
                "suggestions": None::<Vec<Suggestion>>,
                "welcome_channel": channel.0,
                "welcome_message": None::<String>
            });
            db.set(&guild_id.0.to_string(), new_data).await;
            msg.channel_id.say(&ctx.http, &format!("Welcome channel set to {}", channel)).await?;

        }
    }

    Ok(())
}
