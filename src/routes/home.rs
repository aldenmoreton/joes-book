use askama::Template;
use axum::{response::IntoResponse, routing::get, Router};

use crate::{
    auth::AuthSession,
    objects::book::{get_books, BookSubscription},
};

pub fn router() -> Router {
    Router::new().route("/", get(handler))
    // .route("/books", get(book_list))
}

#[derive(Template)]
#[template(path = "pages/home.html")]
struct HomePage {
    admin: bool,
    username: String,
    books: Vec<BookSubscription>,
}

pub async fn handler(session: AuthSession) -> impl IntoResponse {
    let user = session.user.unwrap();
    let username = user.username;

    let crate::auth::BackendPgDB(pool) = session.backend;
    let books = get_books(user.id, &pool).await.unwrap();

    HomePage {
        admin: false,
        username,
        books,
    }
}
