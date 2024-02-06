use axum::http::StatusCode;
use serde::{Serialize, Deserialize};
use sqlx::PgPool;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BookRole {
	Owner,
	Admin,
	Participant,
	Unauthorized
}

impl From<String> for BookRole {
	fn from(value: String) -> Self {
		match value.as_str() {
			"owner" => Self::Owner,
			"admin" => Self::Admin,
			"participant" => Self::Participant,
			_ => Self::Unauthorized
		}
	}
}

impl From<BookRole> for String {
	fn from(val: BookRole) -> Self {
		match val {
			BookRole::Owner => "owner",
			BookRole::Admin => "admin",
			BookRole::Participant => "participant",
			BookRole::Unauthorized => "unauthorized"
		}.to_string()
	}
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BookSubscription {
	#[sqlx(rename = "id")]
	pub book_id: i64,
	pub user_id: i64,
	pub name: String,
	#[sqlx(try_from = "String")]
	pub role: BookRole
}

pub async fn get_books(user_id: i64, pool: &PgPool) -> Result<Vec<BookSubscription>, StatusCode> {
    let result = sqlx::query_as::<_, BookSubscription>(
		r#"	SELECT b.id, b.name, s.role, s.user_id
			FROM books AS b
			INNER JOIN subscriptions AS s ON s.book_id=b.id
			WHERE s.user_id = $1
		"#
	)
        .bind(user_id)
        .fetch_all(pool)
        .await
		.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

	Ok(result)
}

pub async fn get_book(user_id: i64, book_id: i64, pool: &PgPool) -> Result<BookSubscription, StatusCode> {
	sqlx::query_as::<_, BookSubscription>(
		r#"	SELECT b.id, b.name, s.role, s.user_id
			FROM books AS b
			INNER JOIN subscriptions AS s ON s.book_id=b.id
			WHERE s.user_id = $1 AND b.id = $2
		"#
	)
        .bind(user_id)
		.bind(book_id)
        .fetch_one(pool)
        .await
		.map_err(|_| StatusCode::NOT_FOUND)
}
