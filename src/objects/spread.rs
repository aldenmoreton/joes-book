// use leptos::{create_rw_signal, RwSignal, Scope};
use serde::{Serialize, Deserialize};
use derive_builder::Builder;

use super::Pick;

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct Spread {
	#[builder(field(public))]
	pub home_id: i64,
	#[builder(field(public))]
	pub away_id: i64,
	#[builder(field(public))]
	pub home_spread: f64,
	#[builder(field(public), default)]
	pub notes: Option<String>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpreadGroup {
	pub spreads: Vec<Spread>
}

pub struct SpreadGroupPick {
	pub picks: Vec<Pick>
}
