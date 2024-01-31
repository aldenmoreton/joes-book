use askama::Template;
use axum::response::IntoResponse;
use axum_login::AuthSession;
use serde::{Serialize, Deserialize};
use sqlx::PgPool;

use crate::auth::{BackendPgDB, BackendUser};

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

#[derive(Template)]
#[template(path = "book_list.html")]
pub struct BookList {
	books: Vec<BookSubscription>
}

pub async fn book_list(auth_session: AuthSession<BackendPgDB>) -> impl IntoResponse {
	let BackendPgDB(pool) = auth_session.backend;
	let Some(user) = auth_session.user else {
		return "No books".into_response()
	};

	let Ok(books) = get_books(user, &pool).await else {
		return "Error Getting Books".into_response()
	};

	BookList {books}.into_response()
}
