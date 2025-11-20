QFile Name to Write: src/coord/main.rs//! Coordinator binary: starts a basic Axum HTTP API for metadata.

use axum::{routing::get, Router};
use std::net::SocketAddr;

async fn root() -> &'static str {
    "mini-kvstore-v2 Coordinator: alive!"
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(root));

    let addr = SocketAddr::from(([127, 0, 0, 1], 9001));
    println!("Coordinator listening on http://{}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("Coordinator server failed");
}
