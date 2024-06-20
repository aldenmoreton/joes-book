use askama::Template;
use axum::Extension;
use axum_ctx::RespErr;

use crate::{
    auth::{AuthSession, BackendPgDB},
    db::{
        book::{BookRole, BookSubscription},
        chapter::{get_chapters, Chapter},
    },
    AppError,
};

#[derive(Template)]
#[template(path = "pages/book.html")]
pub struct BookPage {
    book_subscription: BookSubscription,
    username: String,
    chapters: Vec<Chapter>,
}

pub async fn handler(
    auth_session: AuthSession,
    Extension(book_subscription): Extension<BookSubscription>,
) -> Result<BookPage, RespErr> {
    let user = auth_session.user.ok_or(AppError::BackendUser)?;
    let BackendPgDB(pool) = auth_session.backend;

    let username = user.username;

    let chapters = get_chapters(book_subscription.book_id, &pool)
        .await
        .map_err(AppError::from)?;

    Ok(BookPage {
        book_subscription,
        username,
        chapters,
    })
}
