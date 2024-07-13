use askama::Template;
use axum::Extension;
use axum_ctx::RespErr;

use crate::{
    auth::{AuthSession, BackendPgDB},
    db::{
        chapter::Chapter,
        event::{get_events, Event},
    },
    AppError,
};

#[derive(Template)]
#[template(path = "pages/chapter_admin.html")]
pub struct AuthChapterPage {
    username: String,
    chapter: Chapter,
    _events: Vec<Event>,
}

pub async fn handler(
    auth_session: AuthSession,
    Extension(chapter): Extension<Chapter>,
) -> Result<AuthChapterPage, RespErr> {
    let user = auth_session.user.ok_or(AppError::BackendUser)?;
    let BackendPgDB(pool) = auth_session.backend;

    let events = get_events(chapter.chapter_id, &pool)
        .await
        .map_err(AppError::from)?;

    Ok(AuthChapterPage {
        username: user.username,
        chapter,
        _events: events,
    })
}

pub async fn update(body: String) {
    println!("Update body: {body}")
}
