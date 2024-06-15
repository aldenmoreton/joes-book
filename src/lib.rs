// TODO: Refactor some routes to end with / so that they can more
// Simply route to the pages under them
use auth::{authz, BackendPgDB};
use axum::{
    middleware,
    response::Html,
    routing::{get, post},
    Router,
};
use axum_login::{login_required, AuthManagerLayer};
use tower_http::services::ServeDir;
use tower_sessions::PostgresStore;

use crate::components::*;
use crate::routes::*;

pub mod auth;

pub mod routes {
    pub mod book;
    pub mod login;
    pub mod signup;
    pub mod home {
        pub mod page;
    }
}

pub mod components {
    pub mod book_list;
    pub mod nav;
}

pub mod objects {
    pub mod book;
    pub mod chapter;
    pub mod event;
    pub mod spread;
    pub mod user_input;
}

pub fn router(auth_layer: AuthManagerLayer<BackendPgDB, PostgresStore>) -> Router {
    let admin_routes = Router::new().route(
        "/",
        get(|| async { Html("<p>You're on the admin page</p>") }),
    );

    let chapter_routes = Router::new()
        .route(
            "/:chapter_id/admin/update",
            post(book::chapter::admin::update),
        )
        .route(
            "/:chapter_id/admin/create/validate",
            post(book::chapter::admin::validate_new_chapter),
        )
        .route(
            "/:chapter_id/admin/create/add",
            get(book::chapter::admin::add_event),
        )
        .route(
            "/:chapter_id/admin/create/",
            get(book::chapter::admin::create),
        )
        .route("/:chapter_id/admin/", get(book::chapter::admin::handler))
        .route_layer(middleware::from_fn(book::chapter::require_admin))
        .route("/:chapter_id/", get(book::chapter::page::handler))
        .route_layer(middleware::from_fn(book::chapter::chapter_ext))
        .route(
            "/create",
            post(book::chapter::create::handler)
                .layer(middleware::from_fn(book::chapter::require_admin)),
        );

    let book_routes = Router::new()
        .nest("/:book_id/chapter", chapter_routes)
        .route("/:book_id", get(book::page::handler))
        .route_layer(middleware::from_fn(book::require_member))
        .route(
            "/create",
            post(book::create::handler).layer(middleware::from_fn(authz::require_site_admin)),
        );

    let home_routes = Router::new()
        .route("/logout", post(auth::logout))
        .nest("/nav", Router::new().route("/", get(nav::user)))
        .nest(
            "/home",
            Router::new()
                .route("/logout", post(auth::logout))
                .nest("/nav", Router::new().route("/", get(nav::user))),
        )
        .route("/", get(home::page::handler));

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
        .fallback(get(|| async { "Could not find your route" })) // TODO: Add funny status page
}
