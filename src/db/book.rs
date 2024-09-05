use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::AppError;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BookRole {
    Owner,
    Admin,
    Participant,
    Guest { chapter_ids: Vec<i32> },
    Unauthorized,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BookSubscription {
    pub id: i32,
    pub user_id: i32,
    pub name: String,
    #[sqlx(json)]
    pub role: BookRole,
}

pub async fn get_books(user_id: i32, pool: &PgPool) -> Result<Vec<BookSubscription>, AppError> {
    let result = sqlx::query_as::<_, BookSubscription>(
        r#"	SELECT b.id AS id, b.name, s.role, s.user_id
			FROM books AS b
			INNER JOIN subscriptions AS s ON s.book_id=b.id
			WHERE s.user_id = $1
		"#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(result)
}

pub async fn get_book(
    user_id: i32,
    book_id: i32,
    pool: &PgPool,
) -> Result<BookSubscription, sqlx::Error> {
    sqlx::query_as::<_, BookSubscription>(
        r#"
            SELECT b.id AS id, b.name, s.role, s.user_id
            FROM books AS b
            INNER JOIN subscriptions AS s ON s.book_id=b.id
            WHERE s.user_id = $1 AND b.id = $2
            "#,
    )
    .bind(user_id)
    .bind(book_id)
    .fetch_one(pool)
    .await
}

pub async fn get_book_users(book_id: i32, pool: &PgPool) -> Result<Vec<(i32, String)>, AppError> {
    Ok(sqlx::query!(
        "
            SELECT users.id, users.username
            FROM users
            JOIN subscriptions ON users.id = subscriptions.user_id
            WHERE subscriptions.book_id = $1
            ",
        book_id
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::from)?
    .into_iter()
    .map(|r| (r.id, r.username))
    .collect())
}
