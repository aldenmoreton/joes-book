use axum_login::{
    tower_sessions::{cookie::time::Duration, Expiry, SessionManagerLayer},
    AuthManagerLayerBuilder,
};
use joes_book::{auth::BackendPgDB, router};
use sqlx::postgres::PgPoolOptions;

use tower_sessions::PostgresStore;

#[shuttle_runtime::main]
pub async fn shuttle(
    #[shuttle_runtime::Secrets] secrets: shuttle_runtime::SecretStore,
    #[shuttle_shared_db::Postgres(local_uri = "postgresql://postgres:postgres@localhost/new")]
    database_url: String,
) -> shuttle_axum::ShuttleAxum {
    let auth_layer = {
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

        AuthManagerLayerBuilder::new(backend, session_layer).build()
    };

    let state: joes_book::AppState = {
        let turnstile_site_key: String = secrets
            .get("TURNSTILE_SITE_KEY")
            .unwrap_or_else(|| "1x00000000000000000000AA".into());

        let turnstile_secret = secrets
            .get("TURNSTILE_SECRET_KEY")
            .unwrap_or_else(|| "1x0000000000000000000000000000000AA".into());

        joes_book::AppState {
            turnstile: joes_book::TurnstileState {
                site_key: turnstile_site_key,
                client: cf_turnstile::TurnstileClient::new(turnstile_secret.into()),
            },
        }
    };

    let app = router()
        .layer(auth_layer)
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .with_state(&*Box::leak(Box::new(state)));

    Ok(shuttle_axum::AxumService(app))
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
