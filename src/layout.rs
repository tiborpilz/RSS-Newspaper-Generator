use leptos::*;
use leptos_meta::*;

#[component]
pub fn Layout(headline: String, children: Children) -> impl IntoView {
    view! {
        <Title text=headline.clone() />
        <nav class="fixed w-full top-0 left-0 h-16 bg-white shadow-lg">
            <div class="ml-4 lg:ml-16 flex items-center justify-between h-full">
                <h1 class="text-2xl font-bold">{headline.clone()}</h1>
            </div>
        </nav>
        <main class="mt-20 px-4 lg:px-16">
            {children()}
        </main>
    }
}
