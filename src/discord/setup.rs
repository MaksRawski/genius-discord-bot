use std::collections::HashSet;
use std::sync::Arc;

use crate::discord::commands::{CARD_GROUP, QUERY_GROUP};
use crate::genius::{GeniusApi, GeniusApiWrapper};

use serenity::{
    async_trait,
    framework::standard::{
        help_commands,
        macros::{help, hook},
        Args, CommandGroup, CommandResult, HelpOptions, StandardFramework,
    },
    http::Http,
    model::prelude::*,
    prelude::*,
};

const PREFIX: &str = "~";

struct Handler;

impl Handler {}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        tracing::info!("Connected as {}", ready.user.name);
    }
}

#[help]
async fn my_help(
    context: &Context,
    msg: &Message,
    args: Args,
    help_options_template: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    let mut help_options = help_options_template.to_owned();

    help_options.strikethrough_commands_tip_in_guild = None;
    help_options.strikethrough_commands_tip_in_dm = None;
    help_options.max_levenshtein_distance = 2;
    help_options.description_label =
        "To get help with an individual command, pass its name as an argument to this command.";

    let _ = help_commands::with_embeds(context, msg, args, &help_options, groups, owners).await;
    Ok(())
}

#[hook]
async fn unknown_command(ctx: &Context, msg: &Message, unknown_command_name: &str) {
    msg.channel_id
        .say(
            ctx,
            format!("Command '{}' not found!", unknown_command_name),
        )
        .await;
    msg.channel_id
        .say(
            ctx,
            format!(
                "Send `{}help` to see the list of possible commands.",
                PREFIX
            ),
        )
        .await;
}
pub struct Discord {
    client: Client,
}

impl Discord {
    pub async fn new(discord_token: &str, genius_token: &str) -> Self {
        let http = Http::new_with_token(discord_token);

        // fetch bot's id.
        let bot_id = match http.get_current_application_info().await {
            Ok(info) => info.id,
            Err(why) => panic!("Could not access application info: {:?}", why),
        };

        let framework = StandardFramework::new()
            .configure(|c| c.on_mention(Some(bot_id)).prefix(PREFIX))
            // .before(f)
            .group(&QUERY_GROUP)
            .group(&CARD_GROUP)
            .help(&MY_HELP)
            .unrecognised_command(unknown_command);

        let client = Client::builder(discord_token)
            .framework(framework)
            .event_handler(Handler)
            .await
            .expect("Error creating client");

        {
            let mut data = client.data.write().await;
            data.insert::<GeniusApiWrapper>(Arc::new(GeniusApi::new(genius_token)));
        }

        Self { client }
    }
    pub async fn start(&mut self) {
        if let Err(why) = self.client.start().await {
            tracing::error!("Client error: {:?}", why);
        }
    }
}
