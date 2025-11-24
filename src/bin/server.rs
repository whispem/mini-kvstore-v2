use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use axum::{
    extract::{Extension, Path},
    routing::{delete, get, post},
    Json, Router,
};
use mini_kvstore_v2::{KVStore, StoreStats};
use serde::{Deserialize, Serialize};
use tokio::task;

//
// --- Request / Response structs ---
//

#[derive(Deserialize)]
struct SetRequest {
    key: String,
    value: String,
}

#[derive(Serialize)]
struct GetResponse {
    key: String,
    value: Option<String>,
}

#[derive(Serialize)]
struct StatsResponse {
    num_keys: usize,
    num_segments: usize,
    total_bytes: u64,
    active_segment_id: usize,
    oldest_segment_id: usize,
}

//
// --- Handlers ---
//

async fn health() -> &'static str {
    "OK"
}

async fn set(
    Extension(store): Extension<Arc<Mutex<KVStore>>>,
    Json(payload): Json<SetRequest>,
) -> Result<&'static str, (axum::http::StatusCode, String)> {
    let mut store = store.lock().unwrap();
    store
        .set(&payload.key, payload.value.as_bytes())
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok("OK")
}

async fn get_value(
    Extension(store): Extension<Arc<Mutex<KVStore>>>,
    Path(key): Path<String>,
) -> Result<Json<GetResponse>, (axum::http::StatusCode, String)> {
    let store = store.lock().unwrap();
    let value = store
        .get(&key)
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(GetResponse {
        key,
        value: value.map(|v| String::from_utf8_lossy(&v).to_string()),
    }))
}

async fn delete_value(
    Extension(store): Extension<Arc<Mutex<KVStore>>>,
    Path(key): Path<String>,
) -> Result<&'static str, (axum::http::StatusCode, String)> {
    let mut store = store.lock().unwrap();
    store
        .delete(&key)
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok("OK")
}

async fn stats(
    Extension(store): Extension<Arc<Mutex<KVStore>>>,
) -> Result<Json<StatsResponse>, (axum::http::StatusCode, String)> {
    let store = store.lock().unwrap();
    let s: StoreStats = store.stats();

    Ok(Json(StatsResponse {
        num_keys: s.num_keys,
        num_segments: s.num_segments,
        total_bytes: s.total_bytes,
        active_segment_id: s.active_segment_id,
        oldest_segment_id: s.oldest_segment_id,
    }))
}

async fn compact(
    Extension(store): Extension<Arc<Mutex<KVStore>>>,
) -> Result<&'static str, (axum::http::StatusCode, String)> {
    let mut store = store.lock().unwrap();
    store
        .compact()
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok("OK")
}

//
// --- Server bootstrap ---
//

#[tokio::main]
async fn main() {
    let data_dir = "data";
    let store = Arc::new(Mutex::new(
        KVStore::open(data_dir).expect("Failed to open KVStore"),
    ));

    // Background compaction every 30s
    let bg_store = store.clone();
    task::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
            if let Ok(mut s) = bg_store.lock() {
                let _ = s.compact();
            }
        }
    });

    let app = Router::new()
        .route("/health", get(health))
        .route("/set", post(set))
        .route("/get/:key", get(get_value))
        .route("/delete/:key", delete(delete_value))
        .route("/stats", get(stats))
        .route("/compact", post(compact))
        .layer(Extension(store));

    let addr = SocketAddr::from(([0, 0, 0, 0], 9002));
    println!("🚀 Server running on http://localhost:9002");

    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}
