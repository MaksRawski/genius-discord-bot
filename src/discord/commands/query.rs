use crate::genius::{GeniusApiWrapper, SongQuery};
use serenity::framework::standard::{macros::*, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use std::time::Duration;

#[group]
#[commands(img, lyrics)]
pub struct Query;

// either returns song_id or String with an error message
pub async fn query(ctx: &Context, msg: &Message, args: &Args) -> Option<SongQuery> {
    let arg = args.message();
    if arg.len() < 2 {
        tracing::error!("Query too short");
        return None;
    }
    let data = ctx.data.read().await;
    let genius_api = data.get::<GeniusApiWrapper>().unwrap();

    msg.channel_id
        .say(ctx, format!("Searching genius for: **{}**\n", arg));

    let results: Vec<SongQuery> = genius_api.search_song(arg).await?;

    match results.len() {
        0 => {
            msg.channel_id.say(ctx, "**No results found!**".to_string());
            None
        }
        1 => {
            let res = results.get(0).unwrap().clone();
            msg.channel_id
                .say(ctx, format!("Singe result found: **{}**", res));
            Some(res)
        }
        _ => {
            let mut options: String = String::new();

            for r in results.iter().enumerate() {
                options.push_str(&format!("{} - {}", r.0, r.1));
            }

            let options_msg = msg.channel_id.say(
                ctx,
                format!(
                    "Multiple results were found, please choose one:\n{}",
                    options
                ),
            );

            if let Some(answer) = &msg
                .author
                .await_reply(ctx)
                .timeout(Duration::from_secs(60))
                .await
            {
                options_msg.await.unwrap().delete(ctx);
                let index = if let Ok(v) = answer.content.parse::<usize>() {
                    v - 1
                } else {
                    msg.channel_id
                        .say(ctx, format!("That's not a valid number!"));
                    return None;
                };

                if let Some(v) = results.get(index) {
                    let v = v.clone();
                    msg.channel_id.say(ctx, format!("You've chosen: **{}**", v));
                    Some(v)
                } else {
                    msg.channel_id
                        .say(ctx, "There is no result with that number.");
                    None
                }
            } else {
                msg.channel_id.say(ctx, format!("Time's up!"));
                None
            }
        }
    }
}

async fn get_thumbnail(ctx: &Context, msg: &Message, args: &Args) -> Option<String> {
    let song_id = query(ctx, msg, args).await?.id;
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
    tracing::info_span!(
        "User: {:?} asked for lyrics of {:?}",
        user = msg.author.name,
        lyrics = &args.message()
    );
    let data = ctx.data.read().await;
    let genius_api = data.get::<GeniusApiWrapper>().unwrap();

    if let Some(s) = query(ctx, msg, &args).await {
        if let Some(l) = genius_api.lyrics(s.id).await {
            msg.channel_id.send_message(ctx, |m| {
                m.embed(|e| {
                    e.description(l);
                    e.color(0xffff64)
                })
            });
        } else {
            msg.channel_id
                .say(ctx, format!("**Error occured while getting lyrics!**"))
                .await;
        };
    };
    Ok(())
}
