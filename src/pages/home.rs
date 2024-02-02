use askama::Template;
use axum::{response::IntoResponse, routing::get, Router};

use crate::{auth::AuthSession, objects::book::{get_books, BookSubscription}};

pub fn router() -> Router {
	Router::new()
		.route("/", get(home))
		// .route("/books", get(book_list))
}

#[derive(Template)]
#[template(path = "pages/home.html")]
struct Home {
	username: String,
	books: Vec<BookSubscription>
}

pub async fn home(session: AuthSession) -> impl IntoResponse {
	let user = session.user.unwrap();
	let username = user.username.clone();

	let crate::auth::BackendPgDB(pool) = session.backend;
	let books = get_books(user, &pool).await.unwrap();

	Home{ username, books }
}
