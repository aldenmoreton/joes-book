pub mod chapter;
pub mod create;
pub mod page;

use std::collections::HashMap;

use axum::{
    extract::{Path, Request},
    http::StatusCode,
    middleware::Next,
    response::IntoResponse,
};

use crate::{
    auth::{AuthSession, BackendPgDB},
    objects::book::{get_book, BookRole, BookSubscription},
};

pub async fn require_member(
    Path(path): Path<HashMap<String, String>>,
    auth_session: AuthSession,
    mut request: Request,
    next: Next,
) -> impl IntoResponse {
    let Some(user) = auth_session.user else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let BackendPgDB(pool) = auth_session.backend;

    let Some(Ok(book_id)) = path.get("book_id").map(|id| id.parse()) else {
        return StatusCode::BAD_REQUEST.into_response();
    };

    let book_subscription = match get_book(user.id, book_id, &pool).await {
        Ok(BookSubscription {
            role: BookRole::Unauthorized,
            ..
        }) => return StatusCode::UNAUTHORIZED.into_response(),
        Err(_) => return (StatusCode::NOT_FOUND, "Where'd your book go?").into_response(), // TODO: Add funny 404 page
        Ok(user) => user,
    };

    request.extensions_mut().insert(book_subscription);
    next.run(request).await.into_response()
}
