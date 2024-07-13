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
use axum::http::Response;
use axum::{response::IntoResponse, Extension, Json};
use axum_ctx::{RespErr, RespErrCtx, RespErrExt, StatusCode};
use itertools::Itertools;
use sqlx::PgPool;

#[derive(Template)]
#[template(path = "pages/chapter.html", whitespace = "suppress")]
pub struct ChapterPage {
    username: String,
    chapter: Chapter,
    user_picks: Vec<UserPick>,
    is_admin: bool,
    relevent_teams: HashMap<i32, (String, Option<String>)>,
}

fn empty_picks(len: usize) -> Vec<serde_json::Value> {
    vec![serde_json::Value::Null; len]
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
    println!("{user_picks:?}");
    Ok(ChapterPage {
        username: user.username,
        chapter,
        user_picks,
        is_admin: book_subscription.role == BookRole::Admin,
        relevent_teams,
    })
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PickSubmission {
    events: Vec<SubmissionEvent>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(
    rename_all = "kebab-case",
    rename_all_fields = "kebab-case",
    tag = "type"
)]
pub enum SubmissionEvent {
    SpreadGroup {
        event_id: String,
        spreads: Vec<SpreadGroupSpread>,
    },
    UserInput {
        user_input: String,
        event_id: String,
    },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct SpreadGroupSpread {
    num_points: String,
    selection: String,
}

pub async fn submit(
    auth_session: AuthSession,
    Extension(chapter): Extension<Chapter>,
    Json(picks): Json<PickSubmission>,
) -> Result<Response<axum::body::Body>, RespErr> {
    let pool = auth_session.backend.0;
    let user_id = auth_session.user.ok_or(AppError::BackendUser)?.id;

    let (event_ids, choices, wagers) = validate_picks(picks.events, &pool).await?;

    sqlx::query!(
        r#"
        INSERT INTO picks (book_id, chapter_id, user_id, event_id, choice, wager)
        SELECT $1 AS book_id, $2 AS chapter_id, $3 AS user_id, event_id, choice, wager
        FROM UNNEST($4::INT[], $5::JSONB[], $6::JSONB[]) AS a(event_id, choice, wager)
        ON CONFLICT (book_id, chapter_id, event_id, user_id)
        DO UPDATE SET
            choice = EXCLUDED.choice,
            wager = EXCLUDED.wager
        "#,
        chapter.book_id,
        chapter.chapter_id,
        user_id,
        &event_ids,
        &choices,
        &wagers
    )
    .execute(&pool)
    .await
    .map_err(AppError::from)?;

    Ok([("HX-Refresh", "true")].into_response())
}

async fn validate_picks(
    events: Vec<SubmissionEvent>,
    pool: &PgPool,
) -> Result<(Vec<i32>, Vec<serde_json::Value>, Vec<serde_json::Value>), RespErr> {
    let (events, choices, wagers): (Vec<_>, Vec<_>, Vec<_>) = events
        .into_iter()
        .map(|event| match event {
            SubmissionEvent::SpreadGroup { event_id, spreads } => {
                let (choices, wagers): (Vec<_>, Vec<_>) = spreads
                    .into_iter()
                    .map(|spread| (spread.selection, spread.num_points))
                    .unzip();
                (
                    event_id,
                    serde_json::to_value(&choices).unwrap(),
                    serde_json::to_value(&wagers).unwrap(),
                )
            }
            SubmissionEvent::UserInput {
                user_input,
                event_id,
            } => (
                event_id,
                serde_json::to_value(user_input).unwrap(),
                serde_json::to_value(1).unwrap(),
            ),
        })
        .multiunzip();

    let event_ids = events
        .iter()
        .map(|event| event.parse::<i32>())
        .collect::<Result<Vec<_>, _>>()
        .ctx(StatusCode::BAD_REQUEST)
        .user_msg("Could not parse event id")
        .log_msg("Could not parse event id")?;

    let unknown_events = sqlx::query!(
        r#"
        SELECT a
        FROM UNNEST($1::INT[]) AS a
        LEFT JOIN events on a = events.id
        WHERE events.id IS NULL
    "#,
        &event_ids
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::from)?;

    if !unknown_events.is_empty() {
        return Err(RespErr::new(StatusCode::BAD_REQUEST).user_msg("Event not found"));
    }

    Ok((event_ids, choices, wagers))
}
