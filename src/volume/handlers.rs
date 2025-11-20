//! HTTP handlers for volume blob operations

use crate::volume::storage::BlobStorage;
use axum::{
    body::Bytes,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Json, Router,
};
use serde::Serialize;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct AppState {
    pub storage: Arc<Mutex<BlobStorage>>,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    volume_id: String,
    keys: usize,
    segments: usize,
    total_mb: f64,
}

async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    let storage = state.storage.lock().unwrap();
    let stats = storage.stats();

    let response = HealthResponse {
        status: "healthy".to_string(),
        volume_id: storage.volume_id().to_string(),
        keys: stats.num_keys,
        segments: stats.num_segments,
        total_mb: stats.total_mb(),
    };

    (StatusCode::OK, Json(response))
}

async fn put_blob(State(state): State<AppState>, Path(key): Path<String>, body: Bytes) -> Response {
    let mut storage = state.storage.lock().unwrap();
    match storage.put(&key, &body) {
        Ok(meta) => (StatusCode::CREATED, Json(meta)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}

async fn get_blob(State(state): State<AppState>, Path(key): Path<String>) -> Response {
    let mut storage = state.storage.lock().unwrap();
    match storage.get(&key) {
        Ok(Some(blob)) => (StatusCode::OK, Json(blob)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Blob not found".to_string(),
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}

async fn delete_blob(State(state): State<AppState>, Path(key): Path<String>) -> Response {
    let mut storage = state.storage.lock().unwrap();
    match storage.delete(&key) {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}

async fn list_blobs(State(state): State<AppState>) -> impl IntoResponse {
    let storage = state.storage.lock().unwrap();
    let keys = storage.list_keys();
    (StatusCode::OK, Json(keys))
}

pub fn create_router(storage: Arc<Mutex<BlobStorage>>) -> Router {
    let state = AppState { storage };

    Router::new()
        .route("/", get(health_check))
        .route("/health", get(health_check))
        .route("/blobs", get(list_blobs))
        .route("/blobs/:key", post(put_blob))
        .route("/blobs/:key", get(get_blob))
        .route("/blobs/:key", delete(delete_blob))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode as HttpStatus};
    use axum::ServiceExt;
    use std::sync::{Arc, Mutex};

    fn setup_test_storage() -> Arc<Mutex<BlobStorage>> {
        Arc::new(Mutex::new(
            BlobStorage::new("test_volume", "test-vol".to_string()).unwrap(),
        ))
    }

    #[tokio::test]
    async fn test_health_endpoint() {
        let storage = setup_test_storage();
        let app = create_router(storage);

        let response = app.oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

        assert_eq!(response.status(), HttpStatus::OK);
    }

    #[tokio::test]
    async fn test_put_and_get_blob() {
        let storage = setup_test_storage();
        let app = create_router(storage);

        let put_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/blobs/test-key")
                    .body(Body::from("test data"))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(put_response.status(), HttpStatus::CREATED);

        let get_response = app
            .oneshot(
                Request::builder()
                    .uri("/blobs/test-key")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(get_response.status(), HttpStatus::OK);
    }
}
