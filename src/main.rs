use dotenv::dotenv;
use genius::discord::Discord;
use std::env;
use tracing::info;
use tracing_subscriber::prelude::*;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let _guard = sentry::init((
        env::var("SENTRY_URL").expect("SENTRY_URL not found!"),
        sentry::ClientOptions {
            release: sentry::release_name!(),
            traces_sample_rate: 1.0,
            ..Default::default()
        },
    ));
    tracing_subscriber::registry()
        .with(sentry_tracing::layer())
        .init();

    info!("logger initalized!");

    let discord_token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN not found!");
    let genius_token = env::var("GENIUS_TOKEN").expect("GENIUS_TOKEN not found!");

    let mut discord = Discord::new(&discord_token, &genius_token).await;
    discord.start().await;
}
