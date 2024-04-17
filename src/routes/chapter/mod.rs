mod create;
mod page;

use axum::{
    routing::{get, post},
    Router,
};

pub fn router() -> Router {
    Router::new()
        .route("/:chapter_id", get(page::handler))
        .route("/create", post(create::handler))
}
