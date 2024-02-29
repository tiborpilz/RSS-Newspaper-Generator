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

use reqwest;
use rss;

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
        .route("/headlines", get(get_headlines));

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

    // Create the table if doesnt exist
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
struct RootTemplate {}

async fn root() -> Html<String> {
    info!("Entering root handler");
    let result = RootTemplate { }.render().unwrap();
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

async fn get_feeds_from_db() -> Vec<Feed> {
    let pool = connect_db().await;
    let feeds = sqlx::query_as::<_, Feed>("SELECT * FROM feeds")
        .fetch_all(&pool)
        .await
        .unwrap();
    return feeds;
}

async fn post_feed(feed: Form<FeedParams>) -> (StatusCode, String) {
    info!("Entering feed handler");

    info!("URL: {}", feed.url);

    let pool = connect_db().await;

    let _ = sqlx::query("INSERT INTO feeds (url) VALUES (?)")
        .bind(feed.url.as_str())
        .execute(&pool)
        .await;

    let feeds = get_feeds_from_db().await;

    let result = FeedTemplate { feeds: &feeds }.render().unwrap();
    return (StatusCode::OK, result);
}

async fn get_feeds() -> Html<String> {
    let feeds = get_feeds_from_db().await;
    let result = FeedTemplate { feeds: &feeds }.render().unwrap();
    return Html(result);
}

async fn fetch_rss_feed(url: &str) -> Result<String, reqwest::Error> {
    let response = reqwest::get(url).await?;
    let body = response.text().await?;
    return Ok(body);
}

fn parse_rss_feed(xml: &str) -> Result<rss::Channel, rss::Error> {
    let channel = rss::Channel::read_from(xml.as_bytes())?;
    return Ok(channel);
}

async fn get_headlines() -> Html<String> {
    let feeds = get_feeds_from_db().await;
    let mut result = String::new();

    for feed in feeds {
        let mut headlines = String::new();
        match fetch_rss_feed(&feed.url).await {
            Ok(xml) => {
                match parse_rss_feed(&xml) {
                    Ok(channel) => {
                        for item in channel.items() {
                            if let Some(title) = item.title() {
                                headlines.push_str(&format!("<li>{}</li>", title));
                            }
                        }
                    }
                    Err(e) => {
                        info!("Error parsing feed: {}", e);
                    }
                }
            }
            Err(e) => {
                info!("Error fetching feed: {}", e);
            }
        }
        result.push_str(&format!("<h2>{}</h2>", feed.url));
        result.push_str(&format!("<ul>{}</ul>", headlines));
    }

    return Html(result);
}
