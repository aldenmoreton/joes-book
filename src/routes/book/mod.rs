mod chapter;
mod create;
mod page;

use axum::{
    middleware,
    routing::{get, post, Router},
};

use crate::auth::authz;

pub fn router() -> Router {
    Router::new()
        .nest("/:book_id/chapter", chapter::router())
        .route("/:book_id", get(page::handler))
        .route_layer(middleware::from_fn(authz::is_member))
        .route("/create", post(create::handler))
}
