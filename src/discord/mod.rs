pub mod commands;
pub mod setup;

pub use setup::Discord;

/// Expects first 2 args to be &Context and &Message,
/// then it works just like a normal `format!`.
#[macro_export]
macro_rules! send_message {
    ($ctx: expr, $og_msg: expr, $msg: literal) => {
        {
            tracing::info!($msg);
            $og_msg.channel_id.say($ctx, $msg.to_string()).await.unwrap()
        }
    };
    ($ctx: expr, $og_msg: expr, $fmt: literal, $($fmt_args: expr),+) => {
        {
            tracing::info!($fmt, $($fmt_args),+);
            $og_msg.channel_id.say($ctx, format!($fmt, $($fmt_args),+)).await.unwrap()
        }
    }
}

/// Similar to `send_message!` except it will bold the entire error message, if it's a simple string.
/// Expects first 2 args to be &Context and &Message,
/// then it works just like a normal `format!`.
#[macro_export]
macro_rules! send_error {
    ($ctx: expr, $og_msg: expr, $msg: literal) => {
        {
            tracing::error!($msg);
            $og_msg.channel_id.say($ctx, format!("**{}**", $msg)).await.unwrap();
        }
    };
    ($ctx: expr, $og_msg: expr, $fmt: literal, $($fmt_args: expr),+) => {
        {
            tracing::error!($fmt, $($fmt_args),+);
            $og_msg.channel_id.say($ctx, format!($fmt, $($fmt_args),+)).await.unwrap();
        }
    };
}
