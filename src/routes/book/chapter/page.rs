use crate::db::event::{get_picks, EventContent, UserPick};
use crate::{
    auth::{AuthSession, BackendPgDB},
    db::{
        book::{BookRole, BookSubscription},
        chapter::Chapter,
        event::{get_events, Event},
    },
    AppError,
};
use askama::Template;
use axum::Extension;
use axum_ctx::RespErr;

#[derive(Template)]
#[template(path = "pages/chapter.html")]
pub struct ChapterPage {
    username: String,
    chapter: Chapter,
    user_picks: Vec<UserPick>,
    is_admin: bool,
}

pub async fn handler(
    auth_session: AuthSession,
    Extension(book_subscription): Extension<BookSubscription>,
    Extension(chapter): Extension<Chapter>,
) -> Result<ChapterPage, RespErr> {
    let user = auth_session.user.ok_or(AppError::BackendUser)?;
    let BackendPgDB(pool) = auth_session.backend;

    let user_picks = get_picks(user.id, chapter.chapter_id, &pool)
        .await
        .map_err(AppError::from)?;

    Ok(ChapterPage {
        username: user.username,
        chapter,
        user_picks,
        is_admin: book_subscription.role == BookRole::Admin,
    })
}
