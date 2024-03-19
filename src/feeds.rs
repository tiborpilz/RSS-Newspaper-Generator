use serde::{Deserialize, Serialize};
use leptos::*;
use leptos_router::*;
use url::Url;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Feed {
    pub id: i64,
    pub url: String,
}

fn is_valid_url(url: String) -> bool {
    let parsed_url = match Url::parse(&url) {
        Ok(url) => url,
        Err(_) => return false,
    };

    match parsed_url.scheme() {
        "http" | "https" => true,
        _ => false,
    }
}

async fn is_valid_rss_feed(url: String) -> bool {
    if !is_valid_url(url.clone()) {
        return false;
    }

    let response = match reqwest::get(url).await {
        Ok(response) => response,
        Err(_) => return false,
    };

    if let Some(content_type) = response.headers().get(reqwest::header::CONTENT_TYPE) {
        let content_type = match content_type.to_str() {
            Ok(content_type) => content_type.to_lowercase(),
            Err(_) => return false,
        };

        if content_type.contains("application/rss+xml")
            || content_type.contains("application/xml")
            || content_type.contains("text/xml")
        {
            return true;
        }
        return false;
    }

    return false;
}

#[server]
pub async fn get_feed(id: i64) -> Result<Feed, ServerFnError> {
    use crate::db::connect_db;

    let pool = connect_db().await;

    let feed = sqlx::query_as::<_, Feed>("SELECT * FROM feeds WHERE id = ?")
        .bind(id)
        .fetch_one(&pool)
        .await?;

    return Ok(feed);
}

#[server]
pub async fn get_feeds() -> Result<Vec<Feed>, ServerFnError> {
    use crate::db::connect_db;
    use sqlx::Row;

    let pool = connect_db().await;

    let feeds = sqlx::query_as::<_, Feed>("SELECT * FROM feeds")
        .fetch_all(&pool)
        .await?;

    return Ok(feeds);
}

#[server]
pub async fn add_feed(url: String) -> Result<(), ServerFnError> {
    use crate::db::connect_db;

    // TODO: display error message to user
    if !is_valid_rss_feed(url.clone()).await {
        return Err(ServerFnError::new("Invalid RSS feed"));
    }

    let pool = connect_db().await;
    let _ = sqlx::query("INSERT INTO feeds (url) VALUES (?)")
        .bind(url)
        .execute(&pool)
        .await;

    return Ok(());
}

#[server]
pub async fn delete_feed(id: i64) -> Result<(), ServerFnError> {
    use crate::db::connect_db;

    let pool = connect_db().await;
    let _ = sqlx::query("DELETE FROM feeds WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await;

    return Ok(());
}

#[component]
fn FeedListItem(feed: Feed) -> impl IntoView {
    let delete_feed = use_context::<Action<DeleteFeed, Result<(), ServerFnError>>>().expect("No delete feed action");

    let on_click = move |_| {
        delete_feed.dispatch(DeleteFeed { id: feed.id });
    };

    view! {
        <li>
            <a href=format!("/feeds/{}", feed.id)>{feed.url}</a>
            <button on:click=on_click>Delete</button>
        </li>
    }
}

#[component]
fn FeedList(feeds: Vec<Feed>) -> impl IntoView {
    view! {
        <ol>
            <For
                each=move || feeds.clone()
                key=|feed| {
                    logging::log!("Key: {}", feed.url);
                    feed.id
                }
                children=|feed| view! {
                    <FeedListItem feed=feed />
                }
            />
        </ol>
    }
}

#[component]
pub fn FeedListView() -> impl IntoView {
    let add_feed = create_server_action::<AddFeed>();
    let delete_feed = create_server_action::<DeleteFeed>();

    let (error_message, set_error_message) = create_signal(String::new());

    // Provide delete action to children
    provide_context(delete_feed);

    // Resource that fetches feeds from the server when either the
    // add or delete feed actions are dispatched
    let feeds = create_resource(
        move || (
            add_feed.version().get(),
            delete_feed.version().get()
        ),
        |_| async move {
            logging::log!("Fetching feeds");
            get_feeds().await.unwrap_or_default()
        }
    );

    // Refetch feeds when the component is mounted
    create_effect(move |_| {
        feeds.refetch();
    });

    // Ref for the input element
    let input_element: NodeRef<html::Input> = create_node_ref();

    // On click handler for the add feed button
    // Dispatches the add feed action and resets the input
    let on_click = move |_| {
        let input_element = input_element().expect("<input> element should be mounted");
        let url = input_element.value();
        if is_valid_url(url.clone()) {
            set_error_message("".to_string());
            add_feed.dispatch(AddFeed { url });
            input_element.set_value("");
        } else {
            set_error_message("Invalid URL".to_string());
        }
    };

    view! {
        <h1>RSS Newspaper Generator</h1>
        <button on:click=on_click>Add Feed</button>
        <input type="text" node_ref=input_element />
        <Show when=move || !error_message.get().is_empty()>
            <p>{error_message.get()}</p>
        </Show>
        <Show when=feeds.loading()>
            <p>Loading...</p>
        </Show>
        <Show when=move || feeds.get().is_some()>
            <FeedList feeds=feeds.get().unwrap() />
        </Show>
    }
}

#[component]
pub fn FeedDetailView() -> impl IntoView {
    let params = use_params_map();
    let id = move || params.with(|params| params.get("id").cloned().unwrap_or_default().parse::<i64>().unwrap_or_default());

    let feed = create_resource(
        move || id(),
        |id| async move {
            get_feed(id).await.unwrap()
        }
    );

    view! {
        <Show when=feed.loading()>
            <p>Loading...</p>
        </Show>
        <Show when=move || feed.get().is_some()>
            <p>{feed.get().unwrap().url}</p>
        </Show>
    }
}
