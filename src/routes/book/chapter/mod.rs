pub mod admin;
pub mod create;
pub mod page;

use axum::{
    extract::Request, http::StatusCode, middleware::Next, response::IntoResponse, Extension,
};

use crate::objects::book::{BookRole, BookSubscription};

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
