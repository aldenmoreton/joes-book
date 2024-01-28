use axum::async_trait;
use axum_login::{AuthnBackend, UserId, AuthUser};
use sqlx::PgPool;
use serde::Deserialize;

#[derive(Clone, Debug)]
pub struct BackendUser {
	pub id: i64,
	pub username: String,
	pub pw_hash: String
}

#[derive(Deserialize)]
pub struct LoginCreds {
    pub username: String,
    pub password: String
}

impl AuthUser for BackendUser {
    type Id = i64;

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
    pub async fn signup(&self, username: &str, password: &str) -> Result<BackendUser, sqlx::Error>{
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
            .map(|result|
                BackendUser {
                    id: result.id,
                    username: result.username,
                    pw_hash: result.password
                }
            )
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
                pw_hash: record.password
            })),
            Ok(None) => Ok(None),
            Err(e) => Err(e)
        }
    }

    async fn get_user(
        &self,
        user_id: &UserId<Self>,
    ) -> Result<Option<Self::User>, Self::Error> {
        let pool = &self.0;

        sqlx::query!(
            "SELECT *
            FROM users
            WHERE id = $1",
            user_id
        )
            .fetch_optional(pool)
            .await
            .map(|result|
                result.map(|record|
                    BackendUser {
                        id: record.id,
                        username: record.username,
                        pw_hash: record.password
                    }
                )
            )
    }
}

pub mod authz {
    use axum::{extract::Request, middleware::Next, response::IntoResponse};

	pub async fn is_member(
		_request: Request,
		_next: Next
	) -> impl IntoResponse {
		todo!()
	}
}
