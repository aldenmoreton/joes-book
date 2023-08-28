use leptos::*;
use cfg_if::cfg_if;

use crate::objects::{ Event, Chapter, EventContent, Pick };

cfg_if! {
	if #[cfg(feature = "ssr")] {
		use crate::{
			server::{pool, get_book, auth},
			objects::{ BookRole }
		};
	}
}

#[server(AddChapter, "/secure")]
pub async fn add_chapter(cx: Scope, book_id: i64, title: String, closing_time: String, events: Vec<EventContent>) -> Result<i64, ServerFnError> {
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
			EventContent::SpreadGroup{..} => "SpreadGroup",
			EventContent::UserInput(_) => "UserInput"
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

#[server(GetChapters, "/secure")]
pub async fn get_chapters(cx: Scope, book_id: i64) -> Result<Vec<Chapter>, ServerFnError> {
	let book_subscription = get_book(cx, book_id).await?;
	match book_subscription.role {
		BookRole::Unauthorized => return Err(ServerFnError::Request("You aren't in this book".into())),
		_ => ()
	}

	let pool = pool(cx)?;

	let chapters = sqlx::query_as_unchecked!(
		Chapter,
		r#"	SELECT id AS chapter_id, book_id, title, is_open, TO_CHAR(closing_time, 'YYYY-MM-DD"T"HH24:MI:SS.MSZ') as closing_time
			FROM chapters
			WHERE book_id = $1
		"#,
		book_id
	)
		.fetch_all(&pool)
		.await.unwrap_or(Vec::new());


	Ok(chapters)
}

#[server(GetChapter, "/secure")]
pub async fn get_chapter(cx: Scope, chapter_id: i64) -> Result<Chapter, ServerFnError> {
	let pool = pool(cx)?;

	let chapter = sqlx::query_as_unchecked!(
		Chapter,
		r#"	SELECT id AS chapter_id, book_id, title, is_open, TO_CHAR(closing_time, 'YYYY-MM-DD"T"HH24:MI:SS.MSZ') AS closing_time
			FROM chapters
			WHERE id = $1
		"#,
		chapter_id
	)
		.fetch_one(&pool)
		.await
		.map_err(|err| ServerFnError::Request(format!("Could not find chapter: {err}")))?;

	let book_subscription = get_book(cx, chapter.book_id).await?;
	match book_subscription.role {
		BookRole::Unauthorized => return Err(ServerFnError::Request("You don't have access to this book's chapters".into())),
		_ => ()
	}

	Ok(chapter)
}

#[server(GetEvents, "/secure")]
pub async fn get_events(cx: Scope, chapter_id: i64) -> Result<Vec<Event>, ServerFnError> {
	get_chapter(cx, chapter_id)
		.await
		.map_err(|err| ServerFnErrorErr::Request(format!("Could not get chapter: {err}")))?;

	let pool = pool(cx)?;

	let events = sqlx::query_as::<_, Event>(
		r#"	SELECT id, book_id, chapter_id, contents, event_type, is_open, TO_CHAR(closing_time, 'YYYY-MM-DD"T"HH24:MI:SS.MSZ') AS closing_time
			FROM events
			WHERE chapter_id = $1
		"#
	)
		.bind(chapter_id)
		.fetch_all(&pool)
		.await?;

	Ok(events)
}

#[server(GetPick, "/secure")]
pub async fn get_pick(cx: Scope, event_id: i64) -> Result<Pick, ServerFnError> {
	let user = auth(cx)?.current_user.unwrap();
	let pool = pool(cx)?;

	let pick = sqlx::query_as::<_, Pick>(
		r#" SELECT *
				FROM picks
				WHERE event_id = $1 AND user_id = $2"#
	)
		.bind(event_id)
		.bind(user.id)
		.fetch_optional(&pool)
		.await
		.map_err(|err| ServerFnError::Args(format!("Could not build pick: {err}")))?;

	let pick = match pick {
		Some(pick) => pick,
		None => {
			let event = sqlx::query_as::<_, Event>(
				r#"	SELECT id, book_id, chapter_id, contents, event_type, is_open, TO_CHAR(closing_time, 'YYYY-MM-DD"T"HH24:MI:SS.MSZ') AS closing_time
					FROM events
					WHERE id = $1
				"#
			)
				.bind(event_id)
				.bind(user.id)
				.fetch_one(&pool)
				.await
				.map_err(|err| ServerFnError::Args(format!("Could not build event: {err}")))?;

			let book_subscription = get_book(cx, event.book_id).await?;
			match book_subscription.role {
				BookRole::Unauthorized => return Err(ServerFnError::Request("You don't have access to this book's chapters".into())),
				_ => ()
			}

			Pick {
				id: None,
				book_id: event.book_id,
				chapter_id: event.chapter_id,
				event_id: event_id,
				wager: None,
				choice: None,
				correct: None
			}
		}
	};
	Ok(pick)
}

#[server(GetPicks, "/secure")]
pub async fn get_picks(cx: Scope, chapter_id: i64) -> Result<Vec<(Event, Pick)>, ServerFnError> {
	let events = get_events(cx, chapter_id)
		.await
		.map_err(|err| ServerFnErrorErr::Request(format!("Could not get events: {err}")))?;

	let mut picks = Vec::new();
	for event in events.iter() {
		picks.push(get_pick(cx, event.id))
	}

	let mut awaited_picks = Vec::new();
	for pick in picks {
		awaited_picks.push(pick.await?)
	}

	Ok(
		events
			.into_iter()
			.zip(awaited_picks)
			.collect()
	)
}
