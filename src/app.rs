use crate::error_template::{AppError, ErrorTemplate};
use crate::feeds::{FeedListView, FeedDetailView};
use crate::article::ArticleView;
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/rss-newspaper-generator.css"/>

        <Router fallback=|| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! {
                <ErrorTemplate outside_errors/>
            }
            .into_view()
        }>
            <Routes>
                <Route path="" view=FeedListView ssr=SsrMode::Async />
                <Route path="/feeds/:id" view=FeedDetailView ssr=SsrMode::PartiallyBlocked />
                <Route path="/article" view=ArticleView ssr=SsrMode::Async />
            </Routes>
        </Router>
    }
}

