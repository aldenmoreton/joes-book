use askama::Template;
use axum::response::IntoResponse;

use crate::auth::AuthSession;

#[derive(Template)]
#[template(path = "components/nav.html")]
struct Nav {
    username: String,
}

pub async fn user(auth_session: AuthSession) -> impl IntoResponse {
    let username = auth_session.user.unwrap().username;

    Nav { username }
}
