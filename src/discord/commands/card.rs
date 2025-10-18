use super::utils::ask_user_for_a_song;
use crate::genius::cards::generate_card;
use crate::genius::{GeniusApiWrapper, Song};
use crate::{send_error, send_message};
use anyhow::{anyhow, Context};
use image::DynamicImage;
use regex::Regex;
use serenity::builder::{CreateComponents, CreateSelectMenuOption};
use serenity::client;
use serenity::framework::standard::{macros::*, Args, CommandResult};
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::{AttachmentType, InteractionResponseType, Message};
use std::path::PathBuf;
use std::time::Duration;

#[group]
#[commands(card, custom_card)]
pub struct Card;

/// returns a path to a downloaded image or None if an error occured
async fn search_img(ctx: &client::Context, q: &Song) -> Result<DynamicImage, anyhow::Error> {
    let data = ctx.data.read().await;
    let genius_api = data.get::<GeniusApiWrapper>().unwrap();

    let img = genius_api.get_cover(q.id).await?;

    Ok(img)
}

async fn create_card_interaction(
    ctx: &client::Context,
    msg: &Message,
    args: &Args,
    lyrics: &str,
) -> Result<PathBuf, anyhow::Error> {
    let q = ask_user_for_a_song(ctx, msg, args)
        .await
        .ok_or(anyhow!("Failed to get a song from the user"))?;
    let img = search_img(ctx, &q).await?;

    let remove_keywords = Regex::new(r"\[.*\]").unwrap();
    let lyrics = remove_keywords.replace_all(lyrics, "");
    if textwrap::wrap(&lyrics, 46).len() > 8 {
        send_error!(ctx, msg, "This lyric is too long!");
        return Err(anyhow!("Too long lyric"));
    };
    match generate_card(img, &lyrics, &q.artist, &q.title) {
        Ok(card) => Ok(card),
        Err(e) => {
            send_error!(ctx, msg, "Failed to generate the card! {}", e);
            Err(anyhow!(format!("Failed to generate the card! {e}")))
        }
    }
}

#[command]
#[aliases(c)]
#[description(
    "Create a lyric card containing a given quote.

Quote should be given as an argument to this command. You can add keywords
to your query by putting them in [square brackets]. This way you can create
a card with a common quote but from a specific artist.
"
)]
async fn card(ctx: &client::Context, msg: &Message, args: Args) -> CommandResult {
    let card = create_card_interaction(ctx, msg, &args, args.message()).await?;

    msg.channel_id
        .send_files(ctx, vec![&card], |m| m.content(""))
        .await?;

    std::fs::remove_file(card).unwrap();
    Ok(())
}

pub async fn card_slash(
    ctx: &client::Context,
    cmd: &ApplicationCommandInteraction,
) -> Result<(), anyhow::Error> {
    let arg0 = cmd
        .data
        .options
        .get(0)
        .ok_or(anyhow!("No option provided as an argument"))?
        .value
        .clone()
        .ok_or(anyhow!("No value in command data options"))?;

    let query = arg0
        .as_str()
        .ok_or(anyhow!("Command data option is not a string!"))?;

    let remove_keywords = Regex::new(r"\[.*\]").unwrap();
    let quote = remove_keywords.replace_all(query, "");

    tracing::info!("A card is created with quote: {}", quote);

    let data = ctx.data.read().await;
    let genius_api = data.get::<GeniusApiWrapper>().unwrap();

    let results: Vec<Song> = genius_api
        .search_for_song(&quote)
        .await
        .with_context(|| "Failed to find a song")?;

    // ACK an interaction
    cmd.create_interaction_response(ctx, |r| {
        r.kind(serenity::model::prelude::InteractionResponseType::DeferredChannelMessageWithSource)
    })
    .await
    .with_context(|| "Failed to ACK an interaction")?;

    // let the user choose the result
    let msg = cmd
        .create_followup_message(ctx, |msg| {
            if results.len() == 0 {
                msg.content("No results were found!")
            } else {
                msg.content("Choose a song").components(|c| {
                    c.create_action_row(|r| {
                        r.create_select_menu(|menu| {
                            menu.custom_id("results_menu");
                            menu.options(|mut ops| {
                                for res in results {
                                    ops = ops.add_option(CreateSelectMenuOption::new(
                                        format!("{} - {}", res.artist, res.title),
                                        res.id,
                                    ));
                                }
                                ops
                            })
                        })
                    })
                })
            }
        })
        .await
        .with_context(|| "Failed to ask the user to pick the song")?;

    let interaction = match msg
        .await_component_interaction(&ctx)
        .timeout(Duration::from_secs(30))
        .await
    {
        Some(interaction) => interaction,
        None => {
            msg.reply(&ctx, "Timed out!").await.unwrap();
            return Ok(());
        }
    };
    let song_id: u32 = interaction.data.values[0].parse()?;
    let song = genius_api
        .get_song_by_id(song_id)
        .await
        .with_context(|| format!("Failed to find a song with song_id: {song_id}"))?;
    let img_data = genius_api
        .get_cover(song_id)
        .await
        .with_context(|| format!("Failed to get cover for a song_id: {song_id}"))?;

    let card_path = generate_card(img_data, &quote, &song.artist, &song.title)
        .with_context(|| "Failed to generate the card!")?;

    interaction
        .create_interaction_response(ctx, |r| {
            r.kind(InteractionResponseType::UpdateMessage)
                .interaction_response_data(|d| {
                    // clear the original response - message with the song choice
                    d.content("")
                        .set_components(CreateComponents::default())
                        .add_file(AttachmentType::Path(&card_path))
                })
        })
        .await
        .with_context(|| "Failed to send the image!")?;

    Ok(())
}

#[command]
#[aliases(cc)]
#[description(
    "Create a lyric card with a custom quote.

Pass some keywords as arguments to this command so i can find a song that you want.
Then you will be able to choose a caption that should be displayed on its card."
)]
async fn custom_card(ctx: &client::Context, msg: &Message, args: Args) -> CommandResult {
    tracing::info!(
        "User \"{}#{}\" is creating a custom card.",
        msg.author.name,
        msg.author.id
    );
    let q = ask_user_for_a_song(ctx, msg, &args).await.ok_or("")?;

    if let Ok(img) = search_img(ctx, &q).await {
        send_message!(ctx, msg, "What should the caption be?");
        let caption = if let Some(answer) = &msg
            .author
            .await_reply(ctx)
            .timeout(Duration::from_secs(60))
            .await
        {
            answer.content.clone()
        } else {
            send_message!(ctx, msg, "Time's up!");
            return Ok(());
        };
        let card = generate_card(img, &caption, &q.artist, &q.title)?;

        msg.channel_id
            .send_files(ctx, vec![&card], |m| m.content(""))
            .await?;

        std::fs::remove_file(card).unwrap();
    } else {
        send_error!(ctx, msg, "Failed to find an image for this song!");
    }
    Ok(())
}
