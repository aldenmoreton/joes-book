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
                WITH spread_cols AS (
                    SELECT (spread->>'home_id')::INT AS home_id, (spread->>'away_id')::INT AS away_id
                    FROM (SELECT jsonb_array_elements(contents->'spread_group') AS spread
                    FROM public.events
                    WHERE contents->'spread_group' IS NOT NULL AND chapter_id = $1)
                )
                SELECT home_id AS ids
                FROM spread_cols
                UNION
                SELECT away_id
                FROM spread_cols
            ) AS cols ON cols.ids=teams.id
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
