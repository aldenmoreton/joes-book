use cfg_if::cfg_if;

cfg_if! {
if #[cfg(feature = "ssr")] {
    use axum::{
        response::{Response, IntoResponse, Redirect},
        routing::get,
        extract::{Path, State, RawQuery},
        http::{Request, header::HeaderMap, StatusCode},
        body::Body as AxumBody,
        Router,
    };
    use joes_book::app::*;
    use joes_book::auth::*;
    use joes_book::state::AppState;
    use joes_book::fallback::file_and_error_handler;
    use leptos_axum::{generate_route_list, LeptosRoutes, handle_server_fns_with_context};
    use leptos::{log, view, provide_context, get_configuration};
    use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
    use axum_session::{SessionConfig, SessionLayer, SessionStore};
    use axum_session_auth::{AuthSessionLayer, AuthConfig, SessionSqlitePool};

    async fn server_fn_handler(State(app_state): State<AppState>, auth_session: AuthSession, path: Path<String>, headers: HeaderMap, raw_query: RawQuery,
    request: Request<AxumBody>) -> impl IntoResponse {

        log!("{:?}", path);

        handle_server_fns_with_context(path, headers, raw_query, move |cx| {
            provide_context(cx, auth_session.clone());
            provide_context(cx, app_state.pool.clone());
        }, request).await
    }

    async fn secure_server_fn_handler(State(app_state): State<AppState>, auth_session: AuthSession, path: Path<String>, headers: HeaderMap, raw_query: RawQuery,
    request: Request<AxumBody>) -> impl IntoResponse {

        log!("--------------Secure API {:?}", path);

        if !auth_session.is_authenticated() {
            return StatusCode::BAD_REQUEST.into_response()
        }

        handle_server_fns_with_context(path, headers, raw_query, move |cx| {
            provide_context(cx, auth_session.clone());
            provide_context(cx, app_state.pool.clone());
        }, request).await.into_response()
    }

    async fn leptos_routes_handler(auth_session: AuthSession, State(app_state): State<AppState>, req: Request<AxumBody>) -> Response{
        let authenticated = auth_session.is_authenticated();
        let uncontrolled_route = req.uri() == "/login" || req.uri() == "/signup";

        let handler = leptos_axum::render_app_to_stream_with_context(
            app_state.leptos_options.clone(),
            move |cx| {
                provide_context(cx, auth_session.clone());
                provide_context(cx, app_state.pool.clone());
            },
            |cx| view! { cx, <App/> }
        );

        match (authenticated, uncontrolled_route) {
            (true, true) => Redirect::to("/").into_response(),
            (false, false) => Redirect::to("/login").into_response(),
            (true, false) | (false, true) => handler(req).await.into_response(),
        }
    }

    #[tokio::main]
    async fn main() {
        simple_logger::init_with_level(log::Level::Info).expect("couldn't initialize logging");

        let pool = SqlitePoolOptions::new()
            .connect("sqlite:Joe.db")
            .await
            .expect("Could not make pool.");

        // Auth section
        let session_config = SessionConfig::default().with_table_name("axum_sessions");
        let auth_config = AuthConfig::<i64>::default();
        let session_store = SessionStore::<SessionSqlitePool>::new(Some(pool.clone().into()), session_config);
        session_store.initiate().await.unwrap();

        sqlx::migrate!()
            .run(&pool)
            .await
            .expect("could not run SQLx migrations");

        let conf = get_configuration(None).await.unwrap();
        let leptos_options = conf.leptos_options;
        let addr = leptos_options.site_addr;
        let routes = generate_route_list(|cx| view! { cx, <App/> }).await;

        let app_state = AppState{
            leptos_options,
            pool: pool.clone(),
        };

        // build our application with a route
        let app = Router::new()
            .route("/api/*fn_name", get(server_fn_handler).post(server_fn_handler))
            .route("/secure/*fn_name", get(secure_server_fn_handler).post(secure_server_fn_handler))
            .leptos_routes_with_handler(routes, get(leptos_routes_handler))
            .fallback(file_and_error_handler)
            .layer(AuthSessionLayer::<User, i64, SessionSqlitePool, SqlitePool>::new(Some(pool.clone()))
            .with_config(auth_config))
            .layer(SessionLayer::new(session_store))
            .with_state(app_state);

        // Run App
        log!("listening on http://{}", &addr);
        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await
            .unwrap();
    }
}

    // client-only stuff for Trunk
    else {
        pub fn main() {
            // This example cannot be built as a trunk standalone CSR-only app.
            // Only the server may directly connect to the database.
        }
    }
}
