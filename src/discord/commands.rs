use serenity::{
    framework::standard::{
        macros::{command, group},
        Args, CommandResult,
    },
    model::channel::Message,
    prelude::*,
};
use std::sync::Arc;

pub use crate::genius_dl::ImageDownloader;
use crate::genius_dl::QueryResult;
use std::time::Duration;

pub struct ImageDownloaderContainer;

impl TypeMapKey for ImageDownloaderContainer {
    type Value = Arc<ImageDownloader>;
}

#[group]
#[commands(query)]
pub struct General;

// TODO generalize this and reuse it
// also have a query command
#[command]
async fn query(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let arg = args.message();
    if arg.len() < 2 {
        msg.channel_id.say(ctx, format!("Query too short")).await;
        return Ok(());
    }
    let data = ctx.data.read().await;
    let image_downloader = data.get::<ImageDownloaderContainer>().unwrap();

    msg.channel_id
        .say(ctx, format!("Searching genius for: **{}**\n", arg))
        .await;

    let results: Vec<QueryResult> = image_downloader.query(arg).await.unwrap();

    let song_id = match results.len() {
        0 => {
            msg.channel_id.say(ctx, "**No results found!**").await;
            return Ok(());
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
                    msg.channel_id
                        .say(ctx, format!("That's not a valid number!"));
                    return Ok(());
                };
                let chosen_result = if let Some(v) = results.get(index) {
                    v
                } else {
                    msg.channel_id
                        .say(ctx, format!("Provided number is too big."))
                        .await;
                    return Ok(());
                };
                msg.channel_id
                    .say(ctx, format!("You've chosen: **{}**", chosen_result))
                    .await;
                chosen_result.id
            } else {
                msg.channel_id.say(ctx, format!("Time's up!")).await;
                return Ok(());
            }
        }
    };
    let url: &str = &image_downloader.img(song_id).await.unwrap();
    msg.channel_id
        .send_files(ctx, vec![url], |m| m.content("TEST"))
        .await;
    Ok(())
}
