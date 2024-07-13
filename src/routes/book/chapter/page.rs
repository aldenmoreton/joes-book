use std::collections::HashMap;

use crate::db::event::{get_picks, EventContent, UserPick};
use crate::db::team::get_chapter_teams;
use crate::{
    auth::{AuthSession, BackendPgDB},
    db::{
        book::{BookRole, BookSubscription},
        chapter::Chapter,
    },
    AppError,
};
use askama::Template;
use axum::{Extension, Json};
use axum_ctx::RespErr;

#[derive(Template)]
#[template(path = "pages/chapter.html", whitespace = "suppress")]
pub struct ChapterPage {
    username: String,
    chapter: Chapter,
    user_picks: Vec<UserPick>,
    is_admin: bool,
    relevent_teams: HashMap<i32, (String, Option<String>)>,
}

pub async fn handler(
    auth_session: AuthSession,
    Extension(book_subscription): Extension<BookSubscription>,
    Extension(chapter): Extension<Chapter>,
) -> Result<ChapterPage, RespErr> {
    let user = auth_session.user.ok_or(AppError::BackendUser)?;
    let BackendPgDB(pool) = auth_session.backend;

    let user_picks = get_picks(user.id, chapter.chapter_id, &pool);
    let relevent_teams = get_chapter_teams(chapter.chapter_id, &pool);

    let user_picks = user_picks.await.map_err(AppError::from)?;
    let relevent_teams = relevent_teams.await.map_err(AppError::from)?;

    Ok(ChapterPage {
        username: user.username,
        chapter,
        user_picks,
        is_admin: book_subscription.role == BookRole::Admin,
        relevent_teams,
    })
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(
    rename_all = "kebab-case",
    rename_all_fields = "kebab-case",
    tag = "type"
)]
pub enum SubmissionEvent {
    SpreadGroup {
        num_points: String,
        selection: String,
        event_id: String,
    },
    UserInput {
        user_input: String,
        event_id: String,
    },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PickSubmission {
    events: Vec<SubmissionEvent>,
}

pub async fn submit(Json(picks): Json<PickSubmission>) {
    println!("{picks:?}")
}
