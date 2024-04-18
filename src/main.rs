use axum::{routing::get, Router};

use axum_login::{
    login_required,
    tower_sessions::{cookie::time::Duration, Expiry, SessionManagerLayer},
    AuthManagerLayerBuilder,
};
use joes_book::{auth::BackendPgDB, routes};
use sqlx::{postgres::PgPoolOptions, PgPool};

use tower_http::services::ServeDir;
use tower_sessions::PostgresStore;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("Unable to read DATABASE_URL ENV");

    let pool = PgPoolOptions::new()
        .connect(&database_url)
        .await
        .expect("Could not make pool.");

    let backend = BackendPgDB(pool.clone());
    backend.init_admin().await.ok();

    let session_store = PostgresStore::new(pool);
    session_store
        .migrate()
        .await
        .expect("Could not migrate database");
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_expiry(Expiry::OnInactivity(Duration::weeks(2)));
    let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();

    let app = Router::new()
        // Site Admin Routes
        .nest("/admin", routes::admin::router())
        // ------------------
        // Book Member Routes
        .nest("/book", routes::book::router())
        // ------------------
        // Logged in Routes
        .merge(routes::home::router())
        // ------------------
        .nest_service("/assets", ServeDir::new("assets"))
        .route_layer(login_required!(BackendPgDB, login_url = "/login"))
        .nest_service("/public", ServeDir::new("public"))
        .nest("/signup", routes::signup::router())
        .nest("/login", routes::login::router())
        .layer(auth_layer)
        .fallback(get(|| async { "Could not find your route" })); // TODO: Add funny status page

    println!();
    println!("Starting server at http://localhost:3000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
