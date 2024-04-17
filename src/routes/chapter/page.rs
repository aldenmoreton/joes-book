use askama::Template;
use axum::{extract::Path, http::StatusCode};

use crate::{
    auth::{AuthSession, BackendPgDB},
    objects::{
        chapter::{get_chapter, Chapter},
        event::{get_events, Event},
    },
};

#[derive(Template)]
#[template(path = "pages/chapter.html")]
pub struct ChapterPage {
    username: String,
    meta: Chapter,
    events: Vec<Event>,
}

pub async fn handler(
    auth_session: AuthSession,
    Path((_book_id, chapter_id)): Path<(i32, i32)>,
) -> Result<ChapterPage, StatusCode> {
    let user = auth_session.user.ok_or(StatusCode::UNAUTHORIZED)?;
    let BackendPgDB(pool) = auth_session.backend;

    let meta = get_chapter(chapter_id, &pool).await?;
    let events = get_events(chapter_id, &pool).await?;

    Ok(ChapterPage {
        username: user.username,
        meta,
        events,
    })
}
