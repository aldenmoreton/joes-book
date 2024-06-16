use itertools::Itertools;

use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgRow, FromRow, PgPool, Row};

use super::{spread::Spread, user_input::UserInput};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: i32,
    pub book_id: i32,
    pub chapter_id: i32,
    pub is_open: bool,
    pub event_type: String,
    pub contents: EventContent,
}

impl FromRow<'_, PgRow> for Event {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        let contents: EventContent = {
            let content_str: String = row.try_get("contents")?;
            serde_json::from_str(&content_str).map_err(|err| sqlx::Error::Decode(Box::new(err)))?
        };

        Ok(Event {
            id: row.try_get("id")?,
            book_id: row.try_get("book_id")?,
            chapter_id: row.try_get("chapter_id")?,
            is_open: row.try_get("is_open")?,
            event_type: row.try_get("event_type")?,
            contents,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventContent {
    SpreadGroup(Spread),
    UserInput(UserInput),
}

// TODO: This is bad
impl From<String> for EventContent {
    fn from(value: String) -> Self {
        serde_json::from_str(&value)
            .unwrap_or_else(|_| panic!("Could not Deserialize Event Contents: {value}"))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Pick {
    pub id: Option<i32>,
    pub book_id: i32,
    pub chapter_id: i32,
    pub event_id: i32,
    pub wager: Option<i32>,
    pub choice: Option<String>,
    pub correct: Option<bool>,
}

pub async fn get_events(chapter_id: i32, pool: &PgPool) -> Result<Vec<Event>, sqlx::Error> {
    sqlx::query_as!(
        Event,
        r#"	SELECT id, book_id, chapter_id, contents, event_type, is_open
			FROM events
			WHERE chapter_id = $1
			ORDER BY event_type
		"#,
        chapter_id
    )
    .fetch_all(pool)
    .await
}

pub async fn get_picks(
    user_id: i32,
    chapter_id: i32,
    pool: &PgPool,
) -> Result<Vec<(String, Vec<(Event, Pick)>)>, sqlx::Error> {
    let events = get_events(chapter_id, pool).await?;

    let mut picks = Vec::new();
    for event in events.iter() {
        picks.push(get_pick(user_id, event.id, pool))
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

pub async fn get_pick(user_id: i32, event_id: i32, pool: &PgPool) -> Result<Pick, sqlx::Error> {
    let pick = sqlx::query_as!(
        Pick,
        r#" SELECT id AS "id?", book_id, chapter_id, event_id, wager AS "wager?", choice AS "choice?", correct AS "correct?"
			FROM picks
			WHERE event_id = $1 AND user_id = $2"#,
        event_id,
        user_id
    )
    .fetch_optional(pool)
    .await?;

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
				.bind(user_id)
				.fetch_one(pool)
				.await?;

            // let book_subscription = get_book(user_id, event.book_id, pool).await?;
            // if book_subscription.role == BookRole::Unauthorized {
            //     return Err(StatusCode::UNAUTHORIZED);
            // }

            Pick {
                id: None,
                book_id: event.book_id,
                chapter_id: event.chapter_id,
                event_id,
                wager: None,
                choice: None,
                correct: None,
            }
        }
    };

    Ok(pick)
}
