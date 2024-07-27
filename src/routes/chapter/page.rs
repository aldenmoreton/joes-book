use crate::db::book::BookRole;
use crate::db::event::get_picks;
use crate::db::team::get_chapter_teams;

use crate::{
    auth::{AuthSession, BackendPgDB},
    db::{book::BookSubscription, chapter::Chapter},
    AppError,
};
use axum::http::Response;
use axum::{response::IntoResponse, Extension, Json};
use axum_ctx::{RespErr, RespErrCtx, RespErrExt, StatusCode};

pub async fn handler(
    auth_session: AuthSession,
    Extension(book_subscription): Extension<BookSubscription>,
    Extension(chapter): Extension<Chapter>,
) -> Result<maud::Markup, RespErr> {
    let user = auth_session.user.ok_or(AppError::BackendUser)?;
    let BackendPgDB(pool) = auth_session.backend;

    let user_picks = get_picks(user.id, chapter.chapter_id, &pool);
    let relevent_teams = get_chapter_teams(chapter.chapter_id, &pool);

    let user_picks = user_picks.await.map_err(AppError::from)?;
    let relevent_teams = relevent_teams.await.map_err(AppError::from)?;

    Ok(crate::templates::chapter_open::markup(
        &user.username,
        chapter,
        user_picks,
        book_subscription.role == BookRole::Admin,
        relevent_teams,
    ))
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
    pool: &sqlx::PgPool,
) -> Result<(Vec<i32>, Vec<serde_json::Value>, Vec<serde_json::Value>), RespErr> {
    let (events, choices, wagers) = events
        .into_iter()
        .map(|event| match event {
            SubmissionEvent::SpreadGroup { event_id, spreads } => {
                let (choices, wagers) =
                        spreads
                            .into_iter()
                            .map(|spread| {
                                Ok((
                                    serde_json::Value::String(spread.selection),
                                    serde_json::Value::Number(
                                        spread
                                            .num_points
                                            .parse::<i32>()
                                            .ctx(StatusCode::BAD_REQUEST)
                                            .user_msg("Could not parse Spread Group Points")?
                                            .into(),
                                    ),
                                ))
                            })
                            .try_fold(
                                (Vec::new(), Vec::new()),
                                |(mut choices, mut wagers),
                                 curr_item: Result<
                                    (serde_json::Value, serde_json::Value),
                                    RespErr,
                                >| {
                                    let (curr_choice, curr_wager) = curr_item?;
                                    choices.push(curr_choice);
                                    wagers.push(curr_wager);
                                    Ok::<_, RespErr>((choices, wagers))
                                },
                            )?;
                Ok((
                    event_id,
                    serde_json::Value::Array(choices),
                    serde_json::Value::Array(wagers),
                ))
            }
            SubmissionEvent::UserInput {
                user_input,
                event_id,
            } => Ok((
                event_id,
                serde_json::Value::String(user_input),
                serde_json::Value::Number(1.into()),
            )),
        })
        .try_fold(
            (Vec::new(), Vec::new(), Vec::new()),
            |(mut events, mut choices, mut wagers),
             curr_item: Result<
                (String, serde_json::Value, serde_json::Value),
                RespErr,
            >| {
                let (curr_event, curr_choices, curr_wager) = curr_item?;
                events.push(curr_event);
                choices.push(curr_choices);
                wagers.push(curr_wager);
                Ok::<_, RespErr>((events, choices, wagers))
            },
        )?;

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
