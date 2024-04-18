use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BookRole {
    Owner,
    Admin,
    Participant,
    Unauthorized,
}

impl From<String> for BookRole {
    fn from(value: String) -> Self {
        match value.as_str() {
            "owner" => Self::Owner,
            "admin" => Self::Admin,
            "participant" => Self::Participant,
            _ => Self::Unauthorized,
        }
    }
}

impl From<BookRole> for String {
    fn from(val: BookRole) -> Self {
        match val {
            BookRole::Owner => "owner",
            BookRole::Admin => "admin",
            BookRole::Participant => "participant",
            BookRole::Unauthorized => "unauthorized",
        }
        .to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BookSubscription {
    #[sqlx(rename = "id")]
    pub book_id: i32,
    pub user_id: i32,
    pub name: String,
    #[sqlx(try_from = "String")]
    pub role: BookRole,
}

pub async fn get_books(user_id: i32, pool: &PgPool) -> Result<Vec<BookSubscription>, sqlx::Error> {
    let result = sqlx::query_as!(
        BookSubscription,
        r#"	SELECT b.id AS book_id, b.name, s.role, s.user_id
			FROM books AS b
			INNER JOIN subscriptions AS s ON s.book_id=b.id
			WHERE s.user_id = $1
		"#,
        user_id
    )
    .fetch_all(pool)
    .await?;

    Ok(result)
}

pub async fn get_book(
    user_id: i32,
    book_id: i32,
    pool: &PgPool,
) -> Result<BookSubscription, sqlx::Error> {
    sqlx::query_as!(
        BookSubscription,
        r#"	SELECT b.id AS book_id, b.name, s.role, s.user_id
			FROM books AS b
			INNER JOIN subscriptions AS s ON s.book_id=b.id
			WHERE s.user_id = $1 AND b.id = $2
		"#,
        user_id,
        book_id
    )
    .fetch_one(pool)
    .await
}
