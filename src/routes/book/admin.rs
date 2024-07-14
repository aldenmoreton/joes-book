use axum::response::IntoResponse;

pub async fn handler() -> impl IntoResponse {
    maud::html! {
        "Admin Page"
    }
}
