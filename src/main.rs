mod modules;

use crate::modules::{core::*, owner::*, utils::*};

use std::{collections::HashSet, env, sync::Arc, fs, io::Write};



use serenity::prelude::*;
use serenity::{
    async_trait,
    client::bridge::gateway::ShardManager,
    framework::standard::{macros::group, StandardFramework},
    http::Http,
    model::gateway::Ready,
};
use tokio::sync::Mutex;

//* add commands to a group, this means you Okkonen!!!!
#[group]
#[commands(ping, about, info, quit, uptime, fullinfo)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    // async fn message(&self, ctx: Context, msg: Message) {
    //     if msg.content == "sping" {
    //         println!("Shard {}", ctx.shard_id);

    //         if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
    //             println!("Error {:?}", why);
    //         }
    //     }
    // }

    async fn ready(&self, ctx: Context, ready: Ready) {
        let guilds = match ready.user.guilds(ctx).await {
            Ok(v) => v.len().to_string(),
            _ => String::from("?"),
        };
        println!(
            "{} is connected & total guilds: {} ",
            ready.user.name, guilds
        );
    }
}

struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Failed to load .env");
    
    let token: String;
    let prefix: &str;

    if cfg!(not(debug_assertions)) {
        println!("Running in release mode");
        token = env::var("DISCORD_TOKEN").expect("token");
        prefix = "s";
    } else { // development mode
        token = env::var("DEV_TOKEN").expect("token");
        prefix = "d"; // d for now...
    }

    if cfg!(debug_assertions) {
        let path = get_pwd().join("data/");

        // println!("{:?}", &path);

        if !path.exists() {
            fs::create_dir(&path.join("data/")).expect("Failed to create data directory");
        }

        // fs::File::create("text.txt").expect("Failed to create text file");

        let a = fs::File::open(path.join("data.json")).unwrap_or_else(|_| {
            let mut b = fs::File::create(path.join("data.json")).unwrap();
            b.write(b"{}").unwrap();
            b
        });
        drop(a);
    }

    let http = Http::new_with_token(&token);

    // fetch your bot's owners and id
    let (owners, bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            if let Some(team) = info.team {
                for member in team.members {
                    owners.insert(member.user.id);
                }
            } else {
                owners.insert(info.owner.id);
            }
            match http.get_current_user().await {
                Ok(bot_id) => (owners, bot_id.id),
                Err(why) => panic!("Could not access the bot id: {:?}", why),
            }
        }
        Err(why) => panic!("Could not access application info: {:?}", why),
    };
    // println!("{:?}", owners);
    let framework = StandardFramework::new()
        .configure(|c| {
            c.prefix(prefix)
                .owners(owners)
                .with_whitespace(true)
                .on_mention(Some(bot_id))
                .delimiters(vec![", ", ","])
        })
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
