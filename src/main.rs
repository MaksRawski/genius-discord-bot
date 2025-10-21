use dotenv::dotenv;
use genius::discord::Discord;
use std::env;
use tracing_subscriber::{filter, fmt, prelude::*, Registry};

// ridiculously good and simple example of how to setup tracing_subscriber
// https://stackoverflow.com/a/70042590
#[tokio::main]
async fn main() {
    dotenv().ok();
    let stdout_log = fmt::layer()
        .pretty()
        // we log everything from this crate and all errors including those from different crates
        .with_filter(filter::filter_fn(|metadata| {
            metadata.target().starts_with("genius") || metadata.level() == &tracing::Level::ERROR
        }));

    Registry::default().with(stdout_log).init();

    tracing::info!("logger initalized!");

    let discord_token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN not found!");
    let genius_token = env::var("GENIUS_TOKEN").expect("GENIUS_TOKEN not found!");

    let mut discord = Discord::new(&discord_token, &genius_token).await;
    discord.start().await;
}
