use rand::distributions::Alphanumeric;
use rand::prelude::*;
use std::ops::Add;
use std::process::Command;

// TODO use lifetimes for the filename
/// returns filename of the output image
/// img must exist and be valid, no checks are done!
pub fn generate_card(
    img: &str,
    caption: &str,
    author: &str,
    track: &str,
) -> Result<String, String> {
    let filename: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect::<String>()
        .add(".jpg");

    Command::new("./scripts/generate.sh")
        .args([img, caption, author, track, &filename])
        .status()
        .map_err(|e| e.to_string())?;

    Ok(filename)
}
