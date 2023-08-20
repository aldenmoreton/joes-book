use leptos::*;

use crate::objects::FrontendUser;

use cfg_if::cfg_if;
cfg_if! {
	if #[cfg(feature = "ssr")] {
		use axum_session_auth::AuthSession;
        use bcrypt::{hash, DEFAULT_COST};
		use crate::{
			server::{
				auth,
				pool
			},
			objects::BackendUser
		};
	}
}

#[server(Signup, "/api")]
pub async fn signup(
    cx: Scope,
    username: String,
    password: String,
    password_confirmation: String,
    remember: Option<String>,
) -> Result<(), ServerFnError> {
    let pool = pool(cx)?;
    let auth = auth(cx)?;

    if password != password_confirmation {
        return Err(ServerFnError::ServerError(
            "Passwords did not match.".to_string(),
        ));
    }

    let password_hashed = hash(password, DEFAULT_COST).unwrap();

    sqlx::query("INSERT INTO users (username, password) VALUES ($1, $2)")
        .bind(username.clone())
        .bind(password_hashed)
        .execute(&pool)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    log!("Signing up");

    let user = BackendUser::get_from_username(username, &pool)
        .await
        .ok_or("Signup failed: User does not exist.")
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    log!("Signing up");

    auth.login_user(user.id);
    auth.remember_user(remember.is_some());

    leptos_axum::redirect(cx, "/");

    Ok(())
}

#[server(Login, "/api")]
pub async fn login(
    cx: Scope,
    username: String,
    password: String,
    remember: Option<String>,
) -> Result<(), ServerFnError> {
    let pool = pool(cx)?;
    let auth = auth(cx)?;

    let user: BackendUser = BackendUser::get_from_username(username, &pool)
        .await
        .ok_or("User does not exist.")
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    match bcrypt::verify(password, &user.password)
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?
    {
        true => {
            auth.login_user(user.id);
            auth.remember_user(remember.is_some());
            leptos_axum::redirect(cx, "/");
            Ok(())
        }
        false => Err(ServerFnError::ServerError(
            "Password does not match.".to_string(),
        )),
    }
}

#[server(Logout, "/secure")]
pub async fn logout(cx: Scope) -> Result<(), ServerFnError> {
    let auth = auth(cx)?;

    auth.logout_user();
    leptos_axum::redirect(cx, "/");

    Ok(())
}

#[server(SearchUser, "/secure")]
pub async fn search_user(cx: Scope, username: String) -> Result<Vec<FrontendUser>, ServerFnError> {
	let pool = pool(cx)?;

	let result = sqlx::query_as::<_, FrontendUser>(
		r#"	SELECT id, username
			FROM users
			WHERE LOWER(username) LIKE '%' || LOWER($1) || '%'
			ORDER BY username LIMIT 5"#
	)
        .bind(username)
        .fetch_all(&pool)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

	Ok(result)
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

#[server(GetPermissions, "/secure")]
pub async fn get_permissions(cx: Scope) -> Result<Vec<String>, ServerFnError> {
    match auth(cx)? {
        AuthSession{current_user: Some(user), ..} => {
            Ok(user.permissions.into_iter().collect())
        },
        _ => Ok(Vec::new())
    }
}
