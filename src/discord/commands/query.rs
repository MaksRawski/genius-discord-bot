use super::utils::ask_user_for_a_song;
use crate::genius::GeniusApiWrapper;
use crate::send_error;
use serenity::framework::standard::{macros::*, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

#[group]
#[commands(img, lyrics)]
pub struct Query;

async fn get_thumbnail(ctx: &Context, song_id: u32) -> Option<String> {
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
#[description(
    "Query a song's thumbnail image.

Pass some keywords as arguments to this command so that i can find a song that you want."
)]
async fn img(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let song_id = ask_user_for_a_song(ctx, msg, &args).await.ok_or("")?.id;

    if let Some(img) = get_thumbnail(ctx, song_id).await {
        msg.channel_id
            .send_files(ctx, vec![&img[..]], |m| m.content(""))
            .await?;
    } else {
        send_error!(ctx, msg, "Failed to get the thumbnail!");
    }
    Ok(())
}

#[command]
#[aliases(l)]
#[description(
    "Return song's lyrics.

Pass some keywords as arguments to this command so that i can find a song that you want."
)]
async fn lyrics(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    tracing::info!(
        "User: {:?} asked for lyrics of {:?}",
        msg.author.name,
        &args.message()
    );
    let data = ctx.data.read().await;
    let genius_api = data.get::<GeniusApiWrapper>().unwrap();

    let song = ask_user_for_a_song(ctx, msg, &args).await.ok_or("")?;

    if let Some(song_url) = genius_api.get_song_url(song.id).await {
        if let Some(lyrics) = genius_api.lyrics(&song_url).await {
            msg.channel_id
                .send_message(ctx, |m| {
                    m.embed(|e| {
                        e.title(song);
                        e.url(song_url);
                        e.description(lyrics);
                        e.color(0xffff64)
                    })
                })
                .await?;
        } else {
            send_error!(ctx, msg, "Failed to download the lyrics!");
        }
    } else {
        send_error!(ctx, msg, "Failed to fetch the lyrics!");
    }

    Ok(())
}
