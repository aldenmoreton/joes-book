use axum::response::ErrorResponse;

use crate::{
    auth::{authz::has_perm, AuthSession},
    db::book::get_books,
    AppError,
};

pub async fn handler(session: AuthSession) -> Result<maud::Markup, ErrorResponse> {
    let user = session.user.ok_or(AppError::BackendUser)?;

    let crate::auth::BackendPgDB(pool) = session.backend;
    let books = get_books(user.id, &pool).await?;

    let is_admin = has_perm("admin", user.id, &pool).await.unwrap_or(false);

    Ok(crate::templates::home_page::markup(
        &user.username,
        is_admin,
        books,
    ))
}
