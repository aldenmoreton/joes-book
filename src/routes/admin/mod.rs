use axum::{response::Html, routing::get, Router};

pub fn router() -> Router {
    Router::new().route(
        "/",
        get(|| async { Html("<p>You're on the admin page</p>") }),
    )
}
