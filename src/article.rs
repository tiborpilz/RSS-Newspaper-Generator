use crate::layout::Layout;
use crate::breadcrumbs::{BreadCrumbs, BreadCrumbItem};
use crate::feeds::get_feed;
use leptos::prelude::*;
use leptos_router::{
    params::Params,
    hooks::use_query_map,
};
use serde::Deserialize;
use leptos::logging;

#[cfg(feature = "ssr")]
use axum::{
    http::{StatusCode, Response, header},
    response,
    body::Body,
    extract::Query,
};

#[cfg(feature = "ssr")]
use genpdf::{elements, fonts, style, Document, Element as _};
#[cfg(feature = "ssr")]
use html_parser::{Dom, Node};

#[cfg(feature = "ssr")]
fn format_article(article: readability::extractor::Product) -> String {
    format!("<h1>{}</h1><p class=\"italic\">{}</p>{}", article.title, article.description, article.content)
}

#[server]
pub async fn scrape_article(url: String) -> Result<String, ServerFnError> {
    use readability::extractor;
    use tokio::task::spawn_blocking;

    match spawn_blocking(move || {
        extractor::scrape(&url)
    }).await {
        Ok(article) => match article {
            Ok(article) => Ok(format_article(article)),
            Err(err) => Err(ServerFnError::new(format!("Error scraping article: {}", err))),
        },
        Err(err) => Err(ServerFnError::new(format!("Error scraping article: {}", err))),
    }
}

#[cfg(feature = "ssr")]
#[derive(Deserialize)]
pub struct ArticlePdfQuery {
    url: String,
}

#[cfg(feature = "ssr")]
pub async fn get_article_pdf(query: Query<ArticlePdfQuery>) -> response::Response {
    let url = query.url.clone();

    use readability::extractor;
    use tokio::task::spawn_blocking;

    logging::log!("Scraping article: {}", url);

    let err_response = |e: String| {
        logging::error!("{}", e);
        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header(header::CONTENT_TYPE, "text/plain")
            .body(Body::from(e))
            .unwrap()
    };

    let article_product = match spawn_blocking(move || { 
        extractor::scrape(&url)
    }).await.unwrap() {
        Ok(product) => product,
        Err(e) => return err_response(format!("Error scraping article: {}", e)),
    };

    let pdf_title = article_product.title.clone(); 
    
    let article_html = format_article(article_product);

    let font_dir = std::env::current_dir().unwrap().join("fonts");
    let font_family = match fonts::from_files(font_dir.clone(), "LiberationSans", None) {
         Ok(family) => family,
         Err(e) => {
             let font_error_msg = format!("Failed to load LiberationSans font family. Ensure LiberationSans fonts are in a ./fonts/ directory. Error: {}. Falling back to Helvetica.", e);
             logging::warn!("{}", font_error_msg);
             match fonts::from_files(font_dir, "Helvetica", Some(fonts::Builtin::Helvetica)) {
                 Ok(fallback_family) => fallback_family,
                 Err(fallback_e) => {
                    return err_response(format!("Failed to load fallback font Helvetica: {}. Original error: {}", fallback_e, font_error_msg));
                 }
             }
         }
    };

    let mut doc = Document::new(font_family);
    doc.set_title(pdf_title.clone()); 
    let mut decorator = genpdf::SimplePageDecorator::new();
    decorator.set_margins(10);
    doc.set_page_decorator(decorator);

    let dom = match Dom::parse(&article_html) {
        Ok(dom) => dom,
        Err(e) => return err_response(format!("Error parsing HTML: {}", e)),
    };

    fn process_node(doc_element: &mut elements::LinearLayout, node: &Node, default_style: style::Style) {
        match node {
            Node::Text(text) => {
                doc_element.push(elements::Paragraph::new(text.trim().to_string()).styled(default_style));
            }
            Node::Element(element) => {
                let mut current_style = default_style;
                let tag_name = element.name.to_lowercase();

                if tag_name == "h1" {
                    current_style = style::Style::new().bold().with_font_size(24);
                } else if tag_name == "h2" {
                    current_style = style::Style::new().bold().with_font_size(18);
                } else if tag_name == "h3" {
                    current_style = style::Style::new().bold().with_font_size(16);
                } else if tag_name == "p" {
                    // Handled by default style for Paragraph elements, but can add specific styling here
                } else if tag_name == "strong" || tag_name == "b" {
                    current_style = default_style.bold();
                } else if tag_name == "em" || tag_name == "i" {
                    current_style = default_style.italic();
                }
                // TODO: Add more tag handling (lists, links, images etc.)

                for child in &element.children {
                    process_node(doc_element, child, current_style);
                }

                // Add a bit of space after block elements like p and headings
                if tag_name == "p" || tag_name.starts_with("h") {
                     doc_element.push(elements::Break::new(1));
                }
            }
            Node::Comment(_) => { /* Ignore comments */ }
        }
    }

    let mut root_layout = elements::LinearLayout::vertical();
    let default_style = style::Style::new(); // Base style

    for node in dom.children {
        process_node(&mut root_layout, &node, default_style);
    }
    doc.push(root_layout);

    let mut pdf_bytes: Vec<u8> = Vec::new();

    doc.render(&mut pdf_bytes).unwrap();
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/pdf")
        .header(header::CONTENT_DISPOSITION, format!("attachment; filename=\"{}.pdf\"", pdf_title.replace(" ", "_").to_lowercase()))
        .body(Body::from(pdf_bytes))
        .unwrap()
}

#[derive(Clone, Params, PartialEq)]
pub struct ArticleQuery {
    url: Option<String>,
    feed_id: Option<i64>,
}

#[component]
pub fn ArticleView() -> impl IntoView {
    let query = use_query_map();
    let url = move || query.read().get("url").unwrap();

    let feed = Resource::new_blocking(
        move || query.read().get("feed_id").unwrap_or_default().to_string(),
        |id| async move { get_feed(id.parse::<i64>().unwrap()).await.unwrap() },
    );

    let article = LocalResource::new(
        move || async move {
            scrape_article(url()).await.unwrap()
        }
    );

    view! {
        <Suspense fallback=|| view! {
            <Layout headline="Article".to_string()>
                <p>Loading...</p>
            </Layout>
        }>
            {move || feed.get().map(|feed| { view! {
                <Layout headline="Article".to_string()>
                    <BreadCrumbs items=vec![
                        BreadCrumbItem { text: "Home".to_string(), url: "/".to_string() },
                        BreadCrumbItem { text: feed.title.clone(), url: format!("/feeds/{}", feed.id) },
                        BreadCrumbItem { text: "Article".to_string(), url: url() },
                    ] />
                    <p>
                        <a download href=format!("/article/pdf?url={}", url())>Download as PDF</a>
                    </p>
                    <Suspense fallback=|| view! {
                        <section class="my-4 p-8 border shadow-lg max-w-[80ch]">
                            <div class="w-[60ch] h-12 rounded bg-slate-100 animate-pulse" />
                        </section>
                    }>
                        {move || article.get().map(|content| { view! {
                            <section class="prose my-4 p-8 border shadow-lg max-w-[80ch]" inner_html=content></section>
                        }})}
                    </Suspense>
                </Layout>
            }})}
        </Suspense>
    }
}
