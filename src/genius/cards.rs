use rand::distributions::Alphanumeric;
use rand::prelude::*;
use std::ops::Add;
use std::process::Command;
use tracing::error;

// TODO use lifetimes for the filename
/// returns filename of the output image
/// img must exist and be valid, no checks are done!
pub fn generate_card(img: &str, caption: &str, author: &str, track: &str) -> Option<String> {
    let filename: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect::<String>()
        .add(".jpg");

    Command::new("./scripts/generate.sh")
        .args([img, caption, author, track, &filename])
        .status()
        .map_err(|e| error!("Error in card generation: {}", e))
        .ok()?;

    Some(filename)
}
