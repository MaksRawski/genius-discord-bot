use std::path::Path;
use std::process::{Command, ExitStatusError};

// TODO use a random name for output and return it
pub fn generate_card(
    img: &str,
    caption: &str,
    author: &str,
    track: &str,
) -> Result<(), ExitStatusError> {
    // TODO check if img exists
    Command::new("./scripts/generate.sh")
        .args([img, caption, author, track])
        .status()
        .unwrap()
        .exit_ok()
}
