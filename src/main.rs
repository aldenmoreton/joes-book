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
    use joes_book::server::*;
    use joes_book::objects::*;
    use joes_book::state::AppState;
    use joes_book::fallback::file_and_error_handler;
    use leptos_axum::{generate_route_list, LeptosRoutes, handle_server_fns_with_context};
    use leptos::{view, provide_context, get_configuration};
    use sqlx::{PgPool, postgres::PgPoolOptions};
    use axum_session::{SessionConfig, SessionLayer, SessionStore};
    use axum_session_auth::{AuthSessionLayer, AuthConfig, SessionPgPool};

    async fn server_fn_handler(
        State(app_state): State<AppState>,
        auth_session: AuthSession,
        path: Path<String>,
        headers: HeaderMap,
        raw_query: RawQuery,
        request: Request<AxumBody>
    ) -> impl IntoResponse {

        let response = handle_server_fns_with_context(path, headers, raw_query, move || {
            provide_context(auth_session.clone());
            provide_context(app_state.pool.clone());
        }, request).await.into_response();

        response
    }

    async fn secure_server_fn_handler(
        State(app_state): State<AppState>,
        auth_session: AuthSession,
        path: Path<String>,
        headers: HeaderMap,
        raw_query: RawQuery,
        request: Request<AxumBody>
    ) -> impl IntoResponse {

        if !auth_session.is_authenticated() {
            return StatusCode::BAD_REQUEST.into_response()
        }

        handle_server_fns_with_context(
            path,
            headers,
            raw_query,
            move || {
                provide_context(auth_session.clone());
                provide_context(app_state.pool.clone());
            },
            request
        ).await.into_response()
    }

    async fn leptos_routes_handler(
        auth_session: AuthSession,
        State(app_state): State<AppState>,
        req: Request<AxumBody>
    ) -> Response{
        let authenticated = auth_session.is_authenticated();
        let uncontrolled_route = req.uri() == "/login" || req.uri() == "/signup";

        let handler = leptos_axum::render_app_to_stream_with_context(
            app_state.leptos_options.clone(),
            move || {
                provide_context(auth_session.clone());
                provide_context(app_state.pool.clone());
            },
            || view!{ <App/> }
        );

        match (authenticated, uncontrolled_route) {
            (true, true) => Redirect::to("/").into_response(),
            (false, false) => Redirect::to("/login").into_response(),
            (true, false) | (false, true) => handler(req).await.into_response(),
        }
    }

    #[tokio::main]
    async fn main() {
        let _ = dotenvy::dotenv();

        simple_logger::init_with_level(log::Level::Info).expect("couldn't initialize logging");

        let database_url = std::env::var("DATABASE_URL").expect("Unable to read DATABASE_URL env var");

        let pool = PgPoolOptions::new()
            .connect(&database_url)
            .await
            .expect("Could not make pool.");

        // Auth section
        let session_config = SessionConfig::default().with_table_name("axum_sessions");

        println!("Attempting Migration");
        // sqlx::migrate!(migrations);
        sqlx::query_file!("migrations/users.sql").execute(&pool).await.ok();
        sqlx::query_file!("migrations/user_permissions.sql").execute(&pool).await.ok();
        sqlx::query_file!("migrations/todos.sql").execute(&pool).await.ok();
        sqlx::query_file!("migrations/books.sql").execute(&pool).await.ok();
        sqlx::query_file!("migrations/subscriptions.sql").execute(&pool).await.ok();
        sqlx::query_file!("migrations/chapters.sql").execute(&pool).await.ok();
        sqlx::query_file!("migrations/events.sql").execute(&pool).await.ok();
        sqlx::query_file!("migrations/picks.sql").execute(&pool).await.ok();

        let auth_config = AuthConfig::<i64>::default();
        let session_store = SessionStore::<SessionPgPool>::new(Some(pool.clone().into()), session_config)
            .await
            .expect("Could not create session store");

        BackendUser::add_to_db(
            std::env::var("OWNER_USERNAME").expect("Unable to read OWNER_USERNAME env var"),
            std::env::var("OWNER_PASSWORD").expect("Unable to read OWNER_PASSWORD env var"),
            vec!["owner".into()],
            pool.clone()
        ).await.expect("Unable to create owner");

        let conf = get_configuration(None).await.unwrap();
        let leptos_options = conf.leptos_options;
        let addr = leptos_options.site_addr;
        let routes = generate_route_list(|| view!{ <App/> });

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
            .layer(AuthSessionLayer::<BackendUser, i64, SessionPgPool, PgPool>::new(Some(pool.clone()))
            .with_config(auth_config))
            .layer(SessionLayer::new(session_store))
            .with_state(app_state);

        // Run App
        // log!("listening on http://{}", &addr);
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
