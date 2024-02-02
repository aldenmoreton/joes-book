use axum::{extract::Path, response::IntoResponse, routing::get, Router};

pub fn router() -> Router {
	Router::new()
		.route("/:chapter_id", get(chapter))
}

async fn chapter(_: Path<(i64, i64)>) -> impl IntoResponse {
	"Chapter page"
}
