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

#[server(Signup, "/api", "Url", "signup")]
pub async fn signup(
    username: String,
    password: String,
    password_confirmation: String,
    remember: Option<String>,
) -> Result<(), ServerFnError> {
    let pool = pool()?;
    let auth = auth()?;

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

    let user = BackendUser::get_from_username(username, &pool)
        .await
        .ok_or("Signup failed: User does not exist.")
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    auth.login_user(user.id);
    auth.remember_user(remember.is_some());

    leptos_axum::redirect("/");

    Ok(())
}

#[server(Login, "/api", "Url", "login")]
pub async fn login(
    username: String,
    password: String,
    remember: Option<String>,
) -> Result<(), ServerFnError> {
    let pool = pool()?;
    let auth = auth()?;

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
            leptos_axum::redirect("/");
            Ok(())
        }
        false => Err(ServerFnError::ServerError(
            "Password does not match.".to_string(),
        )),
    }
}

#[server(Logout, "/secure", "Url", "logout")]
pub async fn logout() -> Result<(), ServerFnError> {
    let auth = auth()?;

    auth.logout_user();
    leptos_axum::redirect("/");

    Ok(())
}

#[server(SearchUser, "/secure", "Url", "search_user")]
pub async fn search_user(username: String) -> Result<Vec<FrontendUser>, ServerFnError> {
	let pool = pool()?;

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

#[server(GetUser, "/secure", "Url", "get_user")]
pub async fn get_user() -> Result<FrontendUser, ServerFnError> {
    let auth = auth()?;
    let BackendUser{ id, username, .. } = auth.current_user.unwrap();
    Ok(FrontendUser{ id, username })
}

#[server(GetUsername, "/secure", "Url", "get_username")]
pub async fn get_username() -> Result<String, ServerFnError> {
    let auth = auth()?;

    Ok(auth.current_user.unwrap().username)
}

#[server(GetUserID, "/secure", "Url", "get_user_id")]
pub async fn get_user_id() -> Result<i64, ServerFnError> {
    let auth = auth()?;

    Ok(auth.current_user.unwrap().id)
}

#[server(HasPermission, "/secure", "Url", "has_permission")]
pub async fn has_permission(permission: String) -> Result<bool, ServerFnError> {
    match auth()? {
        AuthSession{current_user: Some(user), ..} => {
            Ok(user.permissions.contains(&permission))
        },
        _ => Ok(false)
    }
}

#[server(GetPermissions, "/secure", "Url", "get_permissions")]
pub async fn get_permissions() -> Result<Vec<String>, ServerFnError> {
    match auth()? {
        AuthSession{current_user: Some(user), ..} => {
            Ok(user.permissions.into_iter().collect())
        },
        _ => Ok(Vec::new())
    }
}
