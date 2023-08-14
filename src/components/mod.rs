mod signup; pub use signup::Signup;
mod login; pub use login::Login;
mod logout; pub use logout::Logout;
mod header; pub use header::Header;
mod team_search; pub use team_search::TeamSelect;
mod book; pub use book::Book;
mod admin; pub use admin::Admin;

// pub mod event; pub use event::BookEvents;

pub mod pick_six;
pub mod books; pub use books::Books;

use cfg_if::cfg_if;

cfg_if! {
	if #[cfg(feature = "ssr")] {

		use leptos::{ Scope, ServerFnError, use_context };
		pub type AuthSession = axum_session_auth::AuthSession<BackendUser, i64, SessionPgPool, PgPool>;
		use axum_session_auth::SessionPgPool;
		use sqlx::PgPool;

		use crate::auth::BackendUser;

		pub fn pool(cx: Scope) -> Result<PgPool, ServerFnError> {
		   use_context::<PgPool>(cx)
				.ok_or("Pool missing.")
				.map_err(|e| ServerFnError::ServerError(e.to_string()))
		}

	}
}
