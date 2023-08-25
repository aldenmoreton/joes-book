use leptos::RwSignal;
use serde::{Serialize, Deserialize};

use crate::objects::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct Chapter {
	pub book_id: i64,
	pub chapter_id: i64,
	pub events: Vec<Event>
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Event {
	SpreadGroup(SpreadGroup),
	UserInput(TextBet)
}

#[derive(Clone)]
pub enum EventBuilder {
	SpreadGroup(RwSignal<Vec<(i64, RwSignal<SpreadBuilder>)>>)
}

trait Validate {}
