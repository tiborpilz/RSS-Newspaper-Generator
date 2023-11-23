use axum::{
    routing::{
        get,
        post,
    },
    http::StatusCode,
    response::Html,
    Router,
    Form,
};

use std::net::SocketAddr;

use askama::Template;

use serde::Deserialize;

use tracing::info;
use tracing_subscriber;


#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/", get(root))
        .route("/feed", post(feed));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    info!("Listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}


#[derive(Template)]
#[template(path = "root.html")]

struct RootTemplate<'a> {
    name: &'a str,
}

async fn root() -> Html<String> {
    info!("Entering root handler");
    let result = RootTemplate { name: "World" }.render().unwrap();
    return Html(result);
}

#[derive(Deserialize)]
struct FeedParams {
    url: String,
}

async fn feed(feed: Form<FeedParams>) -> (StatusCode, &'static str) {
    info!("Entering feed handler");

    info!("URL: {}", feed.url);
    return (StatusCode::OK, "Submit");
}
