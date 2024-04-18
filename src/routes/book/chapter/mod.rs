mod create;
mod page;

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::{self, Next},
    response::IntoResponse,
    routing::{get, post},
    Extension, Router,
};

use crate::objects::book::{BookRole, BookSubscription};

pub fn router() -> Router {
    Router::new()
        .route("/:chapter_id", get(page::handler))
        .route(
            "/create",
            post(create::handler).layer(middleware::from_fn(create_permissions)),
        )
}

async fn create_permissions(
    Extension(book_subscription): Extension<BookSubscription>,
    request: Request,
    next: Next,
) -> impl IntoResponse {
    if book_subscription.role != BookRole::Admin {
        return (
            StatusCode::UNAUTHORIZED,
            "You cannot create chapters for this book",
        )
            .into_response();
    }

    return next.run(request).await.into_response();
}
