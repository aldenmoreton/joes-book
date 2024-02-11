use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct UserInput {
    pub question: String,
    pub points: i64,
}

#[derive(Debug, Clone, Default)]
pub struct UserInputBuilder {
    pub question: Option<String>,
    pub points: Option<i64>,
}

impl UserInputBuilder {
    pub fn new() -> Self {
        Default::default()
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
