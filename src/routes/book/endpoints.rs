use axum::{
    response::{Html, IntoResponse},
    Form,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateParams {
    #[serde(rename(deserialize = "book-name"))]
    book_name: String,
}

pub async fn create(Form(CreateParams { book_name }): Form<CreateParams>) -> impl IntoResponse {
    println!("{book_name}");
    Html("<p>About to make a book</p>")
}
