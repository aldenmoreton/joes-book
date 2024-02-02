use askama::Template;
use axum::{response::IntoResponse, routing::get, Router};

use crate::auth::AuthSession;

// TODO: Change this, put it somewhere else
pub fn router() -> Router {
	Router::new()
		.route("/", get(user))
}

#[derive(Template)]
#[template(path = "components/nav.html")]
struct Nav {
	username: String
}

async fn user(auth_session: AuthSession) -> impl IntoResponse {
	let username = auth_session.user.unwrap().username;

	Nav { username }
}
