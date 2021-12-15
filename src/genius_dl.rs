use jq_rs;
use rand::distributions::Alphanumeric;
use rand::prelude::*;
use reqwest::{self, Request, RequestBuilder};
use scraper::{element_ref::Select, Html, Selector};
use serde::Deserialize;
use serde_json;
use serenity::http::request;
use std::fmt;
use std::io::{Cursor, Read};

// TODO remove this and instead use an itertor
use std::fmt::Write as FmtWrite;

pub struct GeniusApi {
    client: reqwest::Client,
    genius_token: String,
}

#[derive(Deserialize, Clone)]
pub struct SongQuery {
    pub artist: String,
    pub title: String,
    pub id: u32,
}

impl fmt::Display for SongQuery {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} - {}", self.artist, self.title)
    }
}

impl GeniusApi {
    pub fn new(genius_token: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            genius_token: genius_token.to_string(),
        }
    }

    async fn safe_get(&self, request: RequestBuilder) -> Result<String, String> {
        request
            .send()
            .await
            .map_err(|e| format!("Error occured while trying to make a request: {}", e))?
            .text()
            .await
            .map_err(|e| format!("Error occured while parsing the response: {}", e))
    }

    async fn query_api(&self, path: &str, query: &str) -> Result<String, String> {
        let request = self
            .client
            .get(format!("https://api.genius.com/{}", path))
            .bearer_auth(&self.genius_token)
            .query(&vec![("q", query)]);

        self.safe_get(request).await
    }

    // if there is no match, jq returns error message as String
    fn parse_query(data: &str) -> Result<Vec<SongQuery>, String> {
        let mut jq_query = jq_rs::compile("[.response.hits[] | .result | { artist: .primary_artist.name, title: .title, id: .id }]").unwrap();
        let jq_out = jq_query.run(data).map_err(|v| v.to_string())?;

        serde_json::from_str(&jq_out).map_err(|e| {
            dbg!(e);
            "Error occured while parsing the API response".to_string()
        })
    }

    pub async fn search_song(&self, query: &str) -> Result<Vec<SongQuery>, String> {
        let raw_data = self.query_api("search", query).await?;

        GeniusApi::parse_query(&raw_data)
    }

    /// for a given song_id tries to find a value which matches provided jq query
    pub async fn jq_song_info(&self, song_id: u32, jq: &str) -> Result<String, String> {
        let raw_data = self.query_api(&format!("songs/{}", song_id), "").await?;

        let mut jq = jq_rs::compile(&format!("{}{}", ".response |", jq)).map_err(|_| {
            format!(
                "**This shouldn't happen in production!!!**\nError occured while compiling jq program!",
            )
        })?;
        let jq_out = jq
            .run(&raw_data)
            .map_err(|e| format!("Error occured while parsing the API response: {}", e))?;

        serde_json::from_str::<String>(&jq_out).map_err(|e| {
            dbg!(e);
            "Error occured while parsing the API response".to_string()
        })
    }

    /// returns a URL of song's cover image
    pub async fn img(&self, song_id: u32) -> Result<String, String> {
        self.jq_song_info(song_id, ".song.header_image_url").await
    }

    /// returns a path to downloaded img
    pub async fn download_img(&self, img_url: &str) -> Result<String, String> {
        // TODO many things in here could fail therefore
        // it should be handled more properly
        let resp = self
            .client
            .get(img_url)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let filename: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(30)
            .map(char::from)
            .collect();

        let mut file = std::fs::File::create(&filename)
            .map_err(|_| "Failed to create a new file".to_string())?;
        let mut img_data = Cursor::new(
            resp.bytes()
                .await
                .map_err(|_| "Empty response".to_string())?,
        );
        std::io::copy(&mut img_data, &mut file);

        Ok(filename)
    }

    /// returns formatted lyrics (without annotations)
    pub async fn lyrics(&self, song_id: u32) -> Result<String, String> {
        let song_url = self.jq_song_info(song_id, ".response.song.url").await?;
        let document = self.safe_get(self.client.get(song_url)).await?;

        let html = Html::parse_document(&document);
        let selector = Selector::parse("#lyrics-root > div").unwrap();

        // this will return iterator of paragraphs
        // so it needs to be formatted
        let mut lyrics = String::new();
        // TODO use iterator magic here instead
        for p in html.select(&selector) {
            writeln!(&mut lyrics, "{}\n", p.value().name());
        }
        dbg!(html);
        Ok("".to_string())
        // let selector = Selector::parse("")
    }
}
