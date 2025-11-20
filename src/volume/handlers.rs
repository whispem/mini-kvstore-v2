//! HTTP handlers for volume blob operations

use crate::volume::storage::BlobStorage;
use axum::{
    body::Bytes,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Json, Router, ServiceExt,
};
use serde::Serialize;
use std::sync::{Arc, Mutex};


#[cfg(test)]
mod tests {
    use super::*;
    use crate::volume::storage::BlobStorage;
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

        let response = app
            .clone()
            .oneshot(
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
