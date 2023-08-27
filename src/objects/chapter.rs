use leptos::{RwSignal, SignalGet};
use serde::{Serialize, Deserialize};

use crate::objects::*;

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct Chapter {
// 	pub book_id: i64,
// 	pub chapter_id: i64,
// 	pub closing_time: chrono::DateTime<chrono::Utc>,
// 	pub events: Vec<Event>
// }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
	SpreadGroup(SpreadGroup),
	UserInput(TextBet)
}

#[derive(Clone, Debug)]
pub enum EventBuilder {
	SpreadGroup(RwSignal<Vec<(i64, RwSignal<SpreadBuilder>)>>)
}

impl EventBuilder {
	pub fn build(self) -> Result<Event, String> {
		match self {
			EventBuilder::SpreadGroup(spreads) => {
				let mut new_spreads = Vec::new();
				for (_, spread) in spreads.get() {
					match spread.get().build() {
						Ok(spread) => new_spreads.push(spread),
						Err(e) => return Err(format!("Could not build Spread: {:?}", e).into())
					}
				}
				Ok(Event::SpreadGroup(SpreadGroup { spreads: new_spreads }))
			}
		}
	}
}

trait Validate {}
