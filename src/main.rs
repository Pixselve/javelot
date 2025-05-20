mod cli;
mod dav_server;
mod fake_file_system;
mod shows;
mod torbox_client;

use crate::cli::Cli;
use crate::dav_server::webdav_handler;
use crate::fake_file_system::{FakeFilesystem, File, Folder, Node};
use crate::shows::parse_shows_from_torrents;
use crate::torbox_client::Torbox;
use anyhow::Context;
use axum::Router;
use axum::routing::any;
use clap::Parser;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time;
use tracing::info;

#[derive(Clone)]
struct AppState {
    fake_file_system: Arc<Mutex<FakeFilesystem>>,
    reqwest_client: reqwest::Client,
    torbox_client: Arc<Torbox>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    let fake_fs = FakeFilesystem::new_with_root();

    let client = reqwest::Client::new();
    let torbox_client = Torbox::new(cli.api_key.clone());

    let app_state = AppState {
        fake_file_system: Arc::new(Mutex::new(fake_fs)),
        reqwest_client: client.clone(),
        torbox_client: Arc::new(torbox_client),
    };

    start_refresh_job(app_state.clone(), cli.refresh_interval).await;

    let app = Router::new()
        .route("/", any(webdav_handler))
        .route("/{*path}", any(webdav_handler))
        .with_state(app_state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind(cli.address).await?;
    info!(
        address = listener.local_addr()?.to_string(),
        message = "Starting server"
    );
    axum::serve(listener, app)
        .await
        .context("Failed to run app")?;
    Ok(())
}

async fn refresh_filesystem(app_state: AppState) -> anyhow::Result<()> {
    info!("Refreshing filesystem...");

    let torrents = app_state.torbox_client.list_torrents().await?;
    let shows = parse_shows_from_torrents(torrents)?;

    // Lock the filesystem for updating
    let mut fake_fs = app_state.fake_file_system.lock().unwrap();

    // Reset shows directory
    fake_fs.remove_node(&PathBuf::from("/shows"));
    fake_fs.add_node(
        &PathBuf::from("/shows"),
        Node::Folder(Folder {
            name: "shows".to_string(),
        }),
    );

    // Add all shows again
    for show in shows {
        let path = PathBuf::from(&format!("/shows/{}", &show.title));
        fake_fs.add_node(&path, Node::Folder(Folder { name: show.title }));

        for season in show.seasons.values() {
            let season_name = format!("Season {}", season.number);
            let season_folder = PathBuf::from(&season_name);
            let season_path = path.join(season_folder);
            fake_fs.add_node(&season_path, Node::Folder(Folder { name: season_name }));

            for episode in &season.episodes {
                let episode_folder = PathBuf::from(&episode.file_name);
                let episode_path = season_path.join(episode_folder);
                fake_fs.add_node(
                    &episode_path,
                    Node::File(File {
                        name: episode.file_name.clone(),
                        size: episode.size,
                        download_details: (
                            episode.torbox_file_metadata.torrent_id,
                            episode.torbox_file_metadata.file_id,
                        ),
                    }),
                )
            }
        }
    }

    info!("Filesystem refresh completed");
    Ok(())
}

async fn start_refresh_job(app_state: AppState, refresh_interval: u64) {
    let refresh_interval = time::Duration::from_secs(refresh_interval);
    let mut interval = tokio::time::interval(refresh_interval);

    tokio::spawn(async move {
        loop {
            interval.tick().await;
            if let Err(e) = refresh_filesystem(app_state.clone()).await {
                tracing::error!("Failed to refresh filesystem: {:?}", e);
            }
        }
    });

    info!(
        interval = refresh_interval.as_secs(),
        message = "Started filesystem refresh job"
    );
}
