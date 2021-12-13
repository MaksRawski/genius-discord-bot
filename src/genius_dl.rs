use jq_rs;
use reqwest;
use serde::Deserialize;
use serde_json;
use std::fmt;

pub struct ImageDownloader {
    client: reqwest::Client,
    genius_token: String,
}

#[derive(Deserialize)]
pub struct QueryResult {
    pub artist: String,
    pub title: String,
    pub id: u32,
}

impl fmt::Display for QueryResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} - {}", self.artist, self.title)
    }
}

impl ImageDownloader {
    pub fn new(genius_token: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            genius_token: genius_token.to_string(),
        }
    }

    // if there is no match, jq returns error message as String
    fn parse_data(data: &str) -> Result<Vec<QueryResult>, String> {
        let mut jq_query = jq_rs::compile("[.response.hits[] | .result | { artist: .primary_artist.name, title: .title, id: .id }]").unwrap();
        let jq_out = jq_query.run(data).map_err(|v| v.to_string())?;

        Ok(serde_json::from_str(&jq_out).unwrap())
    }

    async fn query_api(&self, query: &str) -> Result<String, reqwest::Error> {
        let query = self
            .client
            .get("https://api.genius.com/search")
            .bearer_auth(&self.genius_token)
            .query(&[("q", query)]);

        query.send().await?.text().await
    }

    pub async fn query(&self, query: &str) -> Result<Vec<QueryResult>, String> {
        let raw_data = self.query_api(query).await.map_err(|v| v.to_string())?;

        ImageDownloader::parse_data(&raw_data)
    }

    pub async fn img(&self, song_id: u32) -> Result<String, reqwest::Error> {
        let url = format!("https://api.genius.com/songs/{}", song_id);
        let query = self.client.get(url).bearer_auth(&self.genius_token);
        let raw_data = query.send().await?.text().await.unwrap();

        let mut jq = jq_rs::compile(".response.song.header_image_url").unwrap();
        let jq_out = jq.run(&raw_data).unwrap();
        Ok(serde_json::from_str::<String>(&jq_out).unwrap())
    }
}
