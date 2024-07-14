use axum::Extension;
use axum_ctx::RespErr;

use crate::{
    auth::{AuthSession, BackendPgDB},
    db::{book::BookSubscription, chapter::get_chapters},
    AppError,
};

pub async fn handler(
    auth_session: AuthSession,
    Extension(book_subscription): Extension<BookSubscription>,
) -> Result<maud::Markup, RespErr> {
    let user = auth_session.user.ok_or(AppError::BackendUser)?;
    let BackendPgDB(pool) = auth_session.backend;

    let chapters = get_chapters(book_subscription.book_id, &pool)
        .await
        .map_err(AppError::from)?;

    Ok(crate::templates::book_page::markup(
        &user.username,
        book_subscription,
        chapters,
    ))
}
