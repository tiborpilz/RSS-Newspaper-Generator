use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use serde::Deserialize;

#[cfg(feature = "ssr")]
use axum::{
    http::{StatusCode, Response, header},
    response,
    body::Body,
    extract::Query,
};

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

#[derive(Deserialize)]
pub struct ArticlePdfQuery {
    url: String,
}

#[cfg(feature = "ssr")]
pub async fn get_article_pdf(query: Query<ArticlePdfQuery>) -> response::Response {
    let url = query.url.clone();
    use readability::extractor;
    use tokio::task::spawn_blocking;
    use pandoc::{Pandoc, InputKind, InputFormat, OutputFormat, OutputKind};

    logging::log!("Scraping article: {}", url);

    let err_response = |e: String| {
        logging::error!("{}", e);
        return Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header(header::CONTENT_TYPE, "text/plain")
            .body(Body::from(e))
            .unwrap()
    };

    let article_html = match spawn_blocking(move || {
        extractor::scrape(&url)
    }).await.unwrap() {
        Ok(article) => article.content,
        Err(e) => return err_response(format!("Error scraping article: {}", e)),
    };

    let mut pandoc = Pandoc::new();

    // Convert from HTML to PDF
    pandoc.set_input_format(InputFormat::Html, Vec::new());
    pandoc.set_output_format(OutputFormat::Pdf, Vec::new());

    pandoc.set_input(InputKind::Pipe(article_html));
    pandoc.set_output(OutputKind::Pipe);


    // Execute pandoc
    let result = pandoc.execute();
    let pdf_bytes: Vec<u8> = match result {
        Ok(pandoc::PandocOutput::ToBuffer(buffer)) => buffer.into(),
        Ok(pandoc::PandocOutput::ToBufferRaw(buffer)) => buffer,
        Ok(pandoc::PandocOutput::ToFile(_)) => return err_response("Pandoc output to file not supported".to_string()),
        Err(e) => return err_response(format!("Error converting article to PDF: {}", e)),
    };

    return Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/pdf")
        .header(header::CONTENT_DISPOSITION, format!("attachment; filename=\"article.pdf\""))
        .body(Body::from(pdf_bytes))
        .unwrap();
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

    view! {
        <Html lang="en" />
        <Meta name="description" content="Article content" />
        <main>
            <h1>Article</h1>
            <p><a download href=format!("/article/pdf?url={}", url())>Download as PDF</a></p>
            <Suspense fallback=|| { view! { <p>Loading...</p> } }>
                {move || article.get().map(|content| { view! {
                    <section inner_html=content></section>
                }})}
            </Suspense>
        </main>
    }
}
