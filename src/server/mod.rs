mod auth; pub use auth::*;
mod user; pub use user::*;
mod admin_app; pub use admin_app::*;

mod book; pub use book::*;
mod admin_book; pub use admin_book::*;

mod team; pub use team::*;
mod chapter; pub use chapter::*;

use cfg_if::cfg_if;

cfg_if! {
	if #[cfg(feature = "ssr")] {

		use leptos::{ Scope, ServerFnError, use_context };
		pub type AuthSession = axum_session_auth::AuthSession<BackendUser, i64, SessionPgPool, PgPool>;
		use axum_session_auth::SessionPgPool;
		use sqlx::PgPool;

		use crate::objects::BackendUser;

		pub fn pool(cx: Scope) -> Result<PgPool, ServerFnError> {
		   use_context::<PgPool>(cx)
				.ok_or("Pool missing.")
				.map_err(|e| ServerFnError::ServerError(e.to_string()))
		}

	}
}
