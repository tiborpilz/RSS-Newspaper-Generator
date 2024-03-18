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
fn FeedItem(feed: Feed) -> impl IntoView {
    let delete_feed = use_context::<Action<DeleteFeed, Result<(), ServerFnError>>>().expect("No delete feed action");

    let on_click = move |_| {
        delete_feed.dispatch(DeleteFeed { id: feed.id });
    };

    view! {
        <li>
            {feed.url}
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
                    <FeedItem feed=feed />
                }
            />
        </ol>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    let add_feed = create_server_action::<AddFeed>();
    let delete_feed = create_server_action::<DeleteFeed>();

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
        add_feed.dispatch(AddFeed { url: input_element.value() });
        input_element.set_value("");
    };

    view! {
        <h1>RSS Newspaper Generator</h1>
        <button on:click=on_click>Add Feed</button>
        <input type="text" node_ref=input_element />
        <Show when=feeds.loading()>
            <p>Loading...</p>
        </Show>
        <Show when=move || feeds.get().is_some()>
            <FeedList feeds=feeds.get().unwrap() />
        </Show>
    }
}
