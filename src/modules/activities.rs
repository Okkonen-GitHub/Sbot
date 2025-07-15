use serenity::{client::Context, model::gateway::Activity};
use rand::{Rng, thread_rng};
use chrono::offset::Utc;
use std::sync::Arc;
use super::utils::get_sys;

// ShardManager from main.rs
use crate::ShardManagerContainer;


pub async fn set_status(ctx: Arc<Context>) {

    //* Define activies and choose one of them randomly
    let mut activities = Vec::new();

    activities.push(Activity::listening("s help".to_string()));
    activities.push(Activity::watching("s help".to_string()));
    activities.push(Activity::playing(Utc::now().to_rfc2822()));
    activities.push(Activity::watching(get_sys(false).await.get("memory_usage").unwrap()));

    let rng = thread_rng().gen_range(0..activities.len());


    //* Get the bot shards and change the status for each of them
    let data_read = ctx.data.read().await;
    let shard_manager = data_read.get::<ShardManagerContainer>().unwrap();
    let manager = shard_manager.lock().await;
    // complicated I know...
    let runners = manager.runners.lock().await;
    for (_id, runner) in runners.iter() {
        runner.runner_tx.set_activity(Some(activities[rng].clone()));
    }
}