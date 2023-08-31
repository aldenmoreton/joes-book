use leptos::*;

use crate::objects::BookSubscription;

use cfg_if::cfg_if;
cfg_if! {
	if #[cfg(feature = "ssr")] {
		use crate::{
			server::{
				auth,
				pool,
				has_permission
			},
			objects::BookRole
		};
	}
}

#[server(GetBook, "/secure")]
pub async fn get_book(cx: Scope, book_id: i64) -> Result<BookSubscription, ServerFnError> {
	let user = auth(cx)?.current_user.unwrap();
	let pool = pool(cx)?;

    let result = sqlx::query_as::<_, BookSubscription>(
		r#"	SELECT b.id, b.name, s.role, s.user_id
			FROM books AS b
			INNER JOIN subscriptions AS s ON s.book_id=b.id
			WHERE s.user_id = $1 AND b.id = $2
		"#
	)
        .bind(user.id)
		.bind(book_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

	Ok(result)
}

#[server(GetBooks, "/secure")]
pub async fn get_books(cx: Scope) -> Result<Vec<BookSubscription>, ServerFnError> {
	let user = auth(cx)?.current_user.unwrap();
	let pool = pool(cx)?;

    let result = sqlx::query_as::<_, BookSubscription>(
		r#"	SELECT b.id, b.name, s.role, s.user_id
			FROM books AS b
			INNER JOIN subscriptions AS s ON s.book_id=b.id
			WHERE s.user_id = $1
		"#
	)
        .bind(user.id)
        .fetch_all(&pool)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

	Ok(result)
}

#[server(AddBook, "/secure")]
pub async fn add_book(cx: Scope, name: String) -> Result<i64, ServerFnError> {
	if !has_permission(cx, "admin".into()).await? { return Err(ServerFnError::Request("Not permitted to create books".into())) }

	let user = auth(cx)?.current_user.unwrap();
	let pool = pool(cx)?;

	let result = sqlx::query!(
		r#"	WITH inserted_book AS (
				INSERT INTO books (name) VALUES ($1) RETURNING id
			)
			INSERT INTO subscriptions (book_id, user_id, role)
			SELECT id, $2, $3 FROM inserted_book
			RETURNING book_id"#,
			name,
			user.id,
			Into::<String>::into(BookRole::Owner)
	)
		.fetch_one(&pool)
		.await?;

	Ok(result.book_id)
}
