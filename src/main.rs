use axum::{Router, middleware, routing::{get, post}};

use tower_http::services::ServeDir;
use tower_sessions::PostgresStore;
use axum_login::{AuthManagerLayerBuilder, login_required, tower_sessions::{SessionManagerLayer, cookie::time::Duration, Expiry}};
use sqlx::{postgres::PgPoolOptions, PgPool};

use joes_book::{pages, components, auth::{BackendPgDB, authz}};

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().expect("Could not get ENV VARS using dotenvy");
    let database_url = std::env::var("DATABASE_URL").expect("Unable to read DATABASE_URL ENV");

    let pool = PgPoolOptions::new()
        .connect(&database_url)
        .await
        .expect("Could not make pool.");
    // println!("Migrating DB...");
    // migrate(&pool).await;
    // println!("DB Migrated");


    let backend = BackendPgDB(pool.clone());
    // println!("Adding root user");
    // let res = backend.signup("owner", "123").await;
    // if res.is_ok() {
    //     println!("Root user added successfully")
    // } else {
    //     println!("Root user already added")
    // }

    let session_store = PostgresStore::new(pool);
    session_store.migrate().await.expect("Could not migrate database");
    let session_layer = SessionManagerLayer::new(session_store)
            .with_secure(false)
            .with_expiry(Expiry::OnInactivity(Duration::weeks(2)));
    let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer)
        .build();

    let app = Router::new()
        .nest("/book/:id", pages::book::router())
        .route_layer(middleware::from_fn(authz::is_member))
        .nest("/nav", components::nav::router())
        .nest("/home", pages::home::router())
        .route("/", get(pages::home::home))
        .nest_service("/assets", ServeDir::new("assets"))
        .route("/logout", post(joes_book::auth::logout))
        .route_layer(login_required!(BackendPgDB, login_url="/login"))
        .nest_service("/public", ServeDir::new("public"))
        .nest("/signup", pages::signup::router())
        .nest("/login", pages::login::router())
        .layer(auth_layer)
        .fallback(get(|| async {"Could not find your route"}));
        // .with_state(AppState{db: pool});


    println!();
    println!("Starting server at http://localhost:3000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    ()
}

async fn migrate(pool: &PgPool) {
    sqlx::query_file!("migrations/users.sql").execute(pool).await.ok();
    sqlx::query_file!("migrations/user_permissions.sql").execute(pool).await.ok();
    sqlx::query_file!("migrations/todos.sql").execute(pool).await.ok();
    sqlx::query_file!("migrations/books.sql").execute(pool).await.ok();
    sqlx::query_file!("migrations/subscriptions.sql").execute(pool).await.ok();
    sqlx::query_file!("migrations/chapters.sql").execute(pool).await.ok();
    sqlx::query_file!("migrations/events.sql").execute(pool).await.ok();
    sqlx::query_file!("migrations/picks.sql").execute(pool).await.ok();
}
