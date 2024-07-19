use axum::{
    body::Body,
    extract::{Path, Query},
    http::{Response, StatusCode},
    response::IntoResponse,
    Json,
};
use axum_ctx::{RespErr, RespErrCtx, RespErrExt};
use itertools::Itertools;

use crate::{
    auth::AuthSession,
    db::{
        event::{EventContent, EventType},
        spread::Spread,
        user_input::UserInput,
    },
    AppError,
};

pub async fn get(auth_session: AuthSession) -> Result<maud::Markup, RespErr> {
    let username = auth_session.user.ok_or(AppError::BackendUser)?.username;
    Ok(crate::templates::chapter_create::markup(&username))
}

#[derive(serde::Deserialize, Debug)]
#[serde(tag = "type", rename_all(deserialize = "kebab-case"))]
pub enum AddEventType {
    SpreadGroup,
    SpreadGroupAppend,
    UserInput,
}

pub async fn add_event(Query(ty): Query<AddEventType>) -> maud::Markup {
    crate::templates::add_event::markup(ty)
}

#[derive(serde::Deserialize)]
pub struct TeamParams {
    pub id: String,
    pub name: String,
    pub logo: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct TeamSelect {
    pub location: String,
    #[serde(flatten)]
    pub team: TeamParams,
}

pub async fn team_select(Json(team): Json<TeamSelect>) -> maud::Markup {
    crate::templates::team_select::markup(team)
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all(deserialize = "kebab-case"))]
pub struct SpreadSubmission {
    home_id: String,
    away_id: String,
    home_spread: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(
    tag = "type",
    rename_all(deserialize = "kebab-case"),
    rename_all_fields = "kebab-case"
)]
pub enum EventSubmissionType {
    SpreadGroup {
        spreads: Vec<SpreadSubmission>,
    },
    UserInput {
        title: String,
        description: String,
        points: String,
    },
}

#[derive(Debug, serde::Deserialize)]
pub struct EventSubmissions {
    #[serde(rename = "chapter-name")]
    chapter_name: String,
    events: Vec<EventSubmissionType>,
}

fn validate_name(chapter_name: &str) -> Result<(), RespErr> {
    if chapter_name.len() > 30 {
        return Err(
            RespErr::new(StatusCode::BAD_REQUEST).user_msg("Chapter Name too long (> 30 chars)")
        );
    }

    let name_invalid_chars = chapter_name
        .chars()
        .filter(|c| !c.is_alphabetic() && *c != ' ')
        .join(",");

    if !name_invalid_chars.is_empty() {
        return Err(RespErr::new(StatusCode::BAD_REQUEST).user_msg(format!(
            "Chaper name includes invalid characters: {name_invalid_chars}"
        )));
    }

    Ok(())
}

fn validate_events(events: Vec<EventSubmissionType>) -> Result<Vec<EventContent>, RespErr> {
    let events = events
        .into_iter()
        .map(|curr_event| match curr_event {
            EventSubmissionType::SpreadGroup { spreads } => {
                let spreads = spreads
                    .into_iter()
                    .map(
                        |SpreadSubmission {
                             home_id,
                             away_id,
                             home_spread,
                         }| {
                            let home_id = home_id
                                .parse()
                                .map_err(|_| RespErr::new(StatusCode::BAD_REQUEST))?;
                            let away_id = away_id
                                .parse()
                                .map_err(|_| RespErr::new(StatusCode::BAD_REQUEST))?;

                            let home_spread = match home_spread.parse() {
                                Ok(a) if a % 0.5 == 0.0 => a,
                                _ => {
                                    return Err(RespErr::new(StatusCode::BAD_REQUEST)
                                        .user_msg("Could not parse amount"))
                                }
                            };

                            Ok(Spread {
                                home_id,
                                away_id,
                                home_spread,
                                notes: None,
                                answer: None,
                            })
                        },
                    )
                    .collect::<Result<Vec<Spread>, RespErr>>()?;

                Ok(EventContent::SpreadGroup(spreads))
            }
            EventSubmissionType::UserInput {
                title,
                description,
                points,
            } => {
                let description = (!description.is_empty()).then_some(description);
                let points = points
                    .parse()
                    .map_err(|_| RespErr::new(StatusCode::BAD_REQUEST))?;
                Ok(EventContent::UserInput(UserInput {
                    title,
                    description,
                    points,
                    acceptable_answers: None,
                }))
            }
        })
        .collect::<Result<Vec<EventContent>, RespErr>>()?;

    Ok(events)
}

pub async fn post(
    auth_session: AuthSession,
    Path(book_id): Path<i32>,
    Json(chapter_submission): Json<EventSubmissions>,
) -> Result<Response<Body>, RespErr> {
    validate_name(&chapter_submission.chapter_name)?;
    let events = validate_events(chapter_submission.events)?;

    let pool = auth_session.backend.0;

    let mut transaction = pool.begin().await.map_err(AppError::from)?;

    let record = sqlx::query!(
        "INSERT INTO chapters (title, book_id, is_open)
        VALUES ($1, $2, false)
        RETURNING id
        ",
        chapter_submission.chapter_name,
        book_id
    )
    .fetch_one(&mut *transaction)
    .await
    .map_err(AppError::from)?;

    let (event_types, event_contents): (Vec<_>, Vec<_>) = events
        .into_iter()
        .map(|event| {
            let event_type = match event {
                EventContent::SpreadGroup(_) => EventType::SpreadGroup,
                EventContent::UserInput(_) => EventType::UserInput,
            };
            (event_type, serde_json::to_value(event))
        })
        .unzip();

    let event_contents = event_contents
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .ctx(StatusCode::BAD_REQUEST)
        .user_msg("Invalid Inputs")
        .log_msg("Failed to serialize user inputs to json string")?;

    sqlx::query!(
        r#"
        INSERT INTO events (book_id, chapter_id, event_type, contents)
        SELECT $1 AS book_id, $2 AS chapter_id, event_type AS "event_type: EventType", contents
        FROM UNNEST($3::event_types[], $4::jsonb[]) AS a(event_type, contents)
        "#,
        book_id,
        record.id,
        event_types as _,
        &event_contents
    )
    .execute(&mut *transaction)
    .await
    .map_err(AppError::from)?;

    transaction.commit().await.map_err(AppError::from)?;

    let new_chapter_uri = format!("/book/{book_id}/chapter/{}/", record.id);

    Ok([("HX-Redirect", new_chapter_uri)].into_response())
}
