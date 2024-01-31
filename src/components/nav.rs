use axum::{response::IntoResponse, routing::get, Router};
use axum_login::AuthSession;
use maud::html;

use crate::auth::BackendPgDB;

pub fn router() -> Router {
	Router::new()
		.route("/user", get(user))
}

async fn user(auth_session: AuthSession<BackendPgDB>) -> impl IntoResponse {
	let username = auth_session.user.unwrap().username;

	html!{
		p {
			(username)
		}
	}
}
