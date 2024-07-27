// TODO: Refactor some routes to end with / so that they can more
// Simply route to the pages under them
use auth::{authz, BackendPgDB};
use axum::{
    middleware,
    response::Html,
    routing::{get, post},
    Router,
};
use axum_ctx::{RespErr, StatusCode};
use axum_login::{login_required, AuthManagerLayer};
use tower_http::services::ServeDir;
use tower_sessions::PostgresStore;

use crate::routes::*;

pub mod auth;
pub mod team_search;

pub mod routes {
    pub mod book;
    pub mod chapter;
    pub mod home;
    pub mod login;
    pub mod signup;
}

pub mod db {
    pub mod book;
    pub mod chapter;
    pub mod event;
    pub mod spread;
    pub mod team;
    pub mod user_input;
}

pub mod templates;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("No Backend User")]
    BackendUser,
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    #[error("Parsing: {0}")]
    Parse(String),
    #[error("Database Error: {0}")]
    Sqlx(#[from] sqlx::Error),
}

impl From<AppError> for RespErr {
    fn from(value: AppError) -> Self {
        match &value {
            AppError::BackendUser => RespErr::new(StatusCode::INTERNAL_SERVER_ERROR)
                .log_msg(value.to_string())
                .user_msg("Could not get user account"),
            AppError::Unauthorized(_) => RespErr::new(StatusCode::UNAUTHORIZED)
                .user_msg(value.to_string())
                .log_msg(value.to_string()),
            AppError::Parse(_) => RespErr::new(StatusCode::BAD_REQUEST)
                .user_msg(value.to_string())
                .log_msg(value.to_string()),
            AppError::Sqlx(_) => RespErr::new(StatusCode::INTERNAL_SERVER_ERROR)
                .user_msg("Database Error")
                // .user_msg(value.to_string())
                .log_msg(value.to_string()),
        }
    }
}

pub fn router(auth_layer: AuthManagerLayer<BackendPgDB, PostgresStore>) -> Router {
    let admin_routes = Router::new().route(
        "/",
        get(|| async { Html("<p>You're on the admin page</p>") }),
    );

    let chapter_routes = Router::new()
        .nest(
            "/:chapter_id/admin/",
            Router::new()
                .route("/", get(chapter::admin::handler).post(chapter::admin::post))
                .route("/user-input", get(chapter::admin::user_input)),
        )
        .route_layer(middleware::from_fn(book::mw::require_admin))
        .route(
            "/:chapter_id/",
            get(chapter::page::handler).post(chapter::page::submit),
        )
        .route_layer(middleware::from_fn(chapter::mw::chapter_ext))
        .nest(
            "/create/",
            Router::new()
                .route("/", get(chapter::create::get).post(chapter::create::post))
                .route("/add", get(chapter::create::add_event))
                .route("/team-select", post(chapter::create::team_select))
                .route_layer(middleware::from_fn(book::mw::require_admin)),
        );

    let book_routes = Router::new()
        .nest("/:book_id/chapter/", chapter_routes)
        .nest(
            "/:book_id/admin/",
            Router::new()
                .route("/", get(book::admin::handler))
                .route_layer(middleware::from_fn(book::mw::require_admin)),
        )
        .route("/:book_id/", get(book::page::handler))
        .route_layer(middleware::from_fn(book::mw::require_member))
        .route(
            "/create",
            post(book::create::handler).layer(middleware::from_fn(authz::mw::require_site_admin)),
        );

    let home_routes = Router::new()
        .route("/logout", post(auth::logout))
        // .nest("/nav", Router::new().route("/", get(nav::user)))
        .nest(
            "/home",
            Router::new().route("/logout", post(auth::logout)), // .nest("/nav", Router::new().route("/", get(nav::user))),
        )
        .route("/", get(home::handler));

    Router::new()
        // v Site Admin Routes v
        .nest("/admin", admin_routes)
        // ^ Site Admin Routes ^
        // v Book Member Routes v
        .nest("/book", book_routes)
        // ^ Book Member Routes ^
        // v Home Routes v
        .merge(home_routes)
        // ^ Home Routes ^
        .route("/team-search", get(team_search::handler))
        .nest_service("/assets", ServeDir::new("assets"))
        // ------------------^ Logged in Routes ^------------------
        .route_layer(login_required!(BackendPgDB, login_url = "/login"))
        .nest_service("/public", ServeDir::new("public"))
        .nest(
            "/signup",
            Router::new().route("/", get(|| async { "No signup page yet" })),
        )
        .nest(
            "/login",
            Router::new().route(
                "/",
                get(crate::login::login_page).post(crate::login::login_form),
            ),
        )
        .layer(auth_layer)
        .fallback(get(|| async {
            (StatusCode::NOT_FOUND, "Could not find your route")
        })) // TODO: Add funny status page
}
