use askama::Template;
use axum::{response::IntoResponse, Extension};

use crate::{
    auth::{AuthSession, BackendPgDB},
    objects::{
        book::BookSubscription,
        chapter::{get_chapters, Chapter},
    },
};

#[derive(Template)]
#[template(path = "pages/book.html")]
struct BookPage {
    username: String,
    chapters: Vec<Chapter>,
}

pub async fn handler(
    auth_session: AuthSession,
    Extension(book_subscription): Extension<BookSubscription>,
) -> impl IntoResponse {
    let user = auth_session.user.unwrap();
    let BackendPgDB(pool) = auth_session.backend;

    let username = user.username;
    let user_id = user.id;

    let chapters = match get_chapters(user_id, book_subscription.book_id, &pool).await {
        Ok(c) => c,
        Err(e) => return e.into_response(),
    };

    BookPage { username, chapters }.into_response()
}
