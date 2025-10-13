use super::utils::ask_user_for_a_song;
use crate::genius::GeniusApiWrapper;
use crate::send_error;
use rand::distributions::{Alphanumeric, DistString};
use serenity::framework::standard::{macros::*, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

#[group]
#[commands(img, lyrics)]
pub struct Query;

#[command]
#[aliases(i)]
#[description(
    "Query a song's thumbnail image.

Pass some keywords as arguments to this command so that i can find a song that you want."
)]
async fn img(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let genius_api = data.get::<GeniusApiWrapper>().unwrap();
    let song_id = ask_user_for_a_song(ctx, msg, &args).await.ok_or("")?.id;

    let filename: String = Alphanumeric.sample_string(&mut rand::thread_rng(), 30);
    let filename = format!("{filename}.jpg");

    genius_api
        .get_cover(song_id)
        .await?
        .save_with_format(&filename, image::ImageFormat::Jpeg)?;

    msg.channel_id
        .send_files(ctx, vec![&filename[..]], |m| m.content(""))
        .await?;
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

    let song = ask_user_for_a_song(ctx, msg, &args)
        .await
        .ok_or("Failed to get a song from the user")?;

    if let Ok(lyrics) = genius_api.lyrics(&song.url).await {
        msg.channel_id
            .send_message(ctx, |m| {
                m.embed(|e| {
                    e.title(&song);
                    e.url(song.url);
                    e.description(lyrics);
                    e.color(0xffff64)
                })
            })
            .await?;
    } else {
        send_error!(ctx, msg, "Failed to download the lyrics!");
    }

    Ok(())
}
