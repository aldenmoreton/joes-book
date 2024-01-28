use askama::Template;
use axum::{Router, routing::get, response::IntoResponse};

pub fn router() -> Router {
	Router::new()
		.route("/", get(home))
}

#[derive(Template)]
#[template(path = "home.html")]
struct Home;

async fn home() -> impl IntoResponse {
	Home{}
}
