use askama::Template;
use axum::{response::IntoResponse, routing::get, Router};

use crate::components::book_list::book_list;

pub fn router() -> Router {
	Router::new()
		.route("/", get(home))
		.route("/books", get(book_list))
}

#[derive(Template)]
#[template(path = "home.html")]
struct Home;

pub async fn home() -> impl IntoResponse {
	Home{}
}
