use crate::error_template::{AppError, ErrorTemplate};
use crate::feeds::{FeedListView, FeedDetailView};
use crate::article::ArticleView;
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{
    path,
    SsrMode,
    components::*,
};

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <head>
            <Stylesheet id="leptos" href="/pkg/rss-newspaper-generator.css"/>
            <MetaTags />
        </head>

        <Router >
            <Routes fallback=|| {
                let mut outside_errors = Errors::default();
                outside_errors.insert_with_default_key(AppError::NotFound);
                view! {
                    <ErrorTemplate outside_errors />
                }
                .into_view()
            }>
                <Route path=path!("") view=FeedListView ssr=SsrMode::Async />
                <Route path=path!("/feeds") view=FeedListView ssr=SsrMode::Async />
                <Route path=path!("/feeds/:id") view=FeedDetailView ssr=SsrMode::PartiallyBlocked />
                <Route path=path!("/article") view=ArticleView ssr=SsrMode::PartiallyBlocked />
            </Routes>
        </Router>
    }
}

