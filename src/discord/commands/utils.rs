use regex::Regex;
use std::time::Duration;

use crate::{
    genius::{GeniusApiWrapper, Song},
    send_error, send_message,
};
use serenity::{framework::standard::Args, model::prelude::*, prelude::*};

pub async fn ask_user_for_a_song(ctx: &Context, msg: &Message, args: &Args) -> Option<Song> {
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

    let results: Vec<Song> = match genius_api.search_for_song(arg).await {
        Ok(matches) => matches,
        Err(e) => {
            send_error!(ctx, msg, "Failed to search for a song: {}", e);
            return None;
        }
    };

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

            for (i, r) in results.iter().enumerate() {
                options.push_str(&format!("{}. **{}**\n", i + 1, r));
            }

            send_message!(
                ctx,
                msg,
                "Multiple results were found, please choose one:\n{}",
                options
            );

            if let Some(answer) = &msg
                .author
                .await_reply(ctx)
                .timeout(Duration::from_secs(60))
                .await
            {
                // ideally we would create this regex somewhere else just once
                // instead of recreating it here every time
                let re = Regex::new(r"[0-9][0-9]?").unwrap();
                let choice = match re.find(&answer.content) {
                    Some(m) => match m.as_str().parse::<usize>() {
                        Ok(v) => v,
                        Err(_) => {
                            // NOTE: we should never be here but there is no need to panic!
                            send_message!(ctx, msg, "That's not a number!");
                            return None;
                        }
                    },
                    None => {
                        send_message!(ctx, msg, "That's not a number!");
                        return None;
                    }
                };

                if let Some(v) = results.get(choice - 1) {
                    let v = v.clone();
                    send_message!(ctx, msg, "You've chosen: **{}**", v);
                    Some(v)
                } else {
                    send_message!(ctx, msg, "There is no result with that number!");
                    None
                }
            } else {
                send_message!(ctx, msg, "Time's up!");
                None
            }
        }
    }
}
