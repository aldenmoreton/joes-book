use askama::Template;
use axum::{http::StatusCode, response::IntoResponse, Extension};

use crate::{
    auth::{AuthSession, BackendPgDB},
    objects::{
        book::{BookRole, BookSubscription},
        chapter::{get_chapters, Chapter},
    },
};

#[derive(Template)]
#[template(path = "pages/book.html")]
struct BookPage {
    book_subscription: BookSubscription,
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
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    BookPage {
        book_subscription,
        username,
        chapters,
    }
    .into_response()
}
