use askama::Template;
use axum::response::IntoResponse;

use crate::{auth::{AuthSession, BackendPgDB}, objects::book::{get_books, BookSubscription}};

#[derive(Template)]
#[template(path = "book_list.html")]
pub struct BookList {
	books: Vec<BookSubscription>
}

pub async fn book_list(auth_session: AuthSession) -> impl IntoResponse {
	let BackendPgDB(pool) = auth_session.backend;
	let Some(user) = auth_session.user else {
		return "No books".into_response()
	};

	let Ok(books) = get_books(user, &pool).await else {
		return "Error Getting Books".into_response()
	};

	BookList {books}.into_response()
}
