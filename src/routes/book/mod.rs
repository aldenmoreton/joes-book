mod endpoints;
mod page;

use axum::routing::{get, post, Router};

pub fn router() -> Router {
    Router::new()
        .route("/create", post(endpoints::create))
        .route("/:book_id/", get(page::handler))
}
