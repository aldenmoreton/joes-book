use std::collections::HashMap;

use askama::Template;
use axum::Extension;
use axum_ctx::RespErr;

use crate::{
    auth::{AuthSession, BackendPgDB},
    db::{
        chapter::Chapter,
        event::{get_events, Event, EventContent},
        team::get_chapter_teams,
    },
    AppError,
};

#[derive(Template)]
#[template(path = "pages/chapter_admin.html")]
pub struct ChapterAdminPage {
    username: String,
    chapter: Chapter,
    events: Vec<Event>,
    relevent_teams: HashMap<i32, (String, Option<String>)>,
}

pub async fn handler(
    auth_session: AuthSession,
    Extension(chapter): Extension<Chapter>,
) -> Result<ChapterAdminPage, RespErr> {
    let user = auth_session.user.ok_or(AppError::BackendUser)?;
    let BackendPgDB(pool) = auth_session.backend;

    let events = get_events(chapter.chapter_id, &pool)
        .await
        .map_err(AppError::from)?;

    let relevent_teams = get_chapter_teams(chapter.chapter_id, &pool)
        .await
        .map_err(AppError::from)?;

    Ok(ChapterAdminPage {
        username: user.username,
        chapter,
        events,
        relevent_teams,
    })
}

pub async fn update(body: String) {
    println!("Update body: {body}")
}
