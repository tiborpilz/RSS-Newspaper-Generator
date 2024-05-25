use chrono::{DateTime};
use leptos::*;

#[component]
pub fn FormattedDate(date_string: String) -> impl IntoView {
    // Example string Thu, 23 May 2024 14:20:31 +0000
    let datetime = DateTime::parse_from_str(&date_string, "%a, %d %b %Y %H:%M:%S %z")
        .unwrap_or_default();

    let formatted_date = datetime.format("%a, %d %b %Y").to_string();

    view! {
        <time
            datetime=date_string.clone()
            title=date_string.clone()
        >{formatted_date}</time>
    }
}
