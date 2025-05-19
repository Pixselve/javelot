use clap::Parser;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::net::SocketAddr;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Your debrid service API key. Currently only supports TorBox.
    #[clap(long, env = "API_KEY")]
    pub api_key: String,

    /// The address the WebDav server will listen to.
    #[clap(long, env = "ADDRESS", default_value_t = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 3000)
    )]
    pub address: SocketAddr,

    /// Debrid service file refresh interval in seconds.
    #[clap(short, long, default_value_t = 60 * 10, env = "REFRESH_INTERVAL")]
    pub refresh_interval: u64,
}
