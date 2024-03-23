use crate::error_template::{AppError, ErrorTemplate};
use crate::feeds::{FeedListView, FeedDetailView};
use crate::article::ArticleView;
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

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
                    <Route path="" view=FeedListView/>
                    <Route path="/feeds/:id" view=FeedDetailView />
                    <Route path="/article" view=ArticleView ssr=SsrMode::Async />
                </Routes>
            </main>
        </Router>
    }
}

