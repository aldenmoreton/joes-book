// use leptos::{create_rw_signal, RwSignal, Scope};
use serde::{Serialize, Deserialize};
use derive_builder::Builder;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Builder)]
pub struct Spread {
	#[builder(field(public))]
	pub home_id: i64,
	#[builder(field(public))]
	pub away_id: i64,
	pub home_spread: i64,
	pub notes: Option<String>
}

pub enum SpreadTeam {
	Home,
	Away
}

pub struct SpreadPick {
	pub pick: SpreadTeam,
	pub wager: i64
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct SpreadGroup {
	pub spreads: Vec<Spread>
}

pub struct SpreadGroupPick {
	pub picks: Vec<SpreadPick>
}
