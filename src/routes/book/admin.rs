use askama_axum::IntoResponse;

#[derive(askama::Template)]
#[template(path = "pages/book_admin.html")]
pub struct Page;

pub async fn handler() -> impl IntoResponse {
    Page
}
