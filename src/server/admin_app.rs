use leptos::*;

use cfg_if::cfg_if;

cfg_if! {
	if #[cfg(feature = "ssr")] {
		use crate::{
			server::{
				auth,
				pool
			},
			objects::BackendUser
		};
	}
}

#[server(MakeAdmin, "/secure", "Url", "make_admin")]
pub async fn make_admin(id: i64) -> Result<bool, ServerFnError> {
	let auth = auth()?;
	match auth.current_user {
		Some(BackendUser { permissions, ..}) if permissions.contains("owner") => (),
		_ => return Err(ServerFnError::Request("You can't go in there!".into()))
	}

	let pool = pool()?;
	let result = sqlx::query(
		r#"	SELECT user_id
					FROM user_permissions
					WHERE user_id=$1 AND token=$2"#,
	)
		.bind(id)
		.bind("admin")
		.fetch_optional(&pool)
		.await?;

	if result.is_some() { return Ok(false) }

	sqlx::query(
		r#"	INSERT INTO user_permissions (user_id, token)
			VALUES ($1, $2)"#
	)
		.bind(id)
		.bind("admin")
		.execute(&pool)
		.await?;

	Ok(true)
}