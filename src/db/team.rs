use sqlx::PgPool;

#[derive(sqlx::FromRow, serde::Deserialize)]
pub struct Team {
    pub id: i64,
    pub name: String,
    pub logo: Option<String>,
}

pub async fn search(
    team_name: &str,
    limit: Option<i64>,
    pool: &PgPool,
) -> Result<Vec<Team>, sqlx::Error> {
    let limit = limit.unwrap_or(5);
    sqlx::query_as!(
        Team,
        r#"	SELECT *
			FROM teams
			WHERE LOWER(name) LIKE '%' || LOWER($1) || '%'
            ORDER BY name
            LIMIT $2"#,
        team_name,
        limit
    )
    .fetch_all(pool)
    .await
}
