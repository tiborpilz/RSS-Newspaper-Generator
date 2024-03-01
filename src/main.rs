use axum::{
    routing::{delete, get, post},
    Router,
};

use std::net::SocketAddr;

use tracing::info;
use tracing_subscriber;

use crate::db::connect_db;
use crate::routes::{delete_feed, get_feeds, get_headlines, post_feed, root};

mod db;
mod routes;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Initialize database
    let _ = connect_db().await;

    let app = Router::new()
        .route("/", get(root))
        .route("/feeds", get(get_feeds))
        .route("/feeds", post(post_feed))
        .route("/feeds/:id", delete(delete_feed))
        .route("/headlines", get(get_headlines));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    info!("Listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

