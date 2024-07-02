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

#[derive(askama::Template)]
#[template(path = "pages/chapter_create.html")]
pub struct CreateChapter {
    username: String,
}

pub async fn get(auth_session: AuthSession) -> Result<CreateChapter, RespErr> {
    let username = auth_session.user.ok_or(AppError::BackendUser)?.username;
    Ok(CreateChapter { username })
}

#[derive(serde::Deserialize, Debug)]
#[serde(tag = "type", rename_all(deserialize = "kebab-case"))]
pub enum AddEventType {
    SpreadGroup,
    UserInput,
}

#[derive(askama::Template)]
#[template(path = "components/add_event.html", whitespace = "suppress")]
pub struct AddEvent {
    ty: AddEventType,
}

pub async fn add_event(Query(ty): Query<AddEventType>) -> AddEvent {
    AddEvent { ty }
}

#[derive(serde::Deserialize)]
pub struct TeamParams {
    pub id: String,
    pub name: String,
    pub logo: Option<String>,
}

#[derive(askama::Template, serde::Deserialize)]
#[template(path = "components/team_select.html", whitespace = "suppress")]
pub struct TeamSelect {
    location: String,
    #[serde(flatten)]
    team: TeamParams,
}

pub async fn team_select(Json(team): Json<TeamSelect>) -> TeamSelect {
    team
}

#[derive(Debug, serde::Deserialize)]
#[serde(
    tag = "type",
    rename_all(deserialize = "kebab-case"),
    rename_all_fields = "kebab-case"
)]
pub enum EventSubmissionType {
    Spread {
        home_id: String,
        away_id: String,
        home_spread: String,
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
            EventSubmissionType::Spread {
                home_id,
                away_id,
                home_spread,
            } => {
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

                Ok(EventContent::SpreadGroup(Spread {
                    home_id,
                    away_id,
                    home_spread,
                    notes: None,
                }))
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

    let (book_ids, chapter_ids, event_types, event_contents): (Vec<_>, Vec<_>, Vec<_>, Vec<_>) =
        events
            .into_iter()
            .map(|event| {
                let event_type = match event {
                    EventContent::SpreadGroup(_) => EventType::SpreadGroup,
                    EventContent::UserInput(_) => EventType::UserInput,
                };
                (book_id, record.id, event_type, serde_json::to_value(event))
            })
            .multiunzip();

    let event_contents = event_contents
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .ctx(StatusCode::BAD_REQUEST)
        .user_msg("Invalid Inputs")
        .log_msg("Failed to serialize user inputs to json string")?;

    sqlx::query!(
        r#"   INSERT INTO events (book_id, chapter_id, event_type, contents)
            SELECT book_id, chapter_id, event_type AS "event_type: EventType", contents
            FROM UNNEST($1::INT[], $2::INT[], $3::event_types[], $4::jsonb[]) AS a(book_id, chapter_id, event_type, contents)
        "#,
        &book_ids,
        &chapter_ids,
        event_types as _,
        &event_contents
    ).execute(&mut * transaction).await.map_err(AppError::from)?;

    transaction.commit().await.map_err(AppError::from)?;

    let new_chapter_uri = format!("/book/{book_id}/chapter/{}/", record.id);

    Ok([("HX-Redirect", new_chapter_uri)].into_response())
}
