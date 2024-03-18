use crate::error_template::{AppError, ErrorTemplate};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};

#[server]
pub async fn add_feed(url: String) -> Result<(), ServerFnError> {
    use crate::db::connect_db;

    let pool = connect_db().await;
    let _ = sqlx::query("INSERT INTO feeds (url) VALUES (?)")
        .bind(url)
        .execute(&pool)
        .await;

    return Ok(());
}


#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Feed {
    pub id: i64,
    pub url: String,
}

#[server(GetFeeds, "/api")]
pub async fn get_feeds() -> Result<Vec<Feed>, ServerFnError> {
    use crate::db::connect_db;
    use sqlx::Row;

    let pool = connect_db().await;

    let feeds = sqlx::query_as::<_, Feed>("SELECT * FROM feeds")
        .fetch_all(&pool)
        .await?;

    return Ok(feeds);
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/rss-newspaper-generator.css"/>

        // sets the document title
        <Title text="RSS Newspaper Generator"/>

        <Router fallback=|| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! {
                <ErrorTemplate outside_errors/>
            }
            .into_view()
        }>
            <main>
                <Routes>
                    <Route path="" view=HomePage/>
                </Routes>
            </main>
        </Router>
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
                    <li>{feed.url}</li>
                }
            />
        </ol>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    let add_feed = create_server_action::<AddFeed>();

    // Resource that fetches feeds from the server
    let feeds = create_resource(
        move || (add_feed.version().get()),
        |_| async move {
            logging::log!("Fetching feeds");
            get_feeds().await.unwrap_or_default()
        }
    );

    // Refetch feeds when the component is mounted
    create_effect(move |_| {
        feeds.refetch();
    });

    // Signal that holds the value of the new feed input
    let (new_feed, set_new_feed) = create_signal::<String>("".to_string());

    // On click handler for the add feed button
    // Dispatches the add feed action and resets the input
    let on_click = move |_| {
        add_feed.dispatch(AddFeed { url: new_feed.get() });
        set_new_feed("".to_string());
    };

    view! {
        <h1>RSS Newspaper Generator</h1>
        <button on:click=on_click>Add Feed</button>
        <input type="text" on:input=move |ev| set_new_feed(event_target_value(&ev)) />
        <Show when=feeds.loading()>
            <p>Loading...</p>
        </Show>
        <Show when=move || feeds.get().is_some()>
            <FeedList feeds=feeds.get().unwrap() />
        </Show>
    }
}
