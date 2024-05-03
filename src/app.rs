use std::time::Duration;

use crate::error_template::{AppError, ErrorTemplate};
use crate::feeds::{FeedListView, FeedDetailView};
use crate::article::ArticleView;
use crate::navigation::Navigation;
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

#[derive(Copy, Clone)]
pub struct HeadlineSetter(pub WriteSignal<String>);

#[derive(Copy, Clone)]
pub struct HeadlineGetter(pub ReadSignal<String>);

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    let (headline, set_headline) = create_signal("RSS Newspaper Generator".to_string());

    provide_context(HeadlineSetter(set_headline));
    provide_context(HeadlineGetter(headline));

    view! {
        <Stylesheet id="leptos" href="/pkg/rss-newspaper-generator.css"/>

        // sets the document title
        <Title text=headline.get() />


        <Navigation headline />
        <Router fallback=|| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! {
                <ErrorTemplate outside_errors/>
            }
            .into_view()
        }>
            <main class="mt-32 p-10">
                <Routes>
                    <Route path="" view=FeedListView ssr=SsrMode::Async />
                    <Route path="/feeds/:id" view=FeedDetailView ssr=SsrMode::Async />
                    <Route path="/article" view=ArticleView ssr=SsrMode::Async />
                </Routes>
            </main>
        </Router>
    }
}

