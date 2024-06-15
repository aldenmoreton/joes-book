use askama::Template;
use askama_axum::IntoResponse;
use axum::{http::StatusCode, Extension};

use crate::{
    auth::{AuthSession, BackendPgDB},
    objects::{
        book::{BookRole, BookSubscription},
        chapter::Chapter,
        event::{get_events, Event},
    },
};

#[derive(Template)]
#[template(path = "pages/chapter.html")]
pub struct ChapterPage {
    username: String,
    meta: Chapter,
    events: Vec<Event>,
    is_admin: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Database Error")]
    Sqlx(#[from] sqlx::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> askama_axum::Response {
        (StatusCode::INTERNAL_SERVER_ERROR, "Cannot get chapter page").into_response()
    }
}

pub async fn handler(
    auth_session: AuthSession,
    Extension(book_subscription): Extension<BookSubscription>,
    Extension(meta): Extension<Chapter>,
) -> Result<ChapterPage, Error> {
    let user = auth_session.user.unwrap();
    let BackendPgDB(pool) = auth_session.backend;

    let events = get_events(meta.chapter_id, &pool).await?;

    Ok(ChapterPage {
        username: user.username,
        meta,
        events,
        is_admin: book_subscription.role == BookRole::Admin,
    })
}
