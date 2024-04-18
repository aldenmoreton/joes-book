mod chapter;
mod create;
mod page;

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::{self, Next},
    response::IntoResponse,
    routing::{get, post, Router},
};

use crate::auth::{
    authz::{self, has_perm},
    AuthSession,
};

pub fn router() -> Router {
    Router::new()
        .nest("/:book_id/chapter", chapter::router())
        .route("/:book_id", get(page::handler))
        .route_layer(middleware::from_fn(authz::is_member))
        .route(
            "/create",
            post(create::handler).layer(middleware::from_fn(create_permissions)),
        )
}

async fn create_permissions(
    auth_session: AuthSession,
    request: Request,
    next: Next,
) -> impl IntoResponse {
    let user = auth_session.user.unwrap();
    let pool = auth_session.backend.0;

    match has_perm("admin", user.id, &pool).await {
        Ok(true) => (),
        Ok(false) => {
            return (
                StatusCode::UNAUTHORIZED,
                "You do not have permission to create a book",
            )
                .into_response()
        }
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }

    return next.run(request).await;
}
