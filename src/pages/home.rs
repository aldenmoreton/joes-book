use askama::Template;
use axum::{response::IntoResponse, routing::get, Router};

pub fn router() -> Router {
	Router::new()
		.route("/", get(home))
}

#[derive(Template)]
#[template(path = "home.html")]
struct Home;

pub async fn home() -> impl IntoResponse {
	Home{}
}
