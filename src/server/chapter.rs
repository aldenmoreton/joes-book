use leptos::*;
use cfg_if::cfg_if;

use crate::objects::{ EventContent, Chapter, Event, Pick };

cfg_if! {
	if #[cfg(feature = "ssr")] {
		use itertools::Itertools;
		use std::collections::HashMap;
		use sqlx::Row;
		use crate::{
			server::{pool, get_book, auth},
			objects::{ BookRole, Team }
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

	let book_id = sqlx::query!(r#"
		SELECT book_id
		FROM chapters
		WHERE id = $1
	"#, chapter_id)
		.fetch_one(&pool)
		.await?.book_id;
	let book_subscription = get_book(cx, book_id).await?;
	match book_subscription.role {
		BookRole::Unauthorized => return Err(ServerFnError::Request("You aren't in this book".into())),
		_ => ()
	}

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

#[server(IsOpen, "/secure")]
pub async fn is_open(cx: Scope, chapter_id: i64) -> Result<bool, ServerFnError> {
	let pool = pool(cx)?;

	let book_id = sqlx::query!(r#"
		SELECT book_id
		FROM chapters
		WHERE id = $1
	"#, chapter_id)
		.fetch_one(&pool)
		.await?.book_id;
	let book_subscription = get_book(cx, book_id).await?;
	match book_subscription.role {
		BookRole::Unauthorized => return Err(ServerFnError::Request("You aren't in this book".into())),
		_ => ()
	}

	let status = sqlx::query!(r#"
		UPDATE chapters
		SET is_open = CASE
			WHEN is_open = true AND closing_time < NOW() THEN false
			ELSE is_open
		END
		WHERE id = $1
		RETURNING is_open
	"#, chapter_id)
		.fetch_one(&pool)
		.await?.is_open;

	Ok(status)
}

#[server(GetEvents, "/secure")]
pub async fn get_events(cx: Scope, chapter_id: i64) -> Result<Vec<Event>, ServerFnError> {
	let pool = pool(cx)?;

	let book_id = sqlx::query!(r#"
		SELECT book_id
		FROM chapters
		WHERE id = $1
	"#, chapter_id)
		.fetch_one(&pool)
		.await?.book_id;

	let book_subscription = get_book(cx, book_id).await?;
	match book_subscription.role {
		BookRole::Unauthorized => return Err(ServerFnError::Request("You aren't in this book".into())),
		_ => ()
	}

	get_chapter(cx, chapter_id)
		.await
		.map_err(|err| ServerFnErrorErr::Request(format!("Could not get chapter: {err}")))?;


	let events = sqlx::query_as::<_, Event>(
		r#"	SELECT id, book_id, chapter_id, contents, event_type, is_open, TO_CHAR(closing_time, 'YYYY-MM-DD"T"HH24:MI:SS.MSZ') AS closing_time
			FROM events
			WHERE chapter_id = $1
			ORDER BY event_type, id
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
		.map_err(|err| ServerFnError::Args(format!("Could not find pick: {err}")))?;

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

	let chapter_id = sqlx::query!(r#"
		SELECT chapter_id
		FROM events
		WHERE id = $1
	"#, event_id)
		.fetch_one(&pool)
		.await?.chapter_id;

	if is_open(cx, chapter_id).await? {
		return Err(ServerFnError::Request("The chapter isn't closed yet! You can't see everyone's picks!".into()))
	}

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

	let book_id = sqlx::query!(r#"
		SELECT book_id
		FROM picks
		WHERE event_id = $1
	"#, picks.get(0).ok_or(ServerFnError::Args("List of picks must not be empty".into()))?.0)
		.fetch_one(&pool)
		.await.map_err(|_| ServerFnError::ServerError("Could not get book id".into()))?.book_id;
	let book_sub = get_book(cx, book_id).await?;
	match book_sub.role {
		BookRole::Owner => (),
	   _ => return Err(ServerFnError::Request("You can't make answers for someone else's book".into()))
   }

	for (id, answers) in picks {
		let answers = answers.iter().map(|a| format!(r#"'{a}'"#)).collect::<Vec<String>>().join(",");
		let query = format!(r#"
			UPDATE picks
			SET correct = choice IN ({})
			WHERE event_id = $1 AND book_id = $2
		"#, answers);

		sqlx::query(&query)
			.bind(id)
			.bind(book_id)
			.execute(&pool)
			.await?;
	}

	Ok(())
}

#[server(GetChapterTable, "/secure")]
pub async fn get_chapter_table(cx: Scope, chapter_id: i64) -> Result<String, ServerFnError> {
	if is_open(cx, chapter_id).await? {
		return Err(ServerFnError::Request("The chapter isn't closed yet! You can't see everyone's picks!".into()))
	}

	let events = get_events(cx, chapter_id).await?;
	let mut teams: HashMap<i64, Team> = HashMap::new();

	for event in events.iter() {
		match &event.contents {
			EventContent::SpreadGroup(spreads) => {
				let (home, away) = super::get_spread_teams(cx, spreads.home_id, spreads.away_id).await?;
				teams.insert(spreads.home_id, home);
				teams.insert(spreads.away_id, away);
			},
			_ => ()
		}
	}

	let table_header = view!{cx,
		<tr>
			<th></th>
			{
				events
					.iter()
					.map(|event| {
						let description: String = match &event.contents {
							EventContent::SpreadGroup(spreads) =>
								format!(
									"{} at {}({:+})",
									teams.get(&spreads.away_id).unwrap().name,
									teams.get(&spreads.home_id).unwrap().name,
									spreads.home_spread
								),
							EventContent::UserInput(input) =>
								input.question.clone()
						};
						view!{cx,
							<th>
								<h1>{description}</h1>
							</th>
						}
					}
					)
					.collect_view(cx)
			}
		</tr>
	};

	let pool = pool(cx)?;
	let user_points: Vec<(_, _, _)> = sqlx::query!(r#"
		SELECT u.id AS id, u.username AS username, CAST(COALESCE(p.total, 0) AS INTEGER) AS week_total
		FROM (
			SELECT users.id, users.username
			FROM chapters
			INNER JOIN subscriptions ON subscriptions.book_id = chapters.book_id
			INNER JOIN users ON users.id = subscriptions.user_id
			WHERE chapters.id = $1
			GROUP BY users.id, users.username
		) AS u
		LEFT JOIN (
			SELECT user_id, SUM(picks.wager) AS total
			FROM picks
			WHERE picks.chapter_id = $1 AND picks.correct
			GROUP BY user_id
		) AS p
		ON u.id = p.user_id
		ORDER BY week_total DESC, username"#,
		chapter_id
	)
		.fetch_all(&pool)
		.await?
		.into_iter()
		.map(|row| {
			(row.id, row.username, row.week_total.unwrap_or(0))
		})
		.collect();

	let mut user_rows: Vec<View> = Vec::new();
	for (user_id, username, week_total) in user_points {
		let picks: Vec<_> = sqlx::query!(r#"
			SELECT picks.choice, picks.wager, picks.correct, events.event_type, picks.event_id
			FROM picks
			JOIN events ON picks.event_id = events.id
			WHERE picks.chapter_id = $1 AND picks.user_id = $2
			ORDER BY events.event_type, events.id"#,
			chapter_id, user_id
		)
			.fetch_all(&pool)
			.await?
			.into_iter()
			.map(|row|
				(row.choice, row.wager, row.correct, row.event_type, row.event_id)
			)
			.collect();

		let view = view!{cx,
			<tr>
				<td>
					{username}
					<br/>
					{week_total}
				</td>
				{
					picks
						.into_iter()
						.map(|(choice, wager, correct, ty, event_id)| {
							match ty.as_str() {
								"SpreadGroup" => {
									let event_idx = events.iter().position(|x| x.id == event_id);
									let inner_text = if let Some(idx) = event_idx {
										let event = match &events[idx].contents {
											EventContent::SpreadGroup(spread) => spread,
											_ => panic!()
										};
										match choice.as_str() {
											"Home" =>
												format!(
													"{}\n{}",
													teams.get(&event.home_id).unwrap().name.clone(),
													wager
												),
											"Away" =>
												format!(
													"{}\n{}",
													teams.get(&event.away_id).unwrap().name.clone(),
													wager
												),
											_ => "None".into()
										}
									} else {
										"None".into()
									};
									if let Some(correct) = correct {
										if correct {
											view!{cx,
												<td class="bg-green-300">
													<p class="whitespace-pre-wrap">{inner_text}</p>
												</td>
											}
										} else {
											view!{cx,
												<td class="bg-red-300">
													<p class="whitespace-pre-wrap">{inner_text}</p>
												</td>
											}
										}
									} else {
										view!{cx,
											<td>
												<p class="whitespace-pre-wrap">{inner_text}</p>
											</td>
										}
									}
								},
								"UserInput" =>
									match correct {
										Some(true) => view!{cx, <td class="bg-green-300">{choice}</td>},
										Some(false) => view!{cx, <td class="bg-red-300">{choice}</td>},
										None => view!{cx, <td>{choice}</td>}
									},
								_ => view!{cx,
									<td>
										"No table view for this pick type"
									</td>
								}
							}
						})
						.collect_view(cx)
				}
			</tr>
		}.into_view(cx);
		user_rows.push(view)
	}

	let table = view!{cx,
		<div class="h-screen overflow-auto border border-black">
			<table class="picktable">
				{table_header}
				{user_rows}
			</table>
		</div>
	}
		.into_view(cx)
		.render_to_string(cx)
		.to_string();

	Ok(table)
}
