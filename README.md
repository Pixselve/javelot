# Javelot

A WebDAV server ready for Plex consumption.

## What it does

1. Scans the list of torrents on you debrid service.
2. Parses the names of the files.
3. Creates a file system that makes Plex happy.

```text
shows/
├─ my show 1/
│  ├─ Season 1/
│  │  ├─ my episode.mkv
├─ my show 2/
│  ├─ Season 9/
│  │  ├─ my episode.mkv
│  │  ├─ my second episode.mkv
```

## Supported debrid Services

- [TorBox](https://torbox.app/dashboard)

## Usage

> Assuming you have Rust and rclone installed.

```shell
cargo build --release
./target/javelot --address 127.0.0.1:3000 --api-key <YOUR TORBOX API KEY>
```

Configure rclone with a new `webdav` server at `127.0.0.1:3000` with no authentication.

In another terminal session:

```shell
rclone mount <name>:/ <path to mount>
```

You can then point your Plex server to that path.

## Roadmap

- [ ] Improve content parsing
    - Will maybe adopt a strategy like Riven with a separate source of trust with mapping `content <> file(s)`
      while having a backfilling script to populate the database from existing content.
- [ ] Support for movies
- [ ] Support for animes
- [ ] Support for RealDebrid
- [ ] Support for Plex refreshing
- [ ] Content fetching support?
    - TorBox supports downloading from RSS feeds. I want to experiment with having a Torrentio -> RSS API.