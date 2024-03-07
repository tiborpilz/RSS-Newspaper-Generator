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
fn FeedList(feeds: ReadSignal<Vec<String>>) -> impl IntoView {
    view! {
        <ol>
            <For
                each=feeds
                key=|feed| feed.clone()
                children=|feed| view! {
                    <li>{feed}</li>
                }
            />
        </ol>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    let add_feed = create_server_action::<AddFeed>();

    let feeds = create_resource(
        move || (add_feed.version().get()),
        move |_| get_feeds()
    );

    let (new_feed, set_new_feed) = create_signal::<String>("".to_string());

    let on_click = {
        move |_| {
            spawn_local(async move {
                add_feed.dispatch(AddFeed { url: new_feed.get() });
            });
        }
    };

    view! {
        <h1>RSS Newspaper Generator</h1>
        <button on:click=on_click>Add Feed</button>
        <input type="text" on:input=move |ev| set_new_feed(event_target_value(&ev)) />
        <Suspense fallback=move || view! { <p>Loading...</p> }>
            {move || {
                let existing_feeds = {
                    move || {
                        feeds.get()
                            .map(move |feeds| match feeds {
                                Err(e) => {
                                    view! { <p>{format!("Error: {}", e)}</p> }.into_view()
                                }
                                Ok(feeds) => {
                                    if feeds.is_empty() {
                                        view! { <p>No feeds yet</p> }.into_view()
                                    } else {
                                        feeds
                                            .into_iter()
                                            .map(move |feed| {
                                                view! {
                                                    <li>{feed.url}</li>
                                                }
                                            })
                                            .collect_view()
                                        }
                                    }
                                })
                                .unwrap_or_default()
                            }
                    };

                    view! {
                        <ul>
                            {existing_feeds}
                        </ul>
                    }
                }
            }
        </Suspense>
    }
}
