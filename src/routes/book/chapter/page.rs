use askama::Template;
use askama_axum::IntoResponse;
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
    Path((_, chapter_id)): Path<(i32, i32)>,
) -> Result<ChapterPage, Error> {
    let user = auth_session.user.unwrap();
    let BackendPgDB(pool) = auth_session.backend;

    let meta = get_chapter(chapter_id, &pool).await?;
    let events = get_events(chapter_id, &pool).await?;

    Ok(ChapterPage {
        username: user.username,
        meta,
        events,
    })
}
