mod modules;

use crate::modules::core::*;

use std::{
    collections::{HashSet},
    env,
    sync::Arc,
};

use serenity::prelude::*;
use serenity::{
    async_trait,
    client::bridge::gateway::{ShardId, ShardManager},
    framework::standard::{
        macros::{command, group},
        CommandResult,
        StandardFramework,
    },
    http::Http,
    model::{
        channel::{Message},
        gateway::Ready,
    },
};
use tokio::sync::Mutex;



// add commands to a group!
#[group]
#[commands(ping, about)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "sping" {
            println!("Shard {}", ctx.shard_id);

            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                println!("Error {:?}", why);
            }
        }
    }
    
    async fn ready(&self, ctx: Context, ready: Ready) {
        let guilds = match ready.user.guilds(ctx).await {
            Ok(v) => v.len().to_string(),
            _ => String::from("?")
        };
        println!("{} is connected to {} servers",ready.user.name, guilds);
    }
}

struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

#[tokio::main]
async fn main() {

    dotenv::dotenv().expect("Failed to load .env");

    let token = env::var("DISCORD_TOKEN").expect("token");

    let http = Http::new_with_token(&token);

    // fetch your bot's owners and id
    let (owners, bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            if let Some(team) = info.team {
                owners.insert(team.owner_user_id);
            } else {
                owners.insert(info.owner.id);
            }
            match http.get_current_user().await {
                Ok(bot_id) => (owners, bot_id.id),
                Err(why) => panic!("Could not access the bot id: {:?}", why),
            }
        },
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    let framework = StandardFramework::new()
        .configure(|c| c
        .prefix("s")
        .owners(owners)
        .with_whitespace(true)
        .on_mention(Some(bot_id))
        .delimiters(vec![", ", ","])
    )
        
    .group(&GENERAL_GROUP)
    .help(&C_HELP);


    let mut client = Client::builder(token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating the client");

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
    }

    if let Err(why) = client.start_shards(2).await {
        println!("An error oocured while running the client: {:?}", why);
    }
}

#[command]
#[aliases("latency")]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    let latency = {
        let data_read = ctx.data.read().await;
        let shard_manager = data_read.get::<ShardManagerContainer>().unwrap();

        let manager = shard_manager.lock().await;
        let runners = manager.runners.lock().await;

        let runner = runners.get(&ShardId(ctx.shard_id)).unwrap();

        if let Some(duration) = runner.latency {
            format!("{:.2} ms", duration.as_millis())
        } else {
            "? ms".to_string()
        }
    };
    msg.reply(
        ctx,
        format!("Bot latency: {}", latency)
    ).await?;

    Ok(())
}

