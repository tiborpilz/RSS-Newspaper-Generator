use leptos::*;
use leptos_router::*;

#[server]
pub async fn scrape_article(url: String) -> Result<String, ServerFnError> {
    use readability::extractor;
    use tokio::task::spawn_blocking;

    match spawn_blocking(move || {
        return extractor::scrape(&url)
    }).await {
        Ok(article) => match article {
            Ok(article) => Ok(article.content),
            Err(err) => Err(ServerFnError::new(format!("Error scraping article: {}", err))),
        },
        Err(err) => Err(ServerFnError::new(format!("Error scraping article: {}", err))),
    }
}

#[derive(Clone, Params, PartialEq)]
pub struct ArticleQuery {
    url: String,
}

#[component]
pub fn ArticleView() -> impl IntoView {
    let query = use_query::<ArticleQuery>();

    let url = move || {
        query.with(|q| q.clone().unwrap().url)
    };

    let article = create_resource(
        move || url(),
        |url| async move {
            scrape_article(url).await.unwrap()
        }
    );

    create_effect(move |_| {
        article.refetch();
    });

    view! {
        <div>
            <h1>Article</h1>
            <Show when=article.loading()>
                <p>Loading...</p>
            </Show>
            <Show when=move || article.get().is_some()>
                <div class="content" inner_html=article.get().unwrap()></div>
            </Show>
        </div>
    }
}
