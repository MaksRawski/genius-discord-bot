use genius::discord::Discord;
use genius::genius_dl::GeniusApi;
use std::env;

#[tokio::main]
async fn main() {
    // let args = App::new("genius")
    //     .version("0.1.0")
    //     .author("Maks Rawski <maksymilian.rawski@tutanota.com>")
    //     .about("Discord bot to create genius lyric cards")
    //     .get_matches();
    let discord_token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN not found!");
    let genius_token = env::var("GENIUS_TOKEN").expect("GENIUS_TOKEN not found!");

    let mut discord = Discord::new(&discord_token, &genius_token).await;
    discord.start().await;
    // let id = ImageDownloader::new(genius_token);
    // id.query("malik monatana do rana").await.unwrap();
}
