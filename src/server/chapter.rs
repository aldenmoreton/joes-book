use cfg_if::cfg_if;
use leptos::*;

use crate::objects::{Chapter, Event, EventContent, Pick};

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use itertools::Itertools;
        use sqlx::Row;
        use crate::{
            server::{pool, get_book, auth::auth},
            objects::BookRole
        };
    }
}

#[server(AddChapter, "/secure", "Url", "add_chapter")]
pub async fn add_chapter(
    book_id: i32,
    title: String,
    closing_time: String,
    events: Vec<EventContent>,
) -> Result<i32, ServerFnError> {
    let book_sub = get_book(book_id).await?;

    match book_sub.role {
        BookRole::Owner => (),
        _ => {
            return Err(ServerFnError::Request(
                "You can't make chapters on someone else's book".into(),
            ))
        }
    }
    let closing_time = chrono::DateTime::parse_from_rfc3339(&closing_time)
        .map_err(|e| ServerFnError::new(format!("Could not parse closing time: {e}")))?;
    let closing_time: chrono::DateTime<chrono::Utc> = closing_time.into();
    if closing_time <= chrono::Utc::now() {
        return Err(ServerFnError::Args(
            "Your closing time can't be in the past".into(),
        ));
    }

    let pool = pool()?;
    let chapter_id = sqlx::query!(
        r#"	INSERT INTO chapters (title, book_id, is_open)
			VALUES ($1, $2, $3)
			RETURNING id"#,
        title,
        book_id,
        true
    )
    .fetch_one(&pool)
    .await?
    .id;

    for event in events {
        let contents = serde_json::to_string(&event)?;
        let event_type = match event {
            EventContent::SpreadGroup(_) => "SpreadGroup",
            EventContent::UserInput(_) => "UserInput",
        };
        sqlx::query(
            r#"	INSERT INTO events (book_id, chapter_id, is_open, event_type, contents)
				VALUES ($1, $2, $3, $4, $5)
			"#,
        )
        .bind(book_id)
        .bind(chapter_id)
        .bind(true)
        .bind(event_type)
        .bind(contents)
        .execute(&pool)
        .await?;
    }

    Ok(chapter_id)
}

#[server(GetChapters, "/secure", "Url", "get_chapters")]
pub async fn get_chapters(book_id: i32) -> Result<Vec<Chapter>, ServerFnError> {
    let book_subscription = get_book(book_id).await?;
    match book_subscription.role {
        BookRole::Unauthorized => {
            return Err(ServerFnError::Request("You aren't in this book".into()))
        }
        _ => (),
    }

    let pool = pool()?;

    let chapters = sqlx::query_as_unchecked!(
        Chapter,
        r#"	SELECT id AS chapter_id, book_id, title, is_open
			FROM chapters
			WHERE book_id = $1
			ORDER BY created_at DESC
		"#,
        book_id
    )
    .fetch_all(&pool)
    .await
    .unwrap_or(Vec::new());

    Ok(chapters)
}

#[server(GetChapter, "/secure", "Url", "get_chapter")]
pub async fn get_chapter(chapter_id: i32) -> Result<Chapter, ServerFnError> {
    let pool = pool()?;

    let book_id = sqlx::query!(
        r#"
		SELECT book_id
		FROM chapters
		WHERE id = $1
	"#,
        chapter_id
    )
    .fetch_one(&pool)
    .await?
    .book_id;
    let book_subscription = get_book(book_id).await?;
    match book_subscription.role {
        BookRole::Unauthorized => {
            return Err(ServerFnError::Request("You aren't in this book".into()))
        }
        _ => (),
    }

    let chapter = sqlx::query_as_unchecked!(
        Chapter,
        r#"	SELECT id AS chapter_id, book_id, title, is_open
			FROM chapters
			WHERE id = $1
		"#,
        chapter_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|err| ServerFnError::new(format!("Could not find chapter: {err}")))?;

    let book_subscription = get_book(chapter.book_id).await?;
    match book_subscription.role {
        BookRole::Unauthorized => {
            return Err(ServerFnError::Request(
                "You don't have access to this book's chapters".into(),
            ))
        }
        _ => (),
    }

    Ok(chapter)
}

#[server(SetOpen, "/secure", "Url", "set_open")]
pub async fn set_open(chapter_id: i32, new_status: bool) -> Result<(), ServerFnError> {
    let pool = pool()?;

    let book_id = sqlx::query!(
        r#"
		SELECT book_id
		FROM chapters
		WHERE id = $1
	"#,
        chapter_id
    )
    .fetch_one(&pool)
    .await?
    .book_id;
    let book_subscription = get_book(book_id).await?;
    match book_subscription.role {
        BookRole::Owner => (),
        _ => {
            return Err(ServerFnError::Request(
                "You aren't the owner of this book".into(),
            ))
        }
    }

    sqlx::query(
        r#"
		UPDATE chapters
		SET is_open = $1
		WHERE id = $2
	"#,
    )
    .bind(new_status)
    .bind(chapter_id)
    .execute(&pool)
    .await?;

    Ok(())
}

#[server(IsOpen, "/secure", "Url", "is_open")]
pub async fn is_open(chapter_id: i32) -> Result<bool, ServerFnError> {
    let pool = pool()?;

    let book_id = sqlx::query!(
        r#"
		SELECT book_id
		FROM chapters
		WHERE id = $1
	"#,
        chapter_id
    )
    .fetch_one(&pool)
    .await?
    .book_id;
    let book_subscription = get_book(book_id).await?;
    match book_subscription.role {
        BookRole::Unauthorized => {
            return Err(ServerFnError::Request("You aren't in this book".into()))
        }
        _ => (),
    }

    let status = sqlx::query!(
        r#"
		SELECT is_open
		FROM chapters
		WHERE id = $1
	"#,
        chapter_id
    )
    .fetch_one(&pool)
    .await?
    .is_open;

    Ok(status)
}

#[server(GetEvents, "/secure", "Url", "get_events")]
pub async fn get_events(chapter_id: i32) -> Result<Vec<Event>, ServerFnError> {
    let pool = pool()?;

    let book_id = sqlx::query!(
        r#"
		SELECT book_id
		FROM chapters
		WHERE id = $1
	"#,
        chapter_id
    )
    .fetch_one(&pool)
    .await?
    .book_id;

    let book_subscription = get_book(book_id).await?;
    match book_subscription.role {
        BookRole::Unauthorized => {
            return Err(ServerFnError::Request("You aren't in this book".into()))
        }
        _ => (),
    }

    get_chapter(chapter_id)
        .await
        .map_err(|err| ServerFnError::new(format!("Could not get chapter: {err}")))?;

    let events = sqlx::query_as::<_, Event>(
        r#"	SELECT id, book_id, chapter_id, contents, event_type, is_open
			FROM events
			WHERE chapter_id = $1
			ORDER BY event_type, id
		"#,
    )
    .bind(chapter_id)
    .fetch_all(&pool)
    .await?;

    Ok(events)
}

#[server(GetPick, "/secure", "Url", "get_pick")]
pub async fn get_pick(event_id: i32) -> Result<Pick, ServerFnError> {
    let user = auth()?.current_user.unwrap();
    let pool = pool()?;

    let pick = sqlx::query_as::<_, Pick>(
        r#" SELECT *
				FROM picks
				WHERE event_id = $1 AND user_id = $2"#,
    )
    .bind(event_id)
    .bind(user.id)
    .fetch_optional(&pool)
    .await
    .map_err(|err| ServerFnError::new(format!("Could not find pick: {err}")))?;

    let pick = match pick {
        Some(pick) => pick,
        None => {
            let event = sqlx::query_as::<_, Event>(
                r#"	SELECT id, book_id, chapter_id, contents, event_type, is_open
					FROM events
					WHERE id = $1
				"#,
            )
            .bind(event_id)
            .bind(user.id)
            .fetch_one(&pool)
            .await
            .map_err(|err| ServerFnError::new(format!("Could not build event: {err}")))?;

            let book_subscription = get_book(event.book_id).await?;
            match book_subscription.role {
                BookRole::Unauthorized => {
                    return Err(ServerFnError::Request(
                        "You don't have access to this book's chapters".into(),
                    ))
                }
                _ => (),
            }

            Pick {
                id: None,
                book_id: event.book_id,
                chapter_id: event.chapter_id,
                event_id: event_id,
                wager: None,
                choice: None,
                correct: None,
            }
        }
    };
    Ok(pick)
}

#[server(GetPicks, "/secure", "Url", "get_picks")]
pub async fn get_picks(
    chapter_id: i32,
) -> Result<Vec<(String, Vec<(Event, Pick)>)>, ServerFnError> {
    let events = get_events(chapter_id)
        .await
        .map_err(|err| ServerFnError::new(format!("Could not get events: {err}")))?;

    let mut picks = Vec::new();
    for event in events.iter() {
        picks.push(get_pick(event.id))
    }

    let mut awaited_picks = Vec::new();
    for pick in picks {
        awaited_picks.push(pick.await?)
    }

    let event_picks: Vec<(Event, Pick)> = events.into_iter().zip(awaited_picks).collect();

    let mut data_grouped = Vec::new();
    for (key, group) in &event_picks
        .into_iter()
        .group_by(|elt| elt.0.event_type.clone())
    {
        data_grouped.push((key, group.collect()));
    }

    Ok(data_grouped)
}

cfg_if! {
    if #[cfg(feature = "ssr")] {
        pub async fn create_pick(pick: Pick) -> Result<(), ServerFnError> {
            let book_sub = get_book(pick.book_id).await?;
            match book_sub.role {
                BookRole::Unauthorized =>
                    return Err(ServerFnError::Request("You can't make picks for a book you are not a member of".into())),
                _ => ()
            }

            let chapter = get_chapter(pick.chapter_id).await?;
            if !chapter.is_open {
                return Err(ServerFnError::Request("You're too late to make picks".into()))
            }

            let pool = pool()?;
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

        pub async fn update_pick(pick: Pick) -> Result<(), ServerFnError> {
            let book_sub = get_book(pick.book_id).await?;
            match book_sub.role {
                BookRole::Unauthorized =>
                    return Err(ServerFnError::Request("You can't update picks for a book you are not a member of".into())),
                _ => ()
            }

            let chapter = get_chapter(pick.chapter_id).await?;
            if	!chapter.is_open {
                return Err(ServerFnError::Request("You're too late to update picks".into()))
            }

            let pool = pool()?;
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

#[server(SavePicks, "/secure", "Url", "save_picks")]
pub async fn save_picks(picks: Vec<Pick>) -> Result<(), ServerFnError> {
    for pick in picks {
        if pick.id.is_some() {
            update_pick(pick).await?
        } else {
            create_pick(pick).await?
        }
    }

    Ok(())
}

#[server(GetUserInputs, "/secure", "Url", "get_user_inputs")]
pub async fn get_user_inputs(event_id: i32) -> Result<Vec<String>, ServerFnError> {
    let pool = pool()?;

    let chapter_id = sqlx::query!(
        r#"
		SELECT chapter_id
		FROM events
		WHERE id = $1
	"#,
        event_id
    )
    .fetch_one(&pool)
    .await?
    .chapter_id;

    if is_open(chapter_id).await? {
        return Err(ServerFnError::Request(
            "The chapter isn't closed yet! You can't see everyone's picks!".into(),
        ));
    }

    let result = sqlx::query(
        r#" SELECT DISTINCT choice
			FROM picks
			WHERE event_id = $1
		"#,
    )
    .bind(event_id)
    .fetch_all(&pool)
    .await?;

    Ok(result.into_iter().map(|row| row.get("choice")).collect())
}

#[server(SaveAnswers, "/secure", "Url", "save_answers")]
pub async fn save_answers(picks: Vec<(i32, Vec<String>)>) -> Result<(), ServerFnError> {
    let pool = pool()?;

    let book_id = sqlx::query!(
        r#"
		SELECT book_id
		FROM picks
		WHERE event_id = $1
	"#,
        picks
            .get(0)
            .ok_or(ServerFnError::new("List of picks must not be empty"))?
            .0
    )
    .fetch_one(&pool)
    .await
    .map_err(|_| ServerFnError::new("Could not get book id"))?
    .book_id;
    let book_sub = get_book(book_id).await?;
    match book_sub.role {
        BookRole::Owner => (),
        _ => {
            return Err(ServerFnError::Request(
                "You can't make answers for someone else's book".into(),
            ))
        }
    }

    for (id, answers) in picks {
        let answers = answers
            .iter()
            .map(|a| format!(r#"'{a}'"#))
            .collect::<Vec<String>>()
            .join(",");
        let query = format!(
            r#"
			UPDATE picks
			SET correct = choice IN ({})
			WHERE event_id = $1 AND book_id = $2
		"#,
            answers
        );

        sqlx::query(&query)
            .bind(id)
            .bind(book_id)
            .execute(&pool)
            .await?;
    }

    Ok(())
}

// #[server(GetChapterTable, "/secure", "Url", "get_chapter_table")]
// pub async fn get_chapter_table(chapter_id: i32) -> Result<String, ServerFnError> {
// 	if is_open(chapter_id).await? {
// 		return Err(ServerFnError::Request("The chapter isn't closed yet! You can't see everyone's picks!".into()))
// 	}

// 	let events = get_events(chapter_id).await?;
// 	let mut teams: HashMap<i32, Team> = HashMap::new();

// 	for event in events.iter() {
// 		match &event.contents {
// 			EventContent::SpreadGroup(spreads) => {
// 				let (home, away) = super::get_spread_teams(spreads.home_id, spreads.away_id).await?;
// 				teams.insert(spreads.home_id, home);
// 				teams.insert(spreads.away_id, away);
// 			},
// 			_ => ()
// 		}
// 	}

// 	let table_header = view!{
// 		<tr>
// 			<th></th>
// 			{
// 				events
// 					.iter()
// 					.map(|event| {
// 						let description: String = match &event.contents {
// 							EventContent::SpreadGroup(spreads) =>
// 								format!(
// 									"{} at {}({:+})",
// 									teams.get(&spreads.away_id).unwrap().name,
// 									teams.get(&spreads.home_id).unwrap().name,
// 									spreads.home_spread
// 								),
// 							EventContent::UserInput(input) =>
// 								input.question.clone()
// 						};
// 						view!{
// 							<th>
// 								<h1>{description}</h1>
// 							</th>
// 						}
// 					}
// 					)
// 					.collect_view()
// 			}
// 		</tr>
// 	};

// 	let pool = pool()?;
// 	let user_points: Vec<(_, _, _)> = sqlx::query!(r#"
// 		SELECT u.id AS id, u.username AS username, CAST(COALESCE(p.total, 0) AS INTEGER) AS week_total
// 		FROM (
// 			SELECT users.id, users.username
// 			FROM chapters
// 			INNER JOIN subscriptions ON subscriptions.book_id = chapters.book_id
// 			INNER JOIN users ON users.id = subscriptions.user_id
// 			WHERE chapters.id = $1
// 			GROUP BY users.id, users.username
// 		) AS u
// 		LEFT JOIN (
// 			SELECT user_id, SUM(picks.wager) AS total
// 			FROM picks
// 			WHERE picks.chapter_id = $1 AND picks.correct
// 			GROUP BY user_id
// 		) AS p
// 		ON u.id = p.user_id
// 		ORDER BY week_total DESC, username"#,
// 		chapter_id
// 	)
// 		.fetch_all(&pool)
// 		.await?
// 		.into_iter()
// 		.map(|row| {
// 			(row.id, row.username, row.week_total.unwrap_or(0))
// 		})
// 		.collect();

// 	let mut user_rows: Vec<View> = Vec::new();
// 	for (user_id, username, week_total) in user_points {
// 		let picks: Vec<_> = sqlx::query!(r#"
// 			SELECT picks.choice, picks.wager, picks.correct, events.event_type, picks.event_id
// 			FROM picks
// 			JOIN events ON picks.event_id = events.id
// 			WHERE picks.chapter_id = $1 AND picks.user_id = $2
// 			ORDER BY events.event_type, events.id"#,
// 			chapter_id, user_id
// 		)
// 			.fetch_all(&pool)
// 			.await?
// 			.into_iter()
// 			.map(|row|
// 				(row.choice, row.wager, row.correct, row.event_type, row.event_id)
// 			)
// 			.collect();

// 		let view = view!{
// 			<tr>
// 				<td>
// 					{username}
// 					<br/>
// 					{week_total}
// 				</td>
// 				{
// 					picks
// 						.into_iter()
// 						.map(|(choice, wager, correct, ty, event_id)| {
// 							match ty.as_str() {
// 								"SpreadGroup" => {
// 									let event_idx = events.iter().position(|x| x.id == event_id);
// 									let inner_text = if let Some(idx) = event_idx {
// 										let event = match &events[idx].contents {
// 											EventContent::SpreadGroup(spread) => spread,
// 											_ => panic!()
// 										};
// 										match choice.as_str() {
// 											"Home" =>
// 												format!(
// 													"{}\n{}",
// 													teams.get(&event.home_id).unwrap().name.clone(),
// 													wager
// 												),
// 											"Away" =>
// 												format!(
// 													"{}\n{}",
// 													teams.get(&event.away_id).unwrap().name.clone(),
// 													wager
// 												),
// 											_ => "None".into()
// 										}
// 									} else {
// 										"None".into()
// 									};
// 									if let Some(correct) = correct {
// 										if correct {
// 											view!{
// 												<td class="bg-green-300">
// 													<p class="whitespace-pre-wrap">{inner_text}</p>
// 												</td>
// 											}
// 										} else {
// 											view!{
// 												<td class="bg-red-300">
// 													<p class="whitespace-pre-wrap">{inner_text}</p>
// 												</td>
// 											}
// 										}
// 									} else {
// 										view!{
// 											<td>
// 												<p class="whitespace-pre-wrap">{inner_text}</p>
// 											</td>
// 										}
// 									}
// 								},
// 								"UserInput" =>
// 									match correct {
// 										Some(true) => view!{<td class="bg-green-300">{choice}</td>},
// 										Some(false) => view!{<td class="bg-red-300">{choice}</td>},
// 										None => view!{<td>{choice}</td>}
// 									},
// 								_ => view!{
// 									<td>
// 										"No table view for this pick type"
// 									</td>
// 								}
// 							}
// 						})
// 						.collect_view()
// 				}
// 			</tr>
// 		}.into_view();
// 		user_rows.push(view)
// 	}

// 	let table = view!{
// 		<div class="h-screen overflow-auto border border-black">
// 			<table class="picktable">
// 				{table_header}
// 				{user_rows}
// 			</table>
// 		</div>
// 	}
// 		.into_view()
// 		.render_to_string()
// 		.to_string();

// 	Ok(table)
// }
