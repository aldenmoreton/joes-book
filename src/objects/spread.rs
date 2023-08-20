use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Spread {
	pub home_id: i64,
	pub away_id: i64,
	pub home_spread: i64,
	pub notes: String
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
