// use tracing;
// use tracing::{error, info};
// use tracing_subscriber::{EnvFilter, FmtSubscriber};
use std::sync::Arc;
use std::collections::HashSet;

use serenity::{
    async_trait,
    http::Http,
    prelude::*,
    model::prelude::*,
    framework::standard::{
        StandardFramework,
        help_commands, HelpOptions,
        macros::{command, group, help, hook},
        Args, CommandResult, CommandGroup
    }
};
use super::commands::{QUERY_GROUP, GeniusApiWrapper};
use crate::genius_dl::GeniusApi;

struct Handler;

impl Handler {}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("Connected as {}", ready.user.name);
    }
}

#[help]
#[description("To get help with an individual command, pass its name as an argument to this command.")]
async fn my_help(
   context: &Context,
   msg: &Message,
   args: Args,
   help_options: &'static HelpOptions,
   groups: &[&'static CommandGroup],
   owners: HashSet<UserId>
) -> CommandResult {
    let _ = help_commands::with_embeds(context, msg, args, help_options, groups, owners).await;
    Ok(())
}

#[hook]
async fn unknown_command(ctx: &Context, msg: &Message, unknown_command_name: &str) {
    msg.channel_id.say(ctx, format!("Command '{}' not found!", unknown_command_name)).await;
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
            .group(&QUERY_GROUP)
            .help(&MY_HELP)
            .unrecognised_command(unknown_command);

        let client = Client::builder(discord_token)
            .framework(framework)
            .event_handler(Handler)
            .await
            .expect("Err creating client");

        {
            let mut data = client.data.write().await;
            data.insert::<GeniusApiWrapper>(Arc::new(GeniusApi::new(genius_token)));
        }

        Self { client }
    }
    pub async fn start(&mut self) {
        if let Err(why) = self.client.start().await {
            eprintln!("Client error: {:?}", why);
        }
    }
}
