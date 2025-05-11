use crate::torbox_client::{ListTorrentsResponse, Torrent};
use anyhow::Context;
use std::collections::HashMap;
use tokio::fs;
use torrent_name_parser::Metadata;

#[derive(Debug, Clone)]
pub struct Show {
    pub title: String,
    pub seasons: HashMap<i32, ShowSeason>,
}
#[derive(Debug, Clone)]
pub struct ShowSeason {
    pub number: i32,
    pub episodes: Vec<ShowEpisode>,
}
#[derive(Debug, Clone)]
pub struct ShowEpisode {
    pub number: i32,
    pub torbox_file_metadata: TorboxFileMetadata,
    pub size: i64,
    pub file_name: String,
}
#[derive(Debug, Clone)]
pub struct TorboxFileMetadata {
    pub(crate) file_id: i64,
    pub(crate) torrent_id: i64,
}

async fn shows_from_cache() -> anyhow::Result<Vec<Show>> {
    let file_content = fs::read_to_string("src/torbox_cache.json").await?;
    let json: ListTorrentsResponse =
        serde_json::from_str(&file_content).context("failed to parse json")?;
    parse_shows_from_torrents(json.data)
}

pub fn parse_shows_from_torrents(torrents: Vec<Torrent>) -> anyhow::Result<Vec<Show>> {
    let mut shows = HashMap::new();

    for torrent in torrents {
        for file in torrent.files {
            if let Ok(metadata) = Metadata::from(&file.name) {
                if !metadata.is_show() {
                    continue;
                }

                let title = metadata.title();
                let season_number = match metadata.season() {
                    Some(season) => season,
                    None => continue,
                };

                let episode_numbers = metadata.episodes();
                if episode_numbers.len() != 1 {
                    continue;
                }

                let show = shows.entry(title.to_string()).or_insert_with(|| Show {
                    title: title.to_string(),
                    seasons: HashMap::new(),
                });

                let season = show
                    .seasons
                    .entry(season_number)
                    .or_insert_with(|| ShowSeason {
                        number: season_number,
                        episodes: vec![],
                    });

                season.episodes.push(ShowEpisode {
                    number: episode_numbers[0],
                    size: file.size,
                    file_name: file.short_name.clone(),
                    torbox_file_metadata: TorboxFileMetadata {
                        torrent_id: torrent.id,
                        file_id: file.id,
                    },
                });
            }
        }
    }

    Ok(shows.into_values().collect())
}
