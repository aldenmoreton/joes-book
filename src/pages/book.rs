use axum::{response::IntoResponse, routing::{Router, get}};

pub fn router() -> Router {
	Router::new()
		.route("/", get(book))
}

async fn book() -> impl IntoResponse {
	"I'm not real yet"
}
