use cfg_if::cfg_if;
use leptos::*;


cfg_if! {
    if #[cfg(feature = "ssr")] {
        use std::collections::HashSet;

        #[derive(Clone, Debug, PartialEq, Eq)]
        pub struct BackendUser {
            pub id: i64,
            pub username: String,
            pub password: String,
            pub permissions: HashSet<String>,
        }

        use async_trait::async_trait;
        use sqlx::PgPool;
        use axum_session_auth::{SessionPgPool, Authentication, HasPermission as HasPerm};
        pub type AuthSession = axum_session_auth::AuthSession<BackendUser, i64, SessionPgPool, PgPool>;

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

            pub async fn get(id: i64, pool: &PgPool) -> Option<Self> {
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

                log!("GET FROM USERNAME {:?}", user);

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
        impl Authentication<BackendUser, i64, PgPool> for BackendUser {
            async fn load_user(userid: i64, pool: Option<&PgPool>) -> Result<BackendUser, anyhow::Error> {
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
        pub fn auth(cx: Scope) -> Result<AuthSession, ServerFnError> {
			use_context::<AuthSession>(cx)
				.ok_or("Auth session missing.")
				.map_err(|e| ServerFnError::ServerError(e.to_string()))
		}
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct FrontendUser {
    pub id: i64,
    pub username: String
}

#[server(GetUser, "/secure")]
pub async fn get_user(cx: Scope) -> Result<FrontendUser, ServerFnError> {
    let auth = auth(cx)?;
    let BackendUser{ id, username, .. } = auth.current_user.unwrap();
    Ok(FrontendUser{ id, username })
}

#[server(GetUsername, "/secure")]
pub async fn get_username(cx: Scope) -> Result<String, ServerFnError> {
    let auth = auth(cx)?;

    Ok(auth.current_user.unwrap().username)
}

#[server(GetUserID, "/secure")]
pub async fn get_user_id(cx: Scope) -> Result<i64, ServerFnError> {
    let auth = auth(cx)?;

    Ok(auth.current_user.unwrap().id)
}

#[server(HasPermission, "/secure")]
pub async fn has_permission(cx: Scope, permission: String) -> Result<bool, ServerFnError> {
    match auth(cx)? {
        AuthSession{current_user: Some(user), ..} => {
            Ok(user.permissions.contains(&permission))
        },
        _ => Ok(false)
    }
}
