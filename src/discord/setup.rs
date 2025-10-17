use anyhow::Context as anyhowContext;
use serenity::model::prelude::command::Command;
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
    model::prelude::*,
    prelude::*,
};

use super::commands::card::card_slash;

const PREFIX: &str = "~";

struct Handler;

impl Handler {}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        tracing::info!("Connected as {}", ready.user.name);
        let _ = Command::create_global_application_command(ctx, |cmd| {
            cmd.name("card")
                .create_option(|op| {
                    op.name("quote")
                        .kind(command::CommandOptionType::String)
                        .description("Quote you want on the card")
                        .required(true)
                })
                .description("Create a card")
        })
        .await;
    }
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(cmd) = interaction {
            match cmd.data.name.as_str() {
                "card" => {
                    if let Err(err) = card_slash(&ctx, &cmd).await {
                        tracing::error!("{:?}", err); // card_slash error
                        if let Err(create_err_msg_err) = cmd
                            .create_followup_message(ctx, |msg| msg.content(err).ephemeral(true))
                            .await
                            .with_context(|| "Failed to send an error message to the user")
                        {
                            tracing::error!("{:?}", create_err_msg_err); // error when sending the error message
                        }
                    }
                }
                cmd_name => todo!("{}", cmd_name),
            }
        }
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
    help_options.max_levenshtein_distance = 1;
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
        .await
        .unwrap();
    msg.channel_id
        .say(
            ctx,
            format!(
                "Send `{}help` to see the list of possible commands.",
                PREFIX
            ),
        )
        .await
        .unwrap();
}
pub struct Discord {
    client: Client,
}

impl Discord {
    pub async fn new(discord_token: &str, genius_token: &str) -> Self {
        let framework = StandardFramework::new()
            .configure(|c| c.prefix(PREFIX))
            // .before(f)
            .group(&QUERY_GROUP)
            .group(&CARD_GROUP)
            .help(&MY_HELP);

        let client = Client::builder(
            discord_token,
            GatewayIntents::GUILD_MESSAGES
                | GatewayIntents::DIRECT_MESSAGES
                | GatewayIntents::MESSAGE_CONTENT,
        )
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
