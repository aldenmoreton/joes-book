mod books;
pub use books::*;
mod team;
pub use team::*;
mod chapter;
pub use chapter::*;

mod user_input;
pub use user_input::*;
mod spread;
pub use spread::*;

use cfg_if::cfg_if;
cfg_if! {
    if #[cfg(feature = "ssr")] {
        use std::collections::HashSet;

        #[derive(Clone, Debug, PartialEq, Eq)]
        pub struct BackendUser {
            pub id: i32,
            pub username: String,
            pub password: String,
            pub permissions: HashSet<String>,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct FrontendUser {
    pub id: i32,
    pub username: String,
}
