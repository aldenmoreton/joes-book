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
#[template(path = "pages/chapter_admin.html")]
pub struct AuthChapterPage {
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
        match self {
            Error::Sqlx(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        }
        .into_response()
    }
}

pub async fn handler(
    auth_session: AuthSession,
    Path((_, chapter_id)): Path<(i32, i32)>,
) -> Result<AuthChapterPage, Error> {
    let user = auth_session.user.unwrap();
    let BackendPgDB(pool) = auth_session.backend;

    let meta = get_chapter(chapter_id, &pool).await?;
    let events = get_events(chapter_id, &pool).await?;

    Ok(AuthChapterPage {
        username: user.username,
        meta,
        events,
    })
}

pub async fn update(body: String) {
    println!("Update body: {body}")
}
