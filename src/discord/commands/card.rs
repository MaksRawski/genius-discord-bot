use super::utils::ask_user_for_a_song;
use crate::genius::cards::generate_card;
use crate::genius::{GeniusApiWrapper, Song};
use crate::{send_error, send_message};
use regex::Regex;
use serenity::framework::standard::{macros::*, Args, CommandResult};
use serenity::model::prelude::Message;
use serenity::prelude::*;
use std::time::Duration;

#[group]
#[commands(card, custom_card)]
pub struct Card;

/// returns a path to a downloaded image or None if an error occured
async fn search_img(ctx: &Context, q: &Song) -> Option<String> {
    let data = ctx.data.read().await;
    let genius_api = data.get::<GeniusApiWrapper>().unwrap();

    let img_url = genius_api.img(q.id).await?;
    let img = genius_api.download_img(&img_url).await?;

    Some(img)
}

async fn get_quote_from_user(
    ctx: &Context,
    msg: &Message,
    args: &Args,
    lyrics: &str,
) -> Option<String> {
    let q = ask_user_for_a_song(ctx, msg, args).await?;
    let img = search_img(ctx, &q).await?;

    let remove_keywords = Regex::new(r"\[.*\]").unwrap();
    let lyrics = remove_keywords.replace_all(lyrics, "");
    if textwrap::wrap(&lyrics, 46).len() > 8 {
        send_error!(ctx, msg, "This lyric is too long!");
        return None;
    };
    match generate_card(&img, &lyrics, &q.artist, &q.title) {
        Ok(card) => {
            std::fs::remove_file(img).unwrap();
            return Some(card);
        }
        Err(e) => {
            send_error!(ctx, msg, "Failed to generate the card! {e}");
            return None;
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
async fn card(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    tracing::info!(
        "User \"{}#{}\" is creating a card.",
        msg.author.name,
        msg.author.id
    );
    let card = get_quote_from_user(ctx, msg, &args, args.message())
        .await
        .ok_or("")?;
    msg.channel_id
        .send_files(ctx, vec![&card[..]], |m| m.content(""))
        .await?;

    std::fs::remove_file(card).unwrap();
    Ok(())
}

#[command]
#[aliases(cc)]
#[description(
    "Create a lyric card with a custom quote.

Pass some keywords as arguments to this command so i can find a song that you want.
Then you will be able to choose a caption that should be displayed on its card."
)]
async fn custom_card(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    tracing::info!(
        "User \"{}#{}\" is creating a custom card.",
        msg.author.name,
        msg.author.id
    );
    let q = ask_user_for_a_song(ctx, msg, &args).await.ok_or("")?;

    if let Some(img) = search_img(ctx, &q).await {
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
        let card = generate_card(&img, &caption, &q.artist, &q.title)?;
        std::fs::remove_file(img).unwrap();

        msg.channel_id
            .send_files(ctx, vec![&card[..]], |m| m.content(""))
            .await?;

        std::fs::remove_file(card).unwrap();
    } else {
        send_error!(ctx, msg, "Failed to find an image for this song!");
    }
    Ok(())
}
