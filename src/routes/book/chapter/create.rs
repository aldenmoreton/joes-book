use std::collections::HashMap;

use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Redirect},
    Form,
};
use itertools::Itertools;

use crate::auth::AuthSession;

#[derive(serde::Deserialize)]
pub struct Params {
    #[serde(rename(deserialize = "chapter-name"))]
    chapter_name: String,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Chapter Name is too Long")]
    ChapterLen,
    #[error("Chapter Name has invalid Characters: {0}")]
    ChapterCharacters(String),
    #[error("Database Error")]
    Sqlx(#[from] sqlx::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> askama_axum::Response {
        match self {
            Error::Sqlx(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            _ => (StatusCode::BAD_REQUEST, self.to_string()),
        }
        .into_response()
    }
}

pub async fn handler(
    auth_session: AuthSession,
    Path(path): Path<HashMap<String, String>>,
    Form(Params { chapter_name }): Form<Params>,
) -> Result<Redirect, Error> {
    if chapter_name.len() > 30 {
        return Err(Error::ChapterLen);
    }
    let name_invalid_chars = chapter_name
        .chars()
        .filter(|c| !c.is_alphabetic() && *c != ' ')
        .join(",");
    if !name_invalid_chars.is_empty() {
        return Err(Error::ChapterCharacters(name_invalid_chars));
    }

    let pool = auth_session.backend.0;
    let book_id: i32 = path.get("book_id").unwrap().parse().unwrap();

    let record = sqlx::query!(
        "INSERT INTO chapters (title, book_id, is_open)
        VALUES ($1, $2, false)
        RETURNING id
        ",
        chapter_name,
        book_id
    )
    .fetch_one(&pool)
    .await?;

    Ok(Redirect::to(&format!(
        "/book/{book_id}/chapter/{}",
        record.id
    )))
}
