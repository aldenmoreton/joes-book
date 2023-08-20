use serde::{Serialize, Deserialize};

use crate::objects::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct Chapter {
	pub book_id: i64,
	pub chapter_id: i64,
	pub events: Vec<Event>
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Event {
	pub event_id: i64,
	pub event: EventType,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum EventType {
	SpreadGroup(SpreadGroup),
	UserInput(TextBet)
}

trait Validate {}
