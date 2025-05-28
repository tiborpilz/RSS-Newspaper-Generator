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

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/rss-newspaper-generator.css"/>

        <Router >
            <Routes fallback=|| {
                let mut outside_errors = Errors::default();
                outside_errors.insert_with_default_key(AppError::NotFound);
                view! {
                    <ErrorTemplate outside_errors />
                }
                .into_view()
            }>
                <Route path=path!("") view=FeedListView />
                <Route path=path!("/feeds") view=FeedListView />
                <Route path=path!("/feeds/:id") view=FeedDetailView />
                <Route path=path!("/article") view=ArticleView />
            </Routes>
        </Router>
    }
}

