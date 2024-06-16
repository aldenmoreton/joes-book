use askama::Template;
use axum_ctx::RespErr;

use crate::{auth::AuthSession, AppError};

#[derive(Template)]
#[template(path = "components/nav.html")]
pub struct Nav {
    username: String,
}

pub async fn user(auth_session: AuthSession) -> Result<Nav, RespErr> {
    let username = auth_session.user.ok_or(AppError::BackendUser)?.username;

    Ok(Nav { username })
}
