use sqlx::PgPool;

#[derive(Debug, Clone)]
pub struct Chapter {
    pub chapter_id: i32,
    pub book_id: i32,
    pub is_open: bool,
    pub title: String,
}

pub async fn get_chapters(
    _user_id: i32,
    book_id: i32,
    pool: &PgPool,
) -> Result<Vec<Chapter>, sqlx::Error> {
    // let book_subscription = get_book(user_id, book_id, pool).await?;
    // if let BookRole::Unauthorized = book_subscription.role {
    //     return Err(sqlx::Error::);
    // }

    let result = sqlx::query!(
        r#"	SELECT id AS chapter_id, book_id, is_open, title
			FROM chapters
			WHERE book_id = $1
		"#,
        book_id
    )
    .fetch_all(pool)
    .await?;

    Ok(result
        .into_iter()
        .map(|record| Chapter {
            chapter_id: record.chapter_id,
            book_id: record.book_id,
            is_open: record.is_open,
            title: record.title,
        })
        .collect::<Vec<_>>())
}

pub async fn get_chapter(chapter_id: i32, pool: &PgPool) -> Result<Chapter, sqlx::Error> {
    sqlx::query_as!(
        Chapter,
        r#"	SELECT id AS chapter_id, book_id, title, is_open
			FROM chapters
			WHERE id = $1
		"#,
        chapter_id
    )
    .fetch_one(pool)
    .await
}
