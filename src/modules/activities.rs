use serenity::{client::Context, model::gateway::Activity};
use rand::{Rng, thread_rng};
use chrono::offset::Utc;
use std::sync::Arc;
use super::utils::{get_sys, get_ping};
use serenity::model::prelude::OnlineStatus;
// ShardManager from main.rs
use crate::ShardManagerContainer;


pub async fn set_status(ctx: Arc<Context>) {

    //* Define activies and choose one of them randomly
    let mut activities = Vec::new();


    activities.push(Activity::listening("s help".to_string()));
    activities.push(Activity::watching("s help".to_string()));
    activities.push(Activity::playing(Utc::now().to_rfc2822()));
    activities.push(Activity::watching(get_sys(false).await.get("memory_usage").unwrap()));
    activities.push(Activity::listening(get_ping(&ctx).await));

    let statuses = [OnlineStatus::Online, OnlineStatus::Idle, OnlineStatus::DoNotDisturb];

    let rng = thread_rng().gen_range(0..activities.len());
    let status_rng = thread_rng().gen_range(0..statuses.len());
    let status = statuses[status_rng];

    //* Get the bot shards and change the status for each of them
    let data_read = ctx.data.read().await;
    let shard_manager = data_read.get::<ShardManagerContainer>().unwrap();
    let manager = shard_manager.lock().await;
    // complicated I know...
    let runners = manager.runners.lock().await;
    for (_id, runner) in runners.iter() {
        runner.runner_tx.set_presence(Some(activities[rng].clone()), status);
    }
}