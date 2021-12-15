use reqwest;
use serenity::{
    framework::standard::{
        macros::{command, group},
        Args, CommandResult,
    },
    http::request,
    model::channel::Message,
    prelude::*,
};
use std::sync::Arc;

use crate::genius_dl::{GeniusApi, SongQuery};
use crate::genius_img::generate_card;
use std::time::Duration;

pub struct GeniusApiWrapper;

impl TypeMapKey for GeniusApiWrapper {
    type Value = Arc<GeniusApi>;
}

#[group]
#[commands(img, lyrics, quote)]
#[summary("Pretty much all the current commands.")]
pub struct Query;

// either returns song_id or String with an error message
async fn query(ctx: &Context, msg: &Message, args: &Args) -> Result<SongQuery, String> {
    let arg = args.message();
    if arg.len() < 2 {
        return Err("Query too short".to_string());
    }
    let data = ctx.data.read().await;
    let genius_api = data.get::<GeniusApiWrapper>().unwrap();

    msg.channel_id
        .say(ctx, format!("Searching genius for: **{}**\n", arg))
        .await;

    let results: Vec<SongQuery> = genius_api.search_song(arg).await.unwrap();

    let result = match results.len() {
        0 => {
            return Err("**No results found!**".to_string());
        }
        1 => results.get(0).unwrap(),
        _ => {
            let mut _i = 0;
            let options: String = results
                .iter()
                .map(|r| {
                    _i += 1;
                    format!("{}. **{}**\n", _i, r)
                })
                .collect();

            msg.channel_id
                .say(
                    ctx,
                    format!(
                        "Multiple results were found, please choose one:\n{}",
                        options
                    ),
                )
                .await;

            if let Some(answer) = &msg
                .author
                .await_reply(ctx)
                .timeout(Duration::from_secs(60))
                .await
            {
                // let re = Regex::new(r"\d+").unwrap();
                // re.captures
                // TODO use regex here to figure out chosen index
                let index = if let Ok(v) = answer.content.parse::<usize>() {
                    v - 1
                } else {
                    return Err(format!("That's not a valid number!"));
                };
                let chosen_result = results
                    .get(index)
                    .ok_or(format!("Provided number is too big."))?;

                msg.channel_id
                    .say(ctx, format!("You've chosen: **{}**", chosen_result))
                    .await;

                chosen_result
            } else {
                return Err(format!("Time's up!"));
            }
        }
    };
    Ok(result.clone())
}

async fn get_thumbnail(ctx: &Context, msg: &Message, args: &Args) -> Result<String, String> {
    let data = ctx.data.read().await;
    let genius_api = data.get::<GeniusApiWrapper>().unwrap();

    let song_id = query(ctx, msg, args).await?.id;
    let img_url = genius_api
        .img(song_id)
        .await
        .map_err(|_| "A problem occured while downloading the cover image".to_string())?;

    Ok(img_url)
}

#[command]
#[aliases(i, image, cover, art, thumbnail)]
#[description("Query a song's thumbnail")]
async fn img(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let img = match get_thumbnail(ctx, msg, &args).await {
        Ok(img) => img,
        Err(e) => {
            msg.channel_id.say(ctx, e).await;
            return Ok(());
        }
    };
    msg.channel_id
        .send_files(ctx, vec![&img[..]], |m| m.content(""))
        .await;

    Ok(())
}

#[command]
async fn find(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let genius_api = data.get::<GeniusApiWrapper>().unwrap();

    let song = match query(ctx, msg, &args).await {
        Ok(s) => s,
        Err(e) => {
            msg.channel_id.say(ctx, e);
            return Ok(());
        }
    };
    msg.channel_id.say(ctx, song).await;
    Ok(())
}

#[command]
#[aliases(l, text)]
async fn lyrics(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    // msg.channel_id.say(ctx, format!("This command is still work in progress.")).await;
    let data = ctx.data.read().await;
    let genius_api = data.get::<GeniusApiWrapper>().unwrap();

    match query(ctx, msg, &args).await {
        Ok(s) => match genius_api.lyrics(s.id).await {
            Ok(l) => msg.channel_id.say(ctx, l).await,
            Err(e) => {
                msg.channel_id
                    .say(ctx, format!("Problem occured while getting lyrics: {}", e))
                    .await
            }
        },
        Err(e) => {
            msg.channel_id.say(ctx, e);
            return Ok(());
        }
    };

    Ok(())
}

#[command]
#[aliases(card)]
#[description("Create a lyric card containing a given quote")]
async fn quote(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    match quote_fn(ctx, msg, &args).await {
        Ok((card, img)) => {
            msg.channel_id
                .send_files(ctx, vec![&card[..]], |m| m.content(""))
                .await;

            // TODO we would ideally like some pointer magic
            // to automatically remove those files once they
            // are no longer needed
            std::fs::remove_file(img);
            std::fs::remove_file(card);
        }
        Err(e) => {
            msg.channel_id.say(ctx, e).await;
        }
    }
    Ok(())
}

/// returns (card_path, img_path)
async fn quote_fn(ctx: &Context, msg: &Message, args: &Args) -> Result<(String, String), String> {
    let data = ctx.data.read().await;
    let genius_api = data.get::<GeniusApiWrapper>().unwrap();
    let quote = args.message();

    let q = query(ctx, msg, args).await?;

    let img_url = genius_api.img(q.id).await?;
    let img = genius_api.download_img(&img_url).await?;
    let card = generate_card(&img, &quote, &q.artist, &q.title)?;

    Ok((card, img))
}
