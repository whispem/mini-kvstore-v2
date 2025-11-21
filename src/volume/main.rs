use mini_kvstore_v2::volume::server::start_volume_server;
use mini_kvstore_v2::volume::config::VolumeConfig;
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let volume_id = std::env::var("VOLUME_ID").unwrap_or_else(|_| "vol-1".into());
    let data_dir = std::env::var("DATA_DIR").unwrap_or_else(|_| format!("volume_data_{}", volume_id));
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "9002".into())
        .parse()
        .unwrap_or(9002);

    let bind_addr = SocketAddr::from(([127, 0, 0, 1], port));

    let config = VolumeConfig::new(volume_id)
        .with_data_dir(data_dir)
        .with_bind_addr(bind_addr);

    start_volume_server(config).await?;

    Ok(())
}
