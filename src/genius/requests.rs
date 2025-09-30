use image::DynamicImage;
use jq_rs;
use regex::Regex;
use reqwest::{self, RequestBuilder};
use scraper::{Html, Selector};
use serde::Deserialize;
use serde_json;
use serenity::prelude::TypeMapKey;
use std::fmt;
use std::sync::Arc;
use std::time::Duration;
use tracing::error;

#[derive(Clone, Deserialize)]
pub struct Song {
    pub artist: String,
    pub title: String,
    pub id: u32,
}

impl fmt::Display for Song {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} - {}", self.artist, self.title)
    }
}

pub struct GeniusApi {
    client: reqwest::Client,
    genius_token: String,
}

pub struct GeniusApiWrapper;

impl TypeMapKey for GeniusApiWrapper {
    type Value = Arc<GeniusApi>;
}

// errors from these functions shouldn't be visible to the user
// therefore there are just logged and None will be returned
// if anything fails
impl GeniusApi {
    pub fn new(genius_token: &str) -> Self {
        Self {
            client: reqwest::ClientBuilder::new()
                .timeout(Duration::new(10, 0))
                .build()
                .expect("Failed to build client"),
            genius_token: genius_token.to_string(),
        }
    }

    async fn safe_get(&self, request: RequestBuilder) -> Option<String> {
        request
            .send()
            .await
            .map_err(|e| error!("While trying to make a request: {}", e))
            .ok()?
            .text()
            .await
            .map_err(|e| error!("While parsing the response: {}", e))
            .ok()
    }

    async fn query_api(&self, path: &str, query: &str) -> Option<String> {
        let request = self
            .client
            .get(format!("https://api.genius.com/{}", path))
            .bearer_auth(&self.genius_token)
            .query(&vec![("q", query)]);

        self.safe_get(request).await
    }

    fn parse_query(data: &str) -> Option<Vec<Song>> {
        let mut jq_query = jq_rs::compile("[.response.hits[] | .result | { artist: .primary_artist.name, title: .title, id: .id }]").unwrap();
        let jq_out = jq_query.run(data).map_err(|v| v.to_string()).ok()?;

        serde_json::from_str(&jq_out)
            .map_err(|e| tracing::error!("While parsing the API response: {}", e))
            .ok()
    }

    /// returns a vector of upto 10 matching songs or None if an error occured
    pub async fn search_for_song(&self, query: &str) -> Option<Vec<Song>> {
        let raw_data = self.query_api("search", query).await?;

        GeniusApi::parse_query(&raw_data)
    }

    /// for a given song_id tries to find a value which matches provided jq query
    async fn jq_song_info(&self, song_id: u32, jq: &str) -> Option<String> {
        let raw_data = self.query_api(&format!("songs/{}", song_id), "").await?;

        let mut jq = jq_rs::compile(&format!("{}{}", ".response |", jq)).map_err(|_| {
            error!(
                "**This shouldn't happen in production!!!**\nError occured while compiling jq program!",
            )
        }).ok()?;

        let jq_out = jq
            .run(&raw_data)
            .map_err(|e| error!("Error occured while parsing the API response: {}", e))
            .ok()?;

        serde_json::from_str::<String>(&jq_out)
            .map_err(|e| {
                error!("Error occured while parsing the API response: {}", e);
            })
            .ok()
    }

    /// returns a URL of song's cover image
    pub async fn img(&self, song_id: u32) -> Option<String> {
        self.jq_song_info(song_id, ".song.header_image_url").await
    }

    /// returns a path to downloaded img
    pub async fn download_img(&self, img_url: &str) -> Option<DynamicImage> {
        let img_data = self
            .client
            .get(img_url)
            .send()
            .await
            .map_err(|e| error!("{}", e))
            .ok()?
            .bytes()
            .await
            .ok()?;
        let image = image::load_from_memory(&img_data).ok()?;

        Some(image)
    }

    pub async fn get_song_url(&self, song_id: u32) -> Option<String> {
        self.jq_song_info(song_id, ".song.url").await
    }

    /// returns formatted lyrics (without annotations)
    pub async fn lyrics(&self, url: &str) -> Option<String> {
        let document = self.safe_get(self.client.get(url)).await?;

        let html = Html::parse_document(&document);
        let selector = Selector::parse("div[data-lyrics-container='true']").unwrap();
        let raw_lyrics = html
            .select(&selector)
            .map(|verse| verse.inner_html())
            .collect::<String>();

        let remove_annotations = Regex::new("(<br>)|<[^br].*?>").unwrap();
        let lyrics = remove_annotations
            .replace_all(&raw_lyrics, "$1")
            .replace("<br>", "\n");

        Some(lyrics)
    }
}
