use super::utils::query_song;
use crate::genius::cards::generate_card;
use crate::genius::{GeniusApiWrapper, SongQuery};
use crate::send_message;
use serenity::framework::standard::{macros::*, Args, CommandResult};
use serenity::http::Typing;
use serenity::model::prelude::Message;
use serenity::prelude::*;
use std::time::Duration;

#[group]
#[commands(card, custom_card)]
pub struct Card;

async fn search_img(ctx: &Context, msg: &Message, args: &Args) -> Option<(String, SongQuery)> {
    let data = ctx.data.read().await;
    let genius_api = data.get::<GeniusApiWrapper>().unwrap();

    let q = query_song(ctx, msg, args).await?;

    let img_url = genius_api.img(q.id).await?;
    let img = genius_api.download_img(&img_url).await?;

    Some((img, q))
}

async fn quote(ctx: &Context, msg: &Message, args: &Args, lyrics: &str) -> Option<String> {
    let (img, q) = search_img(ctx, msg, args).await?;
    let card = generate_card(&img, &lyrics, &q.artist, &q.title)?;
    std::fs::remove_file(img).unwrap();

    Some(card)
}

#[command]
#[aliases(c)]
#[description("Create a lyric card containing a given quote")]
async fn card(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    tracing::info!(
        "User \"{}#{}\" is creating a card.",
        msg.author.name,
        msg.author.id
    );
    let typing = Typing::start(ctx.http.clone(), msg.channel_id.0).unwrap();
    let card = quote(ctx, msg, &args, args.message())
        .await
        .ok_or("Failed to get card info")?;

    msg.channel_id
        .send_files(ctx, vec![&card[..]], |m| m.content(""))
        .await?;
    typing.stop();

    std::fs::remove_file(card).unwrap();
    Ok(())
}

#[command]
#[aliases(cc)]
#[description("Create a lyric card with a custom quote")]
async fn custom_card(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    tracing::info!(
        "User \"{}#{}\" is creating a custom card.",
        msg.author.name,
        msg.author.id
    );
    let (img, q) = search_img(ctx, msg, &args).await.ok_or("")?;
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
    let card = generate_card(&img, &caption, &q.artist, &q.title).ok_or("")?;
    std::fs::remove_file(img).unwrap();
    msg.channel_id
        .send_files(ctx, vec![&card[..]], |m| m.content(""))
        .await?;

    Ok(())
}
