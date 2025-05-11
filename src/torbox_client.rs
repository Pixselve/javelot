use anyhow::Context;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::info;

#[derive(Clone, Debug)]
pub struct Torbox {
    api_key: String,
    base_url: String,
    client: reqwest::Client,
}

impl Torbox {
    pub fn new(api_key: String) -> Self {
        Torbox {
            api_key,
            base_url: "https://api.torbox.app".to_string(),
            client: reqwest::Client::new(),
        }
    }

    pub async fn list_torrents(&self) -> anyhow::Result<Vec<Torrent>> {
        info!("{}", self.api_key);
        let url = format!("{}/v1/api/torrents/mylist", self.base_url);
        let request = self
            .client
            .request(reqwest::Method::GET, url)
            .bearer_auth(&self.api_key);
        let resp = request.send().await.context("Failed to send request")?;
        info!("Got response: {:?}", resp);
        if !resp.status().is_success() {
            anyhow::bail!("Request failed: {}", resp.status());
        }
        let json = resp
            .json::<ListTorrentsResponse>()
            .await
            .context("Failed to parse json")?;
        Ok(json.data)
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
