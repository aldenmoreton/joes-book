use cfg_if;
use leptos::{RwSignal, SignalGet};
use serde::{Deserialize, Serialize};

use crate::objects::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Chapter {
    #[cfg_attr(feature = "ssr", sqlx(rename = "id"))]
    pub chapter_id: i64,
    pub book_id: i64,
    pub is_open: bool,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: i64,
    pub book_id: i64,
    pub chapter_id: i64,
    pub is_open: bool,
    pub event_type: String,
    pub contents: EventContent,
}

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use sqlx::{FromRow, postgres::PgRow, Row};

        impl FromRow<'_, PgRow> for Event {
            fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
                let contents: EventContent = {
                    let content_str: String = row.try_get("contents")?;
                    serde_json::from_str(&content_str)
                        .map_err(|err| sqlx::Error::Decode(Box::new(err)))?
                };

                Ok(Event {
                    id: row.try_get("id")?,
                    book_id: row.try_get("book_id")?,
                    chapter_id: row.try_get("chapter_id")?,
                    is_open: row.try_get("is_open")?,
                    event_type: row.try_get("event_type")?,
                    contents
                })
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventContent {
    SpreadGroup(Spread),
    UserInput(UserInput),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Pick {
    pub id: Option<i64>,
    pub book_id: i64,
    pub chapter_id: i64,
    pub event_id: i64,
    pub wager: Option<i64>,
    pub choice: Option<String>,
    pub correct: Option<bool>,
}

#[derive(Clone, Debug)]
pub enum EventBuilder {
    SpreadGroup(RwSignal<Vec<(i64, RwSignal<SpreadBuilder>)>>),
    UserInput(RwSignal<UserInputBuilder>),
}

impl EventBuilder {
    pub fn build(self) -> Result<Vec<EventContent>, String> {
        match self {
            EventBuilder::SpreadGroup(spreads) => {
                let mut spread_groups = Vec::new();
                for (_, spread) in spreads.get() {
                    match spread.get().build() {
                        Ok(spread) => spread_groups.push(EventContent::SpreadGroup(spread)),
                        Err(e) => return Err(format!("Could not build Spread: {:?}", e).into()),
                    }
                }
                Ok(spread_groups)
            }
            EventBuilder::UserInput(input) => Ok(vec![input
                .get()
                .build()
                .map(|input| EventContent::UserInput(input))
                .ok_or(format!("Could not build user input"))?]),
        }
    }
}

trait Validate {}
