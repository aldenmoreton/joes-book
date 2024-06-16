use std::collections::HashMap;

use axum::{extract::Path, http::StatusCode, response::Redirect, Form};
use axum_ctx::RespErr;
use itertools::Itertools;

use crate::{auth::AuthSession, AppError};

#[derive(serde::Deserialize)]
pub struct Params {
    #[serde(rename(deserialize = "chapter-name"))]
    chapter_name: String,
}

pub async fn handler(
    auth_session: AuthSession,
    Path(path): Path<HashMap<String, String>>,
    Form(Params { chapter_name }): Form<Params>,
) -> Result<Redirect, RespErr> {
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

    Ok(Redirect::to(&format!(
        "/book/{book_id}/chapter/{}/",
        record.id
    )))
}
