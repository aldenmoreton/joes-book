use std::collections::HashMap;

use sqlx::PgPool;

#[derive(sqlx::FromRow, serde::Deserialize)]
pub struct Team {
    pub id: i64,
    pub name: String,
    pub logo: Option<String>,
}

pub async fn get_chapter_teams(
    chapter_id: i32,
    pool: &PgPool,
) -> Result<HashMap<i32, (String, Option<String>)>, sqlx::Error> {
    Ok(sqlx::query!(
        r#"
            SELECT teams.*
            FROM teams
            JOIN (
                SELECT (contents->'spread_group'->>'home_id')::int AS ids
                FROM public.events
                WHERE contents->'spread_group' IS NOT NULL AND chapter_id = $1
                UNION ALL
                SELECT (contents->'spread_group'->>'away_id')::int
                FROM public.events
                WHERE contents->'spread_group' IS NOT NULL AND chapter_id = $1
            ) AS ids ON teams.id = ids.ids
        "#,
        chapter_id
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|record| (record.id, (record.name, record.logo)))
    .collect::<HashMap<_, _>>())
}

pub async fn get(team_id: i32, pool: &PgPool) -> Result<Option<Team>, sqlx::Error> {
    sqlx::query_as!(
        Team,
        r#"
            SELECT *
            FROM teams
            WHERE id = $1
        "#,
        team_id
    )
    .fetch_optional(pool)
    .await
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
