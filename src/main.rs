use genius::discord::Discord;
use std::env;
use dotenv::dotenv;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let discord_token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN not found!");
    let genius_token = env::var("GENIUS_TOKEN").expect("GENIUS_TOKEN not found!");

    let mut discord = Discord::new(&discord_token, &genius_token).await;
    discord.start().await;
}
