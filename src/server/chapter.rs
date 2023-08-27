use leptos::*;
use cfg_if::cfg_if;

use crate::objects::Event;

cfg_if! {
	if #[cfg(feature = "ssr")] {
		use crate::{
			server::{pool, get_book},
			objects::BookRole
		};
	}
}

#[server(AddChapter, "/secure")]
pub async fn add_chapter(cx: Scope, book_id: i64, title: String, closing_time: String, events: Vec<Event>) -> Result<i64, ServerFnError> {
	let book_sub = get_book(cx, book_id).await?;

	match book_sub.role {
 		BookRole::Owner => (),
		_ => return Err(ServerFnError::Request("You can't make chapters on someone else's book".into()))
	}
	let closing_time = chrono::DateTime::parse_from_rfc3339(&closing_time)
		.map_err(|e| ServerFnError::Args(format!("Could not parse closing time: {e}")))?;
	let closing_time: chrono::DateTime<chrono::Utc> = closing_time.into();
	if closing_time <= chrono::Utc::now() {
		return Err(ServerFnError::Args("Your closing time can't be in the past".into()))
	}

	let pool = pool(cx)?;
	let chapter_id = sqlx::query!(
		r#"	INSERT INTO chapters (title, book_id, is_open, closing_time)
			VALUES ($1, $2, $3, $4)
			RETURNING id"#,
			title,
			book_id,
			true,
			closing_time
	)
		.fetch_one(&pool)
		.await?.id;

	for event in events{
		let contents = serde_json::to_string(&event)?;
		let event_type = match event {
			Event::SpreadGroup(_) => "SpreadGroup",
			Event::UserInput(_) => "UserInput"
		};
		sqlx::query(
			r#"	INSERT INTO events (book_id, chapter_id, is_open, event_type, contents, closing_time)
				VALUES ($1, $2, $3, $4, $5, $6)
			"#
		)
			.bind(book_id)
			.bind(chapter_id)
			.bind(true)
			.bind(event_type)
			.bind(contents)
			.bind(closing_time)
			.execute(&pool)
			.await?;
	}

	Ok(chapter_id)
}