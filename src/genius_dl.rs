use jq_rs;
use reqwest::{self, Request};
use scraper::{Html, Selector};
use serde::Deserialize;
use serde_json;
use serenity::http::request;
use std::fmt;

pub struct GeniusApi {
    client: reqwest::Client,
    genius_token: String,
}

#[derive(Deserialize)]
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

type RequestSettings = dyn FnOnce(reqwest::RequestBuilder) -> reqwest::RequestBuilder;

impl GeniusApi {
    pub fn new(genius_token: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            genius_token: genius_token.to_string(),
        }
    }

    async fn safe_get(
        &self,
        url: &str,
        request_settings: RequestSettings,
    ) -> Result<String, String> {
        request_settings(self.client.get(url))
            .send()
            .await
            .map_err(|e| format!("Error occured while trying to make a request: {}", e))?
            .text()
            .await
            .map_err(|e| format!("Error occured while parsing the response: {}", e))
    }

    // async fn get_api(
    //     &self,
    //     path: &str,
    //     query: Option<Vec<(&str, &str)>>,
    // ) -> Result<String, String> {
    //     let request_settings: &Box<RequestSettings> = if let Some(q) = query {
    //         &Box::new(|r| r.bearer_auth(&self.genius_token).query(&q))
    //     } else {
    //         &Box::new(|r| r.bearer_auth(&self.genius_token))
    //     };
    //     self.safe_get(
    //         &format!("https://api.genius.com/{}", path),
    //         request_settings,
    //     )
    //     .await
    // }

    // fuck lifetimes
    // fuck closures
    // it will be probably best if we just simplify everything
    // OR
    // figure out how to have non static lifetime for RequestSettings
    async fn query_api(&self, path: &str, query: String) -> Result<String, String> {
        let token = self.genius_token.clone();
        let request_settings: Box<RequestSettings> =
            Box::new(move |r: reqwest::RequestBuilder| r.bearer_auth(&token).query(&vec![("q", query)]));

        self.safe_get(
            &format!("https://api.genius.com/{}", path),
            *request_settings,
        )
        .await
    }

    // if there is no match, jq returns error message as String
    fn parse_query(data: &str) -> Result<Vec<SongQuery>, String> {
        let mut jq_query = jq_rs::compile("[.response.hits[] | .result | { artist: .primary_artist.name, title: .title, id: .id }]").unwrap();
        let jq_out = jq_query.run(data).map_err(|v| v.to_string())?;

        Ok(serde_json::from_str(&jq_out).unwrap())
    }

    pub async fn search_song(&self, query: &str) -> Result<Vec<SongQuery>, String> {
        let raw_data = self
            .query_api("search", Some(vec![("q", query)]))
            .await
            .map_err(|v| {
                format!(
                    "Error occured while trying to make a request: {}",
                    v.to_string()
                )
            })?;

        GeniusApi::parse_query(&raw_data)
    }

    // make sure to run only trusted jq input here
    async fn query_string_from_song_object(
        &self,
        song_id: u32,
        jq: &str,
    ) -> Result<String, String> {
        let raw_data = self
            .query_api(&format!("songs/{}", song_id), None)
            .await
            .map_err(|v| {
                format!(
                    "Error occured while trying to make a request: {}",
                    v.to_string()
                )
            })?;

        let mut jq = jq_rs::compile(jq).map_err(|_| {
            format!(
                "**This shouldn't happen in production!!!**\nError occured while compiling jq program!",
            )
        })?;
        let jq_out = jq.run(&raw_data).unwrap();

        Ok(serde_json::from_str::<String>(&jq_out).unwrap())
    }

    pub async fn img(&self, song_id: u32) -> Result<String, String> {
        self.query_string_from_song_object(song_id, ".response.song.header")
            .await
    }

    pub async fn lyrics(&self, song_id: u32) -> Result<String, String> {
        let url = self
            .query_string_from_song_object(song_id, ".response.song.url")
            .await?;
        let document = self.client.get(url).send().await?.text().await?;
        let html = Html::parse_document(&document);
        dbg!(html);
        Ok("".to_string())
        // let selector = Selector::parse("")
    }
}
