pub mod signup;
pub mod login;
pub mod logout;
pub mod header;

pub mod pick_six;
pub mod todo;

use cfg_if::cfg_if;

cfg_if! {
	if #[cfg(feature = "ssr")] {

		use leptos::{ Scope, ServerFnError, use_context };
		pub type AuthSession = axum_session_auth::AuthSession<User, i64, SessionSqlitePool, SqlitePool>;
		use axum_session_auth::SessionSqlitePool;
		use sqlx::SqlitePool;

		use crate::auth::User;

		pub fn pool(cx: Scope) -> Result<SqlitePool, ServerFnError> {
		   use_context::<SqlitePool>(cx)
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
