mod modules;

// use crate::modules::checks::*;
// use crate::modules::music::*;
// use crate::modules::{activities::*, core::*, owner::*, utils::*};

use modules::utils::*;

use std::{
    env, fs,
    io::Write,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use ::serenity::all::FullEvent;
use poise::serenity_prelude as serenity;
use tokio::sync::Mutex;

use songbird;

struct Data {
    songbird: Arc<songbird::Songbird>,
    shutting_down: Mutex<bool>,
}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

//TODO! add commands to a group, this means you Okkonen!!!!
//TODO: Add more groups (suggestions, misc, owner, (moderation), etc)
// #[group]
// #[commands(ping, about, info, quit, uptime, fullinfo, betterping, testadmin)]
// struct General;
//
// #[group]
// #[commands(
//     suggest,
//     set_suggestion_channel,
//     edit_suggestion,
//     accept_suggestion,
//     remove_suggestion
// )]
// struct Suggestions;
//
// #[group]
// #[commands(set_welcome_channel, set_welcome_message)]
// struct Welcome;
//
// #[group]
// #[commands(
//     join, leave, mute, deafen, play, stop, skip, pause, resume, unloop, loop, playing, queue,
//     volume
// )]
// struct Music;

struct Handler {
    activity_loop: AtomicBool,
}

// #[async_trait]
// impl EventHandler for Handler {
//     async fn ready(&self, ctx: Context<'_>, ready: Ready) {
//         let guilds = match ready.user.guilds(ctx.clone()).await {
//             Ok(v) => v.len().to_string(),
//             _ => String::from("?"),
//         };
//         println!(
//             "{} is connected & total guilds: {} ",
//             ready.user.name, guilds
//         );
//
//         let ctx = Arc::new(ctx);
//
//         if !self.activity_loop.load(Ordering::Relaxed) {
//             let context = Arc::clone(&ctx);
//             tokio::spawn(async move {
//                 loop {
//                     // println!("boe");
//                     set_random_status(Arc::clone(&context)).await;
//                     tokio::time::sleep(Duration::from_secs(60)).await;
//                 }
//             });
//         }
//         self.activity_loop.swap(true, Ordering::Relaxed);
//     }
//     // we have to have this in the same impl block (only 1 event handler can exist)
//     // so I just call into a different funtion in welcome.rs to handle all the logic
//     async fn guild_member_addition(&self, ctx: Context<'_>, new_member: serenity::Member) -> () {
//         say_hello(&ctx, &new_member).await;
//     }
// }

// struct ShardManagerContainer;
//
// impl TypeMapKey for ShardManagerContainer {
//     type Value = Arc<Mutex<ShardManager>>;
// }
// struct ShuttingDown;
//
// impl TypeMapKey for ShuttingDown {
//     type Value = AtomicBool;
// }
//

fn event_listeners(
    // ctx: &Context<'_>,
    event: &FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    _user_data: &Data,
) -> Result<(), Error> {
    match event {
        FullEvent::Ready { data_about_bot } => {
            println!(
                "{} is connected to {} servers",
                data_about_bot.user.tag(),
                data_about_bot.guilds.len()
            );
        }
        _ => (),
    }
    Ok(())
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
    } else {
        // development mode
        token = env::var("DEV_TOKEN").expect("token");
        prefix = "d"; // d for now...
    }

    if cfg!(debug_assertions) {
        let path = get_pwd().join("data/");

        // println!("{:?}", &path);

        if !path.exists() {
            fs::create_dir(&path).expect("Failed to create data directory");
        }

        fs::File::open(path.join("data.json")).unwrap_or_else(|_| {
            let mut b = fs::File::create(path.join("data.json")).unwrap();
            b.write(b"{}").unwrap();
            b
        });
        let path = get_pwd().join("data/");
        fs::File::open(path.join("guilds.json")).unwrap_or_else(|_| {
            println!("wtf");
            let mut b = fs::File::create(path.join("guilds.json")).unwrap();
            b.write(b"{}").unwrap();
            b
        });
    }
    // let (owners, bot_id) = get_owners(&token).await;

    //*  Music stuff init
    // tracing_subscriber::fmt::init();

    let prefix_options = poise::PrefixFrameworkOptions {
        prefix: Some(prefix.into()),
        mention_as_prefix: true,
        edit_tracker: Some(Arc::new(poise::EditTracker::for_timespan(
            Duration::from_secs(300),
        ))), // 5 mins
        case_insensitive_commands: true,
        ..Default::default()
    };
    let opts = poise::FrameworkOptions {
        commands: vec![
            modules::core::ping(),
            modules::core::about(),
            modules::core::help(),
            modules::core::betterping(),
            modules::core::info(),
            modules::core::fullinfo(),
            modules::music::play(),
            modules::music::pause(),
            modules::music::resume(),
            modules::music::mute(),
            modules::music::leave(),
            modules::music::join(),
            modules::music::r#loop(),
            modules::music::unloop(),
            modules::music::queue(),
            modules::music::deafen(),
        ],
        prefix_options,
        event_handler: |_ctx, event, framework, data| {
            Box::pin(async move { event_listeners(event, framework, data) })
        },
        ..Default::default()
    };
    let songbird = songbird::Songbird::serenity();

    let data = Data {
        songbird: songbird.clone(),
        shutting_down: Mutex::new(false),
    };

    let framework = poise::Framework::builder()
        .options(opts)
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(data)
            })
        })
        .build();

    let intents = serenity::GatewayIntents::GUILDS
        | serenity::GatewayIntents::GUILD_MESSAGES
        | serenity::GatewayIntents::GUILD_MESSAGE_REACTIONS
        | serenity::GatewayIntents::DIRECT_MESSAGES
        | serenity::GatewayIntents::DIRECT_MESSAGE_REACTIONS
        | serenity::GatewayIntents::MESSAGE_CONTENT
        | serenity::GatewayIntents::GUILD_MEMBERS
        | serenity::GatewayIntents::GUILD_PRESENCES // idk about this one
        | serenity::GatewayIntents::GUILD_VOICE_STATES;

    let mut client = serenity::Client::builder(token, intents)
        // .event_handler(Handler {
        //     activity_loop: AtomicBool::new(false),
        // })
        .framework(framework)
        .voice_manager_arc(songbird)
        .await
        .expect("Error creating the client");

    // {
    //     let mut data = client.data.write().await;
    //     data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
    //     data.insert::<ShuttingDown>(AtomicBool::new(false)); // bot is not shutting down (if false)
    // }

    if let Err(why) = client.start_shards(2).await {
        println!("An error oocured while running the client: {:?}", why);
    }
}
