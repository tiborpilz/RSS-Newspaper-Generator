use crate::layout::Layout;
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

#[cfg(feature = "ssr")]
fn format_article(article: readability::extractor::Product) -> String {
    format!("<h1>{}</h1><p class=\"italic\">{}</p>{}", article.title, article.description, article.content)
}

#[server]
pub async fn scrape_article(url: String) -> Result<String, ServerFnError> {
    use readability::extractor;
    use tokio::task::spawn_blocking;

    match spawn_blocking(move || {
        return extractor::scrape(&url)
    }).await {
        Ok(article) => match article {
            Ok(article) => Ok(format_article(article)),
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
    use std::path::PathBuf;

    use readability::extractor;
    use tokio::task::spawn_blocking;
    use pandoc::{Pandoc, InputKind, InputFormat, OutputFormat, OutputKind, PandocOption};

    logging::log!("Scraping article: {}", url);

    let err_response = |e: String| {
        logging::error!("{}", e);
        return Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header(header::CONTENT_TYPE, "text/plain")
            .body(Body::from(e))
            .unwrap()
    };

    let article = match spawn_blocking(move || {
        extractor::scrape(&url)
    }).await.unwrap() {
        Ok(article) => article,
        Err(e) => return err_response(format!("Error scraping article: {}", e)),
    };

    // Add title to HTML as h1 tag
    let article_html = format_article(article);

    let mut pandoc = Pandoc::new();

    // Convert from HTML to PDF
    pandoc.set_input_format(InputFormat::Html, Vec::new());
    pandoc.set_output_format(OutputFormat::Pdf, Vec::new());

    pandoc.set_input(InputKind::Pipe(article_html));
    pandoc.set_output(OutputKind::Pipe);

    pandoc.add_option(PandocOption::PdfEngine(PathBuf::from("xelatex")));

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
        <Suspense fallback=|| view! {
            <Layout headline="Article".to_string()>
                <p>Loading...</p>
            </Layout>
        }>
            {move || article.get().map(|content| { view! {
                <Layout headline="Article".to_string()>
                    <p>
                        <a download href=format!("/article/pdf?url={}", url())>Download as PDF</a>
                    </p>
                    <section class="prose my-4 p-8 border shadow-lg max-w-[80ch]" inner_html=content></section>
                </Layout>
            }})}
        </Suspense>
    }
}
