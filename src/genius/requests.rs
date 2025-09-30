use image::{load_from_memory, DynamicImage};
use regex::Regex;
use reqwest;
use scraper::{Html, Selector};
use serde::Deserialize;
use serde_json;
use serenity::prelude::TypeMapKey;
use std::fmt;
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone, Deserialize)]
pub struct Song {
    #[serde(rename = "primary_artist_names")]
    pub artist: String,
    pub title: String,
    pub id: u32,
    pub header_image_url: String,
    pub url: String,
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

    async fn query_api(&self, path: &str, query: &str) -> Result<String, anyhow::Error> {
        Ok(self
            .client
            .get(format!("https://api.genius.com/{}", path))
            .bearer_auth(&self.genius_token)
            .query(&vec![("q", query)])
            .send()
            .await?
            .text()
            .await?)
    }

    /// returns a vector of upto 10 matching songs or None if an anyhow::error occured
    pub async fn search_for_song(&self, query: &str) -> Result<Vec<Song>, anyhow::Error> {
        let raw_data = self.query_api("search", query).await?;

        Ok(serde_json::from_str::<SearchResponseTop>(&raw_data)?
            .response
            .hits
            .iter()
            .map(|hit| hit.result.clone())
            .collect())
    }

    pub async fn get_song_by_id(&self, song_id: u32) -> Result<Song, anyhow::Error> {
        Ok(serde_json::from_str::<SongResponseTop>(
            &self.query_api(&format!("songs/{}", song_id), "").await?,
        )?
        .response
        .song)
    }

    pub async fn get_cover(&self, song_id: u32) -> Result<DynamicImage, anyhow::Error> {
        let url = self.get_cover_url(song_id).await?;
        let img_data = self.get_img(&url).await?;
        load_from_memory(&img_data).map_err(|e| e.into())
    }

    /// returns a URL of song's cover image
    async fn get_cover_url(&self, song_id: u32) -> Result<String, anyhow::Error> {
        Ok(serde_json::from_str::<SongResponseTop>(
            &self.query_api(&format!("songs/{}", song_id), "").await?,
        )?
        .response
        .song
        .header_image_url)
    }

    async fn get_img(&self, img_url: &str) -> Result<Vec<u8>, reqwest::Error> {
        self.client
            .get(img_url)
            .send()
            .await?
            .bytes()
            .await
            .map(|b| b.to_vec())
    }

    // pub async fn get_song_url(&self, song_id: u32) -> Result<String, anyhow::Error> {
    //     self.get_field_as_string(song_id, &["song", "url"]).await
    // }

    /// returns formatted lyrics (without annotations)
    pub async fn lyrics(&self, url: &str) -> Result<String, anyhow::Error> {
        let document = self.client.get(url).send().await?.text().await?;

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

        Ok(lyrics)
    }
}

#[derive(Deserialize)]
struct SearchResponseTop {
    response: SearchResponse,
}

#[derive(Deserialize)]
struct SearchResponse {
    hits: Vec<Hit>,
}

#[derive(Deserialize)]
struct Hit {
    result: Song,
}

#[derive(Deserialize)]
struct SongResponseTop {
    response: SongResponse,
}
#[derive(Deserialize)]
struct SongResponse {
    song: Song,
}
