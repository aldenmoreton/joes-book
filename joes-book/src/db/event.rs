use std::collections::HashMap;

use crate::AppError;

use super::{spread::Spread, user_input::UserInput};

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use sqlx::PgPool;

#[derive(Debug, Clone, sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "event_types", rename_all = "snake_case")]
pub enum EventType {
    SpreadGroup,
    UserInput,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Event {
    pub id: i32,
    pub book_id: i32,
    pub chapter_id: i32,
    pub is_open: bool,
    pub event_type: EventType,
    pub contents: Json<EventContent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventContent {
    SpreadGroup(Vec<Spread>),
    UserInput(UserInput),
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Pick {
    pub id: i32,
    pub book_id: i32,
    pub chapter_id: i32,
    pub event_id: i32,
    pub user_id: i32,
    pub wager: serde_json::Value,
    pub choice: serde_json::Value,
    pub points: Option<i32>,
}

pub type UserPick = (Event, Option<Pick>);

pub async fn get_events(chapter_id: i32, pool: &PgPool) -> Result<Vec<Event>, sqlx::Error> {
    sqlx::query_as!(
        Event,
        r#"	SELECT  id,
                    book_id,
                    chapter_id,
                    is_open,
                    contents AS "contents: Json<EventContent>",
                    event_type AS "event_type: EventType"
			FROM events
			WHERE chapter_id = $1
            ORDER BY event_type, id
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
) -> Result<Vec<UserPick>, sqlx::Error> {
    sqlx::query!(
        r#" SELECT  e.id AS event_id,
                    e.book_id AS book_id,
                    e.chapter_id AS chapter_id,
                    e.is_open AS is_open,
                    e.event_type AS "event_type: EventType",
                    e.contents AS "contents: Json<EventContent>",
                    p.id AS "pick_id?",
                    p.user_id AS "user_id?",
                    p.choice AS "choice?",
                    p.wager AS "wager?",
                    p.points
            FROM EVENTS AS e
            LEFT JOIN (
                SELECT *
                FROM picks
                WHERE user_id = $1 AND chapter_id = $2
            ) AS p ON e.id = p.event_id
            WHERE e.chapter_id = $2
            ORDER BY event_type
        "#,
        user_id,
        chapter_id
    )
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(|row| {
                let event = Event {
                    id: row.event_id,
                    book_id: row.book_id,
                    chapter_id: row.chapter_id,
                    is_open: row.is_open,
                    event_type: row.event_type,
                    contents: row.contents,
                };
                let pick = if let (Some(pick_id), Some(user_id), Some(wager), Some(choice)) =
                    (row.pick_id, row.user_id, row.wager, row.choice)
                {
                    Some(Pick {
                        id: pick_id,
                        book_id: row.book_id,
                        chapter_id: row.chapter_id,
                        event_id: row.event_id,
                        user_id,
                        wager,
                        choice,
                        points: row.points,
                    })
                } else {
                    None
                };

                (event, pick)
            })
            .collect_vec()
    })
}

#[derive(sqlx::FromRow)]
pub struct ChapterPicksQuery {
    #[sqlx(flatten)]
    key: ChapterPickHash,
    #[sqlx(json)]
    contents: ChapterPick,
}

#[derive(Hash, Eq, PartialEq, sqlx::FromRow)]
pub struct ChapterPickHash {
    pub event_id: i32,
    pub user_id: i32,
}

#[derive(serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ChapterPick {
    SpreadGroup {
        choice: Vec<String>,
        wager: Vec<i32>,
        points: Option<i32>,
    },
    UserInput {
        choice: String,
        wager: i32,
        points: Option<i32>,
    },
}

pub async fn get_chapter_picks(
    chapter_id: i32,
    pool: &PgPool,
) -> Result<HashMap<ChapterPickHash, ChapterPick>, AppError> {
    let records = sqlx::query_as::<_, ChapterPicksQuery>(
        r#"
            SELECT
                EVENTS.ID AS "event_id",
                PICKS.USER_ID,
                JSONB_BUILD_OBJECT(
                    'type',
                    EVENTS.EVENT_TYPE,
                    'choice',
                    PICKS.CHOICE,
                    'wager',
                    PICKS.WAGER,
                    'points',
                    PICKS.POINTS
                ) AS contents
            FROM
                EVENTS
                JOIN PICKS ON EVENTS.ID = PICKS.EVENT_ID
            WHERE
                EVENTS.CHAPTER_ID = $1
        "#,
    )
    .bind(chapter_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::from)?;

    Ok(records
        .into_iter()
        .map(|record| (record.key, record.contents))
        .collect())
}
