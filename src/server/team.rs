use leptos::*;

use crate::objects::Team;

use cfg_if::cfg_if;
cfg_if! {
	if #[cfg(feature = "ssr")] {
		use crate::server::pool;
	}
}

#[server(GetTeam, "/secure")]
pub async fn get_team(cx: Scope, id: i64) -> Result<Team, ServerFnError> {
    let pool = pool(cx)?;

    let result = sqlx::query_as::<_, Team>(
        r#"SELECT *
            FROM teams
            WHERE id = $1"#
    )
        .bind(id)
        .fetch_one(&pool)
        .await?;

    Ok(result)
}

#[server(GetSpreadTeams, "/secure")]
pub async fn get_spread_teams(cx: Scope, home_id: i64, away_id: i64) -> Result<(Team, Team), ServerFnError> {
    Ok((get_team(cx, home_id).await?, get_team(cx, away_id).await?))
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