// TODO: Refactor some routes to end with / so that they can more
// Simply route to the pages under them
use {
    crate::routes::*,
    auth::{authz, BackendPgDB},
    axum::{
        handler::Handler,
        middleware,
        response::{Html, IntoResponse, Redirect},
        routing::{get, post},
        Extension, Router,
    },
    axum_ctx::{RespErr, StatusCode},
    tower_http::services::ServeDir,
};

pub mod auth;
pub mod search;

pub mod routes {
    pub mod book;
    pub mod chapter;
    pub mod finish_signup;
    pub mod home;
    pub mod session;
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

type AppStateRef = &'static AppState;
pub struct AppState {
    pub pool: sqlx::PgPool,
    pub requests: reqwest::Client,
    pub turnstile: TurnstileState,
    pub google: GoogleState,
}

pub struct TurnstileState {
    pub site_key: String,
    pub client: cf_turnstile::TurnstileClient,
}

pub struct GoogleState {
    pub redirect_url: String,
    pub oauth: oauth2::Client<
        oauth2::StandardErrorResponse<oauth2::basic::BasicErrorResponseType>,
        oauth2::StandardTokenResponse<oauth2::EmptyExtraTokenFields, oauth2::basic::BasicTokenType>,
        oauth2::StandardTokenIntrospectionResponse<
            oauth2::EmptyExtraTokenFields,
            oauth2::basic::BasicTokenType,
        >,
        oauth2::StandardRevocableToken,
        oauth2::StandardErrorResponse<oauth2::RevocationErrorResponseType>,
        oauth2::EndpointSet,
        oauth2::EndpointNotSet,
        oauth2::EndpointNotSet,
        oauth2::EndpointNotSet,
        oauth2::EndpointSet,
    >,
}

pub fn router() -> Router<AppStateRef> {
    let site_admin_routes = Router::new().route(
        "/",
        get(|| async { Html("<p>You're on the admin page</p>") }),
    );

    let chapter_home_page =
        get(
            |auth_session: auth::AuthSession,
             Extension(book_subscription): Extension<db::book::BookSubscription>,
             Extension(chapter): Extension<db::chapter::Chapter>| async move {
                if chapter.is_open {
                    chapter::page::open_book(auth_session, &book_subscription, &chapter).await
                } else {
                    chapter::page::closed_book(auth_session, &book_subscription, &chapter).await
                }
            },
        )
        .post(chapter::page::submit.layer(middleware::from_fn(
            |Extension(chapter): Extension<db::chapter::Chapter>,
             request,
             next: middleware::Next| async move {
                if chapter.is_open {
                    Ok(next.run(request).await)
                } else {
                    Err(AppNotification(
                        StatusCode::LOCKED,
                        "This chapter is closed".into(),
                    ))
                }
            },
        )))
        .layer(middleware::from_fn(
            |Extension(chapter): Extension<db::chapter::Chapter>,
             Extension(book_subscription): Extension<db::book::BookSubscription>,
             request,
             next: middleware::Next| async move {
                match book_subscription.role {
                    db::book::BookRole::Owner | db::book::BookRole::Admin => {
                        Ok(next.run(request).await)
                    }
                    db::book::BookRole::Participant if chapter.is_visible => {
                        Ok(next.run(request).await)
                    }
                    db::book::BookRole::Guest { chapter_ids }
                        if chapter.is_visible && chapter_ids.contains(&chapter.chapter_id) =>
                    {
                        Ok(next.run(request).await)
                    }
                    _ => Err((StatusCode::UNAUTHORIZED, Redirect::to("/"))),
                }
            },
        ));

    let chapter_routes = Router::new()
        .nest(
            "/{chapter_id}/admin/",
            Router::new()
                .route(
                    "/",
                    get(chapter::admin::get)
                        .post(chapter::admin::post)
                        .delete(chapter::admin::delete),
                )
                .route("/user-input", get(chapter::admin::user_input))
                .route("/open", post(chapter::admin::open))
                .route("/visible", post(chapter::admin::visible))
                .route("/unsubmitted-users", get(chapter::admin::unsubmitted_users)),
        )
        .route_layer(middleware::from_fn(book::mw::require_admin))
        .route("/{chapter_id}/", chapter_home_page)
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
        .nest("/{book_id}/chapter/", chapter_routes)
        .nest(
            "/{book_id}/admin/",
            Router::new()
                .route("/", get(book::admin::handler).delete(book::admin::delete))
                .route("/user-search", get(book::admin::search_user))
                .route("/add-user", post(book::admin::add_user))
                .route("/remove-user", post(book::admin::remove_user))
                .route_layer(middleware::from_fn(book::mw::require_admin)),
        )
        .route("/{book_id}/leaderboard", get(book::page::leaderboard))
        .route("/{book_id}/", get(book::page::handler))
        .route_layer(middleware::from_fn(book::mw::require_member))
        .route(
            "/create",
            post(book::create::handler).layer(middleware::from_fn(authz::mw::require_site_admin)),
        );

    let home_routes = Router::new()
        .route("/logout", post(session::logout))
        .route("/", get(home::handler));

    let session_routes = Router::new()
        .route("/api/auth/google", get(session::google::google_oauth))
        .route(
            "/finish-signup",
            get(finish_signup::get).post(finish_signup::post),
        )
        // .route(
        //     "/signup",
        //     get(crate::signup::signup_page).post(crate::signup::signup_form),
        // )
        // .route(
        //     "/legacy-login",
        //     get(crate::session::legacy_login_page).post(crate::session::legacy_login_form),
        // )
        // .route(
        //     "/login/explaination",
        //     get(crate::session::login_explaination),
        // )
        .route("/login", get(crate::session::login_page))
        .route_layer(middleware::from_fn(
            |auth_session: auth::AuthSession, request, next: middleware::Next| async move {
                if auth_session.user.is_some() {
                    return axum::response::Redirect::to("/").into_response();
                }
                next.run(request).await.into_response()
            },
        ));

    Router::new()
        .nest("/admin", site_admin_routes)
        .nest("/book", book_routes)
        .merge(home_routes)
        .route("/team-search", get(search::team))
        // ------------------^ Logged in Routes ^------------------
        .route_layer(axum_login::login_required!(
            BackendPgDB,
            login_url = "/login"
        ))
        .nest_service("/public", ServeDir::new("public"))
        .merge(session_routes)
        .fallback(get((StatusCode::NOT_FOUND, "Could not find your route"))) // TODO: Add funny status page
}

#[derive(Debug, thiserror::Error)]
pub enum AppError<'a> {
    #[error("No Backend User")]
    BackendUser,
    #[error("Unauthorized: {0}")]
    Unauthorized(&'a str),
    #[error("Parsing: {0}")]
    Parse(&'a str),
    #[error("Database Error: {0}")]
    Sqlx(#[from] sqlx::Error),
}

impl From<AppError<'_>> for RespErr {
    fn from(value: AppError) -> Self {
        match &value {
            AppError::BackendUser => {
                RespErr::new(StatusCode::INTERNAL_SERVER_ERROR).log_msg(value.to_string())
            }
            AppError::Unauthorized(_) => RespErr::new(StatusCode::UNAUTHORIZED)
                .user_msg(value.to_string())
                .log_msg(value.to_string()),
            AppError::Parse(_) => RespErr::new(StatusCode::BAD_REQUEST)
                .user_msg(value.to_string())
                .log_msg(value.to_string()),
            AppError::Sqlx(_) => {
                RespErr::new(StatusCode::INTERNAL_SERVER_ERROR).log_msg(value.to_string())
            }
        }
    }
}

impl axum::response::IntoResponse for AppError<'_> {
    fn into_response(self) -> axum::response::Response {
        RespErr::from(self).into_response()
    }
}

pub struct AppNotification(StatusCode, String);

impl axum::response::IntoResponse for AppNotification {
    fn into_response(self) -> axum::response::Response {
        (
            self.0,
            [("HX-Retarget", "body"), ("HX-Reswap", "beforeend")],
            maud::html! {
                script {
                    "alertify.set('notifier', 'position', 'top-center');"
                    @if self.0.is_success() {
                        "alertify.success("(maud::PreEscaped("\"")) (maud::PreEscaped(self.1)) (maud::PreEscaped("\""))");"
                    } @else if self.0.is_server_error() {
                        "alertify.error('Our Fault! Please Try Again.');"
                    } @else {
                        "alertify.error("(maud::PreEscaped("\"")) (maud::PreEscaped(self.1)) (maud::PreEscaped("\""))");"
                    }
                }
            },
        )
            .into_response()
    }
}

impl From<RespErr> for AppNotification {
    fn from(value: RespErr) -> Self {
        let text = value.to_string();
        let status = value.status_code;

        let _ = value.into_response();

        AppNotification(status, text)
    }
}

impl From<AppError<'_>> for AppNotification {
    fn from(value: AppError) -> Self {
        AppNotification::from(RespErr::from(value))
    }
}
