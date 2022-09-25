use super::utils::query_song;
use crate::genius::GeniusApiWrapper;
use crate::send_error;
use serenity::framework::standard::{macros::*, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

#[group]
#[commands(img, lyrics)]
pub struct Query;

async fn get_thumbnail(ctx: &Context, msg: &Message, args: &Args) -> Option<String> {
    let song_id = query_song(ctx, msg, args).await?.id;
    let data = ctx.data.read().await;
    let genius_api = data.get::<GeniusApiWrapper>().unwrap();

    let img_url = genius_api
        .img(song_id)
        .await
        .ok_or_else(|| tracing::error!("Error occured while downloading the cover image"))
        .ok()?;

    Some(img_url)
}

#[command]
#[aliases(i)]
#[description("Query a song's thumbnail image")]
async fn img(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let img = get_thumbnail(ctx, msg, &args)
        .await
        .ok_or("Failed to get Thumbnail")?;

    msg.channel_id
        .send_files(ctx, vec![&img[..]], |m| m.content(""));

    Ok(())
}

#[command]
#[aliases(l)]
async fn lyrics(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    tracing::info!(
        "User: {:?} asked for lyrics of {:?}",
        msg.author.name,
        &args.message()
    );
    let data = ctx.data.read().await;
    let genius_api = data.get::<GeniusApiWrapper>().unwrap();

    if let Some(s) = query_song(ctx, msg, &args).await {
        if let Some(l) = genius_api.lyrics(s.id).await {
            msg.channel_id.send_message(ctx, |m| {
                m.embed(|e| {
                    e.description(l);
                    e.color(0xffff64)
                })
            });
        } else {
            send_error!(ctx, msg, "Error occured while getting lyrics!");
        };
    };
    Ok(())
}
