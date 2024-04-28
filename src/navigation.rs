use crate::app::HeadlineGetter;
use leptos::*;

#[component]
pub fn Navigation(headline: ReadSignal<String>) -> impl IntoView {
    view! {
        <nav class="fixed w-full top-0 left-0 h-32 bg-white shadow-lg">
            <div class="container mx-auto flex items-center justify-between h-full">
                <h1 class="text-2xl font-bold">{headline}</h1>
            </div>
        </nav>
    }
}
