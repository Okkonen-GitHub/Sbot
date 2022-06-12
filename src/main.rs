mod modules;

use crate::modules::{core::*, owner::*, utils::*, activities::*, suggestions::*};

use std::{env, fs, io::Write, sync::{atomic::{AtomicBool, Ordering}, Arc }, time::Duration};


use serenity::prelude::*;
use serenity::{
    async_trait,
    client::bridge::gateway::ShardManager,
    framework::standard::{macros::group, StandardFramework},
    model::gateway::Ready,
};
use tokio::sync::Mutex;

//TODO! add commands to a group, this means you Okkonen!!!!
#[group]
#[commands(ping, about, info, quit, uptime, fullinfo, betterping, suggest, set_suggestion_channel)]
struct General;

struct Handler {
    activity_loop: AtomicBool,
}

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
        let guilds = match ready.user.guilds(ctx.clone()).await {
            Ok(v) => v.len().to_string(),
            _ => String::from("?"),
        };
        println!(
            "{} is connected & total guilds: {} ",
            ready.user.name, guilds
        );
        let ctx = Arc::new(ctx);

        if !self.activity_loop.load(Ordering::Relaxed) {
            let context = Arc::clone(&ctx);
            tokio::spawn(async move {
                loop {
                    // println!("boe");
                    set_random_status(Arc::clone(&context)).await;
                    tokio::time::sleep(Duration::from_secs(10)).await;
                }
            });
        }
        self.activity_loop.swap(true, Ordering::Relaxed);
        
    }
}

struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}
struct ShuttingDown;

impl TypeMapKey for ShuttingDown {
    type Value = AtomicBool;
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

        fs::File::open(path.join("data.json")).unwrap_or_else(|_| {
            let mut b = fs::File::create(path.join("data.json")).unwrap();
            b.write(b"{}").unwrap();
            b
        });
        let path = get_pwd().join("data/");
        println!("{:?}", &path);
        fs::File::open(path.join("guilds.json")).unwrap_or_else(|_| {
            println!("wtf");
            let mut b = fs::File::create(path.join("guilds.json")).unwrap();
            b.write(b"{}").unwrap();
            b
        });
    }
    let (owners, bot_id) = get_owners(&token).await;

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
        .event_handler(Handler {
            activity_loop: AtomicBool::new(false),
        })
        .framework(framework)
        .await
        .expect("Error creating the client");

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
        data.insert::<ShuttingDown>(AtomicBool::new(false)); // bot is not shutting down (if false)
    }

    if let Err(why) = client.start_shards(2).await {
        println!("An error oocured while running the client: {:?}", why);
    }
}
