use crate::app::{HeadlineSetter, HeadlineGetter};
use serde::{Deserialize, Serialize};
use leptos::*;
use leptos_router::*;
use url::Url;
use rss::Channel;
use std::error::Error;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Feed {
    pub id: i64,
    pub url: String,
    pub title: String,
    pub description: String,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct FeedDetails {
    pub feed: Feed,
    pub channel: Channel,
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

async fn fetch_and_parse_rss(url: String) -> Result<Channel, Box<dyn Error>> {
    let content = reqwest::get(url).await?.text().await?;
    let channel = content.parse::<Channel>()?;
    return Ok(channel);
}

#[server]
async fn get_channel(id: i64) -> Result<Channel, ServerFnError> {
    let feed = match get_feed(id).await {
        Ok(feed) => feed,
        Err(err) => return Err(ServerFnError::new(format!("Error fetching feed: {}", err))),
    };

    let channel = match fetch_and_parse_rss(feed.url).await {
        Ok(channel) => channel,
        Err(err) => return Err(ServerFnError::new(format!("Error fetching RSS feed: {}", err))),
    };

    return Ok(channel);
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

    let channel = match fetch_and_parse_rss(url.clone()).await {
        Ok(channel) => channel,
        Err(err) => return Err(ServerFnError::new(format!("Error fetching RSS feed: {}", err))),
    };

    let pool = connect_db().await;
    let _ = sqlx::query("INSERT INTO feeds (url, title, description) VALUES (?, ?, ?)")
        .bind(url)
        .bind(channel.title)
        .bind(channel.description)
        .execute(&pool)
        .await;

    return Ok(());
}

#[server]
pub async fn update_feed_info(id: i64) -> Result<(), ServerFnError> {
    use crate::db::connect_db;

    let pool = connect_db().await;

    let feed = sqlx::query_as::<_, Feed>("SELECT * FROM feeds where id = ?")
        .bind(id)
        .fetch_one(&pool)
        .await?;

    let channel = match fetch_and_parse_rss(feed.url).await {
        Ok(channel) => channel,
        Err(err) => return Err(ServerFnError::new(format!("Error fetching RSS feed: {}", err))),
    };

    let _ = sqlx::query("UPDATE feeds SET title = ?, description = ? WHERE id = ?")
        .bind(channel.title)
        .bind(channel.description)
        .bind(id)
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
        <li class="flex items-center my-2">
            <a class="flex-1" href=format!("/feeds/{}", feed.id)>{feed.title}</a>
            <button class="p-2 ml-2 rounded bg-slate-100" on:click=on_click>Delete</button>
        </li>
    }
}

#[component]
fn FeedList(feeds: Vec<Feed>) -> impl IntoView {
    view! {
        <ul>
            <For
                each=move || feeds.clone()
                key=|feed| feed.id
                children=|feed| view! {
                    <FeedListItem feed=feed />
                }
            />
        </ul>
    }
}

#[component]
pub fn FeedListView() -> impl IntoView {
    let add_feed = create_server_action::<AddFeed>();
    let delete_feed = create_server_action::<DeleteFeed>();

    let (error_message, set_error_message) = create_signal(String::new());

    // Provide delete action to children
    provide_context(delete_feed);

    // Use Headline context
    let set_headline = use_context::<HeadlineSetter>().unwrap().0;
    set_headline("Feeds".to_string());
    let headline = use_context::<HeadlineGetter>().unwrap().0;

    // Resource that fetches feeds from the server when either the
    // add or delete feed actions are dispatched
    let feeds = create_resource(
        move || (
            add_feed.version().get(),
            delete_feed.version().get()
        ),
        |_| async move {
            get_feeds().await.unwrap_or_default()
        }
    );

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
        <div class="max-w-[700px]">
            <div class="flex gap-2">
                <input class="p-2 rounded border flex-1" type="text" node_ref=input_element placeholder="https://example.com" />
                <button class="p-2 rounded bg-slate-100" on:click=on_click>Add Feed</button>
            </div>
            <Show when=move || !error_message.get().is_empty()>
                <p>{error_message.get()}</p>
            </Show>
            <Suspense fallback=|| view! { <p>Loading...</p> }>
                {move || feeds.get().map(|feeds| view! {
                    <FeedList feeds />
                })}
            </Suspense>
        </div>
    }
}

#[derive(Clone, Params, PartialEq)]
pub struct FeedParams {
    id: i64,
}

#[component]
pub fn FeedDetailView() -> impl IntoView {
    let params = use_params::<FeedParams>();

    let channel = create_resource(
        move || params.get().unwrap().id,
        |id| async move {
            get_channel(id).await.unwrap()
        }
    );

    params.with(|p| {
        update_feed_info(p.clone().unwrap().id);
    });

    watch(
        move || channel.get(),
        move |channel, _, _| {
            if let Some(channel) = channel {
                let set_headline = use_context::<HeadlineSetter>().unwrap().0;
                set_headline(channel.title.clone());
            }
        },
        true
    );

    view! {
        <Suspense fallback=|| view! { <p>Loading...</p> }>
            {move || channel.get().map(|c| view! {
                <p>{c.description}</p>
                <For
                    each=move || c.items.clone()
                    key=|item| item.link.clone()
                    children=|item| view! {
                        <section class="p-4 my-4 border shadow-lg">
                            <p class="text-lg">
                                <a href=item.link.clone()>{item.title.clone()}</a>
                            </p>
                            <p>
                                {item.pub_date.clone()}
                            </p>
                            <p>
                                <a href=format!("/article?url={}", item.link.clone().unwrap())>Read</a>
                            </p>
                            <p>
                                <a download href=format!("/article/pdf?url={}", item.link.unwrap())>Download as PDF</a>
                            </p>
                            <div inner_html=item.description.clone()></div>
                        </section>
                    }
                />

            })}
        </Suspense>
    }
}
