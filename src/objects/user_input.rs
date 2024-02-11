use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct UserInput {
    pub question: String,
    pub points: i64,
}

#[derive(Debug, Clone)]
pub struct UserInputBuilder {
    pub question: Option<String>,
    pub points: Option<i64>,
}

impl UserInputBuilder {
    pub fn new() -> Self {
        UserInputBuilder {
            question: None,
            points: None,
        }
    }

    pub fn build(self) -> Option<UserInput> {
        Some(UserInput {
            question: self.question?,
            points: self.points?,
        })
    }

    pub fn question(&mut self, q: String) -> &mut Self {
        self.question = Some(q);
        self
    }
    pub fn points(&mut self, p: i64) -> &mut Self {
        self.points = Some(p);
        self
    }
}
