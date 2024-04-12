use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spread {
    pub home_id: i32,
    pub away_id: i32,
    pub home_spread: f64,
    pub notes: Option<String>,
}
