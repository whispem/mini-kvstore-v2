//! Volume HTTP server implementation

use crate::volume::handlers::create_router;
use crate::volume::storage::BlobStorage;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::signal;

pub async fn start_volume_server(
    bind_addr: SocketAddr,
    volume_id: String,
    data_dir: String,
    compaction_threshold: usize,
    compaction_interval: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Initializing storage: {}", data_dir);
    let storage = BlobStorage::new(&data_dir, volume_id.clone())?;
    let storage = Arc::new(Mutex::new(storage));

    // Background compaction
    let bg_storage = storage.clone();
    let compaction_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(compaction_interval));
        loop {
            interval.tick().await;
            if let Ok(mut s) = bg_storage.lock() {
                let stats = s.stats();
                if stats.num_segments >= compaction_threshold {
                    println!(
                        "Auto-compaction triggered ({} segments >= {} threshold)",
                        stats.num_segments, compaction_threshold
                    );
                    let start = std::time::Instant::now();
                    match s.compact() {
                        Ok(()) => {
                            let elapsed = start.elapsed();
                            println!("✓ Compaction completed in {:.2}s", elapsed.as_secs_f64());
                        }
                        Err(e) => eprintln!("✗ Compaction error: {}", e),
                    }
                }
            }
        }
    });

    let app = create_router(storage.clone());

    println!("✓ Volume server ready");
    println!("  Listening: http://{}", bind_addr);
    println!("  Volume ID: {}", volume_id);
    println!("  Data dir: {}", data_dir);
    println!(
        "  Compaction: {} segments, every {}s",
        compaction_threshold, compaction_interval
    );
    println!("\n📡 Endpoints:");
    println!("  GET    /health");
    println!("  GET    /metrics");
    println!("  GET    /blobs");
    println!("  POST   /blobs/:key");
    println!("  GET    /blobs/:key");
    println!("  DELETE /blobs/:key");
    println!("\nPress Ctrl+C to shutdown gracefully\n");

    let listener = tokio::net::TcpListener::bind(bind_addr).await?;

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(storage))
        .await?;

    // Cancel background tasks
    compaction_task.abort();

    println!("Server stopped");
    Ok(())
}

async fn shutdown_signal(storage: Arc<Mutex<BlobStorage>>) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            println!("\n Received Ctrl+C, shutting down gracefully...");
        },
        _ = terminate => {
            println!("\n Received SIGTERM, shutting down gracefully...");
        },
    }

    // Save snapshot before shutdown
    if let Ok(s) = storage.lock() {
        println!("Saving index snapshot...");
        if let Err(e) = s.save_snapshot() {
            eprintln!("⚠ Failed to save snapshot: {}", e);
        } else {
            println!("✓ Snapshot saved");
        }
    }
}
