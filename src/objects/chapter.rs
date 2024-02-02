use axum::http::StatusCode;
use sqlx::PgPool;

use super::book::{get_book, BookRole};

pub struct Chapter {
	pub chapter_id: i64,
	pub book_id: i64,
	pub is_open: bool,
	pub title: String,
	pub closing_time: String
}

pub async fn get_chapters(user_id: i64, book_id: i64, pool: &PgPool) -> Result<Vec<Chapter>, StatusCode> {
	let book_subscription = get_book(user_id, book_id, pool).await?;
	if let BookRole::Unauthorized = book_subscription.role {
		return Err(StatusCode::UNAUTHORIZED)
	}

	let result = sqlx::query!(
		r#"	SELECT id AS chapter_id, book_id, is_open, title, TO_CHAR(closing_time, 'YYYY-MM-DD"T"HH24:MI:SS.MSZ') as closing_time
			FROM chapters
			WHERE book_id = $1
		"#,
		book_id
	)
		.fetch_all(pool)
		.await
		.map_err(|e| { println!("{e:?}"); StatusCode::INTERNAL_SERVER_ERROR })?;

	Ok(
		result
			.into_iter()
			.map(|record| Chapter {
				chapter_id: record.chapter_id,
				book_id: record.book_id,
				is_open: record.is_open,
				title: record.title,
				closing_time: record.closing_time.unwrap_or_default()
			})
			.collect::<Vec<_>>()
	)
}
