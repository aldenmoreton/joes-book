use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use leptos::*;
        use std::collections::HashSet;
        use crate::objects::BackendUser;
        use async_trait::async_trait;
        use sqlx::PgPool;
        use axum_session_auth::{SessionPgPool, Authentication, HasPermission as HasPerm};
        pub type AuthSession = axum_session_auth::AuthSession<BackendUser, i32, SessionPgPool, PgPool>;

        impl BackendUser {
            pub async fn add_to_db(username: String, password: String, permissions: Vec<String>, pool: PgPool) -> anyhow::Result<Self> {
                let password_hashed = bcrypt::hash(password.clone(), bcrypt::DEFAULT_COST)?;

                sqlx::query!(
                    r#" INSERT INTO users (username, password)
                        VALUES ($1, $2)
                        ON CONFLICT (username) DO NOTHING"#,
                    username,
                    password_hashed
                )
                    .execute(&pool)
                    .await?;

                let new_user = sqlx::query!(
                    r#" SELECT id
                        FROM users
                        WHERE username=$1"#,
                        username
                )
                    .fetch_one(&pool)
                    .await?;

                for permission in permissions.iter() {
                    sqlx::query(
                        r#" INSERT INTO user_permissions (user_id, token)
                            SELECT $1, $2
                            WHERE NOT EXISTS(
                                SELECT 1 FROM user_permissions WHERE user_id=$1 AND token=$2
                            )"#)
                        .bind(new_user.id)
                        .bind(permission)
                        .execute(&pool)
                        .await?;
                }

                Ok(BackendUser{
                    id: new_user.id,
                    username,
                    password,
                    permissions: permissions.into_iter().collect()
                })
            }

            pub async fn get(id: i32, pool: &PgPool) -> Option<Self> {
                let user_fields = sqlx::query!(
                    r#"SELECT id, username, password
                    FROM users
                    WHERE id = $1"#,
                    id
                )
                    .fetch_one(pool)
                    .await
                    .ok()?;

                let mut user = BackendUser{
                    id: user_fields.id,
                    username: user_fields.username,
                    password: user_fields.password,
                    permissions: HashSet::new()
                };

                //lets just get all the tokens the user can use, we will only use the full permissions if modifing them.
                let user_perms = sqlx::query!(
                    "SELECT token FROM user_permissions WHERE user_id = $1;",
                    id
                )
                    .fetch_all(pool)
                    .await;

                user.permissions.extend(
                    user_perms
                        .unwrap_or_default()
                        .into_iter()
                        .map(|p| p.token)
                );

                Some(user)
            }

            pub async fn get_from_username(username: String, pool: &PgPool) -> Option<Self> {
                let user_fields = sqlx::query!("SELECT id, username, password FROM users WHERE username = $1", username)
                    .fetch_one(pool)
                    .await
                    .ok()?;

                let mut user = BackendUser{
                    id: user_fields.id,
                    username: user_fields.username,
                    password: user_fields.password,
                    permissions: HashSet::new()
                };

                //lets just get all the tokens the user can use, we will only use the full permissions if modifing them.
                let user_perms = sqlx::query!(
                    "SELECT token FROM user_permissions WHERE user_id = $1;",
                    user.id
                )
                    .fetch_all(pool)
                    .await;

                user.permissions.extend(
                    user_perms
                        .unwrap_or_default()
                        .into_iter()
                        .map(|p| p.token)
                );

                Some(user)
            }
        }

        #[async_trait]
        impl Authentication<BackendUser, i32, PgPool> for BackendUser {
            async fn load_user(userid: i32, pool: Option<&PgPool>) -> Result<BackendUser, anyhow::Error> {
                let pool = pool.unwrap();

                BackendUser::get(userid, pool)
                    .await
                    .ok_or_else(|| anyhow::anyhow!("Cannot get user"))
            }

            fn is_authenticated(&self) -> bool {
                true
            }

            fn is_active(&self) -> bool {
                true
            }

            fn is_anonymous(&self) -> bool {
                false
            }
        }

        #[async_trait]
        impl HasPerm<PgPool> for BackendUser {
            async fn has(&self, perm: &str, _pool: &Option<&PgPool>) -> bool {
                self.permissions.contains(perm)
            }
        }

        #[inline(always)]
        pub fn auth() -> Result<AuthSession, ServerFnError> {
            use_context::<AuthSession>()
                .ok_or("Auth session missing.")
                .map_err(|e| ServerFnError::ServerError(e.to_string()))
        }
    }
}
