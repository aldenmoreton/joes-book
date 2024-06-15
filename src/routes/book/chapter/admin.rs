use std::num::ParseFloatError;

use askama::Template;
use askama_axum::IntoResponse;
use axum::{extract::Query, http::StatusCode, Extension, Json};

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

pub async fn create_page(
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
    UserInput,
}

#[derive(Template)]
#[template(path = "components/add_event.html", whitespace = "suppress")]
struct AddEvent {
    ty: AddEventType,
}

pub async fn add_event(Query(ty): Query<AddEventType>) -> impl IntoResponse {
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

pub async fn create(Json(EventSubmissions { vals }): Json<EventSubmissions>) {
    let vals = validate(vals);
}
