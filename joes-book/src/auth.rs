use async_trait::async_trait;
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
    #[serde(rename = "cf-turnstile-response")]
    pub turnstile_response: String,
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
        .fetch_optional(&self.0)
        .await
        .and_then(|result| {
            result
                .map(|record| {
                    Ok(BackendUser {
                        id: record.id,
                        username: record.username,
                        pw_hash: record.password,
                    })
                })
                .unwrap_or(Err(sqlx::Error::RowNotFound))
        })
    }
}

pub struct UserCredentials {
    pub username: String,
    pub password: String,
}

#[async_trait]
impl AuthnBackend for BackendPgDB {
    type User = BackendUser;
    type Credentials = UserCredentials;
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
        .await?;

        let Some(user) = res else { return Ok(None) };

        if !bcrypt::verify(creds.password, &user.password).unwrap_or(false) {
            return Ok(None);
        };

        Ok(Some(BackendUser {
            id: user.id,
            username: user.username,
            pw_hash: user.password,
        }))
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

pub mod authz {
    pub async fn add_perm(
        user_id: i32,
        perm: &str,
        pool: &sqlx::PgPool,
    ) -> Result<bool, sqlx::Error> {
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

    pub async fn has_perm(
        perm: &str,
        user_id: i32,
        pool: &sqlx::PgPool,
    ) -> Result<bool, sqlx::Error> {
        sqlx::query!(
            "SELECT token
            FROM user_permissions
            WHERE user_id = $1 AND token = $2",
            user_id,
            perm
        )
        .fetch_optional(pool)
        .await
        .map(|r| r.is_some())
    }

    pub mod mw {
        use axum::{body::Body, extract::Request, http::Response, middleware::Next};

        use crate::{auth::AuthSession, AppError};

        use super::has_perm;

        pub async fn require_site_admin(
            auth_session: AuthSession,
            request: Request,
            next: Next,
        ) -> Result<Response<Body>, AppError<'static>> {
            let user = auth_session.user.ok_or(AppError::BackendUser)?;
            let pool = auth_session.backend.0;

            if !has_perm("admin", user.id, &pool).await? {
                return Err(AppError::Unauthorized(
                    "You don't have permission to create a book",
                ));
            }

            Ok(next.run(request).await)
        }
    }
}
