use axum_ctx::RespErr;

use crate::{
    auth::{authz::has_perm, AuthSession},
    db::book::get_books,
    AppError,
};

pub async fn handler(session: AuthSession) -> Result<maud::Markup, RespErr> {
    let user = session.user.ok_or(AppError::BackendUser)?;

    let crate::auth::BackendPgDB(pool) = session.backend;
    let books = get_books(user.id, &pool).await.map_err(AppError::from)?;

    let is_admin = has_perm("admin", user.id, &pool).await.unwrap_or(false);

    Ok(crate::templates::home_page::markup(
        &user.username,
        is_admin,
        books,
    ))
}
