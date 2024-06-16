use askama::Template;
use axum_ctx::RespErr;

use crate::{
    auth::{authz::has_perm, AuthSession},
    objects::book::{get_books, BookSubscription},
    AppError,
};

#[derive(Template)]
#[template(path = "pages/home.html")]
pub struct HomePage {
    admin: bool,
    username: String,
    books: Vec<BookSubscription>,
}

pub async fn handler(session: AuthSession) -> Result<HomePage, RespErr> {
    let user = session.user.ok_or(AppError::BackendUser)?;
    let username = user.username;

    let crate::auth::BackendPgDB(pool) = session.backend;
    let books = get_books(user.id, &pool).await.map_err(AppError::from)?;

    let admin = has_perm("admin", user.id, &pool).await.unwrap_or(false);

    Ok(HomePage {
        admin,
        username,
        books,
    })
}
