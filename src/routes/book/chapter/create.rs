use axum::{
    body::Body,
    extract::{Path, Query},
    http::{Response, StatusCode},
    response::IntoResponse,
    Json,
};
use axum_ctx::RespErr;
use itertools::Itertools;

use crate::{
    auth::AuthSession,
    db::{event::EventContent, spread::Spread, team::Team, user_input::UserInput},
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

#[derive(askama::Template, serde::Deserialize)]
#[template(path = "components/team_select.html", whitespace = "suppress")]
pub struct TeamSelect {
    location: String,
    #[serde(flatten)]
    team: Team,
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
    let mut spread_group = Vec::new();
    let events = events
        .into_iter()
        .filter_map(|curr_event| match curr_event {
            EventSubmissionType::Spread {
                home_id,
                away_id,
                home_spread,
            } => {
                let home_id = match home_id.parse() {
                    Ok(id) => id,
                    Err(_) => return Some(Err(RespErr::new(StatusCode::BAD_REQUEST))),
                };
                let away_id = match away_id.parse() {
                    Ok(id) => id,
                    Err(_) => return Some(Err(RespErr::new(StatusCode::BAD_REQUEST))),
                };
                let home_spread = match home_spread.parse() {
                    Ok(a) if a % 0.5 == 0.0 => a,
                    _ => {
                        return Some(Err(RespErr::new(StatusCode::BAD_REQUEST)
                            .user_msg("Could not parse amount")))
                    }
                };

                spread_group.push(Spread {
                    home_id,
                    away_id,
                    home_spread,
                    notes: None,
                });
                None
            }
            EventSubmissionType::UserInput {
                title,
                description,
                points,
            } => {
                let description = (!description.is_empty()).then_some(description);
                let points = match points.parse() {
                    Ok(p) => p,
                    Err(_) => return Some(Err(RespErr::new(StatusCode::BAD_REQUEST))),
                };
                Some(Ok(EventContent::UserInput(UserInput {
                    title,
                    description,
                    points,
                })))
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
    let _events = validate_events(chapter_submission.events)?;

    let pool = auth_session.backend.0;

    let record = sqlx::query!(
        "INSERT INTO chapters (title, book_id, is_open)
        VALUES ($1, $2, false)
        RETURNING id
        ",
        chapter_submission.chapter_name,
        book_id
    )
    .fetch_one(&pool)
    .await
    .map_err(AppError::from)?;

    let new_chapter_uri = format!("/book/{book_id}/chapter/{}/", record.id);

    Ok([("HX-Redirect", new_chapter_uri)].into_response())
}
