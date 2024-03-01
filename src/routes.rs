use axum::{
    extract::Path,
    http::StatusCode,
    response::Html,
    Form,
};

use askama::Template;
use crate::db::{connect_db, Feed};
use serde::Deserialize;
use url::Url;
use reqwest;
use rss;
use tracing::info;

use std::vec::Vec;


// Initial template
#[derive(Template)]
#[template(path = "root.html")]
pub struct RootTemplate {}

#[derive(Template)]
#[template(path = "feeds.html")]
pub struct FeedTemplate<'a> {
    feeds: &'a Vec<Feed>,
}

#[derive(Template)]
#[template(path = "entry.html")]
pub struct EntryTemplate<'a> {
    entry: &'a Entry,
}

// Root
pub async fn root() -> Html<String> {
    info!("Entering root handler");
    let result = RootTemplate {}.render().unwrap();
    return Html(result);
}

// Feeds
// Handle Post
async fn is_valid_rss_feed(url: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let parsed_url = match Url::parse(url) {
        Ok(url) => url,
        Err(_) => return Ok(false),
    };

    if parsed_url.scheme() != "http" && parsed_url.scheme() != "https" {
        return Ok(false);
    }

    let response = reqwest::get(url).await?;

    if let Some(content_type) = response.headers().get(reqwest::header::CONTENT_TYPE) {
        let content_type = content_type.to_str()?.to_lowercase();
        if content_type.contains("application/rss+xml")
            || content_type.contains("application/xml")
            || content_type.contains("text/xml")
        {
            return Ok(true);
        }
    }

    return Ok(false);
}

#[derive(Deserialize)]
pub struct FeedParams {
    url: String,
}

pub async fn post_feed(feed: Form<FeedParams>) -> (StatusCode, String) {
    info!("Entering feed handler");

    info!("URL: {}", feed.url);

    if !is_valid_rss_feed(&feed.url).await.unwrap() {
        return (StatusCode::BAD_REQUEST, "Invalid RSS feed".to_string());
    }

    let pool = connect_db().await;

    let _ = sqlx::query("INSERT INTO feeds (url) VALUES (?)")
        .bind(feed.url.as_str())
        .execute(&pool)
        .await;

    let feeds = get_feeds_from_db().await;

    let result = FeedTemplate { feeds: &feeds }.render().unwrap();
    return (StatusCode::OK, result);
}

// Get handler
async fn get_feeds_from_db() -> Vec<Feed> {
    let pool = connect_db().await;
    let feeds = sqlx::query_as::<_, Feed>("SELECT * FROM feeds")
        .fetch_all(&pool)
        .await
        .unwrap();
    return feeds;
}

pub async fn get_feeds() -> Html<String> {
    let feeds = get_feeds_from_db().await;
    let result = FeedTemplate { feeds: &feeds }.render().unwrap();
    return Html(result);
}

// Delete handler
pub async fn delete_feed(Path(id): Path<i64>) -> (StatusCode, String) {
    info!("Entering delete feed handler");

    let pool = connect_db().await;

    let _ = sqlx::query("DELETE FROM feeds WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await;

    let feeds = get_feeds_from_db().await;

    let result = FeedTemplate { feeds: &feeds }.render().unwrap();
    return (StatusCode::OK, result);
}

// Entries
pub struct Entry {
    pub title: String,
    pub url: String,
    pub description: String,
    pub comments: String,
}

struct EntryList {
    feed_url: String,
    entries: Vec<Entry>,
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

async fn fetch_rss_entries() -> Vec<EntryList> {
    let feeds = get_feeds_from_db().await;
    let mut result = Vec::<EntryList>::new();

    for feed in feeds {
        let mut entries = Vec::<Entry>::new();
        match fetch_rss_feed(&feed.url).await {
            Ok(xml) => match parse_rss_feed(&xml) {
                Ok(channel) => {
                    for item in channel.items() {
                        if let Some(title) = item.title() {
                            entries.push(Entry {
                                title: title.to_string(),
                                url: item.link().unwrap_or("").to_string(),
                                description: item.description().unwrap_or("").to_string(),
                                comments: item.comments().unwrap_or("").to_string(),
                            });
                        }
                    }
                }
                Err(e) => {
                    info!("Error parsing feed: {}", e);
                }
            },
            Err(e) => {
                info!("Error fetching feed: {}", e);
            }
        }
        result.push(EntryList {
            feed_url: feed.url,
            entries,
        });
    }

    return result;
}

fn render_feed_html(feeds: Vec<EntryList>) -> String {
    let mut result = String::new();
    for feed in feeds {
        for entry in feed.entries {
            let entry_html = EntryTemplate { entry: &entry }.render().unwrap();
            result.push_str(&format!("<h2>{}</h2>", feed.feed_url));
            result.push_str(&entry_html);
        }
    }
    return result;
}

pub async fn get_headlines() -> Html<String> {
    let feeds = fetch_rss_entries().await;
    let result = render_feed_html(feeds);
    return Html(result);
}
