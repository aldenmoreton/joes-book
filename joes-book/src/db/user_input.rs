use std::collections::HashSet;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInput {
    pub title: String,
    pub description: Option<String>,
    pub points: i32,
    pub acceptable_answers: Option<HashSet<String>>,
}
