use askama::Template;
use askama_axum::IntoResponse;
use axum::{extract::Query, http::StatusCode, Extension};

use crate::{
    auth::{AuthSession, BackendPgDB},
    objects::{
        chapter::Chapter,
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
    Extension(meta): Extension<Chapter>,
) -> Result<AuthChapterPage, Error> {
    let user = auth_session.user.unwrap();
    let BackendPgDB(pool) = auth_session.backend;

    let events = get_events(meta.chapter_id, &pool).await?;

    Ok(AuthChapterPage {
        username: user.username,
        meta,
        events,
    })
}

pub async fn update(body: String) {
    println!("Update body: {body}")
}

#[derive(Template)]
#[template(path = "pages/chapter_create.html")]
struct CreateChapter {
    username: String,
    meta: Chapter,
}

pub async fn create(
    auth_session: AuthSession,
    Extension(meta): Extension<Chapter>,
) -> impl IntoResponse {
    let username = auth_session.user.unwrap().username;
    CreateChapter { username, meta }
}

#[derive(serde::Deserialize, Debug)]
#[serde(tag = "type", rename_all(deserialize = "kebab-case"))]
pub enum AddEventType {
    SpreadGroup,
}

pub async fn add_event(Query(ty): Query<AddEventType>) {
    println!("{ty:?}");
}
