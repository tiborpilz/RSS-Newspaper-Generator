use leptos::*;

#[derive(Clone)]
pub struct BreadCrumbItem {
    pub text: String,
    pub url: String,
}

#[component]
pub fn BreadCrumbs(items: Vec<BreadCrumbItem>) -> impl IntoView {
    let count = items.len();

    view! {
        <nav class="flex gap-2">
            <For
                each=move || items.clone().into_iter().enumerate()
                key=|(_, item)| item.url.clone()
                children=move |(index, item)| view! {
                    <a href=item.url.clone()>{item.text.clone()}</a>
                    {if index.clone() < count.clone() - 1 {
                        view! { <span class="text-gray-500"> > </span> }.into_view()
                    } else {
                        view! { "" }.into_view()
                    }}
                }
            />
        </nav>
    }
}
