//! Volume binary entrypoint.

use mini_kvstore_v2::Config;
use mini_kvstore_v2::volume::server::start_volume_server;
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_env();

    let bind_addr = SocketAddr::from(([0, 0, 0, 0], config.port));

    println!("Starting volume server:");
    println!("  volume_id = {}", config.volume_id);
    println!("  data_dir  = {}", config.data_dir);
    println!("  bind_addr = {}", bind_addr);
    println!("  compaction_threshold = {}", config.compaction_threshold);
    println!(
        "  compaction_interval = {}s",
        config.compaction_interval_secs
    );

    start_volume_server(
        bind_addr,
        config.volume_id,
        config.data_dir,
        config.compaction_threshold,
        config.compaction_interval_secs,
    )
    .await?;

    Ok(())
}
