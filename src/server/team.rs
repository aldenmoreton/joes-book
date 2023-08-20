use leptos::*;

use crate::objects::Team;

use cfg_if::cfg_if;
cfg_if! {
	if #[cfg(feature = "ssr")] {
		use crate::server::pool;
	}
}

#[server(SearchTeam, "/secure")]
pub async fn search_team(cx: Scope, name: String) -> Result<Vec<Team>, ServerFnError> {
    let pool = pool(cx)?;

    let result = sqlx::query_as::<_, Team>("SELECT * FROM teams WHERE LOWER(name) LIKE '%' || LOWER($1) || '%' ORDER BY name LIMIT 5")
        .bind(name)
        .fetch_all(&pool)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

	Ok(result)
}