use axum::{http::StatusCode, response::Redirect, Form};
use axum_ctx::RespErr;
use serde::Deserialize;

use crate::{auth::AuthSession, db::book::BookRole, AppError};

#[derive(Deserialize)]
pub struct Params {
    #[serde(rename(deserialize = "book-name"))]
    book_name: String,
}

pub async fn handler(
    auth_session: AuthSession,
    Form(Params { book_name }): Form<Params>,
) -> Result<Redirect, RespErr> {
    if book_name.len() > 30 {
        return Err(RespErr::new(StatusCode::BAD_REQUEST)
            .user_msg("Book name must be less than 30 characters"));
    }

    let user = auth_session.user.ok_or(AppError::BackendUser)?;
    let pool = auth_session.backend.0;

    let mut transaction = pool.begin().await.map_err(AppError::from)?;

    let record = sqlx::query!(
        "INSERT INTO books (name)
            VALUES ($1)
            ON CONFLICT DO NOTHING
            RETURNING id",
        book_name
    )
    .fetch_one(&mut *transaction)
    .await
    .map_err(AppError::from)?;

    sqlx::query!(
        "INSERT INTO subscriptions (user_id, book_id, role)
            VALUES ($1, $2, $3)",
        user.id,
        record.id,
        serde_json::to_value(BookRole::Admin)
            .map_err(|e| RespErr::new(StatusCode::INTERNAL_SERVER_ERROR).log_msg(e.to_string()))?
    )
    .execute(&mut *transaction)
    .await
    .map_err(AppError::from)?;

    transaction.commit().await.map_err(AppError::from)?;

    Ok(Redirect::to(&format!("/book/{}/", record.id)))
}
