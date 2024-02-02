use axum::{extract::Path, response::IntoResponse, routing::{Router, get}};

pub fn router() -> Router {
	Router::new()
		.route("/:id", get(book))
}

async fn book(Path(book_id): Path<i64>) -> impl IntoResponse {
	format!("I'm not real yet: {book_id}")
}
