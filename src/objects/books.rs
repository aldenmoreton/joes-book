use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BookRole {
    Owner,
    Admin,
    Participant,
    Unauthorized,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct BookSubscription {
    #[cfg_attr(feature = "ssr", sqlx(rename = "id"))]
    pub book_id: i64,
    pub user_id: i64,
    pub name: String,
    #[cfg_attr(feature = "ssr", sqlx(try_from = "String"))]
    pub role: BookRole,
}

impl From<String> for BookRole {
    fn from(value: String) -> Self {
        match value.as_str() {
            "owner" => Self::Owner,
            "admin" => Self::Admin,
            "participant" => Self::Participant,
            _ => Self::Unauthorized,
        }
    }
}

impl Into<String> for BookRole {
    fn into(self) -> String {
        match self {
            Self::Owner => "owner",
            Self::Admin => "admin",
            Self::Participant => "participant",
            Self::Unauthorized => "unauthorized",
        }
        .into()
    }
}
