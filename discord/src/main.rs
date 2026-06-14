mod tracker;

use crate::tracker::{Tracker, TrackerConfig};
use log::{error, info};
use poise::serenity_prelude as serenity;
use std::{collections::HashMap, sync::Arc};

static TRACKER_CONFIGS: &[TrackerConfig] = &[TrackerConfig {
    name: "ye",
    url: "https://yetracker.net/htmlview/sheet?headers=false&gid=34972268",
}];

#[derive(Debug)]
pub struct Data {
    trackers: Arc<HashMap<&'static str, Tracker>>,
}

impl Data {
    pub fn new(trackers: impl IntoIterator<Item = Tracker>) -> Self {
        Self {
            trackers: Arc::new(trackers.into_iter().map(|t| (t.name, t)).collect()),
        }
    }
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

#[poise::command(slash_command, prefix_command)]
async fn search(
    ctx: Context<'_>,
    #[description = "Search Query"]
    #[rest]
    query: String,
) -> Result<(), Error> {
    let tracker = ctx.data().trackers.get("ye").unwrap();

    let results = tracker.search(&query);

    if let Err(e) = tracker.send_embed(ctx, &query, &results).await {
        eprintln!("Pagination error: {e}");
    }

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().unwrap();
    pretty_env_logger::init();

    let token = std::env::var("TOKEN").expect("No 'TOKEN' env var found.");
    let prefix = std::env::var("PREFIX").expect("No 'PREFIX' env var found.");

    info!("Tracker bot starting...");

    let mut set = tokio::task::JoinSet::new();
    for config in TRACKER_CONFIGS.iter() {
        set.spawn(Tracker::build(config));
    }

    let mut trackers = Vec::with_capacity(TRACKER_CONFIGS.len());
    while let Some(res) = set.join_next().await {
        trackers.push(res??);
    }

    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![search()],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some(prefix),
                ..Default::default()
            },
            on_error: |error| {
                Box::pin(async move {
                    match error {
                        poise::FrameworkError::UnknownCommand { .. } => { /* ignored */ }
                        other => {
                            error!("{other:#?}");
                            poise::builtins::on_error(other).await.unwrap();
                        }
                    }
                })
            },
            ..Default::default()
        })
        .setup(|ctx, ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                info!("Bot User \"{}\" is now ready.", ready.user.display_name());
                Ok(Data::new(trackers))
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;

    client.unwrap().start().await.unwrap();

    Ok(())
}
