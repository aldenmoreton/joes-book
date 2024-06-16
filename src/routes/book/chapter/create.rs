use std::{collections::HashMap, num::ParseFloatError};

use axum::{
    body::Body,
    extract::{Path, Query},
    http::{Response, StatusCode},
    response::{IntoResponse, Redirect},
    Form, Json,
};
use axum_ctx::RespErr;
use itertools::Itertools;

use crate::{auth::AuthSession, AppError};

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

#[derive(Debug, serde::Deserialize)]
#[serde(
    tag = "type",
    rename_all(deserialize = "kebab-case"),
    rename_all_fields = "kebab-case"
)]
pub enum EventSubmissionType {
    Spread {
        home_team: String,
        away_team: String,
        amount: String,
    },
    UserInput {
        title: String,
        description: String,
    },
}

#[derive(Debug, serde::Deserialize)]
pub struct EventSubmissions {
    #[serde(rename = "chapter-name")]
    chapter_name: String,
    vals: Vec<EventSubmissionType>,
}

enum ValidEvent {
    Spread {
        home_team: String,
        away_team: String,
        amount: f32,
    },
    UserInput {
        title: String,
        description: Option<String>,
    },
}

fn validate(events: Vec<EventSubmissionType>) -> Option<Vec<ValidEvent>> {
    let events: Result<Vec<ValidEvent>, ParseFloatError> = events
        .into_iter()
        .map(|curr_event| match curr_event {
            EventSubmissionType::Spread {
                home_team,
                away_team,
                amount,
            } => {
                let amount = amount.parse::<f32>()?;
                Ok(ValidEvent::Spread {
                    home_team,
                    away_team,
                    amount,
                })
            }
            EventSubmissionType::UserInput { title, description } => {
                let description = (!description.is_empty()).then_some(description);
                Ok(ValidEvent::UserInput { title, description })
            }
        })
        .collect();

    events.ok()
}

pub async fn post(
    auth_session: AuthSession,
    Path(path): Path<HashMap<String, String>>,
    Json(EventSubmissions { chapter_name, vals }): Json<EventSubmissions>,
) -> Result<Response<Body>, RespErr> {
    println!("Vals: {vals:?}");
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

    let pool = auth_session.backend.0;
    let Some(Ok(book_id)): Option<Result<i32, _>> = path.get("book_id").map(|id| id.parse()) else {
        return Err(AppError::Parse("book id".into()).into());
    };

    let record = sqlx::query!(
        "INSERT INTO chapters (title, book_id, is_open)
        VALUES ($1, $2, false)
        RETURNING id
        ",
        chapter_name,
        book_id
    )
    .fetch_one(&pool)
    .await
    .map_err(AppError::from)?;

    let new_chapter_uri = format!("/book/{book_id}/chapter/{}/", record.id);

    Ok([("HX-Redirect", new_chapter_uri)].into_response())
}
