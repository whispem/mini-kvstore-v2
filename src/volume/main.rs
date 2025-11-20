//! Volume binary: starts a minimal Axum HTTP server for blob storage simulation.

use axum::{routing::get, Router};
use std::net::SocketAddr;

async fn root() -> &'static str {
    "mini-kvstore-v2 Volume: running!"
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(root));

    let addr = SocketAddr::from(([127, 0, 0, 1], 9002));
    println!("Volume listening on http://{}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("Volume server failed");
}
