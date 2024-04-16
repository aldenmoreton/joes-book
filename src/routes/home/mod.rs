use axum::{
    routing::{get, post},
    Router,
};

use crate::{auth, components, routes};

mod page;

pub fn router() -> Router {
    Router::new()
        .route("/logout", post(auth::logout))
        .nest("/nav", components::nav::router())
        .nest("/home", routes::home::page::router())
        .route("/", get(routes::home::page::handler))
}
