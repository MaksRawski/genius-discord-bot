use serenity::{
    framework::standard::{
        macros::{command, group},
        Args, CommandResult,
    },
    model::channel::Message,
    prelude::*,
};
use std::sync::Arc;

pub use crate::genius_dl::GeniusApi;
use crate::genius_dl::QueryResult;
use std::time::Duration;

pub struct GeniusApiWrapper;

impl TypeMapKey for GeniusApiWrapper {
    type Value = Arc<GeniusApi>;
}

#[group]
// #[commands(lyrics, quote)]
#[commands(img)]
pub struct Query;

// either returns song_id or String with an error message
async fn query(ctx: &Context, msg: &Message, args: Args) -> Result<u32, String> {
    let arg = args.message();
    if arg.len() < 2 {
        return Err("Query too short".to_string());
    }
    let data = ctx.data.read().await;
    let genius_api = data.get::<GeniusApiWrapper>().unwrap();

    msg.channel_id
        .say(ctx, format!("Searching genius for: **{}**\n", arg))
        .await;

    let results: Vec<QueryResult> = genius_api.query(arg).await.unwrap();

    let song_id = match results.len() {
        0 => {
            return Err("**No results found!**".to_string());
        }
        1 => results.get(0).unwrap().id,
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
                let chosen_result = if let Some(v) = results.get(index) {
                    v
                } else {
                    return Err(format!("Provided number is too big."));
                };
                msg.channel_id
                    .say(ctx, format!("You've chosen: **{}**", chosen_result))
                    .await;
                chosen_result.id
            } else {
                return Err(format!("Time's up!"));
            }
        }
    };
    Ok(song_id)
}

async fn get_thumbnail(ctx: &Context, msg: &Message, args: Args) -> Result<String, String> {
    let data = ctx.data.read().await;
    let genius_api = data.get::<GeniusApiWrapper>().unwrap();

    let song_id = query(ctx, msg, args).await?;
    let img_url = genius_api
        .img(song_id)
        .await
        .map_err(|_| "A problem occured while downloading the cover image".to_string())?;

    Ok(img_url)
}

#[command]
#[aliases(image, cover, art, thumbnail)]
#[description("Query a song's thumbnail")]
async fn img(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let img = match get_thumbnail(ctx, msg, args).await {
        Ok(img) => img,
        Err(e) => {
            msg.channel_id.say(ctx, e);
            return Ok(())
        },
    };
    msg.channel_id
        .send_files(ctx, vec![&img[..]], |m| m.content(""))
        .await;

    Ok(())
}

// #[command]
// #[aliases(text, find, search)]
// async fn lyrics(ctx: &Context, msg: &Message, args: Args) -> CommandResult {}

// #[command]
// #[aliases(card)]
// #[description("Create a lyric card containing a given quote")]
// async fn quote(ctx: &Context, msg: &Message, args: Args) -> CommandResult {}
