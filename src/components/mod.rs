pub mod signup; pub use signup::Signup;
pub mod login; pub use login::Login;
pub mod logout; pub use logout::Logout;
pub mod header; pub use header::Header;
pub mod team_search; pub use team_search::TeamSelect;
pub mod todo; pub use todo::Todos;
pub mod book; pub use book::Book;

pub mod pick_six;
pub mod books; pub use books::Books;

use cfg_if::cfg_if;

cfg_if! {
	if #[cfg(feature = "ssr")] {

		use leptos::{ Scope, ServerFnError, use_context };
		pub type AuthSession = axum_session_auth::AuthSession<User, i64, SessionPgPool, PgPool>;
		use axum_session_auth::SessionPgPool;
		use sqlx::PgPool;

		use crate::auth::User;

		pub fn pool(cx: Scope) -> Result<PgPool, ServerFnError> {
		   use_context::<PgPool>(cx)
				.ok_or("Pool missing.")
				.map_err(|e| ServerFnError::ServerError(e.to_string()))
		}

		pub fn auth(cx: Scope) -> Result<AuthSession, ServerFnError> {
			use_context::<AuthSession>(cx)
				.ok_or("Auth session missing.")
				.map_err(|e| ServerFnError::ServerError(e.to_string()))
		}

	}
}
