use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInput {
    pub title: String,
    pub description: Option<String>,
    pub points: i32,
}
