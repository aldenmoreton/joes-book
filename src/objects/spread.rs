use serde::{Serialize, Deserialize};

use super::Pick;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spread {
	pub home_id: i64,
	pub away_id: i64,
	pub home_spread: f64,
	pub notes: Option<String>
}

#[derive(Clone, Debug, Default)]
pub struct SpreadBuilder {
	pub home_id: Option<i64>,
	pub away_id: Option<i64>,
	pub home_spread: Option<f64>,
	pub notes: Option<String>
}

impl SpreadBuilder {
	pub fn build(self) -> Result<Spread, Self> {
		match self {
			SpreadBuilder { home_id: Some(home_id), away_id: Some(away_id), home_spread: Some(home_spread), notes } => Ok(Spread {home_id, away_id, home_spread, notes}),
			spread_buidler => Err(spread_buidler)
		}
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpreadGroup {
	pub spreads: Vec<Spread>
}

pub struct SpreadGroupPick {
	pub picks: Vec<Pick>
}
