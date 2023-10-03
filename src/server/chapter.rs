use leptos::*;
use cfg_if::cfg_if;

use crate::objects::{ Event, Chapter, EventContent, Pick };

cfg_if! {
	if #[cfg(feature = "ssr")] {
		use itertools::Itertools;
		use sqlx::Row;
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
			EventContent::SpreadGroup(_) => "SpreadGroup",
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
			ORDER BY created_at DESC
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
			ORDER BY event_type
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
pub async fn get_picks(cx: Scope, chapter_id: i64) -> Result<Vec<(String, Vec<(Event, Pick)>)>, ServerFnError> {
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

	let event_picks: Vec<(Event, Pick)> = events
			.into_iter()
			.zip(awaited_picks)
			.collect();

	let mut data_grouped = Vec::new();
	for (key, group) in &event_picks.into_iter().group_by(|elt| elt.0.event_type.clone()) {
		data_grouped.push((key, group.collect()));
	}

	Ok(data_grouped)
}


cfg_if! {
	if #[cfg(feature = "ssr")] {
		pub async fn create_pick(cx: Scope, pick: Pick) -> Result<(), ServerFnError> {
			let book_sub = get_book(cx, pick.book_id).await?;
			match book_sub.role {
				BookRole::Unauthorized =>
					return Err(ServerFnError::Request("You can't make picks for a book you are not a member of".into())),
				_ => ()
			}

			let chapter = get_chapter(cx, pick.chapter_id).await?;
			let closing_time = chrono::DateTime::parse_from_rfc3339(&chapter.closing_time).unwrap();
			if	closing_time < chrono::Utc::now() {
				return Err(ServerFnError::Request("You're too late to make picks".into()))
			}

			let pool = pool(cx)?;
			sqlx::query(
				r#" INSERT INTO picks(book_id, chapter_id, event_id, user_id, choice, wager)
					VALUES ($1, $2, $3, $4, $5, $6)
				"#
			)
				.bind(pick.book_id)
				.bind(pick.chapter_id)
				.bind(pick.event_id)
				.bind(book_sub.user_id)
				.bind(pick.choice)
				.bind(pick.wager)
				.execute(&pool)
				.await?;

			Ok(())
		}

		pub async fn update_pick(cx: Scope, pick: Pick) -> Result<(), ServerFnError> {
			let book_sub = get_book(cx, pick.book_id).await?;
			match book_sub.role {
				BookRole::Unauthorized =>
					return Err(ServerFnError::Request("You can't update picks for a book you are not a member of".into())),
				_ => ()
			}

			let chapter = get_chapter(cx, pick.chapter_id).await?;
			let closing_time = chrono::DateTime::parse_from_rfc3339(&chapter.closing_time).unwrap();
			if	closing_time < chrono::Utc::now() {
				return Err(ServerFnError::Request("You're too late to update picks".into()))
			}

			let pool = pool(cx)?;
			sqlx::query(
				r#" UPDATE picks
					SET choice = $1, wager = $2
					WHERE id = $3
				"#
			)
				.bind(pick.choice)
				.bind(pick.wager)
				.bind(pick.id)
				.execute(&pool)
				.await?;

			Ok(())
		}
	}
}

#[server(SavePicks, "/secure")]
pub async fn save_picks(cx: Scope, picks: Vec<Pick>) -> Result<(), ServerFnError> {
	for pick in picks {
		if pick.id.is_some() {
			update_pick(cx, pick).await?
		} else {
			create_pick(cx, pick).await?
		}
	}

	Ok(())
}

#[server(GetUserInputs, "/secure")]
pub async fn get_user_inputs(cx: Scope, event_id: i64) -> Result<Vec<String>, ServerFnError> {
	let pool = pool(cx)?;

	let result = sqlx::query(
		r#" SELECT DISTINCT choice
			FROM picks
			WHERE event_id = $1
		"#
	)
		.bind(event_id)
		.fetch_all(&pool)
		.await?;

	Ok(
		result
			.into_iter()
			.map(|row| row.get("choice"))
			.collect()
	)
}

#[server(SaveAnswers, "/secure")]
pub async fn save_answers(cx: Scope, picks: Vec<(i64, Vec<String>)>) -> Result<(), ServerFnError> {
	let pool = pool(cx)?;
	for (id, answers) in picks {
		let answers = answers.iter().map(|a| format!(r#"'{a}'"#)).collect::<Vec<String>>().join(",");
		let query = format!(r#"
			UPDATE picks
			SET correct = choice IN ({})
			WHERE event_id = $1
		"#, answers);

		sqlx::query(&query)
			.bind(id)
			.execute(&pool)
			.await?;
	}

	Ok(())
}

