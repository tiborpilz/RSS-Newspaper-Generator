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

use sqlx::sqlite::{
    SqliteConnectOptions,
    SqlitePool,
};

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Initialize database
    let _ = connect_db().await;

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

// Database
#[derive(sqlx::FromRow)]
struct Feed {
    id: i64,
    url: String,
}

async fn connect_db() -> SqlitePool {
    let db_url = "db.sqlite3";
    let conn_opts = SqliteConnectOptions::new()
        .filename(db_url)
        .create_if_missing(true);

    let pool = SqlitePool::connect_with(conn_opts).await.unwrap();

    // Create table if not exists
    let _ = sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS feeds (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            url TEXT NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await;
    return pool;
}

// Initial template
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

#[derive(Template)]
#[template(path = "feeds.html")]
struct FeedTemplate<'a> {
    feeds: &'a Vec<Feed>,
}

// Handle Post
#[derive(Deserialize)]
struct FeedParams {
    url: String,
}

async fn feed(feed: Form<FeedParams>) -> (StatusCode, String) {
    info!("Entering feed handler");

    info!("URL: {}", feed.url);

    let pool = connect_db().await;

    let _ = sqlx::query("INSERT INTO feeds (url) VALUES (?)")
        .bind(feed.url.as_str())
        .execute(&pool)
        .await;

    let feeds = sqlx::query_as::<_, Feed>("SELECT * FROM feeds")
        .fetch_all(&pool)
        .await
        .unwrap();

    let result = FeedTemplate { feeds: &feeds }.render().unwrap();
    return (StatusCode::OK, result);
}
