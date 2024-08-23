use sqlx::PgPool;

use crate::AppError;

#[derive(Debug, Clone)]
pub struct Chapter {
    pub chapter_id: i32,
    pub book_id: i32,
    pub is_open: bool,
    pub is_visible: bool,
    pub title: String,
}

pub async fn get_chapters(book_id: i32, pool: &PgPool) -> Result<Vec<Chapter>, sqlx::Error> {
    sqlx::query_as!(
        Chapter,
        r#"	SELECT id AS chapter_id, book_id, is_open, title, is_visible
			FROM chapters
			WHERE book_id = $1
		"#,
        book_id
    )
    .fetch_all(pool)
    .await
}

pub async fn get_chapter(chapter_id: i32, pool: &PgPool) -> Result<Chapter, sqlx::Error> {
    sqlx::query_as!(
        Chapter,
        r#"	SELECT id AS chapter_id, book_id, title, is_open, is_visible
			FROM chapters
			WHERE id = $1
		"#,
        chapter_id
    )
    .fetch_one(pool)
    .await
}

pub struct ChapterUser {
    pub user_id: i32,
    pub username: String,
    pub total_points: i32,
}

pub async fn get_chapter_users(
    book_id: i32,
    chapter_id: i32,
    pool: &PgPool,
) -> Result<Vec<ChapterUser>, AppError> {
    sqlx::query_as!(
        ChapterUser,
        r#"
        SELECT user_id, username, COALESCE(total_points, 0)::INT as "total_points!"
        FROM (
            SELECT
                sub1.id AS user_id,
                sub1.USERNAME,
                SUM(COALESCE(picks.POINTS, 0)) AS TOTAL_POINTS
            FROM (
                SELECT users.id, users.username
                FROM users
                JOIN subscriptions on users.id = subscriptions.user_id
                WHERE book_id = $1
            ) as sub1
            LEFT JOIN picks on sub1.id = picks.user_id
            WHERE
                picks.CHAPTER_ID = $2 OR picks.CHAPTER_ID IS NULL
            GROUP BY
                sub1.ID,
                sub1.USERNAME
        ) AS sub2
        ORDER BY total_points DESC, username
        "#,
        book_id,
        chapter_id
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::from)
}
