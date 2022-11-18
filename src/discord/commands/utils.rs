use std::time::Duration;

use crate::{
    genius::{GeniusApiWrapper, SongQuery},
    send_error, send_message,
};
use serenity::{framework::standard::Args, model::prelude::*, prelude::*};

pub async fn ask_user_for_a_song(ctx: &Context, msg: &Message, args: &Args) -> Option<SongQuery> {
    let arg = args.message();
    let mut m = msg.content.to_owned();

    // m will now have the command name itself, without the prefix
    m.remove(0);

    if arg.len() == 0 {
        send_message!(
            ctx,
            msg,
            "Send **~help {}** to see the usage of this command.",
            m
        );
        return None;
    }
    if arg.len() < 2 {
        send_error!(ctx, msg, "Query '{}' is too short!", arg);
        return None;
    } else if textwrap::wrap(&arg, 46).len() > 8 {
        send_error!(ctx, msg, "This query is too long!");
        return None;
    }
    let data = ctx.data.read().await;
    let genius_api = data.get::<GeniusApiWrapper>().unwrap();

    send_message!(ctx, msg, "Searching genius for: **{}**\n", arg);

    let results: Vec<SongQuery> = genius_api.search_song(arg).await?;

    match results.len() {
        0 => {
            send_message!(ctx, msg, "**No results found!**");
            None
        }
        1 => {
            let res = results.get(0).unwrap().clone();
            send_message!(ctx, msg, "Singe result found: **{}**", res);
            Some(res)
        }
        _ => {
            let mut options: String = String::new();

            for r in results.iter().enumerate() {
                options.push_str(&format!("{}. - **{}**\n", r.0 + 1, r.1));
            }

            send_message!(
                ctx,
                msg,
                "Multiple results were found, please choose one:\n{}",
                options
            );
            send_message!(ctx, msg, "Send **c** to **CANCEL**");

            if let Some(answer) = &msg
                .author
                .await_reply(ctx)
                .timeout(Duration::from_secs(60))
                .await
            {
                let index = if let Ok(v) = answer.content.parse::<usize>() {
                    v.max(1) - 1
                } else {
                    if answer.content != "c" {
                        send_message!(ctx, msg, "That's not a valid number!");
                    }
                    return None;
                };

                if let Some(v) = results.get(index) {
                    let v = v.clone();
                    send_message!(ctx, msg, "You've chosen: **{}**", v);
                    Some(v)
                } else {
                    send_message!(ctx, msg, "There is no result with that number.");
                    None
                }
            } else {
                send_message!(ctx, msg, "Time's up!");
                None
            }
        }
    }
}
