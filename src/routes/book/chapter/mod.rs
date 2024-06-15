pub mod admin;
pub mod create;
pub mod page;

use axum::{
    body::Body,
    extract::{Path, Request},
    http::{Response, StatusCode},
    middleware::Next,
    response::IntoResponse,
    Extension,
};

use crate::{
    auth::{AuthSession, BackendPgDB},
    objects::{
        book::{BookRole, BookSubscription},
        chapter::get_chapter,
    },
};

pub async fn require_admin(
    Extension(book_subscription): Extension<BookSubscription>,
    request: Request,
    next: Next,
) -> impl IntoResponse {
    if book_subscription.role != BookRole::Admin {
        return (
            StatusCode::UNAUTHORIZED,
            "You cannot edit/create chapters for this book",
        )
            .into_response();
    }

    next.run(request).await.into_response()
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
