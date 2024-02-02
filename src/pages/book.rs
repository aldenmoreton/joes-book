use askama::Template;
use axum::{extract::Path, response::IntoResponse, routing::{Router, get}};

use crate::{auth::AuthSession, objects::chapter::Chapter};

pub fn router() -> Router {
	Router::new()
		.route("/:book_id/", get(book))
}

#[derive(Template)]
#[template(path = "pages/book.html")]
struct Book {
	username: String,
	chapters: Vec<Chapter>
}

async fn book(auth_session: AuthSession, Path(_book_id): Path<i64>) -> impl IntoResponse {

	let username = auth_session.user.unwrap().username;
	let chapters = vec![
		Chapter { name: "Week 1".to_string(), chapter_id: 1234 }
	];

	Book { username, chapters }
}
