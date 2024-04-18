use axum::{async_trait, http::StatusCode, response::IntoResponse};
use axum_login::{AuthSession as AxumLoginAuthSession, AuthUser, AuthnBackend, UserId};
use serde::Deserialize;
use sqlx::PgPool;

use crate::auth::authz::add_perm;

pub type AuthSession = AxumLoginAuthSession<BackendPgDB>;

#[derive(Clone, Debug)]
pub struct BackendUser {
    pub id: i32,
    pub username: String,
    pub pw_hash: String,
}

#[derive(Deserialize)]
pub struct LoginCreds {
    pub username: String,
    pub password: String,
}

impl AuthUser for BackendUser {
    type Id = i32;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.pw_hash.as_bytes()
    }
}

#[derive(Clone)]
pub struct BackendPgDB(pub PgPool);

impl BackendPgDB {
    pub async fn init_admin(&self) -> Result<Option<BackendUser>, sqlx::Error> {
        let Ok(username) = std::env::var("OWNER_USERNAME") else {
            return Ok(None);
        };
        let Ok(password) = std::env::var("OWNER_PASSWORD") else {
            return Ok(None);
        };

        let user = self.signup(&username, &password).await?;
        add_perm(user.id, "admin", &self.0).await?;

        Ok(Some(user))
    }

    pub async fn signup(&self, username: &str, password: &str) -> Result<BackendUser, sqlx::Error> {
        let password_hashed = bcrypt::hash(password, bcrypt::DEFAULT_COST).unwrap();

        sqlx::query!(
            r#" INSERT INTO users (username, password)
                VALUES ($1, $2)
                ON CONFLICT (username) DO NOTHING
                RETURNING *"#,
            username,
            password_hashed
        )
        .fetch_one(&self.0)
        .await
        .map(|result| BackendUser {
            id: result.id,
            username: result.username,
            pw_hash: result.password,
        })
    }
}

#[async_trait]
impl AuthnBackend for BackendPgDB {
    type User = BackendUser;
    type Credentials = LoginCreds;
    type Error = sqlx::Error;

    async fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        let res = sqlx::query!(
            "SELECT *
            FROM users
            WHERE username = $1",
            creds.username
        )
        .fetch_optional(&self.0)
        .await;

        match res {
            Ok(Some(record)) => Ok(Some(BackendUser {
                id: record.id,
                username: record.username,
                pw_hash: record.password,
            })),
            Ok(None) => Ok(None),
            Err(e) => Err(e),
        }
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        let pool = &self.0;

        sqlx::query!(
            "SELECT *
            FROM users
            WHERE id = $1",
            user_id
        )
        .fetch_optional(pool)
        .await
        .map(|result| {
            result.map(|record| BackendUser {
                id: record.id,
                username: record.username,
                pw_hash: record.password,
            })
        })
    }
}

pub async fn logout(mut auth_session: self::AuthSession) -> impl IntoResponse {
    let res = auth_session.logout().await;

    match res {
        Ok(_) => [("HX-Redirect", "/login")].into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub mod authz {
    use std::collections::HashMap;

    use axum::{
        extract::{Path, Request},
        http::StatusCode,
        middleware::Next,
        response::IntoResponse,
    };
    use sqlx::PgPool;

    use crate::objects::book::{get_book, BookRole, BookSubscription};

    use super::{AuthSession, BackendPgDB};

    pub async fn add_perm(user_id: i32, perm: &str, pool: &PgPool) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            "INSERT INTO user_permissions (user_id, token)
            VALUES ($1, $2)
            ON CONFLICT (user_id, token) DO NOTHING",
            user_id,
            perm
        )
        .fetch_optional(pool)
        .await;

        result.map(|r| r.is_some())
    }

    pub async fn has_perm(perm: &str, user_id: i32, pool: &PgPool) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            "SELECT token
            FROM user_permissions
            WHERE user_id = $1 AND token = $2",
            user_id,
            perm
        )
        .fetch_optional(pool)
        .await;

        result.map(|r| r.is_some())
    }

    pub async fn is_member(
        Path(path): Path<HashMap<String, String>>,
        auth_session: AuthSession,
        mut request: Request,
        next: Next,
    ) -> impl IntoResponse {
        let Some(user) = auth_session.user else {
            return StatusCode::UNAUTHORIZED.into_response();
        };
        let BackendPgDB(pool) = auth_session.backend;

        let Some(Ok(book_id)) = path.get("book_id").map(|id| id.parse()) else {
            return StatusCode::BAD_REQUEST.into_response();
        };

        let book_subscription = match get_book(user.id, book_id, &pool).await {
            Ok(BookSubscription {
                role: BookRole::Unauthorized,
                ..
            }) => return StatusCode::UNAUTHORIZED.into_response(),
            Err(_) => return (StatusCode::NOT_FOUND, "Where'd your book go?").into_response(), // TODO: Add funny 404 page
            Ok(user) => user,
        };

        request.extensions_mut().insert(book_subscription);
        next.run(request).await.into_response()
    }
}