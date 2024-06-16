use askama::Template;
use axum::Extension;
use axum_ctx::RespErr;

use crate::{
    auth::{AuthSession, BackendPgDB},
    db::{
        book::{BookRole, BookSubscription},
        chapter::Chapter,
        event::{get_events, Event},
    },
    AppError,
};

#[derive(Template)]
#[template(path = "pages/chapter.html")]
pub struct ChapterPage {
    username: String,
    meta: Chapter,
    events: Vec<Event>,
    is_admin: bool,
}

pub async fn handler(
    auth_session: AuthSession,
    Extension(book_subscription): Extension<BookSubscription>,
    Extension(meta): Extension<Chapter>,
) -> Result<ChapterPage, RespErr> {
    let user = auth_session.user.ok_or(AppError::BackendUser)?;
    let BackendPgDB(pool) = auth_session.backend;

    let events = get_events(meta.chapter_id, &pool)
        .await
        .map_err(AppError::from)?;

    Ok(ChapterPage {
        username: user.username,
        meta,
        events,
        is_admin: book_subscription.role == BookRole::Admin,
    })
}
