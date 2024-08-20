use axum_login::{
    tower_sessions::{cookie::time::Duration, Expiry, SessionManagerLayer},
    AuthManagerLayerBuilder,
};
use joes_book::{auth::BackendPgDB, router};
use sqlx::{postgres::PgPoolOptions, PgPool};

use tower_sessions::PostgresStore;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}

#[shuttle_runtime::main]
pub async fn shuttle(
    #[shuttle_shared_db::Postgres(local_uri = "postgresql://postgres:postgres@localhost/new")]
    database_url: String,
) -> shuttle_axum::ShuttleAxum {
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

    Ok(shuttle_axum::AxumService(router(auth_layer)))
}

// #[tokio::main]
// async fn main() {
//     dotenvy::dotenv().ok();
//     let database_url = std::env::var("DATABASE_URL").expect("Unable to read DATABASE_URL ENV");

//     let pool = PgPoolOptions::new()
//         .connect(&database_url)
//         .await
//         .expect("Could not make pool.");

//     let backend = BackendPgDB(pool.clone());
//     backend.init_admin().await.ok();

//     let session_store = PostgresStore::new(pool);
//     session_store
//         .migrate()
//         .await
//         .expect("Could not migrate database");
//     let session_layer = SessionManagerLayer::new(session_store)
//         .with_secure(false)
//         .with_expiry(Expiry::OnInactivity(Duration::weeks(2)));
//     let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();

//     let app = router(auth_layer);

//     println!();
//     println!("Starting server at http://localhost:3000");
//     let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
//     axum::serve(listener, app).await.unwrap();
// }
