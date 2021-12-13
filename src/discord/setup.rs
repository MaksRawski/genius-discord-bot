// use tracing;
// use tracing::{error, info};
// use tracing_subscriber::{EnvFilter, FmtSubscriber};
use std::sync::Arc;

use serenity::{
    async_trait,
    http::Http,
    prelude::*, model::prelude::Ready, framework::StandardFramework,
};
use super::commands::{GENERAL_GROUP, ImageDownloaderContainer, ImageDownloader};

struct Handler;

impl Handler {}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("Connected as {}", ready.user.name);
    }
}

pub struct Discord {
    client: Client,
}

impl Discord {
    pub async fn new(discord_token: &str, genius_token: &str) -> Self {
        // let subscriber =
        //       FmtSubscriber::builder().with_env_filter(EnvFilter::from_default_env()).finish();

        //   tracing::subscriber::set_global_default(subscriber).expect("Failed to start the logger");
        // TODO info! doesn't work
        // info!("DUPA");

        let http = Http::new_with_token(discord_token);

        // fetch bot's id.
        let bot_id = match http.get_current_application_info().await {
            Ok(info) => info.id,
            Err(why) => panic!("Could not access application info: {:?}", why),
        };

        let framework = StandardFramework::new()
            .configure(|c| c.on_mention(Some(bot_id)))
            .group(&GENERAL_GROUP);

        let client = Client::builder(discord_token)
            .framework(framework)
            .event_handler(Handler)
            .await
            .expect("Err creating client");

        {
            let mut data = client.data.write().await;
            data.insert::<ImageDownloaderContainer>(Arc::new(ImageDownloader::new(genius_token)));
        }

        Self { client }
    }
    pub async fn start(&mut self) {
        if let Err(why) = self.client.start().await {
            eprintln!("Client error: {:?}", why);
        }
    }
}
