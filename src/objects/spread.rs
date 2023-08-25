// use leptos::{create_rw_signal, RwSignal, Scope};
use serde::{Serialize, Deserialize};
use derive_builder::Builder;

#[derive(Debug, Serialize, Deserialize, Builder)]
pub struct Spread {
	#[builder(field(public))]
	pub home_id: i64,
	#[builder(field(public))]
	pub away_id: i64,
	#[builder(field(public))]
	pub home_spread: f64,
	#[builder(field(public), setter(strip_option))]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct SpreadGroup {
	pub spreads: Vec<Spread>
}

pub struct SpreadGroupPick {
	pub picks: Vec<SpreadPick>
}
