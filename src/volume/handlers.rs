//! HTTP handlers for volume blob operations.

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

/// Shared application state.
#[derive(Clone)]
pub struct AppState {
    /// Thread-safe blob storage instance.
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
        Ok(Some(data)) => (StatusCode::OK, data).into_response(),
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

/// Creates the HTTP router with all blob endpoints.
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
    use std::sync::{Arc, Mutex};
    use tower::ServiceExt;

    fn setup_test_storage(path: &str) -> Arc<Mutex<BlobStorage>> {
        let _ = std::fs::remove_dir_all(path);
        std::fs::create_dir_all(path).unwrap();
        Arc::new(Mutex::new(
            BlobStorage::new(path, "test-vol".to_string()).unwrap(),
        ))
    }

    #[tokio::test]
    async fn test_health_endpoint() {
        let storage = setup_test_storage("tests_data/handler_health");
        let app = create_router(storage);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), HttpStatus::OK);

        let _ = std::fs::remove_dir_all("tests_data/handler_health");
    }

    #[tokio::test]
    async fn test_put_and_get_blob() {
        let storage = setup_test_storage("tests_data/handler_put_get");

        // PUT
        {
            let mut s = storage.lock().unwrap();
            s.put("test-key", b"test data").unwrap();
        }

        // Test PUT via HTTP
        let app = create_router(storage.clone());
        let put_response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/blobs/test-key-2")
                    .body(Body::from("test data"))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(put_response.status(), HttpStatus::CREATED);

        // GET
        let app = create_router(storage);
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

        let _ = std::fs::remove_dir_all("tests_data/handler_put_get");
    }

    #[tokio::test]
    async fn test_get_not_found() {
        let storage = setup_test_storage("tests_data/handler_not_found");
        let app = create_router(storage);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/blobs/nonexistent")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), HttpStatus::NOT_FOUND);

        let _ = std::fs::remove_dir_all("tests_data/handler_not_found");
    }

    #[tokio::test]
    async fn test_delete_blob() {
        let storage = setup_test_storage("tests_data/handler_delete");

        // PUT first
        {
            let mut s = storage.lock().unwrap();
            s.put("to-delete", b"data").unwrap();
        }

        // DELETE
        let app = create_router(storage.clone());
        let delete_response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri("/blobs/to-delete")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(delete_response.status(), HttpStatus::NO_CONTENT);

        // Verify deleted
        let app = create_router(storage);
        let get_response = app
            .oneshot(
                Request::builder()
                    .uri("/blobs/to-delete")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(get_response.status(), HttpStatus::NOT_FOUND);

        let _ = std::fs::remove_dir_all("tests_data/handler_delete");
    }
}
