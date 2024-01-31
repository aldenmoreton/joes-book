use serde::{Serialize, Deserialize};
use sqlx::PgPool;

use crate::auth::BackendUser;


#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl Into<String> for BookRole {
	fn into(self) -> String {
		match self {
			Self::Owner => "owner",
			Self::Admin => "admin",
			Self::Participant => "participant",
			Self::Unauthorized => "unauthorized"
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

pub async fn get_books(user: BackendUser, pool: &PgPool) -> Result<Vec<BookSubscription>, sqlx::Error> {
    let result = sqlx::query_as::<_, BookSubscription>(
		r#"	SELECT b.id, b.name, s.role, s.user_id
			FROM books AS b
			INNER JOIN subscriptions AS s ON s.book_id=b.id
			WHERE s.user_id = $1
		"#
	)
        .bind(user.id)
        .fetch_all(pool)
        .await?;

	Ok(result)
}

pub async fn get_book(user: BackendUser, book_id: i64, pool: &PgPool) -> Result<BookSubscription, sqlx::Error> {
	sqlx::query_as::<_, BookSubscription>(
		r#"	SELECT b.id, b.name, s.role, s.user_id
			FROM books AS b
			INNER JOIN subscriptions AS s ON s.book_id=b.id
			WHERE s.user_id = $1 AND b.id = $2
		"#
	)
        .bind(user.id)
		.bind(book_id)
        .fetch_one(pool)
        .await
}
