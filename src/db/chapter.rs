use sqlx::PgPool;

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
