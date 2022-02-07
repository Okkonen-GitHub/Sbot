use serenity::{
    async_trait,
    client::{
        Client,
        Context,
        EventHandler,
        bridge::gateway::{ShardId, ShardManager}
    },
    model::channel::Message,
    framework::standard::{
        StandardFramework,
        CommandResult,
        macros::{
            command,
            group
        }
    }, futures::lock::Mutex,
    prelude::TypeMapKey
};

use std::{env, process::exit, sync::Arc};

// add commands to a group!
#[group]
#[commands(ping, quit)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {}

struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

#[tokio::main]
async fn main() {

    dotenv::dotenv().expect("Failed to load .env");

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("s"))
        .group(&GENERAL_GROUP);

    let token = env::var("DISCORD_TOKEN").expect("token");

    let mut client = Client::builder(token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating the client");

    if let Err(why) = client.start().await {
        println!("An error oocured while running the client: {:?}", why);
    }
}

#[command]
#[aliases("latency")]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    let latency = "?";
    msg.reply(
        ctx,
        format!("Bot latency: {}", latency)
    ).await?;

    Ok(())
}


#[command]
async fn quit(ctx: &Context, msg: &Message) -> CommandResult {
    if msg.author.id == 357166445630849027 {
        msg.reply(ctx, format!("gonna oof, {}", msg.author.id)).await?;
        exit(0);
    }
    else {
        msg.reply(
            ctx,
            format!("{}", &msg.author.id)
        ).await?;
    }
    Ok(())
}
