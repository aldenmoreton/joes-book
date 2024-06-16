pub mod admin;
pub mod create;
pub mod page;

use axum::{
    body::Body,
    extract::{Path, Request},
    http::{Response, StatusCode},
    middleware::Next,
    Extension,
};
use axum_ctx::RespErr;

use crate::{
    auth::{AuthSession, BackendPgDB},
    db::{
        book::{BookRole, BookSubscription},
        chapter::get_chapter,
    },
    AppError,
};

pub async fn require_admin(
    Extension(book_subscription): Extension<BookSubscription>,
    request: Request,
    next: Next,
) -> Result<Response<Body>, RespErr> {
    if book_subscription.role != BookRole::Admin {
        return Err(AppError::Unauthorized(
            "You do not have admin privilages for this book".into(),
        )
        .into());
    }

    Ok(next.run(request).await)
}

pub async fn chapter_ext(
    auth_session: AuthSession,
    Path((_, chapter_id)): Path<(i32, i32)>,
    mut request: Request,
    next: Next,
) -> Result<Response<Body>, (StatusCode, String)> {
    let BackendPgDB(pool) = auth_session.backend;

    let Ok(chapter) = get_chapter(chapter_id, &pool).await else {
        return Err((StatusCode::NOT_FOUND, "Could not find chapter".to_string()));
    };
    request.extensions_mut().insert(chapter);

    Ok(next.run(request).await)
}
