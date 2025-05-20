use anyhow::Context;
use clap::builder::styling::Reset;
use headers::HeaderValue;
use moka::future::Cache;
use reqwest::Response;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;
use tracing::{debug, info};

#[derive(Clone, Debug)]
pub struct Torbox {
    api_key: String,
    base_url: String,
    client: reqwest::Client,
    cache: Cache<String, String>,
}

impl Torbox {
    pub fn new(api_key: String) -> Self {
        Torbox {
            api_key,
            base_url: "https://api.torbox.app".to_string(),
            client: reqwest::Client::new(),
            cache: Cache::builder()
                .time_to_idle(Duration::from_secs(60 * 60 * 3))
                .build(),
        }
    }

    pub async fn list_torrents(&self) -> anyhow::Result<Vec<Torrent>> {
        let url = format!("{}/v1/api/torrents/mylist", self.base_url);
        let request = self
            .client
            .request(reqwest::Method::GET, url)
            .bearer_auth(&self.api_key);
        let resp = request.send().await.context("Failed to send request")?;
        if !resp.status().is_success() {
            anyhow::bail!("Request failed: {}", resp.status());
        }
        let json = resp
            .json::<ListTorrentsResponse>()
            .await
            .context("Failed to parse json")?;
        let active_torrents: Vec<Torrent> = json
            .data
            .into_iter()
            .filter(|torrent| torrent.download_present)
            .collect();
        Ok(active_torrents)
    }

    pub async fn torrent_stream(
        &self,
        torrent_id: i64,
        file_id: i64,
        range_header: Option<HeaderValue>,
    ) -> anyhow::Result<Response> {
        let key = format!("torrent_id:{},file_id:{}", torrent_id, file_id);

        let url = self
            .cache
            .try_get_with(key, async {
                info!("File {} {} not present in cache", file_id, torrent_id);
                let url = format!("{}/v1/api/torrents/requestdl", &self.base_url);
                let mut request = self
                    .client
                    .request(reqwest::Method::GET, url)
                    .query(&[
                        ("token", self.api_key.to_owned()),
                        ("torrent_id", torrent_id.to_string()),
                        ("file_id", file_id.to_string()),
                    ])
                    .bearer_auth(&self.api_key);
                let resp = request.send().await.context("Failed to send request")?;
                if !resp.status().is_success() {
                    anyhow::bail!("Request failed: {}", resp.status());
                }
                let json = resp
                    .json::<RequestDownloadLinkResponse>()
                    .await
                    .context("Failed to parse json")?;
                let download_url = json.data;
                Ok(download_url)
            })
            .await;

        if let Ok(url) = url {
            let mut request = self
                .client
                .request(reqwest::Method::GET, url)
                .bearer_auth(&self.api_key);

            if let Some(range) = range_header {
                request = request.header("Range", range);
            }

            let resp = request.send().await.context("Failed to send request")?;
            return Ok(resp);
        }

        anyhow::bail!("Failed to download torrent file");
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadLinkResponse {
    pub success: bool,
    pub error: Value,
    pub detail: String,
    pub data: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListTorrentsResponse {
    pub success: bool,
    pub error: Value,
    pub detail: String,
    pub data: Vec<Torrent>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Torrent {
    pub id: i64,
    #[serde(rename = "auth_id")]
    pub auth_id: String,
    pub server: i64,
    pub hash: String,
    pub name: Option<String>,
    pub magnet: Option<String>,
    pub size: i64,
    pub active: bool,
    #[serde(rename = "created_at")]
    pub created_at: String,
    #[serde(rename = "updated_at")]
    pub updated_at: String,
    #[serde(rename = "download_state")]
    pub download_state: String,
    pub seeds: i64,
    pub peers: i64,
    pub ratio: f64,
    pub progress: f64,
    #[serde(rename = "download_speed")]
    pub download_speed: i64,
    #[serde(rename = "upload_speed")]
    pub upload_speed: i64,
    pub eta: i64,
    #[serde(rename = "torrent_file")]
    pub torrent_file: bool,
    #[serde(rename = "expires_at")]
    pub expires_at: Option<String>,
    #[serde(rename = "download_present")]
    pub download_present: bool,
    pub files: Vec<File>,
    #[serde(rename = "download_path")]
    pub download_path: String,
    pub availability: f64,
    #[serde(rename = "download_finished")]
    pub download_finished: bool,
    pub tracker: Option<String>,
    #[serde(rename = "total_uploaded")]
    pub total_uploaded: i64,
    #[serde(rename = "total_downloaded")]
    pub total_downloaded: i64,
    pub cached: bool,
    pub owner: String,
    #[serde(rename = "seed_torrent")]
    pub seed_torrent: bool,
    #[serde(rename = "allow_zipped")]
    pub allow_zipped: bool,
    #[serde(rename = "long_term_seeding")]
    pub long_term_seeding: bool,
    #[serde(rename = "tracker_message")]
    pub tracker_message: Option<String>,
    #[serde(rename = "cached_at")]
    pub cached_at: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct File {
    pub id: i64,
    pub md5: Option<String>,
    pub hash: String,
    pub name: String,
    pub size: i64,
    pub zipped: Option<bool>,
    #[serde(rename = "s3_path")]
    pub s3_path: String,
    pub infected: Option<bool>,
    pub mimetype: String,
    #[serde(rename = "short_name")]
    pub short_name: String,
    #[serde(rename = "absolute_path")]
    pub absolute_path: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RequestDownloadLinkResponse {
    pub success: bool,
    pub error: Value,
    pub detail: String,
    pub data: String,
}
