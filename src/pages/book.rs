use askama::Template;
use axum::{extract::Path, response::IntoResponse, routing::{Router, get}};

use crate::{auth::{AuthSession, BackendPgDB}, objects::chapter::{get_chapters, Chapter}};

pub fn router() -> Router {
	Router::new()
		.route("/:book_id/", get(book))
}

#[derive(Template)]
#[template(path = "pages/book.html")]
struct BookPage {
	username: String,
	chapters: Vec<Chapter>
}

async fn book(auth_session: AuthSession, Path(book_id): Path<i64>) -> impl IntoResponse {
	let user = auth_session.user.unwrap();
	let BackendPgDB(pool) = auth_session.backend;

	let username = user.username;
	let user_id = user.id;

	let chapters = match get_chapters(user_id, book_id, &pool).await {
		Ok(c) => c,
		Err(e) => return e.into_response()
	};

	BookPage { username, chapters }.into_response()
}
